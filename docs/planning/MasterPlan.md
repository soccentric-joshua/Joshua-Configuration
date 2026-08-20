# WebConfig — Master Plan & Production-Readiness Audit

This is the production-readiness audit for WebConfig. It cross-references the
acceptance checklist in `Checkpoint` (same directory — keep both files together).

## Status

**REAL / actively built out.** The engine and HTTP daemon are complete; all
engine + HTTP integration tests pass on a fake sysroot with no hardware. The
remaining open items are tracked in Checkpoint.

## Architecture summary

- One daemon, `current_thread` tokio runtime, rustls (no OpenSSL), SQLite (no DB
  server), vanilla-JS frontend with no build step.
- Layering: HTTP (`webconfigd`) → engine (`webconfig-engine`, HTTP-free) →
  platform (board manifest).
- Schema-driven rendering: a new customer setting is a schema file.
- Flagship networking rollback safety net (provisional → confirm/rollback).

## Production-readiness areas

### 1. Resource budget (spec §2)
- < 15 MB resident RAM, < 10 MB image addition, 3–6 MB binary.
- Release profile is pinned (`opt-level=z`, `lto`, `panic=abort`, `strip=symbols`).
- **Watch item:** host x86_64 release binary is ~7.8 MB; ARM targets are smaller
  and are the real measure. Enforce the budget on the ARM cross artifacts.

### 2. Reproducible, offline builds
- Vendored crates + `CARGO_NET_OFFLINE=1`; pinned toolchain (`rust-toolchain.toml`).
- Cross targets: x86_64, aarch64, armv7.
- **Action:** commit the vendored sources so a fresh offline checkout builds.

### 3. CI parity
- `make build && make test && make lint` gate; all three cross targets built.
- **Action:** CI must pin the same Rust version as `rust-toolchain.toml` and
  include the `x86_64` cross target in the matrix.

### 4. Security hardening (spec §4.8)
- Non-root daemon; hardened systemd unit (`ProtectSystem`, `NoNewPrivileges`,
  `PrivateDevices`, `MemoryDenyWriteExecute`, empty `CapabilityBoundingSet`).
- Sessions httpOnly + Secure; CSRF guard (400 on bad token); HSTS.
- Append-only audit log; secret fields redacted from backups.
- **Action:** replace placeholder firmware-signing pubkey in the Joshua manifest
  at BSP provisioning time.

### 5. Documentation
- Spec, API reference, schema format, board-manifest format, this audit, and
  the acceptance checklist are all maintained under `docs/`.

## Verification workflow

The per-repo validation gate is `.claude/validate.sh` (`quick` for fast
checks, `full` = `make check` = build && test && lint). Exit 0 is pass; 1–76
fail; 77 means a check never ran (not a pass).
