use clap::Parser;
use discord_presence::Client as DiscordClient;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::{error, info};
use url::Url;

mod activity;
mod config;
mod discord;
mod editor;
mod language;
mod logging;
mod state;
mod workspace;

use activity::build_activity;
use config::{Config, TimeTracking};
use editor::{detect_editor, EditorInfo};
use language::detect_language;
use state::{FileState, WorkspaceState};
use workspace::{detect_workspace_name, get_filename_from_uri, is_git_repo, get_git_remote_url};

fn extract_editor_from_init_options(params: &InitializeParams) -> Option<String> {
    let init_options = params.initialization_options.as_ref()?;
    
    if let Some(editor) = init_options.get("editor").and_then(|v| v.as_str()) {
        return Some(editor.to_string());
    }
    
    if let Some(name) = init_options.get("name").and_then(|v| v.as_str()) {
        return Some(name.to_string());
    }
    
    None
}

fn extract_editor_from_client_info(params: &InitializeParams) -> Option<String> {
    params.client_info.as_ref().map(|ci| ci.name.clone())
}

fn determine_editor_name(params: &InitializeParams, cli_editor: Option<String>) -> String {
    if let Some(editor) = cli_editor {
        return editor;
    }
    
    if let Some(editor) = extract_editor_from_init_options(params) {
        return editor;
    }
    
    if let Some(editor) = extract_editor_from_client_info(params) {
        return editor;
    }
    
    "Unknown Editor".to_string()
}

#[derive(Parser, Debug)]
#[command(name = "discord-lsp-presence")]
#[command(about = "Discord Rich Presence for LSP editors", long_about = None)]
struct Args {
    #[arg(long)]
    editor: Option<String>,
}

struct Backend {
    client: Client,
    discord: Arc<Mutex<DiscordClient>>,
    config: Arc<Config>,
    editor: Arc<Mutex<EditorInfo>>,
    current_file: Arc<Mutex<Option<FileState>>>,
    current_workspace: Arc<Mutex<Option<WorkspaceState>>>,
    enabled: Arc<Mutex<bool>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let cli_editor = Args::parse().editor;
        let editor_name = determine_editor_name(&params, cli_editor);
        let editor = detect_editor(&editor_name);
        *self.editor.lock().await = editor.clone();
        
        info!("Detected editor: {} (icon: {})", editor.name, if editor.icon_key.is_empty() { "none" } else { &editor.icon_key });
        
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

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        let command = params.command.as_str();

        match command {
            "discord-presence.enable" => {
                let mut enabled = self.enabled.lock().await;
                if !*enabled {
                    *enabled = true;
                    self.client.log_message(MessageType::INFO, "Discord presence enabled.").await;

                    if let Some(file_state) = self.current_file.lock().await.as_ref() {
                        let language = detect_language(&file_state.filename);
                        let ts = match self.config.get_time_tracking() {
                            TimeTracking::File => file_state.get_start_timestamp(),
                            TimeTracking::Workspace => self.current_workspace.lock().await
                                .as_ref()
                                .map(|ws| ws.get_start_timestamp())
                                .unwrap_or_else(|| file_state.get_start_timestamp()),
                        };
                        discord::update_presence(
                            &self.discord,
                            &self.client,
                            &self.config,
                            &file_state.filename,
                            &file_state.workspace,
                            &language,
                            Some(ts),
                            file_state.git_remote_url.as_deref(),
                            self.editor.lock().await.deref(),
                        ).await;
                    }
                } else {
                    self.client.log_message(MessageType::INFO, "Discord presence is already enabled.").await;
                }
            }
            "discord-presence.disable" => {
                let mut enabled = self.enabled.lock().await;
                if *enabled {
                    *enabled = false;
                    self.client.log_message(MessageType::INFO, "Discord presence disabled.").await;
                    discord::clear_presence(&self.discord, &self.client).await;
                } else {
                    self.client.log_message(MessageType::INFO, "Discord presence is already disabled.").await;
                }
            }
            "discord-presence.toggle" => {
                let mut enabled = self.enabled.lock().await;
                *enabled = !*enabled;

                if *enabled {
                    self.client.log_message(MessageType::INFO, "Discord presence enabled.").await;

                    if let Some(file_state) = self.current_file.lock().await.as_ref() {
                        let language = detect_language(&file_state.filename);
                        let ts = match self.config.get_time_tracking() {
                            TimeTracking::File => file_state.get_start_timestamp(),
                            TimeTracking::Workspace => self.current_workspace.lock().await
                                .as_ref()
                                .map(|ws| ws.get_start_timestamp())
                                .unwrap_or_else(|| file_state.get_start_timestamp()),
                        };
                        discord::update_presence(
                            &self.discord,
                            &self.client,
                            &self.config,
                            &file_state.filename,
                            &file_state.workspace,
                            &language,
                            Some(ts),
                            file_state.git_remote_url.as_deref(),
                            self.editor.lock().await.deref(),
                        ).await;
                    }
                } else {
                    self.client.log_message(MessageType::INFO, "Discord presence disabled.").await;
                    discord::clear_presence(&self.discord, &self.client).await;
                }
            }
            _ => {}
        }

