//! Purge command - clean development artifacts

use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::core::config::Config;
use crate::core::filesystem::{format_size, safe_delete};
use crate::core::paths::DevArtifacts;

/// Found artifact with metadata
#[derive(Debug)]
pub struct FoundArtifact {
    pub project_name: String,
    pub artifact_type: String,
    pub path: PathBuf,
    pub size: u64,
    pub age_days: u64,
    pub selected: bool,
}

/// Scan for development artifacts
pub fn scan_artifacts(paths: &[PathBuf]) -> Vec<FoundArtifact> {
    let patterns = DevArtifacts::new();
    let mut artifacts = Vec::new();

    for scan_path in paths {
        if !scan_path.exists() {
            continue;
        }

        for entry in WalkDir::new(scan_path)
            .max_depth(4)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_dir() {
                continue;
            }

            let dir_name = entry.file_name().to_string_lossy();

            for pattern in &patterns.patterns {
                if dir_name == pattern.dir_name {
                    // Check if parent has marker file
                    if let Some(parent) = entry.path().parent() {
                        let has_marker = pattern.marker_files.is_empty()
                            || pattern
                                .marker_files
                                .iter()
                                .any(|m| parent.join(m).exists());

                        if has_marker {
                            let size = calculate_size(entry.path());
                            let age = calculate_age(entry.path());

                            let project_name = parent
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            artifacts.push(FoundArtifact {
                                project_name,
                                artifact_type: pattern.name.to_string(),
                                path: entry.path().to_path_buf(),
                                size,
                                age_days: age,
                                selected: age > 7, // Select old artifacts by default
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by size descending
    artifacts.sort_by(|a, b| b.size.cmp(&a.size));

    artifacts
}

fn calculate_size(path: &std::path::Path) -> u64 {
    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

fn calculate_age(path: &std::path::Path) -> u64 {
    path.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0)
}

/// Run the purge command
pub fn run(paths: Option<Vec<PathBuf>>, dry_run: bool) -> Result<()> {
    println!("{}", "Mole-RS Project Purge".bold().cyan());
    println!("{}", "═".repeat(60));
    println!();

    let config = Config::load();
    let scan_paths = paths.unwrap_or(config.project_paths);

    println!("{}", "Scanning for development artifacts...".dimmed());
    println!();

    let artifacts = scan_artifacts(&scan_paths);

    if artifacts.is_empty() {
        println!("{}", "No development artifacts found.".yellow());
        return Ok(());
    }

    let total_size: u64 = artifacts.iter().filter(|a| a.selected).map(|a| a.size).sum();
    let selected_count = artifacts.iter().filter(|a| a.selected).count();

    println!("{}", "Found artifacts:".bold());
    println!();

    for artifact in &artifacts {
        let marker = if artifact.selected { "●" } else { "○" };
        let marker_color = if artifact.selected {
            marker.green()
        } else {
            marker.dimmed()
        };

        let age_str = if artifact.age_days == 0 {
            "Today".to_string()
        } else if artifact.age_days == 1 {
            "1 day".to_string()
        } else {
            format!("{} days", artifact.age_days)
        };

        let age_colored = if artifact.age_days < 7 {
            age_str.yellow()
        } else {
            age_str.dimmed()
        };

        println!(
            " {} {:<20} {:>10} | {} | {}",
            marker_color,
            artifact.project_name.bold(),
            format_size(artifact.size).yellow(),
            artifact.artifact_type.dimmed(),
            age_colored
        );
    }

    println!();
    println!(
        "Selected: {} artifacts, {}",
        selected_count.to_string().bold(),
        format_size(total_size).green().bold()
    );
    println!();

    if dry_run {
        println!("{}", "[DRY RUN] No files were deleted.".yellow().bold());
        return Ok(());
    }

    // Perform deletion
    println!("{}", "Cleaning selected artifacts...".dimmed());

    let mut freed = 0u64;
    for artifact in artifacts.iter().filter(|a| a.selected) {
        match safe_delete(&artifact.path, false) {
            Ok(size) => {
                freed += size;
                println!("  {} Removed {}", "✓".green(), artifact.project_name);
            }
            Err(e) => {
                println!("  {} Failed {}: {}", "✗".red(), artifact.project_name, e);
            }
        }
    }

    println!();
    println!("{}", "═".repeat(60));
    println!(
        "{}: {}",
        "Space freed".bold(),
        format_size(freed).green().bold()
    );

    Ok(())
}
