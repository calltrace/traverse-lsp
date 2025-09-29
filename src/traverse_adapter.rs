//! Bridge between LSP server and Traverse analysis library.
//! 
//! Isolates Traverse-specific logic from the LSP protocol layer,
//! making it easier to upgrade or swap analysis engines.

use anyhow::Result;
use traverse_graph::cg::{CallGraph, CallGraphGeneratorPipeline, CallGraphGeneratorInput, CallGraphGeneratorContext};
use traverse_graph::cg_dot::{CgToDot, DotExportConfig};
use traverse_graph::cg_mermaid::{MermaidGenerator, ToSequenceDiagram};
use traverse_graph::parser::{parse_solidity, get_solidity_language};
use traverse_graph::steps::{CallsHandling, ContractHandling};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::config::MermaidConfig;

pub struct TraverseAdapter {}

impl TraverseAdapter {
    pub fn new() -> Result<Self> {
        Ok(TraverseAdapter {})
    }

    pub fn build_call_graph(&self, source: &str) -> Result<CallGraph> {
        let parsed = parse_solidity(source)?;
        let solidity_lang = get_solidity_language();
        let input = CallGraphGeneratorInput {
            source: source.to_string(),
            tree: parsed.tree,
            solidity_lang,
        };
        
        let mut ctx = CallGraphGeneratorContext::default();
        let mut graph = CallGraph::new();
        let config: HashMap<String, String> = HashMap::new();
        
        let mut pipeline = CallGraphGeneratorPipeline::new();
        pipeline.add_step(Box::new(ContractHandling::default()));
        pipeline.add_step(Box::new(CallsHandling::default()));
        pipeline.run(input, &mut ctx, &mut graph, &config)?;
        
        Ok(graph)
    }

    #[allow(dead_code)]
    pub fn generate_mermaid_flowchart(&self, graph: &CallGraph) -> Result<String> {
        let config = MermaidConfig::default();
        self.generate_mermaid_with_config(graph, &config)
            .map(|result| result.content)
    }

    pub fn generate_dot_diagram(&self, graph: &CallGraph) -> Result<String> {
        let config = DotExportConfig::default();
        let dot = graph.to_dot("call_graph", &config);
        Ok(dot)
    }
    
    pub fn generate_mermaid_with_config(&self, graph: &CallGraph, config: &MermaidConfig) -> Result<ChunkedMermaidResult> {
        let generator = MermaidGenerator::new();
        let sequence_diagram = generator.to_sequence_diagram(graph);
        let output = traverse_mermaid::sequence_diagram_writer::write_diagram(&sequence_diagram);
        
        if !config.no_chunk {
            let chunk_dir = Some(config.chunk_dir.as_path());
            
            match traverse_mermaid::mermaid_chunker::chunk_mermaid_diagram(&output, chunk_dir) {
                Ok(chunking_result) => {
                    let first_chunk_path = chunking_result.output_dir.join("chunk_001.mmd");
                    let first_chunk_content = std::fs::read_to_string(&first_chunk_path)
                        .unwrap_or_else(|_| output.clone());
                    
                    Ok(ChunkedMermaidResult {
                        is_chunked: true,
                        content: first_chunk_content,
                        chunks: Some(vec![MermaidChunk {
                            id: 1,
                            content: output.clone(),
                            filename: Some(format!("{} chunks generated", chunking_result.chunk_count)),
                        }]),
                        chunk_dir: Some(chunking_result.output_dir),
                    })
                }
                Err(e) => {
                    eprintln!("Chunking failed: {}, returning as single diagram", e);
                    Ok(ChunkedMermaidResult {
                        is_chunked: false,
                        content: output,
                        chunks: None,
                        chunk_dir: None,
                    })
                }
            }
        } else {
            Ok(ChunkedMermaidResult {
                is_chunked: false,
                content: output,
                chunks: None,
                chunk_dir: None,
            })
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ChunkedMermaidResult {
    pub is_chunked: bool,
    pub content: String,
    pub chunks: Option<Vec<MermaidChunk>>,
    pub chunk_dir: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MermaidChunk {
    pub id: usize,
    pub content: String,
    pub filename: Option<String>,
}