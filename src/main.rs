use discord_presence::Client as DiscordClient;
use serde::Deserialize;
use std::env::home_dir;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use url::Url;

const DISCORD_APPLICATION_ID: u64 = 1470506076574187745;

struct Backend {
    client: Client,
    discord: Arc<Mutex<DiscordClient>>,
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

        if !DiscordClient::is_ready() {
            self.client
                .log_message(
                    MessageType::WARNING,
                    "Discord not ready, skipping activity update",
                )
                .await;
            return;
        }

        let uri = &params.text_document.uri;
        let filename = uri
            .path_segments()
            .and_then(|s| s.last())
            .map(|s| s.to_string());
        let workspace_name = detect_workspace_name(uri);

        if let Some(filename) = filename {
            let mut discord = self.discord.lock().await;
            let state = workspace_name
                .map(|name| format!("in {}", name))
                .unwrap_or_else(|| "in unknown workspace".to_string());

            match discord
                .set_activity(|a| a.details(format!("Editing: {}", filename)).state(&state))
            {
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

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "File changed")
            .await;

        if !DiscordClient::is_ready() {
            self.client
                .log_message(
                    MessageType::WARNING,
                    "Discord not ready, skipping activity update",
                )
                .await;
            return;
        }

        let uri = &params.text_document.uri;
        let filename = uri
            .path_segments()
            .and_then(|s| s.last())
            .map(|s| s.to_string());
        let workspace_name = detect_workspace_name(uri);

        if let Some(filename) = filename {
            let mut discord = self.discord.lock().await;
            let state = workspace_name
                .map(|name| format!("in {}", name))
                .unwrap_or_else(|| "in unknown workspace".to_string());

            match discord
                .set_activity(|a| a.details(format!("Editing: {}", filename)).state(&state))
            {
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

    {
        let drpc = discord.lock().await;

        drpc.on_ready(move |_ctx| {
            eprintln!("Discord client ready");
        })
        .persist();

        drpc.on_error(|_ctx| {
            eprintln!("Discord connection error. Exiting.");
            std::process::exit(1);
        })
        .persist();
    }

    let (service, socket) = LspService::new(move |client| Backend { client, discord });

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout, socket).serve(service).await;
}
