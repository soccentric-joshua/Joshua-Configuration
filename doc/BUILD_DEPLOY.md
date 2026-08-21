# Joshua-Configuration Build and Deployment

## Purpose and scope

Joshua-Configuration is the Rust WebConfig workspace. It contains the `webconfig-engine` library and `webconfigd` daemon, an HTTP/TLS configuration service for the Joshua embedded Linux platform. The root Makefile is the single build, test, lint, cross-build, run, and cleanup entry point.

## 1. Prerequisites

- Rustup and Cargo. The repository pins Rust channel `1.85` in `rust-toolchain.toml`.
- Network access for `make toolchain` to reach rustup and for first dependency resolution, unless the checked-in `vendor/` sources and an appropriate offline Cargo setup are used.
- Rust components `rustfmt` and `clippy`, plus the targets `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and `armv7-unknown-linux-gnueabihf`.
- A host with a usable TCP port. The default development port is `8443`; override it with `WEBCONFIG_PORT=...`.

## 2. Setup

From the repository root, run `make toolchain` once to install the pinned toolchain, `rustfmt`, `clippy`, and the three cross targets. The target is network-dependent and may not work in a restricted environment. The Makefile prefers `$HOME/.cargo/bin/cargo`, while `CARGO=...` can override it.

## 3. Build and compile

- `make build` - debug-build all workspace crates.
- `make release` - build the release profile with `opt-level=z`, LTO, one codegen unit, abort-on-panic, and stripped symbols.
- `make cross` - release-build all three configured cross targets.
- `make vendor` - regenerate the versioned vendored crate tree after dependency changes.
- `make release-artifacts` - run `./tools/release-artifacts.sh` to package the source and vendor tarballs and print SHA-256 checksums. Run `make vendor` first when needed.

Cargo outputs are under `target/`, including target-specific release binaries. The workspace daemon package is `webconfigd`.

## 4. Test and lint

- `make test` - run `cargo test --workspace --all-targets`; tests use fake sysroots and do not require hardware.
- `make lint` - run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`.
- `make fmt` - format the workspace in place.
- `make check` - run `build`, `test`, and `lint`.

## 5. Packaging and artifacts

`make release-artifacts` creates the release tarballs consumed by the Yocto `webconfigd.bb` recipe and reports their real checksums. The release binary is produced by Cargo under `target/release` or a target-specific `target/<triple>/release` directory. This Makefile does not create a DEB/RPM/container image.

## 6. Deployment, install, and run

- `make run-mock` - run `webconfigd` against a temporary fake sysroot with mock backends and no hardware side effects.
- `make run` - run against the real host sysroot (`/`), live `/proc` and `/sys`, real systemd/update backends, and persistent `./data` state. This is a privileged/device-like manual verification path; do not use casually on production hosts.
- `WEBCONFIG_PORT=9443 make run-mock` - run on an alternate port.
- `make stop` - terminate the process listening on `WEBCONFIG_PORT`.

The daemon creates or uses TLS/config state in the configured data directory. No cloud credentials or remote deployment target is defined by the Makefile. Installation into a Yocto image is performed by the consuming OS layer rather than this repository's Makefile.

## 7. Cleanup

- `make clean` runs `cargo clean` and removes `target/`.
- `make vendor` changes checked-in vendored inputs and should be reviewed before committing.

## 8. Hardware, cloud, and manual verification caveats

The test suite and `make run-mock` are host-only and intentionally fake hardware. `make run` reads live host interfaces and can invoke real systemd/update behavior; reserve it for a controlled target or explicitly provisioned test host. Cross compilation validates target compilation, not device boot, TLS provisioning, systemd integration, or actual hardware vitals. There is no cloud deployment workflow in this leaf.
