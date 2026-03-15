mod activity;
mod config;
mod discord;
mod editor;
mod language;
mod logging;
mod server;
mod state;
mod workspace;

use config::Config;
use discord_presence::Client as DiscordClient;
use editor::EditorInfo;
use server::{AppState, Backend};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    logging::init_logging();

    let config = Config::load();
    let enabled = config.is_enabled();
    let state = Arc::new(AppState {
        discord: Mutex::new(DiscordClient::new(config.get_application_id())),
        config,
        editor: Mutex::new(EditorInfo::default()),
        current_file: Mutex::new(None),
        current_workspace: Mutex::new(None),
        enabled: Mutex::new(enabled),
    });

    discord::setup_discord_handlers(state.clone()).await;

    let (service, socket) = tower_lsp::LspService::new(move |client| Backend { client, state });

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}
