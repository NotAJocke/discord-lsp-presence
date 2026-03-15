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
use server::Backend;
use state::{FileState, WorkspaceState};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    logging::init_logging();

    let config = Arc::new(Config::load());
    let discord = Arc::new(Mutex::new(DiscordClient::new(config.get_application_id())));

    let current_file: Arc<Mutex<Option<FileState>>> = Arc::new(Mutex::new(None));
    let current_workspace: Arc<Mutex<Option<WorkspaceState>>> = Arc::new(Mutex::new(None));
    let enabled: Arc<Mutex<bool>> = Arc::new(Mutex::new(config.is_enabled()));
    let editor: Arc<Mutex<editor::EditorInfo>> = Arc::new(Mutex::new(editor::EditorInfo::default()));

    discord::setup_discord_handlers(
        &discord,
        Arc::clone(&enabled),
        Arc::clone(&current_file),
        Arc::clone(&current_workspace),
        Arc::clone(&config),
        Arc::clone(&editor),
    )
    .await;

    let current_file_clone = Arc::clone(&current_file);
    let current_workspace_clone = Arc::clone(&current_workspace);
    let config_clone = Arc::clone(&config);
    let enabled_clone = Arc::clone(&enabled);
    let editor_clone = Arc::clone(&editor);
    let (service, socket) = tower_lsp::LspService::new(move |client| Backend {
        client,
        discord: Arc::clone(&discord),
        config: Arc::clone(&config_clone),
        editor: Arc::clone(&editor_clone),
        current_file: Arc::clone(&current_file_clone),
        current_workspace: Arc::clone(&current_workspace_clone),
        enabled: Arc::clone(&enabled_clone),
    });

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}
