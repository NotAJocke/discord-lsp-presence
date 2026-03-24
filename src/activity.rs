use crate::config::Config;
use crate::editor::EditorInfo;
use crate::language::LanguageInfo;
use discord_presence::models::rich_presence::{
    Activity, ActivityAssets, ActivityButton, ActivityTimestamps,
};

pub fn build_activity(
    config: &Config,
    filename: &str,
    workspace: &str,
    language: &LanguageInfo,
    start_timestamp: Option<u64>,
    git_remote_url: Option<&str>,
    editor: &EditorInfo,
) -> Activity {
    let activity_config = config.activity.clone().unwrap_or_default();
    let (details, state) =
        build_details_and_state(config, filename, workspace, language, &editor.name);

    let large_image_key = activity_config
        .editor_image_key
        .or(activity_config.large_image_key)
        .or_else(|| {
            if !editor.icon_key.is_empty() {
                Some(editor.icon_key.clone())
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty());

    let config_has_text = activity_config
        .editor_image_text
        .as_ref()
        .map(|s| !s.is_empty())
        .unwrap_or(false)
        || activity_config
            .large_image_text
            .as_ref()
            .map(|s| !s.is_empty())
            .unwrap_or(false);

    let large_image_text = if !config_has_text
        && !editor.icon_key.is_empty()
        && large_image_key.is_some()
    {
        Some(replace_placeholders(
            &editor.name,
            filename,
            workspace,
            language,
            &editor.name,
        ))
    } else if activity_config.editor_image_text.is_some()
        || activity_config.large_image_text.is_some()
    {
        activity_config
            .editor_image_text
            .or(activity_config.large_image_text)
            .filter(|s| !s.is_empty())
            .map(|text| replace_placeholders(&text, filename, workspace, language, &editor.name))
    } else {
        None
    };

    let small_image_key = if config.show_language_images() && !language.icon_key.is_empty() {
        Some(language.icon_key.clone())
    } else {
        None
    };
    let small_image_text = small_image_key.as_ref().map(|_| language.name.clone());

    let mut builder = if details.is_empty() && state.is_empty() {
        Activity::new()
    } else if details.is_empty() {
        Activity::new().state(state)
    } else if state.is_empty() {
        Activity::new().details(details)
    } else {
        Activity::new().details(details).state(state)
    };

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

    if let Some(remote_url) = git_remote_url {
        let button_label = activity_config
            .button_label
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "View Repository".to_string());
        builder = builder.append_buttons(|_| {
            ActivityButton::new()
                .label(button_label)
                .url(remote_url.to_string())
        });
    }

    builder
}

pub fn build_details_and_state(
    config: &Config,
    filename: &str,
    workspace: &str,
    language: &LanguageInfo,
    editor_name: &str,
) -> (String, String) {
    let activity_config = config.activity.clone().unwrap_or_default();

    let details_template = activity_config
        .details
        .unwrap_or_else(|| crate::config::DEFAULT_DETAILS.to_string());
    let state_template = activity_config
        .state
        .unwrap_or_else(|| crate::config::DEFAULT_STATE.to_string());

    let details = if details_template.is_empty() {
        String::new()
    } else {
        replace_placeholders(
            &details_template,
            filename,
            workspace,
            language,
            editor_name,
        )
    };
    let state = if state_template.is_empty() {
        String::new()
    } else {
        replace_placeholders(&state_template, filename, workspace, language, editor_name)
    };

    (details, state)
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
