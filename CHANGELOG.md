# Changelog

Maintained by the Loop Engine. One entry per loop that touched this repo,
newest first, appended on loop close (pass or fail) and committed on the loop's
own feature branch. See the master repo's `Joshua-Loop-Orchestration-Spec-v2.md` §11.1.

Entries written by hand are fine; the engine inserts directly under this header.

## Gap-remediation pass (build/CI/docs)
- Committed vendored crate sources (`vendor/`) for offline reproducible builds.
- Removed the `.gitignore` rule that ignored `vendor/`.
- Aligned CI toolchain to the pinned `rust-toolchain.toml` (1.97.1).
- Added `x86_64-unknown-linux-gnu` to the CI cross-build matrix (all three targets).
- Fixed clippy `non-snake-case` lint (`parse_Joshua_manifest` → `parse_joshua_manifest`).
- Resolved a duplicate nav shortcut key (GPS/GNSS now `g 3`) in `frontend/pages/registry.js`.
- Restored the `docs/` set: webconfig-spec, api-reference, schema-format, board-manifest,
  planning/MasterPlan, planning/Checkpoint, project-managers.
