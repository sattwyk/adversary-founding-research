# Whole-guest volatile-state loss experiment

This experimental runner boots a disposable Alpine Linux kernel/initramfs under QEMU/KVM, mounts an ext4 test image, runs a small workload, and terminates QEMU with host `SIGKILL` after an external serial acknowledgment. Recovery boots a fresh kernel against the surviving image. It also runs the real bbolt 1.4.3 implementation.

No host block devices, loop devices, device-mapper configuration or root privileges are used. The data volumes are regular files created by the job. No guest network is configured. QEMU's graceful `quit` command is deliberately not used because it flushes caches.

Requirements: Linux x86_64 with usable `/dev/kvm`, QEMU, `mkfs.ext4`, cpio, Python 3 and Go 1.26 or later for this experiment's worker module. Roughly 50 MiB of pinned boot assets and several hundred MiB of temporary disk images are needed for one repetition. Default three repetitions retain more images.

```bash
python3 experiments/guest-loss/run.py \
  --assets /absolute/path/to/asset-cache \
  --output /absolute/path/to/fresh-experiment-directory \
  --repetitions 1
```

The script downloads missing official Alpine assets, checks them against `assets.json`, builds a static Go PID 1 and bbolt worker, and records results/logs. The upstream `latest-stable` asset URLs are mutable; checksum mismatch stops the experiment. Supply the originally tested cached files rather than silently changing the profile. The cache must be a different directory from the output. Reproduction does not require extracting or installing packages on the host.

To recover repeatedly from one frozen witness, use `--replay-image /path/to/bbolt-unsafe-0.witness.img --replay-impl bbolt --replay-case unsafe --repetitions 3` with a fresh output directory. Each recovery uses a separate copy; the script verifies that the original witness hash remains unchanged.

Tested profile: Alpine kernel 6.18.58-0-virt, QEMU 11.0.1, ext4 defaults, 64 MiB raw test volume, one vCPU, 256 MiB memory, `cache=none,aio=threads`, host kill after guest acknowledgment. Kernel and userspace asset hashes are recorded. The fresh-file image is constructed by `mkfs.ext4` on the host; filesystem tools' version/features belong to the profile as well.

Checks: unsynced overwrite, file sync, synced rename plus directory sync, bbolt relaxed durability, bbolt durable default. The retained `*.witness.img` is copied after capture termination and before recovery mounts it. Recovery may mutate its separate image, so the witness must remain the replay source.

**Meaning:** lose guest memory and guest page cache; retain writes that reached the host-backed virtual disk. This samples a concrete machine-failure subset. It does not model loss/reordering of a volatile device cache, arbitrary sector tears, a host power outage, every ext4 allowed outcome, or deterministic execution. Three observed survivals are not universal fsync verification. This remains a separate research probe until backend qualification and lifecycle packaging are implemented.
