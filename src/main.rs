use discord_presence::Client as DiscordClient;
use serde::Deserialize;
use std::env::home_dir;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

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

        if let Some(filename) = params
            .text_document
            .uri
            .path_segments()
            .and_then(|s| s.last())
        {
            let mut discord = self.discord.lock().await;
            match discord.set_activity(|a| a.state(format!("Editing: {}", filename))) {
                Ok(_) => {
                    self.client
                        .log_message(MessageType::INFO, &format!("Set activity to: {}", filename))
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

        if let Some(filename) = params
            .text_document
            .uri
            .path_segments()
            .and_then(|s| s.last())
        {
            let mut discord = self.discord.lock().await;
            match discord.set_activity(|a| a.state(format!("Editing: {}", filename))) {
                Ok(_) => {
                    self.client
                        .log_message(MessageType::INFO, &format!("Set activity to: {}", filename))
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
        let discord = Arc::clone(&discord);

        drpc.on_ready(move |_ctx| {
            let mut client = discord.blocking_lock();
            eprintln!("Client ready");

            if let Err(e) = client.set_activity(|a| a.state("hello world")) {
                eprintln!("Failed to set activity: {}", e);
            }
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
