use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub analysis: AnalysisConfig,
    pub generation: GenerationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AnalysisConfig {
    pub max_depth: usize,
    pub include_external: bool,
    pub cache_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GenerationConfig {
    pub default_diagram_type: DiagramType,
    pub max_nodes: usize,
    pub include_storage: bool,
    pub include_modifiers: bool,
    pub mermaid: MermaidConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagramType {
    Sequence,
    CallGraph,
    Storage,
    Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct MermaidConfig {
    pub no_chunk: bool,
    pub chunk_dir: PathBuf,
}

impl Default for MermaidConfig {
    fn default() -> Self {
        Self {
            no_chunk: false,
            chunk_dir: PathBuf::from("./traverse-output/sequence-diagrams/chunks/"),
        }
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        AnalysisConfig {
            max_depth: 10,
            include_external: false,
            cache_enabled: true,
        }
    }
}

impl Default for GenerationConfig {
    fn default() -> Self {
        GenerationConfig {
            default_diagram_type: DiagramType::Sequence,
            max_nodes: 100,
            include_storage: true,
            include_modifiers: true,
            mermaid: MermaidConfig::default(),
        }
    }
}
