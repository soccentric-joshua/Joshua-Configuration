//! Board manifest (spec §3 platform layer).
//!
//! The canonical manifest is a BSP-provided, read-only JSON file at
//! `/usr/share/webconfig/board-manifest.json`, optionally overridden by an
//! on-device technician edit at `/etc/webconfig/board.yaml` (see
//! `docs/board-manifest.md`), declaring available I/O
//! (CAN/serial/GPIO/I2C/MCU/FPGA/PRU) with Linux device names and human names.
//! Dashboard tiles and panels are declared as **data, not code**, so
//! board-specific quirks live in this layer rather than in code (spec §3).
//!
//! I/O rendering is 100% manifest-driven — zero hardcoded I/O (spec §4.9).

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("manifest parse error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("manifest YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),
    #[error("manifest io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("manifest not found at {0}")]
    NotFound(String),
    #[error("manifest firmware signing key invalid: {0}")]
    FirmwareKey(String),
}

/// Kind of peripheral a manifest entry describes (spec §4.9 / §4.5 tiles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IoKind {
    #[default]
    Serial,
    Can,
    Gpio,
    I2c,
    Spi,
    Mcu,
    Fpga,
    Pru,
    Camera,
    Display,
    Thermal,
    Net,
}

/// One manifest peripheral entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Peripheral {
    /// Stable id, e.g. "can0", "ttyS2", "gpio-led-status".
    pub id: String,
    /// Linux device name, e.g. "can0", "/dev/ttyS2", "gpiochip3 line 7".
    #[serde(default)]
    pub device: String,
    /// Human-readable name shown in the UI.
    #[serde(default)]
    pub human_name: String,
    pub kind: IoKind,
    /// Capabilities specific to the kind, kept as free-form JSON so new
    /// platform quirks (e.g. GPIO-switchable CAN termination) need no code
    /// change (spec §4.9).
    #[serde(default)]
    pub caps: serde_json::Map<String, serde_json::Value>,
}

impl Peripheral {
    /// Whether this peripheral advertises signed-firmware support.
    pub fn signed_firmware(&self) -> bool {
        self.caps
            .get("signed_fw")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// The Ed25519 verification key this peripheral advertises for its
    /// firmware, if any.
    pub fn firmware_pubkey(&self) -> Option<&str> {
        self.caps.get("firmware_pubkey_ed25519")?.as_str()
    }
}

/// A dashboard tile declared as data (spec §4.5 platform tiles via manifest
/// plugin). The frontend renders these generically.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DashboardTile {
    pub id: String,
    #[serde(default)]
    pub title: String,
    /// Metric ids this tile pulls from the monitoring endpoint.
    #[serde(default)]
    pub metrics: Vec<String>,
    /// Render hint: "sparkline" | "value" | "state" | "bar".
    #[serde(default)]
    pub render: String,
}

/// AP-provisioning-mode radio parameters (spec §4.9: zero hardcoded I/O — the
/// interface name, hw_mode, and channel used to be hardcoded constants in
/// `ApApplier`; a board with a different radio (5 GHz-only, a non-`wlan0`
/// interface name) had no way to override them). Falls back to the previous
/// hardcoded defaults when a manifest omits this section, so existing
/// manifests keep working unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiRadio {
    #[serde(default = "WifiRadio::default_interface")]
    pub interface: String,
    #[serde(default = "WifiRadio::default_hw_mode")]
    pub hw_mode: String,
    #[serde(default = "WifiRadio::default_channel")]
    pub channel: u8,
}

impl WifiRadio {
    fn default_interface() -> String {
        "wlan0".into()
    }
    fn default_hw_mode() -> String {
        "g".into()
    }
    fn default_channel() -> u8 {
        7
    }
}

impl Default for WifiRadio {
    fn default() -> Self {
        Self {
            interface: Self::default_interface(),
            hw_mode: Self::default_hw_mode(),
            channel: Self::default_channel(),
        }
    }
}

