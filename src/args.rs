use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct CliInput {
    #[command(subcommand)]
    pub command: CliSubCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliSubCommand {
    New(CliNewSubCommand),
    Build(CliBuildSubCommand),
    Config(CliFmtSubCommand),
    Fmt(CliFmtSubCommand),
}

#[derive(Args, Debug, Clone)]
pub struct CliNewSubCommand {
    #[arg(long)]
    pub path: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct CliBuildSubCommand {}

#[derive(Args, Debug, Clone)]
pub struct CliFmtSubCommand {
    #[arg(long)]
    pub path: Option<PathBuf>,
    #[arg(short, long)]
    pub quiet: bool,
    #[arg(short, long)]
    pub check: bool,
    #[arg(short, long)]
    pub use_tabs: Option<bool>,
    #[arg(long)]
    pub line_width: Option<u16>,
    #[arg(long)]
    pub indent_width: Option<u8>,
    #[arg(short, long)]
    pub always_semicolons: Option<bool>,
    #[arg(short, long)]
    pub single_quote: Option<bool>,
}
