//! Main LSP Server Entry Point
//! 
//! This server uses stdio for communication to ensure compatibility with any LSP client,
//! whether it's VS Code, Neovim, or Emacs. Heavy computational tasks like diagram generation
//! are offloaded to a dedicated worker thread, keeping the main message loop responsive
//! to user interactions. This architecture prevents UI freezes when analyzing large
//! smart contracts with complex call graphs.

use crate::{
    generator_worker::{GenerationRequest, GeneratorWorker},
    handlers::execute_command,
};
use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{
    request::{ExecuteCommand, Request as _},
    CodeActionOptions, CompletionOptions,
    InitializeParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use std::{
    sync::mpsc,
    thread,
};
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod commands;
mod config;
mod generator_worker;
mod handlers;
mod traverse_adapter;
mod utils;

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Traverse LSP server");

    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::FULL,
        )),
        completion_provider: Some(CompletionOptions::default()),
        hover_provider: None,
        code_lens_provider: None,
        code_action_provider: Some(lsp_types::CodeActionProviderCapability::Options(
            CodeActionOptions {
                ..Default::default()
            },
        )),
        execute_command_provider: None,
        ..Default::default()
    })?;

    let init_params = connection.initialize(server_capabilities)?;
    let init_params: InitializeParams = serde_json::from_value(init_params)?;

    main_loop(connection, init_params)?;

    io_threads.join()?;
    info!("Shutting down Traverse LSP server");
    Ok(())
}

fn main_loop(connection: Connection, _init_params: InitializeParams) -> Result<()> {
    info!("Starting main loop");

    let (generator_tx, generator_rx) = mpsc::channel::<GenerationRequest>();

    let generator_thread = thread::spawn(move || {
        GeneratorWorker::new()
            .unwrap()
            .run(generator_rx);
    });

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    generator_tx.send(GenerationRequest::Shutdown)?;
                    break;
                }

                process_request(&connection, req, &generator_tx);
            }
            Message::Notification(not) => {
                process_notification(not);
            }
            Message::Response(_) => {}
        }
    }

    generator_thread.join().unwrap();

    Ok(())
}

fn process_request(
    conn: &Connection,
    req: Request,
    generator_tx: &mpsc::Sender<GenerationRequest>,
) {
    let req_id = req.id.clone();

    let result = match req.method.as_str() {
        ExecuteCommand::METHOD => execute_command(req, conn, generator_tx),
        _ => {
            info!("Received unhandled request: {}", req.method);
            Ok(())
        }
    };

    if let Err(e) = result {
        let response = Response::new_err(req_id, -32603, e.to_string());
        let _ = conn.sender.send(response.into());
    }
}

fn process_notification(_not: Notification) {}
