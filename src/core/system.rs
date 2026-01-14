//! System information wrapper using sysinfo

use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, System, RefreshKind};

/// System information snapshot
#[derive(Debug)]
pub struct SystemInfo {
    system: System,
    disks: Disks,
    networks: Networks,
}

impl SystemInfo {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system,
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
        }
    }

    /// Refresh all system information
    pub fn refresh(&mut self) {
        self.system.refresh_cpu_specifics(CpuRefreshKind::everything());
        self.system.refresh_memory_specifics(MemoryRefreshKind::everything());
        self.system.refresh_processes();
        self.disks.refresh();
        self.networks.refresh();
    }

    /// Get CPU usage percentage (0-100)
    pub fn cpu_usage(&self) -> f32 {
        let cpus = self.system.cpus();
        if cpus.is_empty() {
            return 0.0;
        }
        cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32
    }

    /// Get per-core CPU usage
    pub fn cpu_per_core(&self) -> Vec<f32> {
        self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect()
    }

    /// Get total memory in bytes
    pub fn total_memory(&self) -> u64 {
        self.system.total_memory()
    }

    /// Get used memory in bytes
    pub fn used_memory(&self) -> u64 {
        self.system.used_memory()
    }

    /// Get memory usage percentage
    pub fn memory_usage(&self) -> f32 {
        let total = self.total_memory() as f32;
        if total == 0.0 {
            return 0.0;
        }
        (self.used_memory() as f32 / total) * 100.0
    }

    /// Get disk information
    pub fn disk_info(&self) -> Vec<DiskInfo> {
        self.disks
            .iter()
            .map(|disk| DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
                file_system: String::from_utf8_lossy(disk.file_system().as_encoded_bytes()).to_string(),
            })
            .collect()
    }

    /// Get network I/O
    pub fn network_io(&self) -> (u64, u64) {
        let mut received = 0u64;
        let mut transmitted = 0u64;

        for (_name, data) in self.networks.iter() {
            received += data.received();
            transmitted += data.transmitted();
        }

        (received, transmitted)
    }

    /// Get system uptime in seconds
    pub fn uptime(&self) -> u64 {
        System::uptime()
    }

    /// Get load average (1, 5, 15 minutes)
    pub fn load_average(&self) -> (f64, f64, f64) {
        let load = System::load_average();
        (load.one, load.five, load.fifteen)
    }

    /// Get top processes by CPU usage
    pub fn top_processes_by_cpu(&self, limit: usize) -> Vec<ProcessInfo> {
        let mut processes: Vec<_> = self
            .system
            .processes()
            .values()
            .map(|p| ProcessInfo {
                name: p.name().to_string(),
                cpu_usage: p.cpu_usage(),
                memory: p.memory(),
            })
            .collect();

        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        processes.truncate(limit);
        processes
    }

    /// Get hostname
    pub fn hostname(&self) -> String {
        System::host_name().unwrap_or_else(|| "unknown".to_string())
    }

    /// Get OS name and version
    pub fn os_info(&self) -> String {
        format!(
            "{} {}",
            System::name().unwrap_or_else(|| "Unknown".to_string()),
            System::os_version().unwrap_or_else(|| "".to_string())
        )
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub file_system: String,
}

impl DiskInfo {
    pub fn used_space(&self) -> u64 {
        self.total_space.saturating_sub(self.available_space)
    }

    pub fn usage_percent(&self) -> f32 {
        if self.total_space == 0 {
            return 0.0;
        }
        (self.used_space() as f32 / self.total_space as f32) * 100.0
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
}
