use discord_presence::Client as DiscordClient;
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
}

#[tokio::main]
async fn main() {
    let discord = Arc::new(Mutex::new(DiscordClient::new(DISCORD_APPLICATION_ID)));

    {
        let drpc = discord.lock().await;
        let discord = Arc::clone(&discord);

        drpc.on_ready(move |_ctx| {
            let mut client = discord.blocking_lock();

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
