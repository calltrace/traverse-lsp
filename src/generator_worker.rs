//! Generator Worker
//! 
//! Diagram generation for complex smart contracts can be computationally expensive,
//! especially when analyzing deep call chains or contracts with hundreds of functions.
//! This worker runs in a dedicated thread to prevent these operations from blocking
//! the LSP message loop. The worker pattern ensures that users can continue editing
//! and navigating their code while diagrams are being generated in the background.
//! Each request is processed synchronously within the worker to avoid the complexity
//! of concurrent graph manipulation while still maintaining overall system responsiveness.

use crate::traverse_adapter::TraverseAdapter;
use anyhow::Result;
use graph::cg::CallGraph;
use lsp_types::Url;
use std::sync::mpsc;
use tokio::sync::oneshot;
use tracing::{debug, info};

pub enum GenerationRequest {
    Shutdown,
    GenerateCallGraphDiagram {
        uri: Url,
        contract_name: Option<String>,
        tx: oneshot::Sender<Result<String>>,
    },
    GenerateMermaidFlowchart {
        uri: Url,
        contract_name: Option<String>,
        tx: oneshot::Sender<Result<String>>,
    },
    GenerateAllDiagrams {
        uri: Url,
        contract_name: Option<String>,
        tx: oneshot::Sender<Result<String>>,
    },
    GenerateStorageLayout {
        uri: Url,
        contract_name: String,
        tx: oneshot::Sender<Result<String>>,
    },
}

pub struct GeneratorWorker {
    adapter: TraverseAdapter,
}

impl GeneratorWorker {
    pub fn new() -> Result<Self> {
        Ok(GeneratorWorker {
            adapter: TraverseAdapter::new()?,
        })
    }

    pub fn run(mut self, rx: mpsc::Receiver<GenerationRequest>) {
        info!("Generator worker started");

        for request in rx.iter() {
            match request {
                GenerationRequest::Shutdown => {
                    info!("Generator worker shutting down");
                    break;
                }
                GenerationRequest::GenerateCallGraphDiagram {
                    uri,
                    contract_name,
                    tx,
                } => {
                    debug!("Generating call graph diagram (DOT) for {:?} in {}", contract_name, uri);
                    let result = self.generate_call_graph_diagram(&uri, contract_name.as_deref());
                    let _ = tx.send(result);
                }
                GenerationRequest::GenerateMermaidFlowchart {
                    uri,
                    contract_name,
                    tx,
                } => {
                    debug!("Generating Mermaid flowchart for {:?} in {}", contract_name, uri);
                    let result = self.generate_mermaid_flowchart(&uri, contract_name.as_deref());
                    let _ = tx.send(result);
                }
                GenerationRequest::GenerateAllDiagrams {
                    uri,
                    contract_name,
                    tx,
                } => {
                    debug!("Generating all diagrams for {:?} in {}", contract_name, uri);
                    let result = self.generate_all_diagrams(&uri, contract_name.as_deref());
                    let _ = tx.send(result);
                }
                GenerationRequest::GenerateStorageLayout {
                    uri,
                    contract_name,
                    tx,
                } => {
                    debug!("Generating storage layout for {} in {}", contract_name, uri);
                    let result = self.generate_storage_layout(&uri, &contract_name);
                    let _ = tx.send(result);
                }
            }
        }
    }

    fn get_or_build_call_graph(&mut self, uri: &Url) -> Result<CallGraph> {
        let path = uri.to_file_path().map_err(|_| anyhow::anyhow!("Invalid URI"))?;
        let content = std::fs::read_to_string(path)?;
        self.adapter.build_call_graph(&content)
    }

    fn generate_call_graph_diagram(&mut self, uri: &Url, _contract_name: Option<&str>) -> Result<String> {
        let call_graph = self.get_or_build_call_graph(uri)?;
        
        let dot_diagram = self.adapter.generate_dot_diagram(&call_graph)?;
        Ok(serde_json::json!({
            "dot": dot_diagram
        }).to_string())
    }

    fn generate_mermaid_flowchart(&mut self, uri: &Url, _contract_name: Option<&str>) -> Result<String> {
        let call_graph = self.get_or_build_call_graph(uri)?;
        
        let mermaid_diagram = self.adapter.generate_mermaid_flowchart(&call_graph)?;
        Ok(serde_json::json!({
            "mermaid": mermaid_diagram
        }).to_string())
    }
    
    fn generate_all_diagrams(&mut self, uri: &Url, _contract_name: Option<&str>) -> Result<String> {
        let call_graph = self.get_or_build_call_graph(uri)?;
        
        let dot_diagram = self.adapter.generate_dot_diagram(&call_graph)?;
        let mermaid_diagram = self.adapter.generate_mermaid_flowchart(&call_graph)?;
        Ok(serde_json::json!({
            "dot": dot_diagram,
            "mermaid": mermaid_diagram
        }).to_string())
    }

    fn generate_storage_layout(&mut self, uri: &Url, _contract_name: &str) -> Result<String> {
        let call_graph = self.get_or_build_call_graph(uri)?;
        
        let storage_summary_map = graph::storage_access::analyze_storage_access(&call_graph);
        let mut md = String::from("# Storage Access Analysis\n\n");
        md.push_str(&format!("**File:** {}\n\n", uri.path()));
        md.push_str("| Endpoint | Reads | Writes |\n");
        md.push_str("|----------|-------|--------|\n");
        
        let mut sorted_entries: Vec<_> = storage_summary_map.iter().collect();
        sorted_entries.sort_by_key(|(node_id, _)| {
            call_graph.nodes.get(**node_id).map_or_else(String::new, |n| {
                format!(
                    "{}.{}",
                    n.contract_name.as_deref().unwrap_or("Global"),
                    n.name
                )
            })
        });
        
        for (func_node_id, summary) in sorted_entries {
            if let Some(func_node) = call_graph.nodes.get(*func_node_id) {
                let endpoint_name = format!(
                    "{}.{}",
                    func_node.contract_name.as_deref().unwrap_or("Global"),
                    func_node.name
                );
                
                let reads_vec: Vec<String> = summary
                    .reads
                    .iter()
                    .map(|id| {
                        call_graph.nodes.get(*id).map_or_else(
                            || format!("UnknownVar({})", id),
                            |n| format!("{}.{}", n.contract_name.as_deref().unwrap_or("?"), n.name),
                        )
                    })
                    .collect();
                
                let writes_vec: Vec<String> = summary
                    .writes
                    .iter()
                    .map(|id| {
                        call_graph.nodes.get(*id).map_or_else(
                            || format!("UnknownVar({})", id),
                            |n| format!("{}.{}", n.contract_name.as_deref().unwrap_or("?"), n.name),
                        )
                    })
                    .collect();
                
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    endpoint_name,
                    reads_vec.join(", "),
                    writes_vec.join(", ")
                ));
            }
        }
        
        Ok(md)
    }
}