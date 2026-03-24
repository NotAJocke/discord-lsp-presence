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

pub fn load_project_config(path: &PathBuf) -> Option<Config> {
    let config_str = std::fs::read_to_string(path).ok()?;
    toml::from_str(&config_str).ok()
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

#[derive(Deserialize, Debug, Clone, Default)]
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

    pub fn merge_with(&self, project_config: &Config) -> Config {
        let merged_activity = match (&self.activity, &project_config.activity) {
            (Some(global), Some(project)) => {
                let mut merged = global.clone();
                if project.details.is_some() {
                    merged.details = project.details.clone();
                }
                if project.state.is_some() {
                    merged.state = project.state.clone();
                }
                if project.large_image_key.is_some() {
                    merged.large_image_key = project.large_image_key.clone();
                }
                if project.large_image_text.is_some() {
                    merged.large_image_text = project.large_image_text.clone();
                }
                if project.editor_image_key.is_some() {
                    merged.editor_image_key = project.editor_image_key.clone();
                }
                if project.editor_image_text.is_some() {
                    merged.editor_image_text = project.editor_image_text.clone();
                }
                if project.language_images.is_some() {
                    merged.language_images = project.language_images.clone();
                }
                if project.button_label.is_some() {
                    merged.button_label = project.button_label.clone();
                }
                Some(merged)
            }
            (Some(global), None) => Some(global.clone()),
            (None, Some(project)) => Some(project.clone()),
            (None, None) => None,
        };

        Config {
            application_id: project_config.application_id.or(self.application_id),
            enabled: project_config.enabled.or(self.enabled),
            time_tracking: project_config.time_tracking.or(self.time_tracking),
            activity: merged_activity,
        }
    }
}
