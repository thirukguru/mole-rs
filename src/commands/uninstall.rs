//! Uninstall command - remove apps and their leftover files

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::core::filesystem::{dir_size, format_size, safe_delete};

/// Installed application info
#[derive(Debug, Clone)]
pub struct InstalledApp {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub app_type: AppType,
    pub leftovers: Vec<LeftoverFile>,
}

/// Type of application
#[derive(Debug, Clone, PartialEq)]
pub enum AppType {
    Deb,        // Installed via apt/dpkg
    Snap,       // Installed via snap
    Flatpak,    // Installed via flatpak
    AppImage,   // AppImage file
    Manual,     // Manually installed (in /opt, ~/.local/bin, etc.)
}

impl std::fmt::Display for AppType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppType::Deb => write!(f, "deb"),
            AppType::Snap => write!(f, "snap"),
            AppType::Flatpak => write!(f, "flatpak"),
            AppType::AppImage => write!(f, "AppImage"),
            AppType::Manual => write!(f, "manual"),
        }
    }
}

/// Leftover file from an uninstalled app
#[derive(Debug, Clone)]
pub struct LeftoverFile {
    pub path: PathBuf,
    pub file_type: LeftoverType,
    pub size: u64,
}

/// Type of leftover file
#[derive(Debug, Clone)]
pub enum LeftoverType {
    Config,      // Configuration files
    Cache,       // Cache files
    Data,        // Application data
    Log,         // Log files
    Desktop,     // Desktop entries
    Autostart,   // Autostart entries
}

impl std::fmt::Display for LeftoverType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeftoverType::Config => write!(f, "config"),
            LeftoverType::Cache => write!(f, "cache"),
            LeftoverType::Data => write!(f, "data"),
            LeftoverType::Log => write!(f, "log"),
            LeftoverType::Desktop => write!(f, "desktop"),
            LeftoverType::Autostart => write!(f, "autostart"),
        }
    }
}

/// Common leftover locations for Ubuntu
fn get_leftover_locations() -> Vec<(PathBuf, LeftoverType)> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    
    vec![
        // Config locations
        (home.join(".config"), LeftoverType::Config),
        (PathBuf::from("/etc"), LeftoverType::Config),
        
        // Cache locations
        (home.join(".cache"), LeftoverType::Cache),
        (PathBuf::from("/var/cache"), LeftoverType::Cache),
        
        // Data locations
        (home.join(".local/share"), LeftoverType::Data),
        (PathBuf::from("/var/lib"), LeftoverType::Data),
        
        // Log locations
        (home.join(".local/share"), LeftoverType::Log),
        (PathBuf::from("/var/log"), LeftoverType::Log),
        
        // Desktop entries
        (home.join(".local/share/applications"), LeftoverType::Desktop),
        (PathBuf::from("/usr/share/applications"), LeftoverType::Desktop),
        
        // Autostart
        (home.join(".config/autostart"), LeftoverType::Autostart),
    ]
}

/// Scan for installed packages (deb only for now)
pub fn scan_installed_apps() -> Result<Vec<InstalledApp>> {
    let mut apps = Vec::new();
    
    // Scan dpkg installed packages
    apps.extend(scan_dpkg_apps()?);
    
    // Scan snap packages
    apps.extend(scan_snap_apps()?);
    
    // Scan flatpak packages
    apps.extend(scan_flatpak_apps()?);
    
    // Sort by name
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    
    Ok(apps)
}

/// Scan dpkg installed packages
fn scan_dpkg_apps() -> Result<Vec<InstalledApp>> {
    let output = std::process::Command::new("dpkg-query")
        .args(["-W", "-f", "${Package}\t${Installed-Size}\n"])
        .output();
    
    let mut apps = Vec::new();
    
    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let size_kb: u64 = parts[1].parse().unwrap_or(0);
                    
                    apps.push(InstalledApp {
                        name: name.clone(),
                        path: PathBuf::from(format!("/var/lib/dpkg/info/{}.list", name)),
                        size: size_kb * 1024, // Convert KB to bytes
                        app_type: AppType::Deb,
                        leftovers: Vec::new(),
                    });
                }
            }
        }
    }
    
    Ok(apps)
}