        Ok(None)
    }
}

impl Backend {
    async fn handle_file_event(&self, uri: &Url) {
        if !*self.enabled.lock().await {
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

            let start_timestamp = match self.config.get_time_tracking() {
                TimeTracking::File => {
                    let state = FileState::new(
                        filename.clone(),
                        workspace.clone(),
                        git_remote_url.clone(),
                    );
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
                    git_remote_url.as_deref(),
                    self.editor.lock().await.deref(),
                )
                .await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    logging::init_logging();

    let config = Arc::new(Config::load());

    let discord = Arc::new(Mutex::new(DiscordClient::new(config.get_application_id())));

    let current_file: Arc<Mutex<Option<FileState>>> = Arc::new(Mutex::new(None));
    let current_workspace: Arc<Mutex<Option<WorkspaceState>>> = Arc::new(Mutex::new(None));
    let enabled: Arc<Mutex<bool>> = Arc::new(Mutex::new(config.is_enabled()));
    let editor: Arc<Mutex<EditorInfo>> = Arc::new(Mutex::new(EditorInfo::default()));
    let current_file_for_ready = Arc::clone(&current_file);
    let current_workspace_for_ready = Arc::clone(&current_workspace);
    let discord_for_ready = Arc::clone(&discord);
    let config_for_ready = Arc::clone(&config);
    let enabled_for_ready = Arc::clone(&enabled);
    let editor_for_ready = Arc::clone(&editor);

    {
        let drpc = discord.lock().await;

        drpc.on_ready(move |_ctx| {
            info!("Discord client ready");

            if !*enabled_for_ready.blocking_lock() {
                info!("Discord presence is disabled, skipping initial presence.");
                return;
            }

            if let Some(file_state) = current_file_for_ready.blocking_lock().as_ref() {
                info!("Setting initial presence for: {}", file_state.filename);
                let ts = match config_for_ready.get_time_tracking() {
                    TimeTracking::File => file_state.get_start_timestamp(),
                    TimeTracking::Workspace => current_workspace_for_ready
                        .blocking_lock()
                        .as_ref()
                        .map(|ws| ws.get_start_timestamp())
                        .unwrap_or_else(|| file_state.get_start_timestamp()),
                };
                let language = detect_language(&file_state.filename);
                let editor = editor_for_ready.blocking_lock().clone();
                let activity = build_activity(
                    &config_for_ready,
                    &file_state.filename,
                    &file_state.workspace,
                    &language,
                    Some(ts),
                    file_state.git_remote_url.as_deref(),
                    &editor,
                );
                let mut client = discord_for_ready.blocking_lock();
                if let Err(e) = client.set_activity(|_| activity) {
                    error!("Failed to set initial activity: {}", e);
                }
            }
        })
        .persist();

        drpc.on_error(|_ctx| {
            error!("Discord connection error. Exiting.");
            std::process::exit(1);
        })
        .persist();
    }

    let current_file_clone = Arc::clone(&current_file);
    let current_workspace_clone = Arc::clone(&current_workspace);
    let config_clone = Arc::clone(&config);
    let enabled_clone = Arc::clone(&enabled);
    let editor_clone = Arc::clone(&editor);
    let (service, socket) = LspService::new(move |client| Backend {
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
    Server::new(stdin, stdout, socket).serve(service).await;
}
