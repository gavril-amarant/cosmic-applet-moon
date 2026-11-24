use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, Config, CosmicConfigEntry};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CosmicConfigEntry)]
#[version = 1]
pub struct MoonConfig {
    /// `None` = automatic OS timezone detection; `Some("Europe/Bucharest")` = manual override.
    pub manual_tz: Option<String>,
        /// Stores e.g. `"Full Moon:2026-11-24T14:53:00+00:00"` so each event only notifies once.
    pub last_notified_event: Option<String>,
    #[serde(default = "default_notifications_enabled")]
    pub notifications_enabled: bool,
    #[serde(default = "default_notify_days")]
    pub notify_days: u32,
    #[serde(default = "default_notify_hours")]
    pub notify_hours: u32,  
}
fn default_notifications_enabled() -> bool { true }
fn default_notify_days() -> u32 { 1 }
fn default_notify_hours() -> u32 { 0 }

impl Default for MoonConfig {
    fn default() -> Self {
        Self {
            manual_tz: None,
            last_notified_event: None,
            notifications_enabled: true,
            notify_days: 1,
            notify_hours: 0,
        }
    }
}

impl MoonConfig {
    fn file_path(app_id: &str) -> Option<std::path::PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config"))
            })?;
        Some(base.join("cosmic").join(app_id).join("v1").join("settings.ron"))
    }

    pub fn load(app_id: &str) -> (Option<Config>, Self) {
        let config = Self::file_path(app_id)
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| ron::from_str(&s).ok())
            .unwrap_or_default();
        (None, config)
    }

    pub fn persist(&self, app_id: &str) {
        if let Some(path) = Self::file_path(app_id) {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(serialized) = ron::to_string(self) {
                let _ = std::fs::write(path, serialized);
            }
        }
    }

    pub fn set_notified(&mut self, _handler: Option<&Config>, event_key: String) {
        self.last_notified_event = Some(event_key);
        self.persist(crate::moon_applet::APP_ID);
    }

    pub fn save(&mut self, _handler: Option<&Config>) {
        self.persist(crate::moon_applet::APP_ID);
    }
}
