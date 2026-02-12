use discord_presence::Client as DiscordClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::Client;
use tower_lsp::lsp_types::MessageType;

pub async fn update_presence(
    discord: &Arc<Mutex<DiscordClient>>,
    client: &Client,
    filename: &str,
    workspace: &str,
) {
    let mut discord = discord.lock().await;
    let state = format!("in {}", workspace);

    match discord.set_activity(|a| {
        a.details(format!("Editing: {}", filename))
            .state(&state)
            .timestamps(|t| t.start(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()))
    }) {
        Ok(_) => {
            client
                .log_message(
                    MessageType::INFO,
                    &format!("Set activity to: Editing: {} {}", filename, state),
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
