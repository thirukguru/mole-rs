//! Analyze command - disk usage visualization

use anyhow::Result;
use colored::Colorize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::core::filesystem::format_size;

/// Directory entry with size info
#[derive(Debug)]
pub struct DirEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

/// Scan a directory and get sorted entries by size
pub fn scan_directory(path: &Path, _depth: u32) -> Result<Vec<DirEntry>> {
    let mut entries = Vec::new();

    if !path.exists() {
        return Ok(entries);
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        let size = if metadata.is_dir() {
            calculate_dir_size(&path)
        } else {
            metadata.len()
        };

        entries.push(DirEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: path.clone(),
            size,
            is_dir: metadata.is_dir(),
        });
    }

    // Sort by size descending
    entries.sort_by(|a, b| b.size.cmp(&a.size));

    Ok(entries)
}

fn calculate_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

/// Run the analyze command
pub fn run(path: String) -> Result<()> {
    let path = PathBuf::from(&path);

    println!("{}", "Mole-RS Disk Analyzer".bold().cyan());
    println!("{}", "═".repeat(60));
    println!();
    println!("Analyzing: {}", path.display().to_string().yellow());
    println!();

    let entries = scan_directory(&path, 0)?;

    if entries.is_empty() {
        println!("{}", "No files found.".dimmed());
        return Ok(());
    }

    let total_size: u64 = entries.iter().map(|e| e.size).sum();

    // Display entries with bar visualization
    for (i, entry) in entries.iter().take(20).enumerate() {
        let percent = if total_size > 0 {
            (entry.size as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        let bar_width: usize = 20;
        let filled = ((percent / 100.0) * bar_width as f64) as usize;
        let bar = format!(
            "{}{}",
            "█".repeat(filled),
            "░".repeat(bar_width.saturating_sub(filled))
        );

        let icon = if entry.is_dir { "📁" } else { "📄" };
        let size_str = format_size(entry.size);

        let name = if entry.name.len() > 30 {
            format!("{}...", &entry.name[..27])
        } else {
            entry.name.clone()
        };

        let bar_colored = if percent > 30.0 {
            bar.red()
        } else if percent > 15.0 {
            bar.yellow()
        } else {
            bar.green()
        };

        println!(
            " {:2}. {} {:>5.1}% {} {:<30} {:>10}",
            i + 1,
            bar_colored,
            percent,
            icon,
            name,
            size_str.yellow()
        );
    }

    if entries.len() > 20 {
        println!();
        println!(
            "  {} {} more items...",
            "...".dimmed(),
            entries.len() - 20
        );
    }

    println!();
    println!("{}", "═".repeat(60));
    println!(
        "Total: {} ({} items)",
        format_size(total_size).green().bold(),
        entries.len()
    );

    Ok(())
}
