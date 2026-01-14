//! Status command - live system monitoring

use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use std::time::Duration;

use crate::core::filesystem::format_size;
use crate::core::system::SystemInfo;

/// Run the status command (non-TUI version)
pub fn run() -> Result<()> {
    let mut sysinfo = SystemInfo::new();

    // Clear screen and hide cursor
    print!("\x1B[2J\x1B[H");
    print!("\x1B[?25l");
    io::stdout().flush()?;

    // Setup Ctrl+C handler
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })
    .ok();

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        sysinfo.refresh();
        render_status(&sysinfo);
        std::thread::sleep(Duration::from_secs(1));
    }

    // Show cursor on exit
    print!("\x1B[?25h");
    io::stdout().flush()?;

    Ok(())
}

fn render_status(sysinfo: &SystemInfo) {
    // Move to top-left
    print!("\x1B[H");

    let width = 60;

    // Header
    println!(
        "{}",
        format!("  Mole-RS Status {:>width$}", sysinfo.hostname(), width = width - 18)
            .bold()
            .on_bright_black()
    );
    println!("{}", "─".repeat(width));

    // CPU
    let cpu_usage = sysinfo.cpu_usage();
    let cpu_bar = progress_bar(cpu_usage as f64, 20);
    println!(
        "  {} {} {:>5.1}%",
        "CPU".bold(),
        cpu_bar,
        cpu_usage
    );

    // Load average
    let (l1, l5, l15) = sysinfo.load_average();
    println!(
        "  {}  {:.2} / {:.2} / {:.2}",
        "Load".dimmed(),
        l1,
        l5,
        l15
    );

    println!();

    // Memory
    let mem_usage = sysinfo.memory_usage();
    let mem_bar = progress_bar(mem_usage as f64, 20);
    let used_mem = format_size(sysinfo.used_memory());
    let total_mem = format_size(sysinfo.total_memory());
    println!(
        "  {} {} {:>5.1}%",
        "Memory".bold(),
        mem_bar,
        mem_usage
    );
    println!(
        "  {}  {} / {}",
        "     ".dimmed(),
        used_mem,
        total_mem
    );

    println!();

    // Disks
    println!("  {}", "Disks".bold());
    for disk in sysinfo.disk_info() {
        if disk.mount_point == "/" || disk.mount_point.starts_with("/home") {
            let usage = disk.usage_percent();
            let bar = progress_bar(usage as f64, 15);
            let used = format_size(disk.used_space());
            let total = format_size(disk.total_space);
            println!(
                "   {:10} {} {:>5.1}%  {} / {}",
                disk.mount_point,
                bar,
                usage,
                used,
                total
            );
        }
    }

    println!();

    // Network I/O
    let (rx, tx) = sysinfo.network_io();
    println!(
        "  {} ↓ {}  ↑ {}",
        "Network".bold(),
        format_size(rx),
        format_size(tx)
    );

    println!();

    // Top processes
    println!("  {} {:>15} {:>10}", "Top Processes".bold(), "CPU%", "Memory");
    for proc in sysinfo.top_processes_by_cpu(5) {
        let name = if proc.name.len() > 15 {
            format!("{}...", &proc.name[..12])
        } else {
            proc.name.clone()
        };
        println!(
            "   {:<15} {:>14.1} {:>10}",
            name,
            proc.cpu_usage,
            format_size(proc.memory)
        );
    }

    println!();

    // Uptime
    let uptime = sysinfo.uptime();
    let days = uptime / 86400;
    let hours = (uptime % 86400) / 3600;
    let mins = (uptime % 3600) / 60;
    println!(
        "  {} {}d {}h {}m",
        "Uptime".dimmed(),
        days,
        hours,
        mins
    );

    println!();
    println!("  {}", "Press Ctrl+C to exit".dimmed());

    io::stdout().flush().ok();
}

fn progress_bar(percent: f64, width: usize) -> String {
    let filled = ((percent / 100.0) * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    if percent > 90.0 {
        bar.red().to_string()
    } else if percent > 70.0 {
        bar.yellow().to_string()
    } else {
        bar.green().to_string()
    }
}
