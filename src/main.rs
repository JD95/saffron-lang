use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

mod parsing;

enum Value {
    Str(String),
    Sym(String),
}

struct Module {
    values: HashMap<String, Value>,
    text: String,
}

struct Backend {
    client: Client,
    text_file: Arc<Mutex<String>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, x: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(MessageType::INFO, "initalizing...")
            .await;
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    ..CompletionOptions::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("did open '{}'", params.text_document.uri.as_str()),
            )
            .await;
        let text = params.text_document.text;
        if let Ok(mut text_file) = self.text_file.lock() {
            *text_file = text;
        }
        self.client
            .log_message(MessageType::INFO, "loaded text".to_string())
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("did change'{}'", params.text_document.uri.as_str()),
            )
            .await;
        for change in params.content_changes {
            if let (Some(range), Some(range_length)) = { (change.range, change.range_length) } {
                self.client
                    .log_message(MessageType::INFO, format!("change '{}'", change.text))
                    .await;

                if let Ok(mut text_file) = self.text_file.lock() {
                    *text_file = change.text;
                }
            }
        }
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .show_message(MessageType::INFO, "Saffron lsp started!")
            .await;
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.client
            .log_message(MessageType::INFO, "completition triggered")
            .await;
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn completion_resolve(&self, _: CompletionItem) -> Result<CompletionItem> {
        self.client
            .log_message(MessageType::INFO, "completion resolve")
            .await;
        Ok(CompletionItem {
            label: "Item!".to_string(),
            ..Default::default()
        })
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let pos = params.text_document_position_params.position;
        self.client
            .log_message(
                MessageType::INFO,
                format!("hover at '{}' '{}'", pos.line, pos.character),
            )
            .await;
        let mut msg = "".to_string();
        if let Ok(text_file) = self.text_file.lock() {
            let str = text_file.as_str();
            if let Ok(tokens) = parsing::lex_line(str) {
                msg = format!("{:?}", tokens);
                if let Some(result) = tokens
                    .iter()
                    .filter(|t| t.position.location_offset() >= pos.character.try_into().unwrap())
                    .last()
                {
                    return Ok(Some(Hover {
                        contents: HoverContents::Scalar(MarkedString::String(
                            format!("You're hovering on a {:?}", result.content).to_string(),
                        )),
                        range: None,
                    }));
                }
            }
        }
        self.client
            .log_message(MessageType::INFO, format!("tokens were '{}'", msg))
            .await;

        return Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                "Not sure what this is".to_string(),
            )),
            range: None,
        }));
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let text_file = Arc::new(Mutex::new("".to_string()));
    let (service, socket) = LspService::new(|client| Backend {
        client: client,
        text_file: text_file,
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