/// The board manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoardManifest {
    /// Platform id (always "joshua" — this repo is Joshua-only).
    pub platform: String,
    #[serde(default)]
    pub model: String,
    /// Human-readable board name.
    #[serde(default)]
    pub board_name: String,
    #[serde(default)]
    pub peripherals: Vec<Peripheral>,
    #[serde(default)]
    pub dashboard_tiles: Vec<DashboardTile>,
    /// Curated service allowlist for this platform image (spec §4.3 — never
    /// enumerate all units).
    #[serde(default)]
    pub service_allowlist: Vec<String>,
    /// Platform-specific capability flags read by the trust panel (spec §4.8).
    #[serde(default)]
    pub has_rtc: bool,
    #[serde(default)]
    pub has_tpm: bool,
    #[serde(default)]
    pub secure_boot: bool,
    /// Optional first-boot values for the live SQLite configuration.
    /// Subsequent edits are stored in SQLite and do not mutate this file.
    #[serde(default)]
    pub config_defaults: Option<crate::store::Config>,
    /// Per-platform default alert rule packs shipped as manifest data (spec
    /// §4.7: "Per-platform default rule packs shipped as schema files
    /// (Joshua: throttling; all: disk-full, watchdog-reset)"). The engine
    /// merges these with the user-configured rules at evaluation time; an
    /// operator can override or disable a default by id.
    #[serde(default)]
    pub default_alert_rules: Vec<crate::store::alerts::AlertRule>,
    /// AP-provisioning-mode radio parameters (spec §4.9). Absent in existing
    /// manifests → falls back to `WifiRadio::default()` (wlan0/g/7, the
    /// previous hardcoded behavior).
    #[serde(default)]
    pub wifi_radio: WifiRadio,
}

impl BoardManifest {
    /// Load the administrator-controlled YAML board definition from a file.
    pub fn load_yaml_path(path: &Path) -> Result<Self, ManifestError> {
        if !path.exists() {
            return Err(ManifestError::NotFound(path.display().to_string()));
        }
        let raw = std::fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&raw)?)
    }

    /// Load from `<share>/board-manifest.json`. Returns `NotFound` if absent.
    pub fn load(share_dir: &Path) -> Result<Self, ManifestError> {
        let path = share_dir.join("board-manifest.json");
        if !path.exists() {
            return Err(ManifestError::NotFound(path.display().to_string()));
        }
        let raw = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    /// Peripherals of a given kind.
    pub fn of_kind(&self, kind: IoKind) -> Vec<&Peripheral> {
        self.peripherals.iter().filter(|p| p.kind == kind).collect()
    }

    /// Whether a peripheral id is declared (used to reject config referencing
    /// hardware the board does not have).
    pub fn has(&self, id: &str) -> bool {
        self.peripherals.iter().any(|p| p.id == id)
    }

    /// Validate that every peripheral advertising signed firmware carries a
    /// real, well-formed Ed25519 verification key. Fails closed: a missing,
    /// placeholder, or malformed key is rejected so a signed-firmware claim
    /// with no usable trust anchor can never pass validation.
    pub fn validate_firmware_keys(&self) -> Result<(), ManifestError> {
        for p in self.peripherals.iter().filter(|p| p.signed_firmware()) {
            let key = p.firmware_pubkey().ok_or_else(|| {
                ManifestError::FirmwareKey(format!(
                    "{} advertises signed firmware but has no firmware_pubkey_ed25519",
                    p.id
                ))
            })?;
            validate_ed25519_key(&p.id, key)?;
        }
        Ok(())
    }
}

