# Joshua — project-managers fleet conventions

Reference for how this repository and its siblings interoperate within the
`project-managers` fleet, and the shared conventions that govern them.

## Repository index (current repo)

- **Repo:** `soccentric-joshua/Joshua-Configuration`
- **Parent project:** `Joshua` (target hardware: **TI Sitara AM62x**)
- **Leaf type:** `<Project>-Configuration`
- **Status:** REAL — actively built out
- **Brand:** SoCcentric WebConfig

See `REPO_MAP.md` at the repo root for the authoritative layout and remote URL.

## Leaf-type conventions

Each project in the fleet has a `-Configuration` leaf. **Important
discrepancy:** other sibling projects' `-Configuration` leaves run a different
Python/FastAPI/Vue dashboard. This repo (Joshua) is architecturally distinct —
a Rust/axum daemon + vanilla-JS frontend, with board-specific data isolated to a
single board manifest. **Do not port fleet-wide `-Configuration` conventions
from one implementation to the other without checking.**

## Shared conventions

- **Build entry point:** a repo-root `Makefile` is the single entry point for
  build / release / test / lint / fmt / vendor / cross / run / clean. Never
  bypass it with ad-hoc cargo commands in CI or PR validation.
- **Validation gate:** `make build && make test && make lint`. The per-repo
  wrapper `.claude/validate.sh` (quick / full) enforces it.
- **Secrets:** never commit secrets (`.env`, `.pem`, `.key` are gitignored).
- **Human-owned tooling:** `.claude/lib`, hooks, commands, and `settings.json`
  are human-owned — run them, never edit them.

## Git & branching

- Default branch: `develop`. PR/CI run against `main` and `develop`.
- Git workflows run the validation gate on every push/PR.

## Cross-project references

- The full 9-project / 8-leaf hierarchy, global conventions, and this
  fleet-doc location are defined in the master `docs/project-managers.md` at the
  `Joshua` project root (not this repo). This file is the repo-local mirror that
  records how this leaf fits into the fleet.

## Related sibling docs

- `docs/webconfig-spec.md` — design specification.
- `docs/api-reference.md` — REST API shapes.
- `docs/schema-format.md` — schema-driven rendering format.
- `docs/board-manifest.md` — board-manifest format.
- `docs/planning/MasterPlan.md` / `docs/planning/Checkpoint.md` — audit + checklist.
