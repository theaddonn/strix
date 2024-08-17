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
pub struct StrixFmtConfig {
    pub use_tabs: bool,
    pub line_width: u16,
    pub indent_width: u8,
    pub always_semicolons: bool,
    pub single_quote: bool,
}

impl Default for StrixFmtConfig {
    fn default() -> Self {
        Self {
            use_tabs: true,
            line_width: 80,
            indent_width: 4,
            always_semicolons: false,
            single_quote: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrixBuildConfig {

}
