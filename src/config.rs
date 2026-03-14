use serde::Deserialize;
use std::env::home_dir;
use std::path::PathBuf;

pub fn get_config_dir() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".config").join("discord-presence-lsp"))
}

pub const DEFAULT_APPLICATION_ID: u64 = 1470506076574187745;
pub const DEFAULT_DETAILS: &str = "Editing: {filename}";
pub const DEFAULT_STATE: &str = "in {workspace}";

pub fn get_config_path() -> Option<PathBuf> {
    get_config_dir().map(|dir| dir.join("config.toml"))
}

#[derive(Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TimeTracking {
    #[default]
    File,
    Workspace,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ActivityConfig {
    pub details: Option<String>,
    pub state: Option<String>,
    pub large_image_key: Option<String>,
    pub large_image_text: Option<String>,
    pub editor_image_key: Option<String>,
    pub editor_image_text: Option<String>,
    pub language_images: Option<bool>,
    pub button_label: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub application_id: Option<u64>,
    #[serde(default)]
    pub activity: Option<ActivityConfig>,
    #[serde(default)]
    pub time_tracking: Option<TimeTracking>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

impl Config {
    pub fn load() -> Self {
        let Some(path) = get_config_path() else {
            return Self::default();
        };

        if !path.exists() {
            return Self::default();
        }

        let config_str = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to read config file: {}. Using defaults.",
                    e
                );
                return Self::default();
            }
        };

        match toml::from_str(&config_str) {
            Ok(config) => config,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to parse config file: {}. Using defaults.",
                    e
                );
                Self::default()
            }
        }
    }

    pub fn get_application_id(&self) -> u64 {
        self.application_id.unwrap_or(DEFAULT_APPLICATION_ID)
    }

    pub fn get_time_tracking(&self) -> TimeTracking {
        self.time_tracking.unwrap_or_default()
    }

    pub fn show_language_images(&self) -> bool {
        self.activity
            .as_ref()
            .and_then(|a| a.language_images)
            .unwrap_or(true)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }
}
