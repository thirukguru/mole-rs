//! Clean command - system cache cleanup

use anyhow::Result;
use colored::Colorize;

use crate::core::filesystem::{clean_directory, dir_size, format_size, is_root};
use crate::core::CleanupPaths;

/// Cleanup category with size information
#[derive(Debug)]
pub struct CleanupCategory {
    pub name: String,
    pub path: std::path::PathBuf,
    pub size: u64,
    pub requires_sudo: bool,
    pub selected: bool,
}

/// Scan all cleanup categories and calculate sizes
pub fn scan_categories() -> Vec<CleanupCategory> {
    let paths = CleanupPaths::new();
    let is_sudo = is_root();

    let mut categories = Vec::new();

    // User caches (no sudo needed)
    for (name, path) in paths.user_caches() {
        if path.exists() {
            let size = dir_size(path).unwrap_or(0);
            if size > 0 {
                categories.push(CleanupCategory {
                    name: name.to_string(),
                    path: path.clone(),
                    size,
                    requires_sudo: false,
                    selected: true,
                });
            }
        }
    }

    // System caches (require sudo)
    if is_sudo {
        for (name, path) in paths.system_caches() {
            if path.exists() {
                let size = dir_size(path).unwrap_or(0);
                if size > 0 {
                    categories.push(CleanupCategory {
                        name: name.to_string(),
                        path: path.clone(),
                        size,
                        requires_sudo: true,
                        selected: true,
                    });
                }
            }
        }
    }

    // Sort by size (largest first)
    categories.sort_by(|a, b| b.size.cmp(&a.size));

    categories
}

/// Run the clean command
pub fn run(dry_run: bool, debug: bool) -> Result<()> {
    println!("{}", "Mole-RS Clean".bold().cyan());
    println!("{}", "═".repeat(50));
    println!();

    println!("{}", "Scanning cache directories...".dimmed());
    let categories = scan_categories();

    if categories.is_empty() {
        println!("{}", "No caches found to clean.".yellow());
        return Ok(());
    }

    let total_size: u64 = categories.iter().map(|c| c.size).sum();

    println!();
    println!("{}", "Found cleanup targets:".bold());
    println!();

    for cat in &categories {
        let size_str = format_size(cat.size);
        let sudo_marker = if cat.requires_sudo { " [sudo]" } else { "" };

        if debug {
            println!(
                "  {} {} {} {}",
                "✓".green(),
                cat.name.bold(),
                size_str.yellow(),
                cat.path.display().to_string().dimmed()
            );
        } else {
            println!(
                "  {} {} {}{}",
                "✓".green(),
                cat.name.bold(),
                size_str.yellow(),
                sudo_marker.dimmed()
            );
        }
    }

    println!();
    println!(
        "{}: {}",
        "Total space to free".bold(),
        format_size(total_size).green().bold()
    );
    println!();

    if dry_run {
        println!("{}", "[DRY RUN] No files were deleted.".yellow().bold());
        return Ok(());
    }

    // Perform cleanup
    println!("{}", "Cleaning...".dimmed());

    let mut freed = 0u64;

    for cat in &categories {
        match clean_directory(&cat.path, false) {
            Ok(size) => {
                freed += size;
                println!("  {} Cleaned {}", "✓".green(), cat.name);
            }
            Err(e) => {
                println!("  {} Failed {}: {}", "✗".red(), cat.name, e);
            }
        }
    }

    println!();
    println!("{}", "═".repeat(50));
    println!(
        "{}: {}",
        "Space freed".bold(),
        format_size(freed).green().bold()
    );

    Ok(())
}
