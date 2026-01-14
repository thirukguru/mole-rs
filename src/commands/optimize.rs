//! Optimize command - system maintenance

use anyhow::Result;
use colored::Colorize;
use std::process::Command;

use crate::core::filesystem::is_root;

/// Optimization task
struct OptimizeTask {
    name: &'static str,
    description: &'static str,
    requires_sudo: bool,
    command: Option<(&'static str, Vec<&'static str>)>,
    action: Option<fn() -> Result<()>>,
}

/// Run the optimize command
pub fn run(dry_run: bool) -> Result<()> {
    println!("{}", "Mole-RS System Optimize".bold().cyan());
    println!("{}", "═".repeat(50));
    println!();

    let is_sudo = is_root();

    let tasks = vec![
        OptimizeTask {
            name: "Clear thumbnail cache",
            description: "Remove cached thumbnails",
            requires_sudo: false,
            command: None,
            action: Some(clear_thumbnails),
        },
        OptimizeTask {
            name: "Update font cache",
            description: "Rebuild font cache",
            requires_sudo: false,
            command: Some(("fc-cache", vec!["-f"])),
            action: None,
        },
        OptimizeTask {
            name: "Clear APT cache",
            description: "Remove downloaded package files",
            requires_sudo: true,
            command: Some(("apt-get", vec!["clean"])),
            action: None,
        },
        OptimizeTask {
            name: "Remove orphan packages",
            description: "Remove unused dependencies",
            requires_sudo: true,
            command: Some(("apt-get", vec!["autoremove", "-y"])),
            action: None,
        },
        OptimizeTask {
            name: "Vacuum journal logs",
            description: "Limit journal size to 100M",
            requires_sudo: true,
            command: Some(("journalctl", vec!["--vacuum-size=100M"])),
            action: None,
        },
    ];

    let available_tasks: Vec<_> = tasks
        .iter()
        .filter(|t| !t.requires_sudo || is_sudo)
        .collect();

    if available_tasks.is_empty() {
        println!("{}", "No optimization tasks available.".yellow());
        println!(
            "{}",
            "Run with sudo for system-level optimizations.".dimmed()
        );
        return Ok(());
    }

    println!("{}", "Optimization tasks:".bold());
    println!();

    for task in &available_tasks {
        let sudo_marker = if task.requires_sudo { " [sudo]" } else { "" };
        println!(
            "  {} {} {}",
            "→".cyan(),
            task.name.bold(),
            sudo_marker.dimmed()
        );
        println!("    {}", task.description.dimmed());
    }

    println!();

    if dry_run {
        println!("{}", "[DRY RUN] No changes were made.".yellow().bold());
        return Ok(());
    }

    // Execute tasks
    println!("{}", "Running optimizations...".dimmed());
    println!();

    for task in &available_tasks {
        print!("  {} {}... ", "→".cyan(), task.name);

        let result = if let Some((cmd, args)) = &task.command {
            run_command(cmd, args)
        } else if let Some(action) = task.action {
            action()
        } else {
            Ok(())
        };

        match result {
            Ok(_) => println!("{}", "done".green()),
            Err(e) => println!("{} {}", "failed:".red(), e),
        }
    }

    println!();
    println!("{}", "═".repeat(50));
    println!("{}", "System optimization completed.".green().bold());

    if !is_sudo {
        println!();
        println!(
            "{}",
            "Tip: Run with sudo for additional optimizations.".dimmed()
        );
    }

    Ok(())
}

fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(cmd).args(args).output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn clear_thumbnails() -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let thumb_dir = home.join(".cache/thumbnails");

    if thumb_dir.exists() {
        std::fs::remove_dir_all(&thumb_dir)?;
        std::fs::create_dir_all(&thumb_dir)?;
    }

    Ok(())
}
