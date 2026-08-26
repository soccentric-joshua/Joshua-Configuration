# WebConfig — Board-Manifest Format

The board manifest is the **platform layer** of WebConfig (spec §3): all
board-specific behavior lives here as **data, not code**. I/O rendering is 100%
manifest-driven — zero hardcoded I/O (spec §4.9).

## Sources

- **Canonical:** a BSP-provided, read-only JSON file at
  `/usr/share/webconfig/board-manifest.json` (on Joshua, installed by the
  `webconfig-manifest` recipe from `board-manifests/joshua.json`).
- **Override:** an optional on-device technician edit at
  `/etc/webconfig/board.yaml` (YAML), e.g. for a generic Linux host
  (`board-manifests/generic.yaml`).

The repo root `board-manifests/` holds the canonical copies referenced by the
engine's tests and by the Yocto recipes — there is exactly one source of truth.

## Top-level fields

| field | type | meaning |
|---|---|---|
| `platform` | string | Platform id (`joshua`, or `generic-linux`). |
| `model` | string | Model id, e.g. `ti-sitara-am62a`. |
| `board_name` | string | Human-readable board name. |
| `peripherals` | array | Declared I/O peripherals (below). |
| `dashboard_tiles` | array | Dashboard tiles rendered generically (below). |
| `service_allowlist` | array | Curated systemd units shown/manageable (never all units). |
| `has_rtc` | bool | RTC present (trust panel). |
| `has_tpm` | bool | TPM present (trust panel). |
| `secure_boot` | bool | Secure boot capability (trust panel). |
| `config_defaults` | object | Optional first-boot values for the live SQLite config (see `generic.yaml` for the full shape). |
| `default_alert_rules` | array | Per-platform default alert rule packs (below). |
| `wifi_radio` | object | AP-provisioning radio params: `interface`, `hw_mode`, `channel` (falls back to `wlan0`/`g`/`7` when absent). |

## Peripherals

Each entry declares an id, Linux device, human name, kind, and free-form
capabilities (so new platform quirks need no code change):

```json
{
  "id": "can0",
  "device": "can0",
  "human_name": "CAN channel 0 (MCAN)",
  "kind": "can",
  "caps": { "fd": true, "termination_gpio": "gpio-can0-term" }
}
```

Supported `kind` values (lowercase): `serial`, `can`, `gpio`, `i2c`, `spi`,
`mcu`, `fpga`, `pru`, `camera`, `display`, `thermal`, `net`.

Examples of `caps`:
- `serial`: `{ "modes": ["rs232"] }`
- `can`: `{ "fd": true, "termination_gpio": "gpio-can0-term" }`
- `gpio`: `{ "direction": "in|out", "active_low": bool, "function": "led", "role": "system" }`
- `mcu`: `{ "protocol": "rpmsg", "signed_fw": true, "settings_schema": "app:mcu-stm32", "firmware_pubkey_ed25519": "..." }`

The engine's `has(id)` rejects config referencing hardware the board does not
declare; `of_kind(kind)` filters peripherals for a given kind.

## Dashboard tiles

Declared as data; the frontend renders them generically:

```json
{
  "id": "cpu",
  "title": "CPU utilization",
  "metrics": ["cpu.util"],
  "render": "sparkline"
}
```

`render` is one of `sparkline | value | state | bar`; `metrics` are ids pulled
from the monitoring endpoint.

## Default alert rules

```json
{
  "id": "disk-full",
  "enabled": true,
  "metric": "disk.pct",
  "comparison": ">",
  "threshold": 90,
  "duration_secs": 60,
  "actions": ["ui"]
}
```

The engine merges these with user-configured rules at evaluation time; an
operator can override or disable a default by id.

## Example manifests

- `board-manifests/joshua.json` — the real Joshua (TI SK-AM62A-LP) manifest:
  19 peripherals (serial/CAN/I2C/SPI/GPIO/PRU/MCU/camera/thermal/Ethernet/Wi-Fi),
  4 dashboard tiles, 4 default alert rules, curated service allowlist, `wifi_radio`
  `wlan0`/`g`/7, RTC/TPM/secure-boot all true.
- `board-manifests/generic.yaml` — a generic Linux host template with an empty
  peripheral list and the full `config_defaults` shape; copy to
  `/etc/webconfig/board.yaml` to drive the engine on any Linux host.

## Firmware signing key (signed firmware trust anchor)

A peripheral advertising `"signed_fw": true` must carry a valid Ed25519
verification key in `caps.firmware_pubkey_ed25519` (32 bytes, base64).
`BoardManifest::validate_firmware_keys()` **fails closed**: a missing,
placeholder, or malformed key is rejected so a signed-firmware claim with no
usable trust anchor can never pass validation (see `manifest.rs` unit tests).

### Key rotation procedure

1. **Generate** a new keypair with the BSP firmware toolchain (e.g.
   `openssl genpkey -algorithm ED25519`); keep the private key in the signing
   keystore/HMS, never in the repo.
2. **Sign firmware** with the new private key, and encode the new public key
   as base64 (32 bytes).
3. **Update** `board-manifests/joshua.json`'s
   `caps.firmware_pubkey_ed25519` for the MCU, replacing the old value.
4. **Validate**: run the Configuration test suite — `validate_firmware_keys`
   must pass and the placeholder-rejection tests must still pass.
5. **Stage** the key via a signed image (the provisioning/update path refuses
   an unverifiable image when the trust anchor is absent).
6. **Retire** the old key from the signing toolchain only after the new image
   (signed with the new key) is confirmed on hardware.

## Consistency guarantees

- The polkit allowlist (`meta-webconfig/recipes-core/webconfig/files/50-webconfigd.polkit.rules`)
  must match `service_allowlist` exactly; a unit test (`manifest.rs`) fails if
  the two drift apart.
- Joshua's `mcu-c55` peripheral declares `settings_schema: "app:mcu-stm32"`; the
  schema must actually exist (guarded by a test).
