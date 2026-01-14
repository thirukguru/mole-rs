//! Security module - path validation and protection

use std::path::{Path, PathBuf};
use std::os::unix::fs::MetadataExt;

/// Critical system paths that should NEVER be deleted
/// These form an "Iron Dome" around the system
pub const BLOCKED_PATHS: &[&str] = &[
    // Root filesystem
    "/",
    
    // Core system directories (Linux)
    "/bin",
    "/boot",
    "/dev",
    "/etc",
    "/lib",
    "/lib32",
    "/lib64",
    "/libx32",
    "/proc",
    "/root",
    "/run",
    "/sbin",
    "/srv",
    "/sys",
    "/usr",
    "/var",
    
    // Critical /var subdirectories
    "/var/lib",
    "/var/log",
    "/var/run",
    
    // Ubuntu specific
    "/snap/core",
    "/snap/snapd",
];

/// Paths that require extra caution (warning but allowed)
pub const CAUTION_PATHS: &[&str] = &[
    "/opt",
    "/home",
    "/tmp",
    "/var/tmp",
    "/var/cache",
];

/// Validation result for path operations
#[derive(Debug, Clone, PartialEq)]
pub enum PathValidation {
    /// Path is safe to delete
    Safe,
    /// Path is blocked - never delete
    Blocked { reason: String },
    /// Path requires caution - warn user
    Caution { reason: String },
    /// Path is a symlink - needs special handling
    Symlink { target: PathBuf },
    /// Path validation failed
    Invalid { reason: String },
}

/// Security validator for filesystem operations
pub struct SecurityValidator {
    /// User-defined whitelist (protected paths)
    whitelist: Vec<PathBuf>,
    /// Maximum size for automatic deletion (bytes)
    large_deletion_threshold: u64,
    /// Whether to allow symlink following
    allow_symlinks: bool,
}

impl SecurityValidator {
    /// Create a new security validator
    pub fn new() -> Self {
        Self {
            whitelist: Self::load_whitelist(),
            large_deletion_threshold: 1024 * 1024 * 1024, // 1GB
            allow_symlinks: false,
        }
    }

    /// Load whitelist from config file
    fn load_whitelist() -> Vec<PathBuf> {
        let whitelist_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mole-rs")
            .join("whitelist");

        if !whitelist_path.exists() {
            return Vec::new();
        }

        std::fs::read_to_string(&whitelist_path)
            .unwrap_or_default()
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .map(|line| {
                let expanded = if line.starts_with('~') {
                    dirs::home_dir()
                        .map(|h| h.join(&line[2..]))
                        .unwrap_or_else(|| PathBuf::from(line))
                } else {
                    PathBuf::from(line)
                };
                expanded
            })
            .collect()
    }

    /// Validate a path before deletion
    pub fn validate_path(&self, path: &Path) -> PathValidation {
        // Check for empty path
        if path.as_os_str().is_empty() {
            return PathValidation::Invalid {
                reason: "Empty path".to_string(),
            };
        }

        // Require absolute paths
        if !path.is_absolute() {
            return PathValidation::Invalid {
                reason: "Path must be absolute".to_string(),
            };
        }

        // Check for path traversal attempts
        let path_str = path.to_string_lossy();
        if path_str.contains("/../") || path_str.ends_with("/..") {
            return PathValidation::Invalid {
                reason: "Path traversal detected".to_string(),
            };
        }

        // Check against blocked paths
        for blocked in BLOCKED_PATHS {
            if path_str == *blocked || path_str.starts_with(&format!("{}/", blocked)) {
                // Special exception: Allow cleaning specific cache subdirectories
                if self.is_safe_cache_subdir(path) {
                    continue;
                }
                return PathValidation::Blocked {
                    reason: format!("System path protected: {}", blocked),
                };
            }
        }

        // Check if path is whitelisted (user protected)
        if self.is_whitelisted(path) {
            return PathValidation::Blocked {
                reason: "Path is whitelisted by user".to_string(),
            };
        }

        // Check for symlinks
        if let Ok(metadata) = std::fs::symlink_metadata(path) {
            if metadata.file_type().is_symlink() {
                if let Ok(target) = std::fs::read_link(path) {
                    return PathValidation::Symlink { target };
                }
            }
        }

        // Check against caution paths
        for caution in CAUTION_PATHS {
            if path_str == *caution {
                return PathValidation::Caution {
                    reason: format!("Deleting {} requires confirmation", caution),
                };
            }
        }

        PathValidation::Safe
    }

    /// Check if path is in user's whitelist
    pub fn is_whitelisted(&self, path: &Path) -> bool {
        self.whitelist.iter().any(|w| path.starts_with(w))
    }

