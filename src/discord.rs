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
) {
    let mut discord = discord.lock().await;
    let activity = config.build_activity(filename, workspace, language, start_timestamp);

    match discord.set_activity(|_| activity) {
        Ok(_) => {
            let (details, state) = config.build_details_and_state(filename, workspace, language);
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
