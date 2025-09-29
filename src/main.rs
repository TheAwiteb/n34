// n34 - A CLI to interact with NIP-34 and other stuff related to code in Nostr
// Copyright (C) 2025 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://gnu.org/licenses/gpl-3.0.html>.

/// Command line interface module
pub mod cli;
/// N34 errors
pub mod error;
/// Nostr keyring
pub mod nostr_keyring;
/// Nostr utils module
pub mod nostr_utils;

use std::{
    fs::File,
    process::ExitCode,
    sync::atomic::{AtomicBool, Ordering},
};

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{Layer, filter, layer::SubscriberExt};

use self::cli::Cli;

/// Whether the editor is currently open. Prevents logging while the editor is
/// open.
static EDITOR_OPEN: AtomicBool = AtomicBool::new(false);

/// Returns the stderr log layer
fn stderr_log_layer<S>() -> impl Layer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_writer(std::io::stderr)
        .without_time()
}

/// Configures the logging level based on the provided verbosity.
///
/// When verbosity is set to TRACE, includes file and line numbers in logs.
fn set_log_level(verbosity: Verbosity, logs_file: File) {
    let editor_filter = filter::dynamic_filter_fn(move |m, _| {
        // Disable all logs while editor is open
        verbosity.tracing_level().unwrap_or(Level::ERROR) >= *m.level()
            && (m.name().starts_with("event src") || m.name().contains("nostr"))
            && !EDITOR_OPEN.load(Ordering::Relaxed)
    });

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(logs_file)
        .with_file(true)
        .with_line_number(true)
        .with_filter(LevelFilter::TRACE);

    let subscriber = tracing_subscriber::registry()
        .with(stderr_log_layer().with_filter(editor_filter))
        .with(file_layer);
    tracing::subscriber::set_global_default(subscriber).ok();
}

async fn try_main() -> error::N34Result<()> {
    // Initialize a thread-local subscriber for logging during CLI parsing and
    // post-processing.
    let guard =
        tracing::subscriber::set_default(tracing_subscriber::registry().with(stderr_log_layer()));

    let cli = cli::post_cli(Cli::parse()).await?;
    let logs_file = cli::utils::logs_file()?;

    // Replace the thread-local subscriber with a global default subscriber based on
    // the CLI verbosity level.
    drop(guard);
    set_log_level(cli.verbosity, logs_file);

    cli.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = try_main().await {
        eprintln!("{err}");
        return err.exit_code();
    }

    ExitCode::SUCCESS
}
