//! Filesystem operations with safety checks

use crate::core::errors::{MoleError, Result};
use crate::core::security::{SecurityValidator, PathValidation};
use std::path::Path;
use walkdir::WalkDir;

/// Calculate the size of a directory recursively
pub fn dir_size(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }

    let mut total = 0u64;

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }

    Ok(total)
}

/// Format bytes into human-readable string
pub fn format_size(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}

/// Check if we have permission to delete a path
pub fn can_delete(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    // Check if we can write to parent directory
    if let Some(parent) = path.parent() {
        if let Ok(metadata) = std::fs::metadata(parent) {
            return !metadata.permissions().readonly();
        }
    }

    false
}

/// Check if running as root
pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Safely delete a file or directory with security validation
pub fn safe_delete(path: &Path, dry_run: bool) -> Result<u64> {
    // Security validation
    let validator = SecurityValidator::new();
    
    match validator.validate_path(path) {
        PathValidation::Safe => {}
        PathValidation::Blocked { reason } => {
            return Err(MoleError::PermissionDenied {
                path: format!("{}: {}", path.display(), reason),
            });
        }
        PathValidation::Caution { reason } => {
            // Log warning but proceed
            tracing::warn!("Caution: {} - {}", path.display(), reason);
        }
        PathValidation::Symlink { target } => {
            // For symlinks, validate the target too
            if is_root() {
                match validator.validate_path(&target) {
                    PathValidation::Blocked { reason } => {
                        return Err(MoleError::PermissionDenied {
                            path: format!("Symlink target blocked: {}", reason),
                        });
                    }
                    _ => {}
                }
            }
        }
        PathValidation::Invalid { reason } => {
            return Err(MoleError::Other(format!("Invalid path: {}", reason)));
        }
    }

    if !path.exists() {
        return Ok(0);
    }

    let size = dir_size(path)?;

    // Check for large deletion
    if validator.is_large_deletion(size) && !dry_run {
        tracing::warn!(
            "Large deletion: {} ({} bytes). Proceeding with caution.",
            path.display(),
            size
        );
    }

    if dry_run {
        return Ok(size);
    }

    if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                MoleError::PermissionDenied {
                    path: path.display().to_string(),
                }
            } else {
                MoleError::Io(e)
            }
        })?;
    } else {
        std::fs::remove_file(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                MoleError::PermissionDenied {
                    path: path.display().to_string(),
                }
            } else {
                MoleError::Io(e)
            }
        })?;
    }

    Ok(size)
}

/// Delete contents of a directory but keep the directory itself
pub fn clean_directory(path: &Path, dry_run: bool) -> Result<u64> {
    if !path.exists() || !path.is_dir() {
        return Ok(0);
    }

    // Validate the parent directory first
    let validator = SecurityValidator::new();
    match validator.validate_path(path) {
        PathValidation::Blocked { reason } => {
            return Err(MoleError::PermissionDenied {
                path: format!("{}: {}", path.display(), reason),
            });
        }
        _ => {}
    }

    let mut total_freed = 0u64;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        
        // Validate each entry before deletion
        match validator.validate_path(&entry_path) {
            PathValidation::Safe | PathValidation::Caution { .. } => {
                total_freed += safe_delete(&entry_path, dry_run)?;
            }
            PathValidation::Blocked { reason } => {
                tracing::debug!("Skipping blocked path: {} - {}", entry_path.display(), reason);
            }
            PathValidation::Symlink { target } => {
                // Skip symlinks to protected paths
                if let PathValidation::Blocked { .. } = validator.validate_path(&target) {
                    tracing::debug!("Skipping symlink to protected path: {}", entry_path.display());
                    continue;
                }
                total_freed += safe_delete(&entry_path, dry_run)?;
            }
            PathValidation::Invalid { reason } => {
                tracing::debug!("Skipping invalid path: {} - {}", entry_path.display(), reason);
            }
        }
    }

    Ok(total_freed)
}

/// Count files in a directory
pub fn count_files(path: &Path) -> usize {
    if !path.exists() {
        return 0;
    }

    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count()
}

/// Check if a path is a symlink
pub fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

/// Get the target of a symlink
pub fn symlink_target(path: &Path) -> Option<std::path::PathBuf> {
    if is_symlink(path) {
        std::fs::read_link(path).ok()
    } else {
        None
    }
}
