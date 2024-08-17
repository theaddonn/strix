use serde::{Deserialize, Serialize};

// commonly stored in a .strix file
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrixConfig {
    name: String,
    description: String,
    authors: Option<Vec<String>>,
    fmt: StrixFmtConfig,
    build: StrixBuildConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrixFmtConfig {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrixBuildConfig {}