/// Scan snap packages
fn scan_snap_apps() -> Result<Vec<InstalledApp>> {
    let output = std::process::Command::new("snap")
        .args(["list"])
        .output();
    
    let mut apps = Vec::new();
    
    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for (i, line) in stdout.lines().enumerate() {
                if i == 0 { continue; } // Skip header
                
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    let name = parts[0].to_string();
                    let snap_path = PathBuf::from(format!("/snap/{}", name));
                    let size = dir_size(&snap_path).unwrap_or(0);
                    
                    apps.push(InstalledApp {
                        name,
                        path: snap_path,
                        size,
                        app_type: AppType::Snap,
                        leftovers: Vec::new(),
                    });
                }
            }
        }
    }
    
    Ok(apps)
}

/// Scan flatpak packages
fn scan_flatpak_apps() -> Result<Vec<InstalledApp>> {
    let output = std::process::Command::new("flatpak")
        .args(["list", "--app", "--columns=application,name,size"])
        .output();
    
    let mut apps = Vec::new();
    
    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    let app_id = parts[0].to_string();
                    let name = parts.get(1).unwrap_or(&parts[0]).to_string();
                    
                    apps.push(InstalledApp {
                        name,
                        path: PathBuf::from(format!("/var/lib/flatpak/app/{}", app_id)),
                        size: 0, // Would need to calculate
                        app_type: AppType::Flatpak,
                        leftovers: Vec::new(),
                    });
                }
            }
        }
    }
    
    Ok(apps)
}

/// Find leftover files for a given app name
pub fn find_leftovers(app_name: &str) -> Vec<LeftoverFile> {
    let mut leftovers = Vec::new();
    let locations = get_leftover_locations();
    
    // Normalize app name for matching
    let normalized = normalize_app_name(app_name);
    let patterns = generate_search_patterns(&normalized);
    
    for (base_path, file_type) in locations {
        if !base_path.exists() {
            continue;
        }
        
        // Search only first level to avoid deep recursion
        if let Ok(entries) = std::fs::read_dir(&base_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let entry_name = entry.file_name().to_string_lossy().to_lowercase();
                
                for pattern in &patterns {
                    if entry_name.contains(pattern) {
                        let path = entry.path();
                        let size = dir_size(&path).unwrap_or(0);
                        
                        leftovers.push(LeftoverFile {
                            path,
                            file_type: file_type.clone(),
                            size,
                        });
                        break;
                    }
                }
            }
        }
    }
    
    leftovers
}

/// Normalize app name for matching
fn normalize_app_name(name: &str) -> String {
    name.to_lowercase()
        .replace('-', "")
        .replace('_', "")
        .replace(' ', "")
}

/// Generate search patterns from app name
fn generate_search_patterns(normalized: &str) -> Vec<String> {
    let mut patterns = vec![normalized.to_string()];
    
    // Add common variations
    if normalized.len() > 3 {
        // First 5 chars if long enough
        if normalized.len() >= 5 {
            patterns.push(normalized[..5].to_string());
        }
    }
    
    patterns
}