    /// Check if path is a safe cache subdirectory
    fn is_safe_cache_subdir(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // Allow specific cache directories
        let safe_patterns = [
            "/var/cache/apt/archives",
            "/var/cache/apt/pkgcache.bin",
            "/var/cache/apt/srcpkgcache.bin",
        ];

        safe_patterns.iter().any(|p| path_str.starts_with(p))
    }

    /// Check if deletion exceeds size threshold
    pub fn is_large_deletion(&self, size: u64) -> bool {
        size >= self.large_deletion_threshold
    }

    /// Validate path for deletion with sudo
    pub fn validate_sudo_operation(&self, path: &Path) -> PathValidation {
        let base_validation = self.validate_path(path);

        // Additional checks for sudo operations
        if base_validation != PathValidation::Safe {
            return base_validation;
        }

        // Verify path is not a symlink pointing to system directory
        if let Ok(canonical) = path.canonicalize() {
            let canonical_str = canonical.to_string_lossy();
            for blocked in BLOCKED_PATHS {
                if canonical_str == *blocked || canonical_str.starts_with(&format!("{}/", blocked)) {
                    if !self.is_safe_cache_subdir(&canonical) {
                        return PathValidation::Blocked {
                            reason: format!("Symlink resolves to protected path: {}", canonical_str),
                        };
                    }
                }
            }
        }

        PathValidation::Safe
    }

    /// Check if running as root (for security warnings)
    pub fn is_running_as_root() -> bool {
        unsafe { libc::geteuid() == 0 }
    }

    /// Validate that we're operating in user's home directory
    pub fn is_in_home_directory(path: &Path) -> bool {
        if let Some(home) = dirs::home_dir() {
            path.starts_with(&home)
        } else {
            false
        }
    }
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Check for potentially dangerous characters in path
pub fn contains_dangerous_chars(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Check for control characters
    path_str.chars().any(|c| c.is_control()) ||
    // Check for null byte
    path_str.contains('\0') ||
    // Check for newline (command injection risk)
    path_str.contains('\n') || path_str.contains('\r')
}

/// Sanitize path by removing dangerous components
pub fn sanitize_path(path: &Path) -> Option<PathBuf> {
    if contains_dangerous_chars(path) {
        return None;
    }

    // Normalize the path
    let mut components = Vec::new();
    for component in path.components() {
        use std::path::Component;
        match component {
            Component::ParentDir => {
                // Don't allow .. to escape beyond root
                if !components.is_empty() {
                    components.pop();
                }
            }
            Component::Normal(c) => {
                components.push(c);
            }
            Component::RootDir => {
                components.clear();
                components.push(std::ffi::OsStr::new("/"));
            }
            _ => {}
        }
    }

    if components.is_empty() {
        return None;
    }

    let mut result = PathBuf::new();
    for c in components {
        result.push(c);
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_paths() {
        let validator = SecurityValidator::new();
        
        assert!(matches!(
            validator.validate_path(Path::new("/")),
            PathValidation::Blocked { .. }
        ));
        assert!(matches!(
            validator.validate_path(Path::new("/etc/passwd")),
            PathValidation::Blocked { .. }
        ));
        assert!(matches!(
            validator.validate_path(Path::new("/usr/bin/ls")),
            PathValidation::Blocked { .. }
        ));
    }

    #[test]
    fn test_relative_path_rejected() {
        let validator = SecurityValidator::new();
        
        assert!(matches!(
            validator.validate_path(Path::new("../etc/passwd")),
            PathValidation::Invalid { .. }
        ));
        assert!(matches!(
            validator.validate_path(Path::new("relative/path")),
            PathValidation::Invalid { .. }
        ));
    }

    #[test]
    fn test_path_traversal_rejected() {
        let validator = SecurityValidator::new();
        
        assert!(matches!(
            validator.validate_path(Path::new("/home/user/../etc/passwd")),
            PathValidation::Invalid { .. }
        ));
    }

    #[test]
    fn test_safe_path() {
        let validator = SecurityValidator::new();
        let home = dirs::home_dir().unwrap();
        let safe_path = home.join(".cache/test");
        
        assert!(matches!(
            validator.validate_path(&safe_path),
            PathValidation::Safe
        ));
    }

    #[test]
    fn test_dangerous_chars() {
        assert!(contains_dangerous_chars(Path::new("/path/with\nnewline")));
        assert!(contains_dangerous_chars(Path::new("/path/with\0null")));
        assert!(!contains_dangerous_chars(Path::new("/normal/path")));
    }

    #[test]
    fn test_large_deletion_threshold() {
        let validator = SecurityValidator::new();
        
        assert!(!validator.is_large_deletion(500 * 1024 * 1024)); // 500MB
        assert!(validator.is_large_deletion(2 * 1024 * 1024 * 1024)); // 2GB
    }
}
