use crate::{
    commands, 
    generator_worker::GenerationRequest, 
    handlers::common::send_request_to_worker,
};
use anyhow::Result;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::{ExecuteCommandParams, MessageType, ShowMessageParams, Url};
use serde::de::DeserializeOwned;
use std::sync::mpsc;
use tracing::{debug, error, info};

pub fn execute_command(
    req: Request,
    conn: &Connection,
    generator_tx: &mpsc::Sender<GenerationRequest>,
) -> Result<()> {
    let (id, params) = req.extract::<ExecuteCommandParams>("workspace/executeCommand")?;
    debug!("Executing command: {}", params.command);

    let response = match params.command.as_str() {
        commands::GENERATE_CALL_GRAPH_WORKSPACE => {
            workspace_command(conn, id.clone(), params, generator_tx, |uris, tx| {
                show_message(conn, MessageType::INFO, format!("Analyzing {} files...", uris.len()))?;
                Ok(GenerationRequest::GenerateCallGraphDiagram {
                    uris,
                    contract_name: None,
                    tx,
                })
            })
        }
        commands::GENERATE_SEQUENCE_DIAGRAM_WORKSPACE => {
            let args = extract_args::<WorkspaceArgs>(&params, &id);
            let no_chunk = args.as_ref().map(|a| a.no_chunk).unwrap_or(false);
            workspace_command(conn, id.clone(), params, generator_tx, move |uris, tx| {
                show_message(conn, MessageType::INFO, format!("Generating diagram for {} files...", uris.len()))?;
                Ok(GenerationRequest::GenerateMermaidFlowchart {
                    uris,
                    contract_name: None,
                    no_chunk,
                    tx,
                })
            })
        }
        commands::GENERATE_ALL_WORKSPACE => {
            workspace_command(conn, id.clone(), params, generator_tx, |uris, tx| {
                show_message(conn, MessageType::INFO, format!("Generating all for {} files...", uris.len()))?;
                Ok(GenerationRequest::GenerateAllDiagrams {
                    uris,
                    contract_name: None,
                    tx,
                })
            })
        }
        commands::ANALYZE_STORAGE_WORKSPACE => {
            workspace_command(conn, id.clone(), params, generator_tx, |uris, tx| {
                show_message(conn, MessageType::INFO, format!("Analyzing storage for {} files...", uris.len()))?;
                Ok(GenerationRequest::GenerateStorageLayout {
                    uris,
                    contract_name: String::new(),
                    tx,
                })
            })
        }
        
        _ => Ok(Response::new_err(
            id,
            -32601,
            format!("Unknown command: {}", params.command),
        )),
    }?;

    conn.sender.send(Message::Response(response))?;
    Ok(())
}

fn workspace_command(
    conn: &Connection,
    id: lsp_server::RequestId,
    params: ExecuteCommandParams,
    generator_tx: &mpsc::Sender<GenerationRequest>,
    build_request: impl FnOnce(Vec<Url>, tokio::sync::oneshot::Sender<Result<String>>) -> Result<GenerationRequest>,
) -> Result<Response> {
    let workspace_args = match extract_args::<WorkspaceArgs>(&params, &id) {
        Ok(args) => args,
        Err(response) => return Ok(response),
    };
    let sol_files = find_solidity_files(&workspace_args.workspace_folder)?;
    
    if sol_files.is_empty() {
        show_message(
            conn,
            MessageType::WARNING,
            "No Solidity files found in workspace".into(),
        )?;
        return Ok(Response::new_ok(id, serde_json::json!(null)));
    }
    
    info!("Found {} Solidity files in workspace", sol_files.len());
    
    let result = send_request_to_worker(generator_tx, |tx| build_request(sol_files, tx).unwrap());
    match result {
        Ok(res) => generation_result(conn, id, Ok(res)),
        Err(_) => Ok(Response::new_err(id, -32603, "Failed to send request".into())),
    }
}

fn generation_result(
    conn: &Connection,
    id: lsp_server::RequestId,
    result: Result<Result<String>>,
) -> Result<Response> {
    match result {
        Ok(Ok(diagram_data)) => {
            if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&diagram_data) {
                Ok(Response::new_ok(id, serde_json::json!({ 
                    "success": true,
                    "data": json_data 
                })))
            } else {
                Ok(Response::new_ok(id, serde_json::json!({ 
                    "success": true,
                    "diagram": diagram_data 
                })))
            }
        }
        Ok(Err(e)) => {
            error!("Failed to generate diagram: {}", e);
            show_message(
                conn,
                MessageType::ERROR,
                format!("Failed to generate: {e}"),
            )?;
            Ok(Response::new_err(
                id,
                -32603,
                e.to_string(),
            ))
        }
        Err(e) => {
            error!("Channel error: {}", e);
            Ok(Response::new_err(id, -32603, "Internal error".into()))
        }
    }
}

fn extract_args<T: DeserializeOwned>(
    params: &ExecuteCommandParams,
    id: &lsp_server::RequestId,
) -> Result<T, Response> {
    let Some(args_value) = params.arguments.first() else {
        return Err(Response::new_err(id.clone(), -32602, "Missing arguments".into()));
    };
    
    serde_json::from_value::<T>(args_value.clone())
        .map_err(|_| Response::new_err(id.clone(), -32602, "Invalid parameters".into()))
}

fn find_solidity_files(workspace_folder: &str) -> Result<Vec<Url>> {
    use walkdir::WalkDir;
    
    let mut sol_files = Vec::new();
    
    for entry in WalkDir::new(workspace_folder)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            !e.path()
                .components()
                .any(|c| {
                    matches!(c.as_os_str().to_str(), Some("node_modules" | "build" | "cache" | ".git"))
                })
        })
    {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("sol") {
            let uri = Url::from_file_path(entry.path())
                .map_err(|_| anyhow::anyhow!("Invalid path"))?;
            sol_files.push(uri);
        }
    }
    
    Ok(sol_files)
}

fn show_message(conn: &Connection, typ: MessageType, message: String) -> Result<()> {
    let params = ShowMessageParams { typ, message };
    let notification = Notification::new("window/showMessage".to_string(), params);
    conn.sender.send(Message::Notification(notification))?;
    Ok(())
}

#[derive(serde::Deserialize)]
struct WorkspaceArgs {
    workspace_folder: String,
    #[serde(default)]
    no_chunk: bool,
}
