//! Traverse Adapter
//! 
//! This adapter acts as the bridge between the LSP server and the Traverse graph analysis
//! library. By encapsulating all Traverse-specific operations here, we keep the LSP protocol
//! handling code clean and focused on communication concerns. This separation makes it easier
//! to test the analysis logic independently and to upgrade or replace the underlying analysis
//! engine without touching the protocol layer. The adapter pattern also allows us to add
//! caching or other optimizations transparently in the future.

use anyhow::Result;
use graph::cg::{CallGraph, CallGraphGeneratorPipeline, CallGraphGeneratorInput, CallGraphGeneratorContext};
use graph::cg_dot::{CgToDot, DotExportConfig};
use graph::cg_mermaid::{MermaidGenerator, ToSequenceDiagram};
use graph::parser::{parse_solidity, get_solidity_language};
use graph::steps::{CallsHandling, ContractHandling};
use std::collections::HashMap;

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

    pub fn generate_mermaid_flowchart(&self, graph: &CallGraph) -> Result<String> {
        let generator = MermaidGenerator::new();
        let sequence_diagram = generator.to_sequence_diagram(graph);
        let output = mermaid::sequence_diagram_writer::write_diagram(&sequence_diagram);
        Ok(output)
    }

    pub fn generate_dot_diagram(&self, graph: &CallGraph) -> Result<String> {
        let config = DotExportConfig::default();
        let dot = graph.to_dot("call_graph", &config);
        Ok(dot)
    }
}