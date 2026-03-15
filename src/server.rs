use crate::config::{Config, TimeTracking};
use crate::discord;
use crate::editor::{detect_editor, determine_editor_name, EditorInfo};
use crate::language::detect_language;
use crate::state::{FileState, WorkspaceState};
use crate::workspace::{detect_workspace_name, get_filename_from_uri, get_git_remote_url, is_git_repo};
use clap::Parser;
use discord_presence::Client as DiscordClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use url::Url;

#[derive(Parser, Debug)]
#[command(name = "discord-lsp-presence")]
#[command(about = "Discord Rich Presence for LSP editors", long_about = None)]
struct Args {
    #[arg(long)]
    editor: Option<String>,
}

pub struct AppState {
    pub discord: Mutex<DiscordClient>,
    pub config: Config,
    pub editor: Mutex<EditorInfo>,
    pub current_file: Mutex<Option<FileState>>,
    pub current_workspace: Mutex<Option<WorkspaceState>>,
    pub enabled: Mutex<bool>,
}

pub struct Backend {
    pub client: Client,
    pub state: Arc<AppState>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let cli_editor = Args::parse().editor;
        let editor_name = determine_editor_name(&params, cli_editor);
        let editor = detect_editor(&editor_name);
        *self.state.editor.lock().await = editor.clone();
        
        tracing::info!("Detected editor: {} (icon: {})", editor.name, if editor.icon_key.is_empty() { "none" } else { &editor.icon_key });
        
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "discord-lsp-presence".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "discord-presence.enable".to_string(),
                        "discord-presence.disable".to_string(),
                        "discord-presence.toggle".to_string(),
                    ],
                    ..Default::default()
                }),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        let mut discord = self.state.discord.lock().await;
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

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        let command = params.command.as_str();

        match command {
            "discord-presence.enable" => {
                let mut enabled = self.state.enabled.lock().await;
                if !*enabled {
                    *enabled = true;
                    self.client.log_message(MessageType::INFO, "Discord presence enabled.").await;

                    if let Some(file_state) = self.state.current_file.lock().await.as_ref() {
                        let language = detect_language(&file_state.filename);
                        let ts = match self.state.config.get_time_tracking() {
                            TimeTracking::File => file_state.get_start_timestamp(),
                            TimeTracking::Workspace => self.state.current_workspace.lock().await
                                .as_ref()
                                .map(|ws| ws.get_start_timestamp())
                                .unwrap_or_else(|| file_state.get_start_timestamp()),
                        };
                        discord::update_presence(
                            &self.state.discord,
                            &self.client,
                            &self.state.config,
                            &file_state.filename,
                            &file_state.workspace,
                            &language,
                            Some(ts),
                            file_state.git_remote_url.as_deref(),
                            &*self.state.editor.lock().await,
                        ).await;
                    }
                } else {
                    self.client.log_message(MessageType::INFO, "Discord presence is already enabled.").await;
                }
            }
            "discord-presence.disable" => {
                let mut enabled = self.state.enabled.lock().await;
                if *enabled {
                    *enabled = false;
                    self.client.log_message(MessageType::INFO, "Discord presence disabled.").await;
                    discord::clear_presence(&self.state.discord, &self.client).await;
                } else {
                    self.client.log_message(MessageType::INFO, "Discord presence is already disabled.").await;
                }
            }
            "discord-presence.toggle" => {
                let mut enabled = self.state.enabled.lock().await;
                *enabled = !*enabled;

                if *enabled {
                    self.client.log_message(MessageType::INFO, "Discord presence enabled.").await;

                    if let Some(file_state) = self.state.current_file.lock().await.as_ref() {
                        let language = detect_language(&file_state.filename);
                        let ts = match self.state.config.get_time_tracking() {
                            TimeTracking::File => file_state.get_start_timestamp(),
                            TimeTracking::Workspace => self.state.current_workspace.lock().await
                                .as_ref()
                                .map(|ws| ws.get_start_timestamp())
                                .unwrap_or_else(|| file_state.get_start_timestamp()),
                        };
                        discord::update_presence(
                            &self.state.discord,
                            &self.client,
                            &self.state.config,
                            &file_state.filename,
                            &file_state.workspace,
                            &language,
                            Some(ts),
                            file_state.git_remote_url.as_deref(),
                            &*self.state.editor.lock().await,
                        ).await;
                    }
                } else {
                    self.client.log_message(MessageType::INFO, "Discord presence disabled.").await;
                    discord::clear_presence(&self.state.discord, &self.client).await;
                }
            }
            _ => {}
        }

        Ok(None)
    }
}

impl Backend {
    async fn handle_file_event(&self, uri: &Url) {
        if !*self.state.enabled.lock().await {
            return;
        }

        let filename = get_filename_from_uri(uri);
        let workspace_name = detect_workspace_name(uri);
        let git_repo = is_git_repo(uri);
        let git_remote_url = if git_repo {
            get_git_remote_url(uri)
        } else {
            None
        };

        if let Some(filename) = filename {
            let workspace = workspace_name.unwrap_or_else(|| "unknown workspace".to_string());
            let language = detect_language(&filename);

            let start_timestamp = match self.state.config.get_time_tracking() {
                TimeTracking::File => {
                    let state = FileState::new(
                        filename.clone(),
                        workspace.clone(),
                        git_remote_url.clone(),
                    );
                    let ts = state.get_start_timestamp();
                    *self.state.current_file.lock().await = Some(state);
                    Some(ts)
                }
                TimeTracking::Workspace => {
                    let mut current_workspace = self.state.current_workspace.lock().await;
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
                    &self.state.discord,
                    &self.client,
                    &self.state.config,
                    &filename,
                    &workspace,
                    &language,
                    start_timestamp,
                    git_remote_url.as_deref(),
                    &*self.state.editor.lock().await,
                )
                .await;
            }
        }
    }
}
