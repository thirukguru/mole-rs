//! Linux distribution detection

use std::fs;
use std::path::Path;

/// Supported Linux distributions
#[derive(Debug, Clone, PartialEq)]
pub enum Distro {
    Ubuntu,
    Debian,
    Fedora,
    CentOS,
    RHEL,
    Arch,
    Manjaro,
    OpenSUSE,
    Alpine,
    Gentoo,
    Unknown(String),
}

impl std::fmt::Display for Distro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Distro::Ubuntu => write!(f, "Ubuntu"),
            Distro::Debian => write!(f, "Debian"),
            Distro::Fedora => write!(f, "Fedora"),
            Distro::CentOS => write!(f, "CentOS"),
            Distro::RHEL => write!(f, "Red Hat Enterprise Linux"),
            Distro::Arch => write!(f, "Arch Linux"),
            Distro::Manjaro => write!(f, "Manjaro"),
            Distro::OpenSUSE => write!(f, "openSUSE"),
            Distro::Alpine => write!(f, "Alpine Linux"),
            Distro::Gentoo => write!(f, "Gentoo"),
            Distro::Unknown(name) => write!(f, "{}", name),
        }
    }
}

/// Package manager type
#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    Apt,      // Debian, Ubuntu
    Dnf,      // Fedora, RHEL 8+
    Yum,      // CentOS, RHEL 7
    Pacman,   // Arch, Manjaro
    Zypper,   // openSUSE
    Apk,      // Alpine
    Portage,  // Gentoo
    Unknown,
}

impl PackageManager {
    /// Get the clean cache command for this package manager
    pub fn clean_cache_cmd(&self) -> Option<Vec<&'static str>> {
        match self {
            PackageManager::Apt => Some(vec!["apt-get", "clean"]),
            PackageManager::Dnf => Some(vec!["dnf", "clean", "all"]),
            PackageManager::Yum => Some(vec!["yum", "clean", "all"]),
            PackageManager::Pacman => Some(vec!["pacman", "-Sc", "--noconfirm"]),
            PackageManager::Zypper => Some(vec!["zypper", "clean", "--all"]),
            PackageManager::Apk => Some(vec!["apk", "cache", "clean"]),
            PackageManager::Portage => None, // Complex, skip for now
            PackageManager::Unknown => None,
        }
    }

    /// Get the autoremove command for this package manager
    pub fn autoremove_cmd(&self) -> Option<Vec<&'static str>> {
        match self {
            PackageManager::Apt => Some(vec!["apt-get", "autoremove", "-y"]),
            PackageManager::Dnf => Some(vec!["dnf", "autoremove", "-y"]),
            PackageManager::Yum => Some(vec!["yum", "autoremove", "-y"]),
            PackageManager::Pacman => Some(vec!["pacman", "-Rns", "$(pacman -Qdtq)", "--noconfirm"]),
            PackageManager::Zypper => None, // No direct equivalent
            PackageManager::Apk => None,
            PackageManager::Portage => Some(vec!["emerge", "--depclean"]),
            PackageManager::Unknown => None,
        }
    }

    /// Get the list installed packages command
    pub fn list_packages_cmd(&self) -> Option<Vec<&'static str>> {
        match self {
            PackageManager::Apt => Some(vec!["dpkg-query", "-W", "-f", "${Package}\t${Installed-Size}\n"]),
            PackageManager::Dnf | PackageManager::Yum => Some(vec!["rpm", "-qa", "--queryformat", "%{NAME}\t%{SIZE}\n"]),
            PackageManager::Pacman => Some(vec!["pacman", "-Q"]),
            PackageManager::Zypper => Some(vec!["rpm", "-qa"]),
            PackageManager::Apk => Some(vec!["apk", "list", "--installed"]),
            PackageManager::Portage => Some(vec!["qlist", "-I"]),
            PackageManager::Unknown => None,
        }
    }

    /// Get cache directories for this package manager
    pub fn cache_paths(&self) -> Vec<&'static str> {
        match self {
            PackageManager::Apt => vec!["/var/cache/apt/archives"],
            PackageManager::Dnf => vec!["/var/cache/dnf"],
            PackageManager::Yum => vec!["/var/cache/yum"],
            PackageManager::Pacman => vec!["/var/cache/pacman/pkg"],
            PackageManager::Zypper => vec!["/var/cache/zypp"],
            PackageManager::Apk => vec!["/var/cache/apk"],
            PackageManager::Portage => vec!["/var/cache/distfiles"],
            PackageManager::Unknown => vec![],
        }
    }
}

/// System information including distro and package manager
#[derive(Debug, Clone)]
pub struct DistroInfo {
    pub distro: Distro,
    pub version: Option<String>,
    pub package_manager: PackageManager,
    pub has_snap: bool,
    pub has_flatpak: bool,
}

impl DistroInfo {
    /// Detect the current Linux distribution
    pub fn detect() -> Self {
        let (distro, version) = detect_distro();
        let package_manager = detect_package_manager(&distro);
        let has_snap = command_exists("snap");
        let has_flatpak = command_exists("flatpak");

        Self {
            distro,
            version,
            package_manager,
            has_snap,
            has_flatpak,
        }
    }

