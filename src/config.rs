use crate::args::{CliInput, CliSubCommand};
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::exit;

pub const STRIX_CONFIG: &str = "strix.json";

pub fn get_config(command: &CliInput) -> Option<StrixConfig> {
    match command.command {
        CliSubCommand::New(_) => None,
        _ => config_read(),
    }
}

fn config_read() -> Option<StrixConfig> {
    match fs::read_to_string(STRIX_CONFIG) {
        Ok(text) => match serde_json::from_str(&text) {
            Ok(v) => Some(v),
            Err(err) => {
                error!("An unexpected Error occurred while trying to load {STRIX_CONFIG:?} {err}");
                exit(1);
            }
        },
        Err(_) => None,
    }
}

// commonly stored in a .strix file
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StrixConfig {
    pub name: String,
    pub description: String,
    pub authors: Option<Vec<String>>,
    pub project_type: StrixConfigProjectType,
    pub projects: HashMap<String, StrixConfigPackType>,
    pub build: StrixBuildConfig,
    pub fmt: StrixFmtConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum StrixConfigProjectType {
    #[default]
    Vanilla,
    Regolith,
    Dash,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum StrixConfigPackType {
    #[default]
    Behaviour,
    Resource,
    Skin,
    WorldTemplate
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
    pub build_path: String,
    pub default_profile: String,
    pub profiles: HashMap<String, StrixBuildConfigProfile>,
}

impl Default for StrixBuildConfig {
    fn default() -> Self {
        Self {
            build_path: String::from("target"),
            default_profile: String::from("debug"),
            profiles: HashMap::from([
                (
                    String::from("debug"),
                    StrixBuildConfigProfile {
                        minify: false,
                        obfuscate: false,
                        compress: false,
                        encrypt: false,
                        dev_folder: true,
                        package: false,
                    },
                ),
                (
                    String::from("release"),
                    StrixBuildConfigProfile {
                        minify: true,
                        obfuscate: true,
                        compress: true,
                        encrypt: true,
                        dev_folder: false,
                        package: true,
                    },
                ),
            ]),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrixBuildConfigProfile {
    /// Minify text and code in the Addon
    pub minify: bool,
    /// Obfuscate code in the Addon
    pub obfuscate: bool,
    /// Compress Assets for the addon, like images and audio
    pub compress: bool,
    /// Encrypt the addon
    pub encrypt: bool,
    /// If the building process should build
    pub dev_folder: bool,
    /// Package all projects into one `.mcaddon` file
    pub package: bool
}
