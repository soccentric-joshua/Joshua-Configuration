# WebConfig — REST API Reference

All routes below are mounted under **`/api/v1`**. The daemon is **TLS-only**;
plain HTTP just redirects to HTTPS. Every mutating route requires an
authenticated session and a valid **`X-CSRF-Token`** header. A missing or
invalid CSRF token returns **400 BadRequest** (not 403).

> The frontend wires handlers to these contracts in `frontend/core/api.js` and
> `frontend/pages/*.js`. When changing the engine, match these shapes exactly.

## Authentication

| Method | Path | Purpose |
|---|---|---|
| POST | `/auth/first-boot` | Create the initial admin (first boot). |
| POST | `/auth/login` | Log in; establishes the session. |
| POST | `/auth/logout` | Destroy the session. |
| GET  | `/auth/whoami` | Current user + role. |

## Meta (spec §3)

| Method | Path | Purpose |
|---|---|---|
| GET | `/manifest` | Board manifest (platform data). |
| GET | `/schemas/:section` | Schema descriptor for a section. |
| GET | `/schemas` | All schema descriptors. |

## System (§4.1)

| Method | Path | Purpose |
|---|---|---|
| GET/PUT | `/system` | Get / set system config. |
| POST | `/system/apply` | Apply appliers to the live system. |
| GET | `/system/info` | Returns per-interface `macs` plus `uptime_secs`. |
| POST | `/system/reboot` | Reboot the device. |
| POST | `/system/shutdown` | Power off. |
| POST | `/system/factory-reset` | Full factory reset (wipes certs + alert history). |
| POST | `/system/root-password` | Set the root password. |
| GET/POST | `/system/users` | List / create users. |
| DELETE | `/system/users/:name` | Delete a user. |
| PUT | `/system/users/:name/role` | Set a user's role. |
| PUT | `/system/users/:name/password` | Change a user's password. |
| GET/POST | `/system/groups` | List / create groups. |
| DELETE | `/system/groups/:name` | Delete a group. |
| PUT/DELETE | `/system/groups/:group/members/:user` | Add / remove group member. |

## Networking (§4.2)

| Method | Path | Purpose |
|---|---|---|
| GET/PUT | `/network` | Get / set network config (provisional). |
| POST | `/network/apply` | Apply + arm the confirm/rollback window. |
| POST | `/network/confirm` | Explicitly confirm a provisional change. |
| POST | `/network/rollback` | Manually roll back a provisional change. |
| GET | `/network/scan` | Wi-Fi scan results. |
| POST | `/network/connectivity-test` | Test connectivity to an upstream. |

## Services & Apps (§4.3)

| Method | Path | Purpose |
|---|---|---|
| GET | `/services` | Curated service list (manifest allowlist). |
| GET | `/services/health` | Service health strip. |
| POST | `/services/:unit/start` | Start a unit. |
| POST | `/services/:unit/stop` | Stop a unit. |
| POST | `/services/:unit/restart` | Restart a unit. |
| PUT | `/services/:unit/enabled` | Set enable/disable. |
| GET | `/services/:unit/journal` | Journal for a unit. |
| GET/PUT | `/services/mqtt` | MQTT broker config. |
| POST | `/services/mqtt/test` | Test MQTT connectivity. |
| GET/PUT | `/apps/:id/settings` | Get / set app settings (validated by schema stem). |

## Updates (§4.4)

