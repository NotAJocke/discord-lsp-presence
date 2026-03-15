use crate::activity::{build_activity, build_details_and_state};
use crate::config::{Config, TimeTracking};
use crate::editor::EditorInfo;
use crate::language::{detect_language, LanguageInfo};
use crate::state::{FileState, WorkspaceState};
use discord_presence::Client as DiscordClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::Client;
use tower_lsp::lsp_types::MessageType;
use tracing::{error, info};

pub async fn update_presence(
    discord: &Arc<Mutex<DiscordClient>>,
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
    discord: &Arc<Mutex<DiscordClient>>,
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

pub async fn setup_discord_handlers(
    discord: &Arc<Mutex<DiscordClient>>,
    enabled: Arc<Mutex<bool>>,
    current_file: Arc<Mutex<Option<FileState>>>,
    current_workspace: Arc<Mutex<Option<WorkspaceState>>>,
    config: Arc<Config>,
    editor: Arc<Mutex<EditorInfo>>,
) {
    let discord = Arc::clone(discord);
    let current_file_for_ready = Arc::clone(&current_file);
    let current_workspace_for_ready = Arc::clone(&current_workspace);
    let config_for_ready = Arc::clone(&config);
    let enabled_for_ready = Arc::clone(&enabled);
    let editor_for_ready = Arc::clone(&editor);

    let discord_for_ready = discord.clone();
    discord.lock().await.on_ready(move |_ctx| {
        info!("Discord client ready");

        if !*enabled_for_ready.blocking_lock() {
            info!("Discord presence is disabled, skipping initial presence.");
            return;
        }

        if let Some(file_state) = current_file_for_ready.blocking_lock().as_ref() {
            info!("Setting initial presence for: {}", file_state.filename);
            let ts = match config_for_ready.get_time_tracking() {
                TimeTracking::File => file_state.get_start_timestamp(),
                TimeTracking::Workspace => current_workspace_for_ready
                    .blocking_lock()
                    .as_ref()
                    .map(|ws| ws.get_start_timestamp())
                    .unwrap_or_else(|| file_state.get_start_timestamp()),
            };
            let language = detect_language(&file_state.filename);
            let editor = editor_for_ready.blocking_lock().clone();
            let activity = build_activity(
                &config_for_ready,
                &file_state.filename,
                &file_state.workspace,
                &language,
                Some(ts),
                file_state.git_remote_url.as_deref(),
                &editor,
            );
            if let Err(e) = discord_for_ready.blocking_lock().set_activity(|_| activity) {
                error!("Failed to set initial activity: {}", e);
            }
        }
    })
    .persist();

    let _discord_for_error = discord.clone();
    discord.lock().await.on_error(move |_ctx| {
        error!("Discord connection error. Exiting.");
        std::process::exit(1);
    })
    .persist();
}
