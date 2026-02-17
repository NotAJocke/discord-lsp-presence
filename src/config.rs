use discord_presence::models::rich_presence::{Activity, ActivityAssets};
use serde::Deserialize;
use std::env::home_dir;
use std::path::PathBuf;

pub fn get_config_dir() -> Option<PathBuf> {
    home_dir().and_then(|home| Some(home.join(".config").join("discord-presence-lsp")))
}

pub fn ensure_config() -> std::result::Result<PathBuf, &'static str> {
    let Some(path) = get_config_dir() else {
        return Err("Couldn't get config directory");
    };

    if !path.exists() {
        if std::fs::create_dir_all(&path).is_err() {
            return Err("Couldn't create config dir");
        }
    }

    let path = path.join("config.toml");

    if !path.exists() {
        let default_config = r#"application_id = 1470506076574187745

[activity]
details = "Editing: {filename}"
state = "in {workspace}"
"#;
        if std::fs::write(&path, default_config).is_err() {
            return Err("Couldn't create config file");
        }
    }

    Ok(path)
}

#[derive(Deserialize, Debug, Clone)]
pub struct ActivityConfig {
    pub details: Option<String>,
    pub state: Option<String>,
    pub large_image_key: Option<String>,
    pub large_image_text: Option<String>,
    pub small_image_key: Option<String>,
    pub small_image_text: Option<String>,
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            details: Some("Editing: {filename}".to_string()),
            state: Some("in {workspace}".to_string()),
            large_image_key: None,
            large_image_text: None,
            small_image_key: None,
            small_image_text: None,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub application_id: u64,
    pub activity: Option<ActivityConfig>,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn build_activity(&self, filename: &str, workspace: &str) -> Activity {
        let activity_config = self.activity.clone().unwrap_or_default();

        let details = activity_config.details.map(|s| {
            s.replace("{filename}", filename)
                .replace("{workspace}", workspace)
        });

        let state = activity_config.state.map(|s| {
            s.replace("{filename}", filename)
                .replace("{workspace}", workspace)
        });

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

        let mut builder = Activity::new();

        if let Some(d) = details {
            builder = builder.details(d);
        }

        if let Some(s) = state {
            builder = builder.state(s);
        }

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
