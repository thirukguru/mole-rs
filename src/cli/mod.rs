//! CLI argument parsing using clap

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Mole-RS: Deep clean and optimize your Ubuntu system
#[derive(Parser, Debug)]
#[command(name = "mo")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    /// Enable debug output
    #[arg(long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Deep system cleanup - free up disk space
    Clean {
        /// Preview changes without deleting
        #[arg(long)]
        dry_run: bool,

        /// Show detailed debug information
        #[arg(long)]
        debug: bool,
    },

    /// Analyze disk usage with visual breakdown
    Analyze {
        /// Path to analyze (defaults to home directory)
        #[arg(default_value_t = default_analyze_path())]
        path: String,
    },

    /// Monitor live system status
    Status,

    /// Clean development project artifacts
    Purge {
        /// Directories to scan for projects
        #[arg(long, value_delimiter = ',')]
        paths: Option<Vec<PathBuf>>,

        /// Preview changes without deleting
        #[arg(long)]
        dry_run: bool,
    },

    /// System optimization and maintenance
    Optimize {
        /// Preview changes without executing
        #[arg(long)]
        dry_run: bool,
    },
}

fn default_analyze_path() -> String {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string())
}
