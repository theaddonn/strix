use crate::args::{CliInput, CliSubCommand};
use crate::fmt::fmt;
use crate::new::new;
use chrono::Local;
use clap::Parser;
use fern::colors::{Color, ColoredLevelConfig};
use log::info;
use std::process::exit;
use tokio::runtime::Builder;
use tokio::time::Instant;

mod args;
mod config;
mod fmt;
mod new;

fn setup_logger() {
    let colors = ColoredLevelConfig::new()
        .debug(Color::Magenta)
        .info(Color::Cyan)
        .warn(Color::Yellow)
        .error(Color::Red);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                Local::now().format("%H:%M:%S%.3f"),
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout())
        .apply()
        .unwrap_or_else(|err| {
            eprintln!("An unexpected Error occurred while trying to setup the logger, Err: {err}");
            exit(1);
        });
}

fn main() {
    let num_threads = num_cpus::get();

    let runtime = Builder::new_multi_thread()
        .worker_threads(num_threads)
        .enable_all()
        .build()
        .unwrap_or_else(|err| {
            eprintln!(
                "An unexpected Error occurred while trying to setup the tokio runtime, Err: {err}"
            );
            exit(1);
        });

    runtime.block_on(tokio_main());
}

async fn tokio_main() {
    setup_logger();
    let start = Instant::now();

    let args = CliInput::parse();

    let error = match args.command {
        CliSubCommand::New(v) => new(v).await,
        CliSubCommand::Build(_) => false,
        CliSubCommand::Fmt(v) => fmt(v).await,
    };

    info!(
        "Finished in {}",
        humantime::format_duration(Instant::now().duration_since(start)).to_string()
    );

    if error {
        exit(1);
    }
}
