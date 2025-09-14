
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagramType {
    Sequence,
    CallGraph,
    Storage,
    Architecture,
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
        }
    }
}