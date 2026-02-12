use discord_presence::Client as DiscordClient;
use serde::Deserialize;
use std::env::home_dir;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use url::Url;

const DISCORD_APPLICATION_ID: u64 = 1470506076574187745;

struct FileState {
    filename: String,
    workspace: String,
    start_time: Instant,
}

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

        let uri = &params.text_document.uri;
        let filename = uri
            .path_segments()
            .and_then(|s| s.last())
            .map(|s| s.to_string());
        let workspace_name = detect_workspace_name(uri);

        if let Some(filename) = filename {
            let workspace = workspace_name.unwrap_or_else(|| "unknown workspace".to_string());
            let state = FileState {
                filename: filename.clone(),
                workspace: workspace.clone(),
                start_time: Instant::now(),
            };
            *self.current_file.lock().await = Some(state);

            if DiscordClient::is_ready() {
                self.update_presence(&filename, &workspace).await;
            }
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "File changed")
            .await;

        let uri = &params.text_document.uri;
        let filename = uri
            .path_segments()
            .and_then(|s| s.last())
            .map(|s| s.to_string());
        let workspace_name = detect_workspace_name(uri);

        if let Some(filename) = filename {
            let workspace = workspace_name.unwrap_or_else(|| "unknown workspace".to_string());
            let state = FileState {
                filename: filename.clone(),
                workspace: workspace.clone(),
                start_time: Instant::now(),
            };
            *self.current_file.lock().await = Some(state);

            if DiscordClient::is_ready() {
                self.update_presence(&filename, &workspace).await;
            }
        }
    }
}

impl Backend {
    async fn update_presence(&self, filename: &str, workspace: &str) {
        let mut discord = self.discord.lock().await;
        let state = format!("in {}", workspace);

        match discord.set_activity(|a| {
            a.details(format!("Editing: {}", filename))
                .state(&state)
                .timestamps(|t| t.start(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()))
        }) {
            Ok(_) => {
                self.client
                    .log_message(
                        MessageType::INFO,
                        &format!("Set activity to: Editing: {} {}", filename, state),
                    )
                    .await;
            }
            Err(e) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        &format!("Failed to set activity: {}", e),
                    )
                    .await;
            }
        }
    }
}

fn detect_workspace_name(uri: &Url) -> Option<String> {
    let path = uri.to_file_path().ok()?;
    let mut current_dir = path.parent()?;

    loop {
        if current_dir.join(".git").exists() {
            return current_dir
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string());
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent,
            None => break,
        }
    }

    path.parent()
        .and_then(|dir| dir.file_name())
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
}

fn get_config_dir() -> Option<PathBuf> {
    home_dir().and_then(|home| Some(home.join(".config").join("discord-presence-lsp")))
}

fn ensure_config() -> std::result::Result<PathBuf, &'static str> {
    let Some(path) = get_config_dir() else {
        return Err("Couldn't get config directory");
    };

    if !path.exists() {
        if std::fs::create_dir_all(&path).is_err() {
            return Err("Couldn't create config dir");
        }
    }

    let path = path.join("config.toml");

    if !path.exists() {
        if std::fs::write(&path, "foo = 'bar'").is_err() {
            return Err("Couldn't create config file");
        }
    }

    Ok(path)
}

#[derive(Deserialize, Debug)]
struct Config {
    foo: String,
}

#[tokio::main]
async fn main() {
    let maybe_config = ensure_config();

    if let Err(e) = maybe_config {
        eprintln!("{e}");
    };

    let config = std::fs::read_to_string(maybe_config.unwrap()).unwrap();

    let config: Config = toml::from_str(&config).unwrap();

    let _ = dbg!(config);

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

    let (service, socket) = LspService::new(move |client| Backend { client, discord, current_file: Arc::clone(&current_file) });

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout, socket).serve(service).await;
}
