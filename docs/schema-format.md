# WebConfig — Schema Format

Schema-driven rendering is the extension point of WebConfig: **adding a
customer setting is a schema file — zero UI code.** This document is the
canonical reference for the on-disk `*.schema.json` format.

## Location

Schemas are dropped into **`/usr/share/webconfig/schemas/`** by Yocto recipes.
Only files matching `*.schema.json` are loaded (a bare `.json` in that directory
is ignored, so it can never mis-key a validator or break `load_dir`). The repo
root `schemas/` directory is the canonical source used by the engine's tests.

## File layout

A schema file has two roles, driven by the same document:

- **Descriptor** (for the frontend renderer) — `id`, `title`, `groups`, `fields`.
- **JSON Schema (draft-07) validator** (for backend re-validation) — the
  `validate` subschema; if absent, the whole descriptor is used as the
  validator (render-only schemas may omit it).

```json
{
  "id": "app:example",
  "title": "Example customer app",
  "groups": [ { "id": "general", "title": "General", "fields": [ ... ] } ],
  "validate": { "$schema": "...", "type": "object", "properties": { ... } }
}
```

## Routing

Validators and descriptors are keyed by the **schema file stem** (e.g. a file
`app-example.schema.json` has stem `app-example`), **not** by the `app:`-prefixed
descriptor id.

- A descriptor whose `id` starts with **`app:`** governs an **app setting**
  stored under `config.app_settings[<stem>]`.
- Any other descriptor governs a **top-level section** (`config[<stem>]`).

> Route app-vs-top-level by `descriptor.id` (`starts_with("app:")`), never by
> `name.starts_with("app:")` on the file stem — that historically never matched,
> so app settings were never server-validated.

## Field types (`field_type`)

The fixed, closed set (spec §3):

| type | meaning |
|---|---|
| `string` | Free text; `validation.pattern` / `min_len` / `max_len`. |
| `int` | Integer; `validation.min` / `max`. |
| `bool` | Boolean toggle. |
| `enum` | Choice; `options[]` of `{ value, label }`. |
| `secret` | Secret stored on-device only, **never returned in backups or diagnostics**. |

## Field structure

```json
{
  "id": "poll_interval_secs",
  "title": "Poll interval (s)",
  "field_type": "int",
  "required": true,
  "validation": { "min": 1, "max": 3600 },
  "help": "How often the app polls the upstream server.",
  "apply_hook": { "kind": "restart", "target": "my-app.service" }
}
```

### Validation rules

| key | type | applies to |
|---|---|---|
| `min` / `max` | number | `int` |
| `pattern` | string (regex) | `string` |
| `min_len` / `max_len` | integer | `string`, `secret` |

### Apply hook

`apply_hook` declares a side effect to run after a successful apply:
`kind` is `restart`, `sighup`, or `reload`; `target` is a systemd unit name (for
`restart`/`reload`) or a process name (for `sighup`).

## Grouping

`groups[]` groups fields for layout: each has `id`, optional `title`, and a
`fields[]` array.

## Validation

Backend validation uses the `jsonschema` crate. The `validate` subschema is
compiled per stem; `is_valid()` checks the target (app settings by stem, else
top-level section). Missing targets (`null`) are skipped. When the schemas
directory is absent, the engine falls back to **permissive mode** (accepts
anything) — used by tests and first boot before schemas are installed.

## Bundled schemas

- `schemas/app-example.schema.json` — example customer app (documentation).
- `schemas/app-mcu-stm32.schema.json` — the STM32 companion MCU settings, the
  `app:mcu-stm32` schema referenced by the Joshua board manifest's `mcu-c55`
  peripheral (`settings_schema: "app:mcu-stm32"`).

These are duplicated into `meta-webconfig/recipes-core/webconfig/files/` and
installed by the `webconfig-schemas.bb` recipe. Additional schemas are added via
a bbappend (`WEBCONFIG_SCHEMAS_EXTRA:append = " file://my-app.schema.json"`).
