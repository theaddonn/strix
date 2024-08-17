use crate::args::CliFmtSubCommand;
use console::Style;
use dprint_plugin_biome::configuration::{
    ArrowParentheses, Configuration, IndentStyle, QuoteProperties, QuoteStyle, Semicolons,
    TrailingComma,
};
use dprint_plugin_biome::format_text as format_biome;
use log::{error, info, warn};
use similar::{ChangeTag, TextDiff};
use std::env::current_dir;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use clap::builder::Str;
use walkdir::{DirEntry, WalkDir};
use crate::config::StrixConfig;

const SUPPORTED_EXTENSIONS: &[&str] = &["json", "js", "ts"];

#[inline(always)]
fn try_rm_prefix(path: &Path) -> PathBuf {
    path.strip_prefix(current_dir().unwrap_or_default())
        .unwrap_or(path)
        .to_path_buf()
}

fn fmt_build_config(fmt: &CliFmtSubCommand, config: &Option<StrixConfig>) -> Configuration {
    let config = config.clone().unwrap_or(StrixConfig::default());

    Configuration {
        javascript_indent_style: match fmt.use_tabs {
            Some(true) => Some(IndentStyle::Tab),
            _ => match config.fmt.use_tabs {
                true => Some(IndentStyle::Tab),
                false => Some(IndentStyle::Space),
            },
        },
        javascript_indent_size: match fmt.indent_width {
            Some(v) => Some(v),
            None => Some(config.fmt.indent_width),
        },
        javascript_line_width: match fmt.line_width {
            Some(v) => Some(v),
            None => Some(config.fmt.line_width),
        },
        json_indent_style: match fmt.use_tabs {
            Some(true) => Some(IndentStyle::Tab),
            _ => match config.fmt.use_tabs {
                true => Some(IndentStyle::Tab),
                false => Some(IndentStyle::Space),
            },
        },
        json_indent_size: match fmt.indent_width {
            Some(v) => Some(v),
            None => Some(config.fmt.indent_width),
        },
        json_line_width: match fmt.line_width {
            Some(v) => Some(v),
            None => Some(config.fmt.line_width),
        },
        semicolons: match fmt.always_semicolons {
            Some(true) => Some(Semicolons::Always),
            _ => match config.fmt.always_semicolons {
                true => Some(Semicolons::Always),
                false => Some(Semicolons::AsNeeded),
            },
        },
        quote_style: match fmt.single_quote {
            Some(true) => Some(QuoteStyle::Single),
            _ => match config.fmt.single_quote {
                true => Some(QuoteStyle::Single),
                false => Some(QuoteStyle::Double),
            },
        },
        jsx_quote_style: match fmt.single_quote {
            Some(true) => Some(QuoteStyle::Single),
            _ => match config.fmt.single_quote {
                true => Some(QuoteStyle::Single),
                false => Some(QuoteStyle::Double),
            },
        },
        quote_properties: Some(QuoteProperties::Preserve),
        arrow_parentheses: Some(ArrowParentheses::AsNeeded),
        trailing_comma: Some(TrailingComma::Es5),
    }
}

pub async fn fmt(fmt: CliFmtSubCommand, config: Option<StrixConfig>) -> bool {
    let config = Arc::new(fmt_build_config(&fmt, &config));

    let mut count = 0;
    let walk: Vec<_> = WalkDir::new(fmt.path.unwrap_or(current_dir().unwrap()))
        .into_iter()
        .filter(|v| {
            if let Ok(v) = v {
                v.file_type().is_file()
            } else {
                false
            }
        })
        .inspect(|_| count += 1)
        .collect();

    let mut handles = vec![];
    let error_out = Arc::new(AtomicBool::new(false));

    for entry in walk.into_iter().flatten() {
        let config = config.clone();
        let error_out = error_out.clone();

        if let Some(ext) = entry.path().extension().and_then(OsStr::to_str) {
            if SUPPORTED_EXTENSIONS.contains(&ext) {
                handles.push(tokio::task::spawn_blocking(move || {
                    fmt_handle_entry(&entry, fmt.check, fmt.quiet, &config, &error_out);
                }));
            }
        } else if let Some(file_name) = entry.file_name().to_str() {
            if file_name.starts_with('.') {
                // We'll just ignore all dot files
            }
        } else {
            warn!(
                "Found file without extension: {}",
                try_rm_prefix(entry.path()).display()
            );
        };
    }

    for handle in handles {
        handle.await.unwrap();
    }

    unsafe { *error_out.as_ptr() }
}

