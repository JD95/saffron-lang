use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use std::collections::HashMap;
use std::cell::Cell;
use std::sync::Mutex;
use std::sync::Arc;

enum Value {
    Str(String),
    Sym(String)
}

struct Module {
    values: HashMap<String, Value>,
    text: String
}

struct Backend {
    client: Client,
    modules: Arc<Mutex<HashMap<String, Module>>>
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, x: InitializeParams) -> Result<InitializeResult> {
        self.client.log_message(MessageType::INFO, "initalizing...").await;
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions{
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
        self.client.log_message(MessageType::INFO, format!("did open '{}'", params.text_document.uri.as_str())).await;
        let path = params.text_document.uri.as_str();
        let values = HashMap::new();
        let text = params.text_document.text;
        let module = Module{values, text};
        if let Ok(mut modules) = self.modules.lock() {
            modules.insert(path.to_string(), module);
        }
    } 

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client.log_message(MessageType::INFO, format!("did change'{}'", params.text_document.uri.as_str())).await;
        for change in params.content_changes {
            if let (Some(range), Some(range_length)) = { (change.range, change.range_length) } { 
                self.client.log_message(MessageType::INFO, format!("change '{}'", change.text)).await;
            }
        }
    } 

    async fn initialized(&self, _: InitializedParams) {
        self.client.show_message(MessageType::INFO, "Saffron lsp started!").await;
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.client.log_message(MessageType::INFO, "completition triggered").await;
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string())
        ])))
    }

    async fn completion_resolve(&self, _: CompletionItem) -> Result<CompletionItem> {
        self.client.log_message(MessageType::INFO, "completion resolve").await;
        Ok(CompletionItem{
            label: "Item!".to_string(),
            ..Default::default()
        })
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        self.client.log_message(MessageType::INFO, "hover triggered").await;
        Ok(Some(Hover {
            contents: HoverContents::Scalar(
                MarkedString::String("You're hovering!".to_string())
            ),
            range: None
        }))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let modules = Arc::new(Mutex::new(HashMap::new()));
    let (service, socket) = LspService::new(|client| Backend { client: client, modules: modules });
    Server::new(stdin, stdout, socket).serve(service).await;
}