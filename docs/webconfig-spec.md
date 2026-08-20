# SoCcentric WebConfig — Design Specification

Version 1.0 · Joshua platform (TI Sitara AM62x) · SoCcentric internal IV&V tooling

This document is the authoritative design specification for **WebConfig**: the
on-device, browser-based configuration web UI shipped into Joshua-OS as the
`meta-webconfig` Yocto layer. It is a single-document companion to the REST API
reference (`docs/api-reference.md`), the schema format (`docs/schema-format.md`),
and the board-manifest format (`docs/board-manifest.md`).

## 1. Purpose

A field engineer or operator points a browser at the device
(`https://<device>:8443`) and configures it: system identity, networking,
services, updates, monitoring, logs, alerts, security, and I/O — all behind
authenticated, role-based access with a commit/confirm rollback safety net for
networking changes.

## 2. Non-negotiable constraints

- **One daemon** (`webconfigd`), tokio `current_thread` runtime.
- **rustls** for TLS — no OpenSSL anywhere in the dependency tree.
- **No database server** — SQLite (bundled `rusqlite`) + ring log files.
- **Frontend** is vanilla JS with **no build step**, no Node at build time,
  under 500 KB total, served by the daemon.
- **Footprint:** < 15 MB resident RAM, < 10 MB image addition, target binary
  3–6 MB (release profile is pinned in `Cargo.toml` and must not be relaxed:
  `opt-level = "z"`, `lto`, `codegen-units = 1`, `panic = "abort"`, `strip =
  "symbols"`).
- **Reproducible builds:** vendored crates + `CARGO_NET_OFFLINE=1`; cross
  targets `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
  `armv7-unknown-linux-gnueabihf`.

## 3. Layering

```
HTTP layer (webconfigd)  →  config engine (webconfig-engine)  →  platform (board manifest)
```

- **Engine (`webconfig-engine`)** is deliberately free of HTTP/async code so it
  can be unit-tested against a tempdir fake sysroot on x86 CI with no hardware.
  It owns the canonical config store (SQLite at `/data/webconfig/config.db`,
  WAL, atomic commits, backup-snapshot row).
- **HTTP layer (`webconfigd`)** talks to the engine over a channel and runs
  appliers on blocking tasks. Mounted under `/api/v1/...`.
- **Platform layer** is the board manifest (`board-manifests/joshua.json` on
  Joshua) — the only board-specific data. I/O rendering is 100% manifest-driven:
  zero hardcoded I/O.
- **Schema-driven rendering (spec §3):** adding a customer setting is a schema
  file (`*.schema.json` in `/usr/share/webconfig/schemas/`) — zero UI code.
  The same schema drives both frontend rendering and backend validation.

## 4. Functional sections

### §4.1 System
Identity (hostname, asset tag, hardware revision, serial), clock/NTP/PTP/GPS,
SSH & console, users/groups, roles, reboot/shutdown, root password,
factory-reset. `GET /api/v1/system/info` returns per-interface `macs` plus
`uptime_secs`.

### §4.2 Networking (flagship: rollback safety net)
Network changes are **provisional**. The operator must reconnect and `confirm`
within a window (`confirm_window_secs`, default 90) or the change **auto-rolls
back**. Includes Wi-Fi scanning, connectivity testing, firewall, AP mode, VLAN,
bridges, cellular.

### §4.3 Services & Apps
Curated service allowlist from the manifest (never enumerate all units), plus
arbitrary customer "apps" configured via `app:<stem>.schema.json`. App settings
are validated against the schema by file stem.

### §4.4 Updates
RAUC-style slot listing, install, upload (streamed to disk), confirm, factory
reset, OTA, and **backup/restore**. Backups are encrypted (AEAD with a
passphrase) and **secret fields never appear in backups** (redacted).

### §4.5 Monitoring
Vitals (`/monitoring/vitals`), eMMC wear, dashboard tiles declared as manifest
data, telemetry publish.

### §4.6 Logs
Journal read/download, sources, storage config, and an **append-only audit log**
(`/logs/audit`). The audit log is only wiped by a full factory reset — there is
no `DELETE /logs/audit`.

### §4.7 Alerts
Rule evaluation on a timer inside the daemon on the same data the dashboard
reads. Per-platform default rule packs shipped in the manifest
(`default_alert_rules`). Rules, webhook/push targets, history.

### §4.8 Security
Idle timeout, plain-HTTP opt-in (default TLS-only), WireGuard, certificate
store, TLS identity upload, CSR generation, trust panel (TPM/secure-boot/RTC
capabilities from the manifest), role-based access (`admin` / `operator` /
`viewer`). Sessions are **httpOnly + Secure** (TLS-only daemon). A missing or
invalid `X-CSRF-Token` on a mutating route returns **400 BadRequest**. Every
service unit listed in the polkit rule must match the manifest's
`service_allowlist` exactly.

### §4.9 I/O
Manifest-driven peripherals: CAN, serial, GPIO, I2C, SPI, MCU (with firmware
signature verification — Ed25519), camera, display, PRU, thermal. AP radio
parameters (`interface`, `hw_mode`, `channel`) come from the manifest's
`wifi_radio` section (falling back to `wlan0`/`g`/`7`).

## 5. Auth, sessions, CSRF

- First boot creates an admin; protected routes are gated until then.
- Sessions: tower-sessions, MemoryStore, `Secure` + `httpOnly` + `SameSite=Lax`.
- Role-based access on mutating routes.
- CSRF guard on all mutating routes (missing/invalid token → **400**, not 403).
- `Strict-Transport-Security` header on every response; plaintext listener only
  redirects to HTTPS.

## 6. Testing & CI

- Engine unit tests + in-crate HTTP integration tests (binary crate, no lib
  target — external `tests/` cannot import crate internals, so tests live in
  `crates/webconfigd/src/tests.rs`).
- `make build && make test && make lint` is the validation gate.
- CI builds all three cross targets on every commit.

## 7. Deployment

`meta-webconfig` Yocto layer; `webconfigd` runs as non-root user with a hardened
systemd unit (see `meta-webconfig/README.md`). Four recipes: `webconfigd`,
`webconfig-static`, `webconfig-schemas`, `webconfig-manifest`.

## 8. Notes / assumptions

- `cargo.bbclass` from oe-core (scarthgap+) is used; `meta-rust` not required.
- Release tarball SHA-256s in `webconfigd.bb` are filled by a release script.
