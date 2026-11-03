//! Adapter transport only: no replacement scheduler or production runtime.
pub use adversary_protocol::Request;
use adversary_protocol::{Frame, MAX_FRAME_BYTES, VERSION};
use serde_json::Value;
use std::io::{self, BufRead, Read, Write};

fn read_frame(input: &mut dyn BufRead) -> Result<Option<String>, String> {
    let mut bytes = Vec::new();
    let count = (&mut *input)
        .take((MAX_FRAME_BYTES + 1) as u64)
        .read_until(b'\n', &mut bytes)
        .map_err(|e| e.to_string())?;
    if count == 0 {
        return Ok(None);
    }
    if count > MAX_FRAME_BYTES || bytes.last() != Some(&b'\n') {
        return Err("oversize or incomplete protocol frame".into());
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|e| e.to_string())
}

pub struct Context<'a> {
    id: &'a str,
    input: &'a mut dyn BufRead,
    output: &'a mut dyn Write,
}

impl Context<'_> {
    /// Emits a named cooperative boundary and waits for continue or process death.
    pub fn checkpoint(&mut self, name: &str) -> Result<(), String> {
        let frame = Frame {
            v: VERSION,
            id: self.id.into(),
            kind: "checkpoint".into(),
            name: Some(name.into()),
            result: None,
            error: None,
        };
        serde_json::to_writer(&mut self.output, &frame).map_err(|e| e.to_string())?;
        writeln!(self.output)
            .and_then(|_| self.output.flush())
            .map_err(|e| e.to_string())?;
        let text = read_frame(self.input)?.ok_or("controller disconnected")?;
        let control: Frame = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if control.v != VERSION
            || control.id != self.id
            || control.kind != "continue"
            || control.name.as_deref() != Some(name)
        {
            return Err("invalid checkpoint continuation".into());
        }
        Ok(())
    }
}

pub fn serve(
    handler: impl FnMut(&Request, &mut Context<'_>) -> Result<Value, String>,
) -> Result<(), String> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    serve_io(&mut input, &mut output, handler)
}

pub fn serve_io(
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    mut handler: impl FnMut(&Request, &mut Context<'_>) -> Result<Value, String>,
) -> Result<(), String> {
    while let Some(text) = read_frame(input)? {
        let req: Request = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if req.v != VERSION {
            return Err("unsupported protocol version".into());
        }
        let result = handler(
            &req,
            &mut Context {
                id: &req.id,
                input,
                output,
            },
        );
        let mut frame = Frame::response(&req.id, Value::Null);
        match result {
            Ok(v) => frame.result = Some(v),
            Err(e) => {
                frame.result = None;
                frame.error = Some(e);
            }
        }
        serde_json::to_writer(&mut *output, &frame).map_err(|e| e.to_string())?;
        writeln!(output)
            .and_then(|_| output.flush())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checkpoint_requires_matching_continue_then_returns_response() {
        let input=b"{\"v\":1,\"id\":\"op-1\",\"method\":\"apply\",\"params\":{}}\n{\"v\":1,\"id\":\"op-1\",\"kind\":\"continue\",\"name\":\"written\"}\n";
        let mut output = vec![];
        serve_io(&mut std::io::Cursor::new(input), &mut output, |_, ctx| {
            ctx.checkpoint("written")?;
            Ok(serde_json::json!({"accepted":true}))
        })
        .unwrap();
        let frames = String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str::<Frame>(s).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].kind, "checkpoint");
        assert_eq!(frames[1].kind, "response");
    }
    #[test]
    fn mismatched_continuation_does_not_acknowledge_operation() {
        let input=b"{\"v\":1,\"id\":\"op-1\",\"method\":\"apply\",\"params\":{}}\n{\"v\":1,\"id\":\"other\",\"kind\":\"continue\",\"name\":\"written\"}\n";
        let mut output = vec![];
        serve_io(&mut std::io::Cursor::new(input), &mut output, |_, ctx| {
            ctx.checkpoint("written")?;
            Ok(serde_json::json!({"accepted":true}))
        })
        .unwrap();
        let frame: Frame =
            serde_json::from_str(String::from_utf8(output).unwrap().lines().last().unwrap())
                .unwrap();
        assert!(frame.error.is_some());
        assert!(frame.result.is_none());
    }
    #[test]
    fn invalid_version_and_oversize_input_are_errors() {
        assert!(
            serve_io(
                &mut std::io::Cursor::new(
                    b"{\"v\":2,\"id\":\"x\",\"method\":\"x\",\"params\":{}}\n"
                ),
                &mut vec![],
                |_, _| Ok(Value::Null)
            )
            .is_err()
        );
        assert!(read_frame(&mut std::io::Cursor::new(vec![b'x'; MAX_FRAME_BYTES + 1])).is_err());
    }
}