| Method | Path | Purpose |
|---|---|---|
| GET | `/updates/slots` | Active/inactive slot status. |
| POST | `/updates/install` | Install a bundle (JSON `bundle_path` reference). |
| POST | `/updates/upload` | Stream an OS bundle to disk (up to 512 MB). |
| POST | `/updates/confirm` | Confirm the update as good (marks + audits). |
| POST | `/updates/reboot-to-apply` | Reboot into the new slot. |
| GET/PUT | `/updates/ota` | OTA update configuration. |
| GET | `/updates/ota/check` | Check for OTA updates. |
| GET/PUT | `/updates/backup/config` | Backup/restore policy. |
| GET | `/updates/backup/check` | Backup health check. |
| GET/POST | `/updates/backup` | Download / create (optionally encrypted) backup. |
| POST | `/updates/restore` | Restore from a backup (JSON or encrypted envelope). |
| GET | `/updates/diagnostics` | Diagnostics bundle (journal + network state). |
| POST | `/updates/factory-reset` | Factory reset via the update path. |

## Monitoring (§4.5)

| Method | Path | Purpose |
|---|---|---|
| GET | `/monitoring/vitals` | Live vitals. |
| GET | `/monitoring/emmc/:dev` | eMMC wear stats. |
| GET | `/monitoring/tiles` | Dashboard tiles (from manifest data). |
| GET/PUT | `/monitoring/telemetry` | Telemetry publish config. |

## Logs (§4.6)

| Method | Path | Purpose |
|---|---|---|
| GET | `/logs` | Read the journal. |
| GET | `/logs/sources` | Available log sources. |
| GET/PUT | `/logs/config` | Log storage config. |
| GET | `/logs/audit` | Append-only audit log. |
| GET | `/logs/download` | Download logs. |

## Alerts (§4.7)

| Method | Path | Purpose |
|---|---|---|
| GET/PUT | `/alerts` | Alert rules + push targets. |
| GET | `/alerts/history` | Alert history. |

## Security (§4.8)

| Method | Path | Purpose |
|---|---|---|
| GET/PUT | `/security` | Security config. |
| POST | `/security/apply` | Apply persisted security appliers. |
| GET | `/security/wireguard/pubkey` | WireGuard public key. |
| POST | `/security/csr` | Generate a CSR. |
| POST | `/security/tls` | Upload a TLS identity (cert + key PEM). |
| GET | `/security/exposure` | Exposure surface report. |
| GET | `/security/trust` | Trust panel status (TPM / secure-boot / RTC). |
| GET/POST | `/security/certs` | List / upload a CA cert. |
| DELETE | `/security/certs/:id` | Delete a cert. |

## I/O (§4.9)

| Method | Path | Purpose |
|---|---|---|
| GET/PUT | `/io` | I/O config (with `If-Match` stale-guard on PUT). |
| GET | `/io/usb` | USB devices. |
| GET | `/io/pci` | PCI devices. |
| GET | `/io/i2c` | I2C devices. |
| GET | `/io/spi` | SPI devices. |
| POST | `/io/can/apply` | Apply CAN config. |
| POST | `/io/serial/apply` | Apply serial config. |
| POST | `/io/gpio/:name/toggle` | Toggle a GPIO. |
| POST | `/io/i2c/scan` | Scan an I2C bus. |
| GET | `/io/mcu` | MCU status. |
| POST | `/io/mcu/firmware` | Verify/upload MCU firmware (Ed25519 signature). |
| GET | `/io/platform-panel` | Manifest-driven I/O panel. |

## SSE (§3 / §4.6)

| Method | Path | Purpose |
|---|---|---|
| GET | `/events` | Server-sent events stream (vitals etc.), auth required, concurrency-capped. |

---

## Common behavior

- **Request-body limits** are enforced per route (TLS upload 64 KB, cert 64 KB,
  backup secret 4 KB, restore 8 MB, install JSON 4 KB, bundle upload 512 MB).
  Oversized bodies are rejected before full buffering.
- **CSRF:** all mutating (POST/PUT/DELETE) routes require `X-CSRF-Token`;
  missing/invalid → **400 BadRequest**.
- **Roles:** `viewer` cannot mutate admin-only routes; `operator` can mutate
  apps but not admin-only routes; `admin` is unrestricted.
- **Errors:** handled via `ApiError`; `NotFound` is a **unit** variant (no
  message payload).
