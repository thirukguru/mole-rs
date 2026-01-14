//! Mole-RS: Deep clean and optimize your Ubuntu system
//!
//! A Rust-based system cleanup tool inspired by tw93/Mole

use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod core;
mod tui;

use cli::Args;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    match args.command {
        Some(cli::Command::Clean { dry_run, debug }) => {
            commands::clean::run(dry_run, debug)?;
        }
        Some(cli::Command::Analyze { path }) => {
            commands::analyze::run(path)?;
        }
        Some(cli::Command::Status) => {
            commands::status::run()?;
        }
        Some(cli::Command::Purge { paths, dry_run }) => {
            commands::purge::run(paths, dry_run)?;
        }
        Some(cli::Command::Optimize { dry_run }) => {
            commands::optimize::run(dry_run)?;
        }
        None => {
            // Launch interactive TUI
            tui::run()?;
        }
    }

    Ok(())
}