    /// Check if this is a Debian-based distro
    pub fn is_debian_based(&self) -> bool {
        matches!(self.distro, Distro::Ubuntu | Distro::Debian)
    }

    /// Check if this is a Red Hat-based distro
    pub fn is_redhat_based(&self) -> bool {
        matches!(self.distro, Distro::Fedora | Distro::CentOS | Distro::RHEL)
    }

    /// Check if this is an Arch-based distro
    pub fn is_arch_based(&self) -> bool {
        matches!(self.distro, Distro::Arch | Distro::Manjaro)
    }
}

/// Detect the Linux distribution from /etc/os-release
fn detect_distro() -> (Distro, Option<String>) {
    // Try /etc/os-release first (most modern distros)
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        return parse_os_release(&content);
    }

    // Fallback to /etc/lsb-release (older Ubuntu)
    if let Ok(content) = fs::read_to_string("/etc/lsb-release") {
        if content.contains("Ubuntu") {
            let version = extract_value(&content, "DISTRIB_RELEASE");
            return (Distro::Ubuntu, version);
        }
    }

    // Check for specific files
    if Path::new("/etc/debian_version").exists() {
        return (Distro::Debian, None);
    }
    if Path::new("/etc/fedora-release").exists() {
        return (Distro::Fedora, None);
    }
    if Path::new("/etc/arch-release").exists() {
        return (Distro::Arch, None);
    }

    (Distro::Unknown("Linux".to_string()), None)
}

/// Parse /etc/os-release content
fn parse_os_release(content: &str) -> (Distro, Option<String>) {
    let id = extract_value(content, "ID").unwrap_or_default().to_lowercase();
    let version = extract_value(content, "VERSION_ID");

    let distro = match id.as_str() {
        "ubuntu" => Distro::Ubuntu,
        "debian" => Distro::Debian,
        "fedora" => Distro::Fedora,
        "centos" => Distro::CentOS,
        "rhel" => Distro::RHEL,
        "arch" => Distro::Arch,
        "manjaro" => Distro::Manjaro,
        "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" => Distro::OpenSUSE,
        "alpine" => Distro::Alpine,
        "gentoo" => Distro::Gentoo,
        _ => {
            let name = extract_value(content, "NAME").unwrap_or_else(|| id.clone());
            Distro::Unknown(name)
        }
    };

    (distro, version)
}

/// Extract a value from key=value format
fn extract_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with(key) {
            if let Some(value) = line.split('=').nth(1) {
                return Some(value.trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Detect the package manager based on distro or available commands
fn detect_package_manager(distro: &Distro) -> PackageManager {
    match distro {
        Distro::Ubuntu | Distro::Debian => PackageManager::Apt,
        Distro::Fedora => PackageManager::Dnf,
        Distro::CentOS | Distro::RHEL => {
            if command_exists("dnf") {
                PackageManager::Dnf
            } else {
                PackageManager::Yum
            }
        }
        Distro::Arch | Distro::Manjaro => PackageManager::Pacman,
        Distro::OpenSUSE => PackageManager::Zypper,
        Distro::Alpine => PackageManager::Apk,
        Distro::Gentoo => PackageManager::Portage,
        Distro::Unknown(_) => {
            // Try to detect based on available commands
            if command_exists("apt-get") {
                PackageManager::Apt
            } else if command_exists("dnf") {
                PackageManager::Dnf
            } else if command_exists("yum") {
                PackageManager::Yum
            } else if command_exists("pacman") {
                PackageManager::Pacman
            } else if command_exists("zypper") {
                PackageManager::Zypper
            } else if command_exists("apk") {
                PackageManager::Apk
            } else {
                PackageManager::Unknown
            }
        }
    }
}

/// Check if a command exists
pub fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_distro() {
        let info = DistroInfo::detect();
        // Should detect something
        println!("Detected: {} ({:?})", info.distro, info.package_manager);
    }

    #[test]
    fn test_parse_ubuntu_os_release() {
        let content = r#"
NAME="Ubuntu"
VERSION="22.04.3 LTS (Jammy Jellyfish)"
ID=ubuntu
VERSION_ID="22.04"
"#;
        let (distro, version) = parse_os_release(content);
        assert_eq!(distro, Distro::Ubuntu);
        assert_eq!(version, Some("22.04".to_string()));
    }

    #[test]
    fn test_parse_fedora_os_release() {
        let content = r#"
NAME="Fedora Linux"
VERSION="39 (Workstation Edition)"
ID=fedora
VERSION_ID=39
"#;
        let (distro, version) = parse_os_release(content);
        assert_eq!(distro, Distro::Fedora);
        assert_eq!(version, Some("39".to_string()));
    }

    #[test]
    fn test_package_manager_commands() {
        let apt = PackageManager::Apt;
        assert!(apt.clean_cache_cmd().is_some());
        assert!(apt.autoremove_cmd().is_some());

        let dnf = PackageManager::Dnf;
        assert!(dnf.clean_cache_cmd().is_some());
    }
}
