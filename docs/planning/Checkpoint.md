# WebConfig — Acceptance Checklist (Checkpoint)

Acceptance criteria for WebConfig, cross-referenced from `MasterPlan` (same
directory — keep both files together). Each item is **PASS** (verified by test)
or **OPEN** (still to do). The corresponding test/guard is noted where one
exists.

## Build & tooling

- [x] Reproducible release profile pinned in `Cargo.toml` (`opt-level=z`, `lto`,
      `panic=abort`, `strip=symbols`). — verified by `make release`.
- [x] No OpenSSL in the dependency tree (rustls only).
- [x] No Node / no build step for the frontend (< 500 KB).
- [x] Engine + HTTP integration tests pass (89/89) on a fake sysroot with no
      hardware. — `make test`.
- [x] `make lint` (clippy `-D warnings` + rustfmt) clean.
- [ ] Vendored `vendor/` committed so a fresh offline checkout builds.
      — guard: `CARGO_NET_OFFLINE=true cargo build --workspace` on a fresh clone.
- [ ] CI pins the same Rust version as `rust-toolchain.toml` (1.97.1).
- [ ] CI cross-build matrix includes `x86_64-unknown-linux-gnu` (all three
      targets).
- [ ] Release binary within the spec §2 3–6 MB budget on the ARM cross
      artifacts (host x86_64 is ~7.8 MB and is not the authoritative measure).

## Functionality (verified by integration tests)

- [x] First-boot admin creation + authenticated session. — `first_boot_creates_admin_and_authenticates`.
- [x] Session cookie is `Secure` + `httpOnly` on the wire.
- [x] CSRF guard rejects mutations without a token (400). — `csrf_guard_rejects_mutations_without_token`.
- [x] Role-based access (viewer/operator/admin). — `viewer_role_*`, `operator_role_*`.
- [x] Networking provisional apply + confirm + auto-rollback. — `network_apply_confirm_and_rollback_via_http`,
      `confirm_timer_rolls_back_when_window_expires`.
- [x] Update install / confirm / OTA backup-restore. — `updates_install_ota_backup_restore_via_http`.
- [x] Upload streams to disk and reaches install; oversized upload rejected. — `updates_upload_*`.
- [x] TLS identity upload replaces cert and rejects mismatch. — `tls_identity_upload_*`.
- [x] Security PUT/apply via HTTP. — `security_put_apply_and_exposure_via_http`.
- [x] Services verbs report real failure (not fake ok). — `services_verbs_report_real_failure_not_fake_ok`.
- [x] Serial apply writes baud/unit, rejects conflicts. — `serial_apply_writes_baud_unit_and_rejects_conflicts`.
- [x] MCU firmware signature verification (real signature passes, tampering
      rejected). — `mcu_firmware_verifies_real_signature_and_rejects_tampering`.
- [x] App-schema secret fields never appear in backup or diagnostics. — `app_schema_secret_field_never_appears_in_backup_or_diagnostics`.
- [x] Cellular 5G is an untyped placeholder with redacted secrets. — `cellular_5g_is_an_untyped_placeholder_with_redacted_secrets_in_backup`.
- [x] Factory reset wipes certs + alert history; partial failure reported.
      — `factory_reset_wipes_certs_and_alert_history`, `factory_reset_reports_partial_failure_*`.
- [x] Monitoring/logs/io happy paths via HTTP. — `monitoring_logs_and_io_happy_paths_via_http`.
- [x] Cert store upload/list/delete round trip. — `cert_store_upload_list_delete_round_trip`.
- [x] Trust status distinguishes unavailable from real read. — `trust_status_distinguishes_unavailable_from_real_read`.
- [x] SSE requires auth + emits vitals; concurrency-capped. — `events_sse_*`.
- [x] Network provisional state surfaces when a later applier fails. — `network_apply_surfaces_provisional_state_*`.
- [x] Reboot/shutdown/root-password/last-admin/group guards. — `reboot_shutdown_root_password_last_admin_guard_and_groups_via_http`.

## Schema & manifest consistency

- [x] Joshua manifest declares its hardware blocks (stm32, can). — validate gate.
- [x] Polkit allowlist matches manifest `service_allowlist`. — `polkit_allowlist_matches_manifest_service_allowlist`.
- [x] `app:mcu-stm32` schema loads and renders real fields. — `real_mcu_stm32_schema_loads_and_renders_real_fields`.
- [x] App schemas validated by file stem, not `app:`-prefixed id. — `app_schema_validates_app_settings_by_stem`.
- [ ] Placeholder firmware-signing pubkey (`PLACEHOLDER`) in Joshua manifest
      replaced with the real M4F key at BSP provisioning time.

## Documentation

- [x] `docs/webconfig-spec.md` present.
- [x] `docs/api-reference.md` present.
- [x] `docs/schema-format.md` present.
- [x] `docs/board-manifest.md` present.
- [x] `docs/planning/MasterPlan.md` + `docs/planning/Checkpoint.md` present and
      co-located.
- [x] `docs/project-managers.md` present.

> The ticked-but-OPEN documentation items reflect the state after the docs were
> restored; the OPEN build/CI/size items remain to be closed in CI.
