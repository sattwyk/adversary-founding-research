# Working research prototypes

These are research probes for the Adversary decision, not a production CLI. They use real redb and bbolt implementations. Unsafe modes intentionally relax durability while the external checker still demands durable acknowledgment. Their failures are expected contract/configuration mismatches, not new library defects.

Requirements: Linux x86_64, Rust >=1.90, Go >=1.26 for native probes, Python 3, git, strace, a C compiler for Gosim, and network access for dependencies. Tested under WSL2. The source directory may be Windows-mounted; generated data and build outputs use Linux temporary directories.

From the extracted bundle directory:

```bash
bash prototypes/run.sh
```

If the archive is extracted so these files are at another location, invoke `bash /absolute/path/to/prototypes/run.sh`. The script resolves its own directory. It prints a fresh results directory containing raw JSON, positional-write traces, effect records, baseline files, and failing witness images. A successful command means the research workflow completed; inspect the result JSON for intentional contract violations. It is not a correctness pass for all configurations.

To reproduce the existing-tool Go baseline:

```bash
bash /absolute/path/to/prototypes/run-gosim.sh
```

This clones the exact tested Gosim revision, adds the supplied 86-line scenario, and invokes a downloaded Go 1.23.2 toolchain with cgo enabled. Five seeds test both safe and deliberately relaxed pair updates. The test expects unsafe mode to recover `(0,0)`, making the contract mismatch visible in the log. Using Go 1.26.4 against that checkout failed during translation in the measured environment.

To reproduce native syscall errors with Faultbox, compile its tested revision:

```bash
git clone https://github.com/faultbox/Faultbox.git /tmp/adv-faultbox-checkout
git -C /tmp/adv-faultbox-checkout checkout 9ed11f9b90036904c069a4e82e04503e56842f51
cd /tmp/adv-faultbox-checkout
go build -o /tmp/adv-faultbox ./cmd/faultbox
```

Then use the binary paths and campaign result directory from a native build:

```bash
python3 /absolute/path/to/prototypes/baseline.py \
  --faultbox /tmp/adv-faultbox \
  --rust /absolute/path/to/rust-target/release/adversary-redb-probe \
  --go /absolute/path/to/go-worker \
  --campaign-root /absolute/path/to/results
```

To replay a saved image (five fresh copies by default):

```bash
python3 /absolute/path/to/prototypes/replay.py \
  --worker /absolute/path/to/go-worker \
  --image /absolute/path/to/evidence/native-campaign/go/unsafe-witness.db
```

`probe.py` captures real successful positional writes and completed sync calls, reconstructs selected storage images, and invokes actual recovery code. It is limited to a pre-existing single-file model; it does not simulate a filesystem, directory persistence, arbitrary mmap, partial syscalls, io_uring, SSD firmware or a full deterministic runtime. Raw effects are retained so the model's assumptions can be inspected. `test_fidelity.py` checks several critical rejection and contract semantics. There is no claim of exhaustive exploration.

`synctest-probe` demonstrates scheduling variation with fake time. `Cargo.lock` and `go.sum` pin native dependencies. All generated build caches and upstream repositories are outside the deliverable. Exact original environment and raw logs are in the sibling `evidence` directory. Nothing here requires root or changes host block devices.

