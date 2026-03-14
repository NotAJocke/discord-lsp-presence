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
        });

    let config_has_text =
        activity_config.editor_image_text.is_some() || activity_config.large_image_text.is_some();

    let large_image_text = if !config_has_text && !editor.icon_key.is_empty() {
        Some(editor.name.clone())
    } else {
        activity_config
            .editor_image_text
            .or(activity_config.large_image_text)
            .map(|text| replace_placeholders(&text, filename, workspace, language, &editor.name))
    };

    let small_image_key = if config.show_language_images() && !language.icon_key.is_empty() {
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

    if let Some(remote_url) = git_remote_url {
        let button_label = activity_config
            .button_label
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

    let details = replace_placeholders(
        &details_template,
        filename,
        workspace,
        language,
        editor_name,
    );
    let state = replace_placeholders(&state_template, filename, workspace, language, editor_name);

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
