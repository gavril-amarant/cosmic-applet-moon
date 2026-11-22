use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, Config, CosmicConfigEntry};
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u64 = 1;

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
    /// Loads the saved config from `~/.config/cosmic/<APP_ID>/v1`, falling back to defaults.
    pub fn load(app_id: &str) -> (Option<Config>, Self) {
        match Config::new(app_id, CONFIG_VERSION) {
            Ok(handler) => {
                let config = match MoonConfig::get_entry(&handler) {
                    Ok(entry) => entry,
                    Err((_errors, partial)) => partial,
                };
                (Some(handler), config)
            }
            Err(_) => (None, MoonConfig::default()),
        }
    }

    pub fn set_notified(&mut self, handler: Option<&Config>, event_key: String) {
        self.last_notified_event = Some(event_key.clone());
        if let Some(h) = handler {
            let _ = self.set_last_notified_event(h, Some(event_key));
        }
    }


    /// Persists the updated `manual_tz` value to disk.
    pub fn save(&mut self, handler: Option<&Config>) {
        if let Some(h) = handler {
            let _ = self.set_manual_tz(h, self.manual_tz.clone());
        }
    }
}