/// Validate an Ed25519 verification key (32-byte, base64-encoded). Rejects
/// placeholder text and malformed encodings so a manifest can never advertise
/// signed firmware without a usable trust anchor.
fn validate_ed25519_key(id: &str, key: &str) -> Result<(), ManifestError> {
    let lower = key.to_ascii_lowercase();
    if lower.contains("placeholder") || lower.contains("change-me") || lower.contains("todo") {
        return Err(ManifestError::FirmwareKey(format!(
            "{} has a placeholder firmware_pubkey_ed25519",
            id
        )));
    }
    use base64::Engine as _;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(key)
        .map_err(|_| {
            ManifestError::FirmwareKey(format!("{} has a malformed (non-base64) key", id))
        })?;
    if decoded.len() != 32 {
        return Err(ManifestError::FirmwareKey(format!(
            "{} key must decode to 32 bytes, got {}",
            id,
            decoded.len()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_joshua_manifest() {
        let raw = include_str!("../../../board-manifests/joshua.json");
        let m: BoardManifest = serde_json::from_str(raw).unwrap();
        assert_eq!(m.platform, "joshua");
        assert!(!m.peripherals.is_empty());
    }

    #[test]
    fn polkit_allowlist_matches_manifest_service_allowlist() {
        // Checkpoint 3.3: the polkit rule's allowed-unit list must exactly
        // match board-manifests/joshua.json's service_allowlist (the curated
        // base set: sshd, systemd-networkd, systemd-resolved, wpa_supplicant,
        // webconfigd) — this test fails loudly the moment the two drift apart
        // again, instead of relying on a one-time manual diff.
        let manifest: BoardManifest =
            serde_json::from_str(include_str!("../../../board-manifests/joshua.json")).unwrap();
        let polkit = include_str!(
            "../../../meta-webconfig/recipes-core/webconfig/files/50-webconfigd.polkit.rules"
        );
        let start = polkit
            .find("var allowed = [")
            .expect("polkit allowlist array present")
            + "var allowed = [".len();
        let end = start
            + polkit[start..]
                .find(']')
                .expect("polkit allowlist array closed");
        let polkit_allowed: std::collections::BTreeSet<String> = polkit[start..end]
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let manifest_allowed: std::collections::BTreeSet<String> =
            manifest.service_allowlist.iter().cloned().collect();
        assert_eq!(
            polkit_allowed, manifest_allowed,
            "polkit allowlist and board-manifests/joshua.json's service_allowlist have drifted apart"
        );
    }

    #[test]
    fn generic_yaml_manifest_parses_with_defaults() {
        let manifest: BoardManifest =
            serde_yaml::from_str(include_str!("../../../board-manifests/generic.yaml")).unwrap();
        assert_eq!(manifest.platform, "generic-linux");
        assert!(manifest.config_defaults.is_some());
        assert!(manifest.config_defaults.unwrap().system.time.ntp_enabled);
    }

    #[test]
    fn rejects_placeholder_firmware_key() {
        use base64::Engine as _;
        // A 32-byte, valid-base64 Ed25519 key for the "good" case.
        let valid = base64::engine::general_purpose::STANDARD.encode([0u8; 32]);
        let manifest: BoardManifest = serde_json::from_value(serde_json::json!({
            "platform": "joshua",
            "peripherals": [{
                "id": "mcu-c55",
                "kind": "mcu",
                "caps": { "signed_fw": true, "firmware_pubkey_ed25519": "PLACEHOLDER" }
            }]
        }))
        .unwrap();
        let err = manifest.validate_firmware_keys().unwrap_err();
        assert!(err.to_string().contains("placeholder"));
        // Same manifest with a real key must validate.
        let ok: BoardManifest = serde_json::from_value(serde_json::json!({
            "platform": "joshua",
            "peripherals": [{
                "id": "mcu-c55",
                "kind": "mcu",
                "caps": { "signed_fw": true, "firmware_pubkey_ed25519": valid }
            }]
        }))
        .unwrap();
        assert!(ok.validate_firmware_keys().is_ok());
    }

    #[test]
    fn rejects_malformed_firmware_key() {
        use base64::Engine as _;
        // 16 bytes encoded is a legal base64 string but not a valid Ed25519 key.
        let short = base64::engine::general_purpose::STANDARD.encode([0u8; 16]);
        let manifest: BoardManifest = serde_json::from_value(serde_json::json!({
            "platform": "joshua",
            "peripherals": [{
                "id": "mcu-c55",
                "kind": "mcu",
                "caps": { "signed_fw": true, "firmware_pubkey_ed25519": short }
            }]
        }))
        .unwrap();
        let err = manifest.validate_firmware_keys().unwrap_err();
        assert!(err.to_string().contains("32 bytes"));

        // Non-base64 garbage is rejected.
        let garbage: BoardManifest = serde_json::from_value(serde_json::json!({
            "platform": "joshua",
            "peripherals": [{
                "id": "mcu-c55",
                "kind": "mcu",
                "caps": { "signed_fw": true, "firmware_pubkey_ed25519": "!!not-base64!!" }
            }]
        }))
        .unwrap();
        assert!(garbage.validate_firmware_keys().is_err());
    }

    #[test]
    fn unsigned_peripheral_skips_key_validation() {
        let manifest: BoardManifest = serde_json::from_value(serde_json::json!({
            "platform": "joshua",
            "peripherals": [{
                "id": "cam0",
                "kind": "camera",
                "caps": { "signed_fw": false }
            }]
        }))
        .unwrap();
        assert!(manifest.validate_firmware_keys().is_ok());
    }
}