/// Uninstall an app based on its type
pub fn uninstall_app(app: &InstalledApp, dry_run: bool, remove_leftovers: bool) -> Result<u64> {
    let mut freed = 0u64;
    
    println!();
    println!(
        "Uninstalling {} ({})...",
        app.name.bold(),
        app.app_type.to_string().dimmed()
    );
    
    if dry_run {
        println!("  {} Would remove app", "→".cyan());
    } else {
        // Uninstall based on type
        let result = match app.app_type {
            AppType::Deb => uninstall_deb(&app.name),
            AppType::Snap => uninstall_snap(&app.name),
            AppType::Flatpak => uninstall_flatpak(&app.name),
            AppType::AppImage => uninstall_appimage(&app.path),
            AppType::Manual => uninstall_manual(&app.path),
        };
        
        match result {
            Ok(_) => {
                println!("  {} Removed app", "✓".green());
                freed += app.size;
            }
            Err(e) => {
                println!("  {} Failed: {}", "✗".red(), e);
            }
        }
    }
    
    // Handle leftovers
    if remove_leftovers {
        let leftovers = find_leftovers(&app.name);
        
        if !leftovers.is_empty() {
            println!("  {} Found {} leftover locations", "→".cyan(), leftovers.len());
            
            for leftover in &leftovers {
                if dry_run {
                    println!(
                        "    {} Would remove {} ({})",
                        "→".dimmed(),
                        leftover.path.display(),
                        format_size(leftover.size).yellow()
                    );
                    freed += leftover.size;
                } else {
                    match safe_delete(&leftover.path, false) {
                        Ok(size) => {
                            println!(
                                "    {} Removed {} ({})",
                                "✓".green(),
                                leftover.path.display(),
                                format_size(size)
                            );
                            freed += size;
                        }
                        Err(e) => {
                            println!(
                                "    {} Failed {}: {}",
                                "✗".red(),
                                leftover.path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }
    }
    
    Ok(freed)
}

fn uninstall_deb(name: &str) -> Result<()> {
    let status = std::process::Command::new("sudo")
        .args(["apt-get", "remove", "-y", name])
        .status()?;
    
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("apt-get remove failed"))
    }
}

fn uninstall_snap(name: &str) -> Result<()> {
    let status = std::process::Command::new("sudo")
        .args(["snap", "remove", name])
        .status()?;
    
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("snap remove failed"))
    }
}

fn uninstall_flatpak(name: &str) -> Result<()> {
    let status = std::process::Command::new("flatpak")
        .args(["uninstall", "-y", name])
        .status()?;
    
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("flatpak uninstall failed"))
    }
}

fn uninstall_appimage(path: &Path) -> Result<()> {
    std::fs::remove_file(path)?;
    Ok(())
}

fn uninstall_manual(path: &Path) -> Result<()> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Run the uninstall command
pub fn run(app_name: Option<String>, dry_run: bool, list_only: bool) -> Result<()> {
    println!("{}", "Mole-RS Uninstall".bold().cyan());
    println!("{}", "═".repeat(50));
    println!();
    
    if list_only {
        // Just list installed apps
        println!("{}", "Scanning installed applications...".dimmed());
        
        let apps = scan_installed_apps()?;
        
        println!();
        println!("Found {} installed packages:", apps.len().to_string().bold());
        println!();
        
        // Group by type
        let mut by_type: HashMap<String, Vec<&InstalledApp>> = HashMap::new();
        for app in &apps {
            by_type
                .entry(app.app_type.to_string())
                .or_default()
                .push(app);
        }
        
        for (app_type, type_apps) in &by_type {
            println!("  {} ({}):", app_type.bold(), type_apps.len());
            for app in type_apps.iter().take(10) {
                println!(
                    "    {} {} {}",
                    "•".dimmed(),
                    app.name,
                    format_size(app.size).dimmed()
                );
            }
            if type_apps.len() > 10 {
                println!("    {} ... and {} more", "".dimmed(), type_apps.len() - 10);
            }
            println!();
        }
        
        return Ok(());
    }
    
    if let Some(name) = app_name {
        // Uninstall specific app
        println!("Searching for '{}'...", name.yellow());
        
        let apps = scan_installed_apps()?;
        let matching: Vec<_> = apps
            .iter()
            .filter(|a| a.name.to_lowercase().contains(&name.to_lowercase()))
            .collect();
        
        if matching.is_empty() {
            println!("{}", "No matching applications found.".yellow());
            return Ok(());
        }
        
        println!();
        println!("Found {} matching apps:", matching.len());
        
        let mut total_freed = 0u64;
        
        for app in matching {
            total_freed += uninstall_app(app, dry_run, true)?;
        }
        
        println!();
        println!("{}", "═".repeat(50));
        
        if dry_run {
            println!(
                "{}: {} (dry-run)",
                "Would free".bold(),
                format_size(total_freed).green().bold()
            );
        } else {
            println!(
                "{}: {}",
                "Space freed".bold(),
                format_size(total_freed).green().bold()
            );
        }
    } else {
        println!("{}", "Usage:".bold());
        println!("  mo uninstall <app-name>     Uninstall an app");
        println!("  mo uninstall --list         List installed apps");
        println!("  mo uninstall <name> --dry-run  Preview uninstall");
    }
    
    Ok(())
}
