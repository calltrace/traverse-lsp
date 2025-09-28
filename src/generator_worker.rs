//! Background worker for computationally expensive operations.
//! 
//! Prevents diagram generation from blocking the LSP message loop,
//! ensuring the editor remains responsive during analysis.

use crate::traverse_adapter::TraverseAdapter;
use crate::config::MermaidConfig;
use anyhow::Result;
use traverse_graph::cg::CallGraph;
use lsp_types::Url;
use std::sync::mpsc;
use std::path::PathBuf;
use tokio::sync::oneshot;
use tracing::{debug, info};

pub enum GenerationRequest {
    Shutdown,
    GenerateCallGraphDiagram {
        uris: Vec<Url>,
        contract_name: Option<String>,
        tx: oneshot::Sender<Result<String>>,
    },
    GenerateMermaidFlowchart {
        uris: Vec<Url>,
        contract_name: Option<String>,
        no_chunk: bool,
        tx: oneshot::Sender<Result<String>>,
    },
    GenerateAllDiagrams {
        uris: Vec<Url>,
        contract_name: Option<String>,
        tx: oneshot::Sender<Result<String>>,
    },
    GenerateStorageLayout {
        uris: Vec<Url>,
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
                    uris,
                    contract_name,
                    tx,
                } => {
                    debug!("Generating call graph diagram (DOT) for {:?} in {} files", contract_name, uris.len());
                    let result = self.generate_call_graph_diagram(&uris, contract_name.as_deref());
                    let _ = tx.send(result);
                }
                GenerationRequest::GenerateMermaidFlowchart {
                    uris,
                    contract_name,
                    no_chunk,
                    tx,
                } => {
                    debug!("Generating Mermaid flowchart for {:?} in {} files (no_chunk: {})", contract_name, uris.len(), no_chunk);
                    let result = self.generate_mermaid_flowchart(&uris, contract_name.as_deref(), no_chunk);
                    let _ = tx.send(result);
                }
                GenerationRequest::GenerateAllDiagrams {
                    uris,
                    contract_name,
                    tx,
                } => {
                    debug!("Generating all diagrams for {:?} in {} files", contract_name, uris.len());
                    let result = self.generate_all_diagrams(&uris, contract_name.as_deref());
                    let _ = tx.send(result);
                }
                GenerationRequest::GenerateStorageLayout {
                    uris,
                    contract_name,
                    tx,
                } => {
                    debug!("Generating storage layout for {} in {} files", contract_name, uris.len());
                    let result = self.generate_storage_layout(&uris, &contract_name);
                    let _ = tx.send(result);
                }
            }
        }
    }

    fn get_or_build_call_graph(&mut self, uris: &[Url]) -> Result<CallGraph> {
        let mut combined_source = String::new();
        
        for uri in uris {
            let path = uri.to_file_path().map_err(|_| anyhow::anyhow!("Invalid URI"))?;
            let content = std::fs::read_to_string(&path)?;
            combined_source.push_str(&content);
            combined_source.push('\n');
        }
        
        self.adapter.build_call_graph(&combined_source)
    }

    fn generate_call_graph_diagram(&mut self, uris: &[Url], _contract_name: Option<&str>) -> Result<String> {
        let call_graph = self.get_or_build_call_graph(uris)?;
        
        let dot_diagram = self.adapter.generate_dot_diagram(&call_graph)?;
        Ok(serde_json::json!({
            "dot": dot_diagram
        }).to_string())
    }

    fn generate_mermaid_flowchart(&mut self, uris: &[Url], _contract_name: Option<&str>, no_chunk: bool) -> Result<String> {
        let call_graph = self.get_or_build_call_graph(uris)?;
        
        let config = MermaidConfig {
            no_chunk,
            chunk_dir: PathBuf::from("./mermaid-chunks/"),
        };
        
        let result = self.adapter.generate_mermaid_with_config(&call_graph, &config)?;
        
        if result.is_chunked {
            Ok(serde_json::json!({
                "mermaid": result.content,
                "is_chunked": true,
                "chunks": result.chunks,
                "chunk_dir": result.chunk_dir,
            }).to_string())
        } else {
            Ok(serde_json::json!({
                "mermaid": result.content,
                "is_chunked": false,
            }).to_string())
        }
    }
    
    fn generate_all_diagrams(&mut self, uris: &[Url], _contract_name: Option<&str>) -> Result<String> {
        let call_graph = self.get_or_build_call_graph(uris)?;
        
        let dot_diagram = self.adapter.generate_dot_diagram(&call_graph)?;
        let mermaid_config = MermaidConfig::default();
        let mermaid_result = self.adapter.generate_mermaid_with_config(&call_graph, &mermaid_config)?;
        
        Ok(serde_json::json!({
            "dot": dot_diagram,
            "mermaid": mermaid_result.content,
            "is_chunked": mermaid_result.is_chunked,
            "chunk_dir": mermaid_result.chunk_dir
        }).to_string())
    }

    fn generate_storage_layout(&mut self, uris: &[Url], _contract_name: &str) -> Result<String> {
        let call_graph = self.get_or_build_call_graph(uris)?;
        
        let storage_summary_map = traverse_graph::storage_access::analyze_storage_access(&call_graph);
        let mut md = String::from("# Storage Access Analysis\n\n");
        md.push_str(&format!("**Files analyzed:** {} Solidity files\n\n", uris.len()));
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