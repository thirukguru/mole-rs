//! Optimize command - system maintenance with distro detection

use anyhow::Result;
use colored::Colorize;
use std::process::Command;

use crate::core::distro::{DistroInfo, PackageManager};
use crate::core::filesystem::is_root;

/// Optimization task
struct OptimizeTask {
    name: String,
    description: String,
    requires_sudo: bool,
    command: Option<(String, Vec<String>)>,
    action: Option<fn() -> Result<()>>,
}

/// Run the optimize command
pub fn run(dry_run: bool) -> Result<()> {
    let distro = DistroInfo::detect();
    
    println!("{}", "Mole-RS System Optimize".bold().cyan());
    println!("{}", "═".repeat(50));
    println!();
    println!(
        "Detected: {} ({})",
        distro.distro.to_string().green(),
        format!("{:?}", distro.package_manager).dimmed()
    );
    println!();

    let is_sudo = is_root();
    let tasks = build_tasks(&distro);

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
            run_command(cmd, &args.iter().map(|s| s.as_str()).collect::<Vec<_>>())
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

/// Build tasks based on detected distro
fn build_tasks(distro: &DistroInfo) -> Vec<OptimizeTask> {
    let mut tasks = Vec::new();

    // Universal tasks
    tasks.push(OptimizeTask {
        name: "Clear thumbnail cache".to_string(),
        description: "Remove cached thumbnails".to_string(),
        requires_sudo: false,
        command: None,
        action: Some(clear_thumbnails),
    });

    tasks.push(OptimizeTask {
        name: "Update font cache".to_string(),
        description: "Rebuild font cache".to_string(),
        requires_sudo: false,
        command: Some(("fc-cache".to_string(), vec!["-f".to_string()])),
        action: None,
    });

    // Package manager specific tasks
    if let Some(cmd) = distro.package_manager.clean_cache_cmd() {
        tasks.push(OptimizeTask {
            name: format!("Clear {} cache", format!("{:?}", distro.package_manager)),
            description: "Remove downloaded package files".to_string(),
            requires_sudo: true,
            command: Some((cmd[0].to_string(), cmd[1..].iter().map(|s| s.to_string()).collect())),
            action: None,
        });
    }

    if let Some(cmd) = distro.package_manager.autoremove_cmd() {
        // Skip pacman's complex command for now
        if distro.package_manager != PackageManager::Pacman {
            tasks.push(OptimizeTask {
                name: "Remove orphan packages".to_string(),
                description: "Remove unused dependencies".to_string(),
                requires_sudo: true,
                command: Some((cmd[0].to_string(), cmd[1..].iter().map(|s| s.to_string()).collect())),
                action: None,
            });
        }
    }

    // Journal cleanup (systemd-based distros)
    if std::path::Path::new("/usr/bin/journalctl").exists() {
        tasks.push(OptimizeTask {
            name: "Vacuum journal logs".to_string(),
            description: "Limit journal size to 100M".to_string(),
            requires_sudo: true,
            command: Some(("journalctl".to_string(), vec!["--vacuum-size=100M".to_string()])),
            action: None,
        });
    }

    // Snap cleanup (if available)
    if distro.has_snap {
        tasks.push(OptimizeTask {
            name: "Clean old snap revisions".to_string(),
            description: "Remove disabled snap versions".to_string(),
            requires_sudo: true,
            command: None,
            action: Some(clean_old_snaps),
        });
    }

    // Flatpak cleanup (if available)
    if distro.has_flatpak {
        tasks.push(OptimizeTask {
            name: "Clean unused Flatpak runtimes".to_string(),
            description: "Remove unused Flatpak dependencies".to_string(),
            requires_sudo: false,
            command: Some(("flatpak".to_string(), vec!["uninstall".to_string(), "--unused".to_string(), "-y".to_string()])),
            action: None,
        });
    }

    tasks
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

fn clean_old_snaps() -> Result<()> {
    // List disabled snaps and remove them
    let output = Command::new("snap")
        .args(["list", "--all"])
        .output()?;

    if !output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 && parts[5] == "disabled" {
            let name = parts[0];
            let revision = parts[2];
            let _ = Command::new("sudo")
                .args(["snap", "remove", name, "--revision", revision])
                .output();
        }
    }

    Ok(())
}
