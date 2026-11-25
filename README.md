# Joshua-Configuration

## Project Summary

Joshua-Configuration is the on-device WebConfig service for Joshua’s Yocto/embedded-Linux target. It
combines a Rust workspace with an Axum daemon, a reusable configuration engine, a no-build-step
vanilla JavaScript frontend, JSON schemas, board manifests, and a Yocto layer for installation. The
daemon is designed to run with a single-thread Tokio runtime and rustls, using JSON and ring files
rather than a database. Its HTTP surface covers system identity, networking, services, updates,
monitoring, logs, alerts, security, and I/O, with authentication, role checks, CSRF protection, and
SSE for live state.

The engine owns configuration storage, schema validation, manifests, alert and monitoring state, and
system appliers. The daemon bridges those operations to networkd, systemd, journald, firewall, GPIO,
CAN, and other backends. Networking changes have a commit/confirm workflow intended to prevent
remote lockout. Real RAUC/zbus interactions are paired with fake-sysroot and mock-oriented test
paths so host tests do not require physical hardware. The frontend renders forms from
`schemas/*.schema.json`, while board-specific values come from `board-manifests/joshua.json` with a
generic fallback. Make targets document build, release, test, lint, vendoring, cross-compilation,
and fake-sysroot execution. API, schema, board-manifest, planning, and audit documents provide
further contracts; deployment behavior still needs confirmation.

## Feature Inventory

| # | Name / Description | Implemented? | Test Coverage? | Manual Test |
| --- | --- | --- | --- | --- |
|   1 | Rust workspace with engine and daemon crates | Implemented  | `make test` scope                         | [ ] Manual verification required |
|   2 | Axum daemon and static frontend              | Implemented  | Partial HTTP integration tests            | [ ] Manual verification required |
|   3 | rustls TLS                                   | Implemented  | Unknown deployment coverage               | [ ] Manual verification required |
|   4 | Authentication, roles, and CSRF              | Implemented  | Partial daemon tests                      | [ ] Manual verification required |
|   5 | Schema-driven forms and validation           | Implemented  | Unknown browser coverage                  | [ ] Manual verification required |
|   6 | Joshua manifest and generic fallback         | Implemented  | Unknown                                   | [ ] Manual verification required |
|   7 | Nine configuration/API sections              | Implemented  | Partial; `crates/webconfigd/src/tests.rs` | [ ] Manual verification required |
|   8 | Engine persistence and system appliers       | Implemented  | Partial engine tests                      | [ ] Manual verification required |
|   9 | Network confirm/rollback workflow            | Implemented  | Unknown timeout/reconnect coverage        | [ ] Manual verification required |
|  10 | Real and fake system backends                | Implemented  | Partial fake-sysroot harness              | [ ] Manual verification required |
|  11 | Server-sent events                           | Implemented  | Unknown                                   | [ ] Manual verification required |
|  12 | Yocto `meta-webconfig` packaging             | Implemented  | Unknown image integration                 | [ ] Manual verification required |
|  13 | Offline vendoring and three cross targets    | Implemented  | Build/lint commands documented            | [ ] Manual verification required |
|  14 | Size-oriented release profile                | Implemented  | Unknown                                   | [ ] Manual verification required |

## Build and Test

journald, firewall, GPIO, CAN, and other backend adapters. Networking changes have a commit/confirm
An **on-device, browser-based configuration web UI** for the Joshua Yocto/embedded-Linux platform
(TI Sitara AM62x). One small Rust daemon serves a vanilla-JS frontend over TLS; board-specific data
lives in a single board-manifest file (`board-manifests/joshua.json`).

> The full design specification lives in [`docs/webconfig-spec.md`](docs/webconfig-spec.md).

## What it is

A field engineer or operator points a browser at the device (`https://<device>:8443`) and configures
it: system identity, networking, services, updates, monitoring, logs, alerts, security, and I/O —
all behind authenticated, role-based access with a commit/confirm rollback safety net for networking
Cargo.toml                  # workspace, pinned deps, exact release profile
Map your BSP `MACHINE` to its board manifest in
`meta-webconfig/recipes-core/webconfig/webconfig-manifest.bbappend`. See
[`meta-webconfig/README.md`](meta-webconfig/README.md).

## Documentation

- [Design specification](docs/webconfig-spec.md)
- [REST API reference](docs/api-reference.md)
- [Schema format](docs/schema-format.md)
- [Board-manifest format](docs/board-manifest.md)
- [Production-readiness audit](docs/planning/MasterPlan.md) and its
  [acceptance checklist](docs/planning/Checkpoint.md)

## License

MIT. See [LICENSE](LICENSE).
