use crate::activity::{build_activity, build_details_and_state};
use crate::config::{Config, TimeTracking};
use crate::editor::EditorInfo;
use crate::language::{detect_language, LanguageInfo};
use crate::server::AppState;
use discord_presence::Client as DiscordClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::Client;
use tower_lsp::lsp_types::MessageType;
use tracing::{error, info};

pub async fn update_presence(
    discord: &Mutex<DiscordClient>,
    client: &Client,
    config: &Config,
    filename: &str,
    workspace: &str,
    language: &LanguageInfo,
    start_timestamp: Option<u64>,
    git_remote_url: Option<&str>,
    editor: &EditorInfo,
) {
    let mut discord = discord.lock().await;
    let activity = build_activity(config, filename, workspace, language, start_timestamp, git_remote_url, editor);

    match discord.set_activity(|_| activity) {
        Ok(_) => {
            let (details, state) = build_details_and_state(config, filename, workspace, language, &editor.name);
            info!("Set activity to: {} {}", details, state);
            client
                .log_message(
                    MessageType::INFO,
                    &format!("Set activity to: {} {}", details, state),
                )
                .await;
        }
        Err(e) => {
            error!("Failed to set activity: {}", e);
            client
                .log_message(
                    MessageType::ERROR,
                    &format!("Failed to set activity: {}", e),
                )
                .await;
        }
    }
}

pub async fn clear_presence(
    discord: &Mutex<DiscordClient>,
    client: &Client,
) {
    let mut discord = discord.lock().await;

    match discord.clear_activity() {
        Ok(_) => {
            info!("Discord presence cleared.");
            client
                .log_message(MessageType::INFO, "Discord presence cleared.")
                .await;
        }
        Err(e) => {
            error!("Failed to clear presence: {}", e);
            client
                .log_message(MessageType::ERROR, &format!("Failed to clear presence: {}", e))
                .await;
        }
    }
}

pub async fn setup_discord_handlers(state: Arc<AppState>) {
    let state_for_ready = state.clone();
    state.discord.lock().await.on_ready(move |_ctx| {
        let discord = &state_for_ready.discord;
        let config = &state_for_ready.config;
        let enabled = &state_for_ready.enabled;
        let current_file = &state_for_ready.current_file;
        let current_workspace = &state_for_ready.current_workspace;
        let editor = &*state_for_ready.editor.blocking_lock();

        info!("Discord client ready");

        if !*enabled.blocking_lock() {
            info!("Discord presence is disabled, skipping initial presence.");
            return;
        }

        if let Some(file_state) = current_file.blocking_lock().as_ref() {
            info!("Setting initial presence for: {}", file_state.filename);
            let ts = match config.get_time_tracking() {
                TimeTracking::File => file_state.get_start_timestamp(),
                TimeTracking::Workspace => current_workspace
                    .blocking_lock()
                    .as_ref()
                    .map(|ws| ws.get_start_timestamp())
                    .unwrap_or_else(|| file_state.get_start_timestamp()),
            };
            let language = detect_language(&file_state.filename);
            let activity = build_activity(
                config,
                &file_state.filename,
                &file_state.workspace,
                &language,
                Some(ts),
                file_state.git_remote_url.as_deref(),
                editor,
            );
            if let Err(e) = discord.blocking_lock().set_activity(|_| activity) {
                error!("Failed to set initial activity: {}", e);
            }
        }
    })
    .persist();

    state.discord.lock().await.on_error(|_ctx| {
        error!("Discord connection error. Exiting.");
        std::process::exit(1);
    })
    .persist();
}