fn fmt_handle_entry(
    entry: &DirEntry,
    check: bool,
    quiet: bool,
    config: &Arc<Configuration>,
    error_out: &Arc<AtomicBool>,
) {
    if !quiet {
        info!("Processing: {}", try_rm_prefix(entry.path()).display());
    }

    if !entry.path().is_file() {
        return;
    }

    match fs::read_to_string(entry.path()) {
        Ok(text) => {
            if text.trim().is_empty() {
                warn!(
                    "Found empty file at {}",
                    try_rm_prefix(entry.path()).display()
                )
            } else if check {
                match fmt_check(entry.path(), &text, config) {
                    Ok(_) => {}
                    Err(Ok(diff)) => {
                        error!(
                            "Difference found in file {}:",
                            try_rm_prefix(entry.path()).display()
                        );
                        println!("{diff}");
                        unsafe { *error_out.as_ptr() = true };
                    }
                    Err(Err(err)) => {
                        error!(
                            "An unexpected Error occurred while trying to check {}",
                            try_rm_prefix(entry.path()).display()
                        );
                        println!("Err: {err}");
                        unsafe { *error_out.as_ptr() = true };
                    }
                };
            } else {
                match fmt_reformat(entry.path(), &text, config) {
                    Ok(Some(text)) => {
                        info!("Reformating file {}", try_rm_prefix(entry.path()).display());

                        fs::write(entry.path(), text).unwrap_or_else(|err| {
                            error!(
                                "An unexpected Error occurred while trying to write {}",
                                try_rm_prefix(entry.path()).display()
                            );
                            println!("Err: {err}");
                            unsafe { *error_out.as_ptr() = true };
                        })
                    }
                    Ok(None) => {}
                    Err(err) => {
                        error!(
                            "An unexpected Error occurred while trying to reformat {}",
                            try_rm_prefix(entry.path()).display()
                        );
                        println!("Err: {err}");
                        unsafe { *error_out.as_ptr() = true };
                    }
                };
            }
        }
        Err(err) => {
            error!(
                "An unexpected Error occurred while trying to read {}",
                try_rm_prefix(entry.path()).display()
            );
            println!("Err: {err}");
            unsafe { *error_out.as_ptr() = true };
        }
    }
}

fn fmt_reformat(path: &Path, text: &str, config: &Configuration) -> Result<Option<String>, String> {
    match format_biome(path, text, config) {
        Ok(text) => Ok(text),
        Err(err) => Err(format!("{err}")),
    }
}

fn fmt_check(path: &Path, old: &str, config: &Configuration) -> Result<(), Result<String, String>> {
    match format_biome(path, old, config) {
        Ok(Some(new)) => {
            let diff = TextDiff::from_lines(old, &new);

            let mut string = String::new();

            for op in diff.ops() {
                for change in diff.iter_changes(op) {
                    let (sign, style) = match change.tag() {
                        ChangeTag::Delete => ("-", Style::new().red()),
                        ChangeTag::Insert => ("+", Style::new().green()),
                        ChangeTag::Equal => (" ", Style::new()),
                    };
                    string += &format!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
                }
            }

            Err(Ok(string))
        }
        Ok(None) => Ok(()),
        Err(err) => Err(Err(format!("{err}"))),
    }
}
