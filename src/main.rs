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

use config::Config;
use state::FileState;
use workspace::{detect_workspace_name, get_filename_from_uri};

struct Backend {
    client: Client,
    discord: Arc<Mutex<DiscordClient>>,
    config: Arc<Config>,
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
                discord::update_presence(&self.discord, &self.client, &self.config, &filename, &workspace).await;
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

    let config_path = maybe_config.unwrap();
    let config = match Config::load(&config_path) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    let discord = Arc::new(Mutex::new(DiscordClient::new(config.application_id)));

    let current_file: Arc<Mutex<Option<FileState>>> = Arc::new(Mutex::new(None));
    let current_file_for_ready = Arc::clone(&current_file);
    let discord_for_ready = Arc::clone(&discord);
    let config_for_ready = Arc::clone(&config);

    {
        let drpc = discord.lock().await;

        drpc.on_ready(move |_ctx| {
            eprintln!("Discord client ready");
            
            if let Some(file_state) = current_file_for_ready.blocking_lock().as_ref() {
                eprintln!("Setting initial presence for: {}", file_state.filename);
                let activity = config_for_ready.build_activity(&file_state.filename, &file_state.workspace);
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
    let config_clone = Arc::clone(&config);
    let (service, socket) = LspService::new(move |client| Backend { 
        client, 
        discord: Arc::clone(&discord), 
        config: Arc::clone(&config_clone),
        current_file: Arc::clone(&current_file_clone) 
    });

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout, socket).serve(service).await;
}
