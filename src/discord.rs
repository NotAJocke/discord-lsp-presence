use crate::activity::{build_activity, build_details_and_state};
use crate::config::Config;
use crate::language::LanguageInfo;
use discord_presence::Client as DiscordClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::Client;
use tower_lsp::lsp_types::MessageType;

pub async fn update_presence(
    discord: &Arc<Mutex<DiscordClient>>,
    client: &Client,
    config: &Config,
    filename: &str,
    workspace: &str,
    language: &LanguageInfo,
    start_timestamp: Option<u64>,
    git_remote_url: Option<&str>,
) {
    let mut discord = discord.lock().await;
    let activity = build_activity(config, filename, workspace, language, start_timestamp, git_remote_url);

    match discord.set_activity(|_| activity) {
        Ok(_) => {
            let (details, state) = build_details_and_state(config, filename, workspace, language);
            client
                .log_message(
                    MessageType::INFO,
                    &format!("Set activity to: {} {}", details, state),
                )
                .await;
        }
        Err(e) => {
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
            client
                .log_message(MessageType::INFO, "Discord presence cleared.")
                .await;
        }
        Err(e) => {
            client
                .log_message(MessageType::ERROR, &format!("Failed to clear presence: {}", e))
                .await;
        }
    }
}
