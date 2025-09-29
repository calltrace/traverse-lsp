use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
