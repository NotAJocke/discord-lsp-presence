use discord_presence::Client as DiscordClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use url::Url;

mod config;
mod discord;
mod language;
mod state;
mod workspace;

use config::{Config, TimeTracking};
use language::detect_language;
use state::{FileState, WorkspaceState};
use workspace::{detect_workspace_name, get_filename_from_uri};

struct Backend {
    client: Client,
    discord: Arc<Mutex<DiscordClient>>,
    config: Arc<Config>,
    current_file: Arc<Mutex<Option<FileState>>>,
    current_workspace: Arc<Mutex<Option<WorkspaceState>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "discord-lsp-presence".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        let mut discord = self.discord.lock().await;
        discord.start();

        self.client
            .log_message(MessageType::INFO, "Discord client started.")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "Opened file")
            .await;

        self.handle_file_event(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "File changed")
            .await;

        self.handle_file_event(&params.text_document.uri).await;
    }
}

impl Backend {
    async fn handle_file_event(&self, uri: &Url) {
        let filename = get_filename_from_uri(uri);
        let workspace_name = detect_workspace_name(uri);

        if let Some(filename) = filename {
            let workspace = workspace_name.unwrap_or_else(|| "unknown workspace".to_string());
            let language = detect_language(&filename);

            let start_timestamp = match self.config.get_time_tracking() {
                TimeTracking::File => {
                    let state = FileState::new(filename.clone(), workspace.clone());
                    let ts = state.get_start_timestamp();
                    *self.current_file.lock().await = Some(state);
                    Some(ts)
                }
                TimeTracking::Workspace => {
                    let mut current_workspace = self.current_workspace.lock().await;
                    let ts = match current_workspace.as_ref() {
                        Some(ws) if ws.workspace == workspace => ws.get_start_timestamp(),
                        _ => {
                            let new_ws = WorkspaceState::new(workspace.clone());
                            let ts = new_ws.get_start_timestamp();
                            *current_workspace = Some(new_ws);
                            ts
                        }
                    };
                    Some(ts)
                }
            };

            if DiscordClient::is_ready() {
                discord::update_presence(
                    &self.discord,
                    &self.client,
                    &self.config,
                    &filename,
                    &workspace,
                    &language,
                    start_timestamp,
                )
                .await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Arc::new(Config::load());

    let discord = Arc::new(Mutex::new(DiscordClient::new(config.get_application_id())));

    let current_file: Arc<Mutex<Option<FileState>>> = Arc::new(Mutex::new(None));
    let current_workspace: Arc<Mutex<Option<WorkspaceState>>> = Arc::new(Mutex::new(None));
    let current_file_for_ready = Arc::clone(&current_file);
    let current_workspace_for_ready = Arc::clone(&current_workspace);
    let discord_for_ready = Arc::clone(&discord);
    let config_for_ready = Arc::clone(&config);

    {
        let drpc = discord.lock().await;

        drpc.on_ready(move |_ctx| {
            eprintln!("Discord client ready");

            if let Some(file_state) = current_file_for_ready.blocking_lock().as_ref() {
                eprintln!("Setting initial presence for: {}", file_state.filename);
                let ts = match config_for_ready.get_time_tracking() {
                    TimeTracking::File => file_state.get_start_timestamp(),
                    TimeTracking::Workspace => current_workspace_for_ready
                        .blocking_lock()
                        .as_ref()
                        .map(|ws| ws.get_start_timestamp())
                        .unwrap_or_else(|| file_state.get_start_timestamp()),
                };
                let language = detect_language(&file_state.filename);
                let activity = config_for_ready.build_activity(
                    &file_state.filename,
                    &file_state.workspace,
                    &language,
                    Some(ts),
                );
                let mut client = discord_for_ready.blocking_lock();
                if let Err(e) = client.set_activity(|_| activity) {
                    eprintln!("Failed to set initial activity: {}", e);
                }
            }
        })
        .persist();

        drpc.on_error(|_ctx| {
            eprintln!("Discord connection error. Exiting.");
            std::process::exit(1);
        })
        .persist();
    }

    let current_file_clone = Arc::clone(&current_file);
    let current_workspace_clone = Arc::clone(&current_workspace);
    let config_clone = Arc::clone(&config);
    let (service, socket) = LspService::new(move |client| Backend {
        client,
        discord: Arc::clone(&discord),
        config: Arc::clone(&config_clone),
        current_file: Arc::clone(&current_file_clone),
        current_workspace: Arc::clone(&current_workspace_clone),
    });

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout, socket).serve(service).await;
}
