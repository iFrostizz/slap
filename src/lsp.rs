use crate::{
    cli::Transport,
    detectors::{Detectors, LspMessage},
};
use serde_json::Value;
use std::{path::PathBuf, str::FromStr};
use tokio::net::{TcpListener, TcpStream, UnixListener};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
pub enum LspError {
    //
}

#[derive(Debug)]
pub struct Backend {
    client: Client,
    detectors: Detectors,
}

impl Backend {
    pub fn new(client: Client, detectors: Detectors) -> Self {
        Self { client, detectors }
    }

    async fn update_lsp(&self, uri: Url) {
        let uri_string = uri.to_string();
        let (prefix, suffix) = uri_string.split_at(7);
        let path = PathBuf::from_str(suffix).unwrap();
        if prefix == "file://" && path.extension().is_some_and(|ext| ext == "sol") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let diags = self.detectors.run(&path, content).await;
                let diags = diags
                    .into_iter()
                    .filter_map(|diag| match diag {
                        LspMessage::Diagnostics { path: _, diags } => Some(diags),
                        _ => unimplemented!(),
                    })
                    .flatten()
                    .collect();
                self.client
                    .publish_diagnostics(uri.clone(), diags, None)
                    .await
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        log::info!("initialize");

        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "dummy.do_something".to_string(),
                        "format.execute".to_string(),
                        "linter.some_lint.execute".to_string(),
                    ],
                    work_done_progress_options: Default::default(),
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        log::debug!("initialized");

        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        log::debug!("initialized");

        self.client
            .log_message(MessageType::INFO, "shutdown!")
            .await;

        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        log::debug!("did_change_workspace_folders");

        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        log::debug!("did_change_configuration");

        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        log::debug!("did_change_watched_files");

        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        log::debug!("execute_command");

        self.client
            .log_message(MessageType::INFO, "command executed!")
            .await;

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        log::debug!("did_open");

        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;

        let uri = params.text_document.uri;
        self.update_lsp(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        log::debug!("did_change");

        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;

        let DidChangeTextDocumentParams {
            text_document,
            content_changes: _,
        } = params;
        self.update_lsp(text_document.uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;

        let DidSaveTextDocumentParams {
            text_document,
            text: _,
        } = params;
        self.update_lsp(text_document.uri).await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        // Ok(Some(CompletionResponse::Array(vec![
        //     CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
        //     CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        // ])))

        Ok(None)
    }
}

pub struct SlapServer {
    path: PathBuf,
    transport: Transport,
}

impl std::fmt::Debug for SlapServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("SlapServer");
        fmt.field("path", &self.path);
        fmt.field("transport", &self.transport);
        Ok(())
    }
}

impl SlapServer {
    pub fn new(path: PathBuf, transport: Transport) -> Self {
        Self { path, transport }
    }

    // pub async fn serve(&self, detectors: Vec<Box<dyn Detector>>) {
    pub async fn serve(&self, detectors: Detectors) {
        // TODO dyn
        let (service, socket) = LspService::new(|client| Backend::new(client, detectors));

        let transport = self.transport.clone();
        // tokio::spawn(async move {
        match transport {
            Transport::Stdio => {
                let stdin = tokio::io::stdin();
                let stdout = tokio::io::stdout();
                let server = Server::new(stdin, stdout, socket);
                server.serve(service).await;
            }
            Transport::IPC => {
                let path = "/tmp/slap.ipc";
                let _ = std::fs::remove_file(path);
                let listener = UnixListener::bind(path).unwrap();
                let (stream, _addr) = listener.accept().await.unwrap();
                let (read, write) = stream.into_split();
                let server = Server::new(read, write, socket);
                server.serve(service).await;
            }
            Transport::TCP => {
                // https://github.com/ebkalderon/tower-lsp/blob/master/examples/tcp.rs

                let listen = true;
                let port = 9257;
                let addr = format!("127.0.0.1:{port}");

                let stream = if listen {
                    let listener = TcpListener::bind(addr).await.unwrap();
                    let (stream, _) = listener.accept().await.unwrap();
                    stream
                } else {
                    TcpStream::connect(addr).await.unwrap()
                };

                let (read, write) = tokio::io::split(stream);

                let server = Server::new(read, write, socket);
                server.serve(service).await;
            }
        }
    }
}
