use std::collections::HashMap;
use std::fs;
use std::process::exit;
use log::error;
use serde::{Deserialize, Serialize};
use crate::args::{CliInput, CliSubCommand};

pub fn get_config(command: &CliInput) -> Option<StrixConfig> {
    match command.command {
        CliSubCommand::New(_) => { None }
        _ => config_read()
    }
}

fn config_read() -> Option<StrixConfig> {
    match fs::read_to_string(".strix") {
        Ok(text) => { match serde_json::from_str(&text) {
            Ok(v) => Some(v),
            Err(err) => {
                error!("An unexpected Error occurred while trying to load `.strix` {err}");
                exit(1);
            }
        }}
        Err(_) => None
    }
}

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
    pub default: String,
    pub profiles: HashMap<String, StrixBuildConfigProfile>,
}

impl Default for StrixBuildConfig {
    fn default() -> Self {
        Self {
            default: String::from("debug"),
            profiles: HashMap::from([
                (String::from("debug"), StrixBuildConfigProfile {
                    minify: false,
                    compression: false,
                    encryption: false,
                }),
                (String::from("release"), StrixBuildConfigProfile {
                    minify: true,
                    compression: true,
                    encryption: true,
                })
            ])
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrixBuildConfigProfile {
    pub minify: bool,
    pub compression: bool,
    pub encryption: bool,
}


