use discord_presence::Client as DiscordClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use url::Url;

mod config;
mod discord;
mod state;
mod workspace;

use state::FileState;
use workspace::{detect_workspace_name, get_filename_from_uri};

const DISCORD_APPLICATION_ID: u64 = 1470506076574187745;

struct Backend {
    client: Client,
    discord: Arc<Mutex<DiscordClient>>,
    current_file: Arc<Mutex<Option<FileState>>>,
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
            let state = FileState::new(filename.clone(), workspace.clone());
            *self.current_file.lock().await = Some(state);

            if DiscordClient::is_ready() {
                discord::update_presence(&self.discord, &self.client, &filename, &workspace).await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let maybe_config = config::ensure_config();

    if let Err(e) = maybe_config {
        eprintln!("{e}");
    };

    let config_str = std::fs::read_to_string(maybe_config.unwrap()).unwrap();
    let _config: config::Config = toml::from_str(&config_str).unwrap();

    let discord = Arc::new(Mutex::new(DiscordClient::new(DISCORD_APPLICATION_ID)));

    let current_file: Arc<Mutex<Option<FileState>>> = Arc::new(Mutex::new(None));
    let current_file_for_ready = Arc::clone(&current_file);
    let discord_for_ready = Arc::clone(&discord);

    {
        let drpc = discord.lock().await;

        drpc.on_ready(move |_ctx| {
            eprintln!("Discord client ready");
            
            if let Some(file_state) = current_file_for_ready.blocking_lock().as_ref() {
                eprintln!("Setting initial presence for: {}", file_state.filename);
                let state = format!("in {}", file_state.workspace);
                let mut client = discord_for_ready.blocking_lock();
                if let Err(e) = client.set_activity(|a| {
                    a.details(format!("Editing: {}", file_state.filename))
                        .state(&state)
                        .timestamps(|t| t.start(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()))
                }) {
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
    let (service, socket) = LspService::new(move |client| Backend { 
        client, 
        discord: Arc::clone(&discord), 
        current_file: Arc::clone(&current_file_clone) 
    });

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout, socket).serve(service).await;
}
