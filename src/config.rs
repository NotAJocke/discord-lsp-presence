use discord_presence::models::rich_presence::{Activity, ActivityAssets, ActivityTimestamps};
use serde::Deserialize;
use std::env::home_dir;
use std::path::PathBuf;

use crate::language::LanguageInfo;

pub fn get_config_dir() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".config").join("discord-presence-lsp"))
}

const DEFAULT_APPLICATION_ID: u64 = 1470506076574187745;
const DEFAULT_DETAILS: &str = "Editing: {filename}";
const DEFAULT_STATE: &str = "in {workspace}";
const DEFAULT_EDITOR_NAME: &str = "Helix";

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
    pub editor_name: Option<String>,
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

    pub fn get_editor_name(&self) -> &str {
        self.editor_name.as_deref().unwrap_or(DEFAULT_EDITOR_NAME)
    }

    pub fn show_language_images(&self) -> bool {
        self.activity
            .as_ref()
            .and_then(|a| a.language_images)
            .unwrap_or(true)
    }

    pub fn build_details_and_state(
        &self,
        filename: &str,
        workspace: &str,
        language: &LanguageInfo,
    ) -> (String, String) {
        let activity_config = self.activity.clone().unwrap_or_default();
        let editor_name = self.get_editor_name();
        let details_template = activity_config
            .details
            .unwrap_or_else(|| DEFAULT_DETAILS.to_string());
        let state_template = activity_config
            .state
            .unwrap_or_else(|| DEFAULT_STATE.to_string());

        let details = replace_placeholders(
            &details_template,
            filename,
            workspace,
            language,
            editor_name,
        );
        let state =
            replace_placeholders(&state_template, filename, workspace, language, editor_name);
        (details, state)
    }

    pub fn build_activity(
        &self,
        filename: &str,
        workspace: &str,
        language: &LanguageInfo,
        start_timestamp: Option<u64>,
    ) -> Activity {
        let activity_config = self.activity.clone().unwrap_or_default();
        let editor_name = self.get_editor_name();
        let (details, state) = self.build_details_and_state(filename, workspace, language);

        let large_image_key = activity_config
            .editor_image_key
            .or(activity_config.large_image_key);
        let large_image_text = activity_config
            .editor_image_text
            .or(activity_config.large_image_text)
            .map(|text| replace_placeholders(&text, filename, workspace, language, editor_name));

        let small_image_key = if self.show_language_images() && !language.icon_key.is_empty() {
            Some(language.icon_key.clone())
        } else {
            None
        };
        let small_image_text = small_image_key.as_ref().map(|_| language.name.clone());

        let mut builder = Activity::new().details(details).state(state);

        if let Some(ts) = start_timestamp {
            builder = builder.timestamps(|_| ActivityTimestamps::new().start(ts));
        }

        if large_image_key.is_some() || small_image_key.is_some() {
            builder = builder.assets(|_| {
                let mut assets = ActivityAssets::new();
                if let Some(key) = large_image_key {
                    assets = assets.large_image(key);
                    if let Some(t) = large_image_text {
                        assets = assets.large_text(t);
                    }
                }
                if let Some(key) = small_image_key {
                    assets = assets.small_image(key);
                    if let Some(t) = small_image_text {
                        assets = assets.small_text(t);
                    }
                }
                assets
            });
        }

        builder
    }
}

fn replace_placeholders(
    text: &str,
    filename: &str,
    workspace: &str,
    language: &LanguageInfo,
    editor: &str,
) -> String {
    text.replace("{filename}", filename)
        .replace("{workspace}", workspace)
        .replace("{language}", &language.name)
        .replace("{editor}", editor)
}
