use crate::config::Config;
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
) {
    let mut discord = discord.lock().await;
    let activity = config.build_activity(filename, workspace);

    match discord.set_activity(|_| activity) {
        Ok(_) => {
            let details = config
                .activity
                .as_ref()
                .and_then(|a| a.details.as_ref())
                .map(|d| d.replace("{filename}", filename).replace("{workspace}", workspace))
                .unwrap_or_else(|| format!("Editing: {}", filename));
            let state = config
                .activity
                .as_ref()
                .and_then(|a| a.state.as_ref())
                .map(|s| s.replace("{filename}", filename).replace("{workspace}", workspace))
                .unwrap_or_else(|| format!("in {}", workspace));
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
