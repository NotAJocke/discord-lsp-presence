use discord_presence::models::rich_presence::{Activity, ActivityAssets};
use serde::Deserialize;
use std::env::home_dir;
use std::path::PathBuf;

pub fn get_config_dir() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".config").join("discord-presence-lsp"))
}

const DEFAULT_APPLICATION_ID: u64 = 1470506076574187745;
const DEFAULT_DETAILS: &str = "Editing: {filename}";
const DEFAULT_STATE: &str = "in {workspace}";

pub fn get_config_path() -> Option<PathBuf> {
    get_config_dir().map(|dir| dir.join("config.toml"))
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ActivityConfig {
    pub details: Option<String>,
    pub state: Option<String>,
    pub large_image_key: Option<String>,
    pub large_image_text: Option<String>,
    pub small_image_key: Option<String>,
    pub small_image_text: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub application_id: Option<u64>,
    #[serde(default)]
    pub activity: Option<ActivityConfig>,
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

    pub fn build_activity(&self, filename: &str, workspace: &str) -> Activity {
        let activity_config = self.activity.clone().unwrap_or_default();

        let details = activity_config
            .details
            .unwrap_or_else(|| DEFAULT_DETAILS.to_string())
            .replace("{filename}", filename)
            .replace("{workspace}", workspace);

        let state = activity_config
            .state
            .unwrap_or_else(|| DEFAULT_STATE.to_string())
            .replace("{filename}", filename)
            .replace("{workspace}", workspace);

        let large_image = activity_config.large_image_key.map(|key| {
            let text = activity_config.large_image_text.map(|t| {
                t.replace("{filename}", filename)
                    .replace("{workspace}", workspace)
            });
            (key, text)
        });

        let small_image = activity_config.small_image_key.map(|key| {
            let text = activity_config.small_image_text.map(|t| {
                t.replace("{filename}", filename)
                    .replace("{workspace}", workspace)
            });
            (key, text)
        });

        let mut builder = Activity::new().details(details).state(state);

        if large_image.is_some() || small_image.is_some() {
            builder = builder.assets(|_| {
                let mut assets = ActivityAssets::new();
                if let Some((key, text)) = large_image {
                    assets = assets.large_image(key);
                    if let Some(t) = text {
                        assets = assets.large_text(t);
                    }
                }
                if let Some((key, text)) = small_image {
                    assets = assets.small_image(key);
                    if let Some(t) = text {
                        assets = assets.small_text(t);
                    }
                }
                assets
            });
        }

        builder
    }
}
