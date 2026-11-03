use crate::model::Adapter;
use adversary_protocol::{Frame, MAX_FRAME_BYTES, Request, VERSION};
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    os::fd::AsRawFd,
    os::unix::process::{CommandExt, ExitStatusExt},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

pub enum Call {
    Response(Value),
    Interrupted,
}
pub struct Process {
    child: Child,
    input: ChildStdin,
    output: mpsc::Receiver<Result<String, String>>,
    journal: File,
    timeout: Duration,
    campaign_deadline: Instant,
    started: Instant,
    sequence: u64,
    killed: bool,
    pub fault_landed: bool,
}

impl Process {
    pub fn start(
        adapter: &Adapter,
        data: &Path,
        logs: &Path,
        timeout_ms: u64,
        campaign_deadline: Instant,
    ) -> Result<Self> {
        ensure!(
            Instant::now() < campaign_deadline,
            "campaign budget exhausted"
        );
        fs::create_dir_all(data)?;
        fs::create_dir_all(logs)?;
        let data = data.canonicalize()?;
        let journal = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(logs.join("events.ndjson"))?;
        let mut cmd = Command::new(&adapter.command[0]);
        cmd.args(&adapter.command[1..])
            .envs(&adapter.env)
            .env("ADVERSARY_DATA_DIR", data)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(File::create(logs.join("stderr.log"))?)
            .process_group(0);
        if let Some(cwd) = &adapter.cwd {
            cmd.current_dir(cwd);
        }
        let mut child = cmd.spawn().context("start adapter")?;
        let input = child.stdin.take().unwrap();
        let flags = unsafe { libc::fcntl(input.as_raw_fd(), libc::F_GETFL) };
        if flags < 0
            || unsafe { libc::fcntl(input.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) }
                < 0
        {
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.wait();
            return Err(std::io::Error::last_os_error().into());
        }
        let stdout = child.stdout.take().unwrap();
        let (tx, output) = mpsc::sync_channel(128);
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut bytes = Vec::new();
                match reader
                    .by_ref()
                    .take((MAX_FRAME_BYTES + 1) as u64)
                    .read_until(b'\n', &mut bytes)
                {
                    Ok(0) => break,
                    Ok(n) if n > MAX_FRAME_BYTES || bytes.last() != Some(&b'\n') => {
                        let _ = tx.send(Err("oversize or incomplete adapter frame".into()));
                        break;
                    }
                    Ok(_) => {
                        if tx
                            .send(String::from_utf8(bytes).map_err(|e| e.to_string()))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string()));
                        break;
                    }
                }
            }
        });
        Ok(Self {
            child,
            input,
            output,
            journal,
            timeout: Duration::from_millis(timeout_ms),
            campaign_deadline,
            started: Instant::now(),
            sequence: 0,
            killed: false,
            fault_landed: false,
        })
    }

    fn record(&mut self, event: &Value) -> Result<()> {
        let record = json!({"sequence":self.sequence,"controller_elapsed_ns":self.started.elapsed().as_nanos(),"event":event});
        self.sequence += 1;
        serde_json::to_writer(&mut self.journal, &record)?;
        writeln!(self.journal)?;
        self.journal.flush()?;
        Ok(())
    }
    fn send(&mut self, value: &impl serde::Serialize, deadline: Instant) -> Result<()> {
        let mut bytes = serde_json::to_vec(value)?;
        bytes.push(b'\n');
        ensure!(
            bytes.len() <= MAX_FRAME_BYTES,
            "outgoing request exceeds protocol frame limit"
        );
        let mut sent = 0;
        while sent < bytes.len() {
            ensure!(Instant::now() < deadline, "adapter write deadline exceeded");
            match self.input.write(&bytes[sent..]) {
                Ok(0) => bail!("adapter input closed"),
                Ok(n) => sent += n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(1))
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
    pub fn call(&mut self, request: &Request, kill_at: Option<&str>) -> Result<Call> {
        let deadline = (Instant::now() + self.timeout).min(self.campaign_deadline);
        self.record(&json!({"direction":"request", "frame":request}))?;
        self.send(request, deadline)?;
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .context("adapter deadline exceeded")?;
            let line = self
                .output
                .recv_timeout(remaining)
                .context("adapter timeout/disconnect")?
                .map_err(anyhow::Error::msg)?;
            let frame: Frame =
                serde_json::from_str(&line).context("invalid adapter protocol JSON")?;
            self.record(&json!({"direction":"event", "frame":frame}))?;
            ensure!(
                frame.v == VERSION && frame.id == request.id,
                "version or operation ID mismatch"
            );
            match frame.kind.as_str() {
                "checkpoint" => {
                    ensure!(
                        frame.result.is_none() && frame.error.is_none(),
                        "invalid checkpoint frame"
                    );
                    let name = frame.name.as_deref().context("checkpoint name missing")?;
                    if kill_at == Some(name) {
                        self.kill()?;
                        self.fault_landed = true;
                        self.record(&json!({"direction":"controller","fault":"SIGKILL","boundary":"checkpoint","name":name,"confirmed":true}))?;
                        return Ok(Call::Interrupted);
                    }
                    let control = Frame {
                        v: VERSION,
                        id: request.id.clone(),
                        kind: "continue".into(),
                        name: frame.name.clone(),
                        result: None,
                        error: None,
                    };
                    self.send(&control, deadline)?;
                    self.record(&json!({"direction":"control","frame":control}))?;
                }
                "response" => {
                    ensure!(
                        frame.name.is_none(),
                        "response must not contain checkpoint name"
                    );
                    if let Some(error) = frame.error {
                        bail!("adapter returned an unclassified error: {error}");
                    }
                    ensure!(
                        kill_at.is_none(),
                        "requested checkpoint never landed; completed operation is inconclusive"
                    );
                    return Ok(Call::Response(
                        frame.result.context("response result missing")?,
                    ));
                }
                _ => bail!("unexpected protocol frame kind"),
            }
        }
    }
    pub fn kill(&mut self) -> Result<()> {
        if self.killed {
            return Ok(());
        }
        // Do not reap before signaling the owned group: keeping the child PID
        // reserved prevents PID/PGID reuse during cleanup of an exited adapter.
        self.record(&json!({"direction":"controller","fault":"SIGKILL","phase":"requested"}))?;
        // The process group was created by us, with this child's PID as PGID.
        let rc = unsafe { libc::kill(-(self.child.id() as i32), libc::SIGKILL) };
        if rc != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
            return Err(std::io::Error::last_os_error().into());
        }
        let status = self.child.wait()?;
        self.killed = true;
        ensure!(
            status.signal() == Some(libc::SIGKILL),
            "adapter did not terminate from SIGKILL; fault unconfirmed"
        );
        self.record(&json!({"direction":"controller","fault":"SIGKILL","phase":"confirmed","signal":libc::SIGKILL}))?;
        Ok(())
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        if !self.killed {
            unsafe {
                libc::kill(-(self.child.id() as i32), libc::SIGKILL);
            }
            let _ = self.child.wait();
            self.killed = true;
        }
    }
}
