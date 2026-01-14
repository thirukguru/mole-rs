//! Unit tests for core modules

#[cfg(test)]
mod tests {
    use super::*;

    mod filesystem_tests {
        use crate::core::filesystem::*;
        use std::fs::{self, File};
        use std::io::Write;
        use tempfile::TempDir;

        #[test]
        fn test_format_size_bytes() {
            assert_eq!(format_size(0), "0 B");
            assert_eq!(format_size(100), "100 B");
            assert_eq!(format_size(1023), "1023 B");
        }

        #[test]
        fn test_format_size_kilobytes() {
            assert_eq!(format_size(1024), "1 KiB");
            assert_eq!(format_size(2048), "2 KiB");
        }

        #[test]
        fn test_format_size_megabytes() {
            assert_eq!(format_size(1024 * 1024), "1 MiB");
            assert_eq!(format_size(5 * 1024 * 1024), "5 MiB");
        }

        #[test]
        fn test_format_size_gigabytes() {
            assert_eq!(format_size(1024 * 1024 * 1024), "1 GiB");
        }

        #[test]
        fn test_dir_size_empty() {
            let temp = TempDir::new().unwrap();
            let size = dir_size(temp.path()).unwrap();
            assert_eq!(size, 0);
        }

        #[test]
        fn test_dir_size_with_files() {
            let temp = TempDir::new().unwrap();
            
            // Create a file with known size
            let file_path = temp.path().join("test.txt");
            let mut file = File::create(&file_path).unwrap();
            file.write_all(b"Hello, World!").unwrap(); // 13 bytes
            
            let size = dir_size(temp.path()).unwrap();
            assert_eq!(size, 13);
        }

        #[test]
        fn test_dir_size_nested() {
            let temp = TempDir::new().unwrap();
            
            // Create nested directory
            let nested = temp.path().join("subdir");
            fs::create_dir(&nested).unwrap();
            
            // Create files in both directories
            let mut file1 = File::create(temp.path().join("file1.txt")).unwrap();
            file1.write_all(b"12345").unwrap(); // 5 bytes
            
            let mut file2 = File::create(nested.join("file2.txt")).unwrap();
            file2.write_all(b"67890").unwrap(); // 5 bytes
            
            let size = dir_size(temp.path()).unwrap();
            assert_eq!(size, 10);
        }

        #[test]
        fn test_dir_size_nonexistent() {
            let size = dir_size(std::path::Path::new("/nonexistent/path")).unwrap();
            assert_eq!(size, 0);
        }

        #[test]
        fn test_safe_delete_file() {
            let temp = TempDir::new().unwrap();
            let file_path = temp.path().join("delete_me.txt");
            
            let mut file = File::create(&file_path).unwrap();
            file.write_all(b"delete this").unwrap();
            
            assert!(file_path.exists());
            
            let freed = safe_delete(&file_path, false).unwrap();
            assert_eq!(freed, 11); // "delete this" = 11 bytes
            assert!(!file_path.exists());
        }

        #[test]
        fn test_safe_delete_dry_run() {
            let temp = TempDir::new().unwrap();
            let file_path = temp.path().join("keep_me.txt");
            
            let mut file = File::create(&file_path).unwrap();
            file.write_all(b"keep this").unwrap();
            
            let freed = safe_delete(&file_path, true).unwrap();
            assert_eq!(freed, 9); // "keep this" = 9 bytes
            assert!(file_path.exists()); // File should still exist
        }

        #[test]
        fn test_clean_directory() {
            let temp = TempDir::new().unwrap();
            
            // Create some files
            File::create(temp.path().join("file1.txt")).unwrap();
            File::create(temp.path().join("file2.txt")).unwrap();
            
            let subdir = temp.path().join("subdir");
            fs::create_dir(&subdir).unwrap();
            File::create(subdir.join("file3.txt")).unwrap();
            
            clean_directory(temp.path(), false).unwrap();
            
            // Directory should still exist but be empty
            assert!(temp.path().exists());
            assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
        }

        #[test]
        fn test_is_root() {
            // This test will pass on non-root systems
            let is_sudo = is_root();
            // We can't assert a specific value, but we can ensure it doesn't panic
            assert!(is_sudo == true || is_sudo == false);
        }
    }

    mod paths_tests {
        use crate::core::paths::*;

        #[test]
        fn test_cleanup_paths_new() {
            let paths = CleanupPaths::new();
            
            // System paths should be absolute
            assert!(paths.apt_cache.is_absolute());
            assert!(paths.system_logs.is_absolute());
            
            // User paths should contain home directory
            assert!(paths.user_cache.to_string_lossy().contains(".cache"));
            assert!(paths.trash.to_string_lossy().contains("Trash"));
        }

        #[test]
        fn test_user_caches_not_empty() {
            let paths = CleanupPaths::new();
            let user_caches = paths.user_caches();
            
            assert!(!user_caches.is_empty());
            assert!(user_caches.len() >= 5);
        }

        #[test]
        fn test_system_caches_not_empty() {
            let paths = CleanupPaths::new();
            let system_caches = paths.system_caches();
            
            assert!(!system_caches.is_empty());
            assert!(system_caches.len() >= 4);
        }

        #[test]
        fn test_dev_artifacts_patterns() {
            let artifacts = DevArtifacts::new();
            
            // Should have common patterns
            let dir_names: Vec<_> = artifacts.patterns.iter().map(|p| p.dir_name).collect();
            
            assert!(dir_names.contains(&"node_modules"));
            assert!(dir_names.contains(&"target"));
            assert!(dir_names.contains(&"venv"));
        }
    }

    mod config_tests {
        use crate::core::config::*;

        #[test]
        fn test_config_default() {
            let config = Config::default();
            
            assert!(config.whitelist.is_empty());
            assert!(!config.project_paths.is_empty());
            assert_eq!(config.skip_recent_days, 7);
            assert_eq!(config.journal_max_size, "100M");
        }

        #[test]
        fn test_config_path() {
            let path = Config::config_path();
            
            assert!(path.to_string_lossy().contains("mole-rs"));
            assert!(path.to_string_lossy().contains("config.toml"));
        }

        #[test]
        fn test_config_load_default() {
            // If no config file exists, should return defaults
            let config = Config::load();
            
            assert_eq!(config.skip_recent_days, 7);
        }
    }

    mod system_tests {
        use crate::core::system::*;

        #[test]
        fn test_system_info_new() {
            let sysinfo = SystemInfo::new();
            
            // Should return valid values
            assert!(sysinfo.total_memory() > 0);
        }

        #[test]
        fn test_cpu_usage_range() {
            let sysinfo = SystemInfo::new();
            let cpu = sysinfo.cpu_usage();
            
            // CPU usage should be between 0 and 100
            assert!(cpu >= 0.0);
            assert!(cpu <= 100.0);
        }

        #[test]
        fn test_memory_usage_range() {
            let sysinfo = SystemInfo::new();
            let mem = sysinfo.memory_usage();
            
            // Memory usage should be between 0 and 100
            assert!(mem >= 0.0);
            assert!(mem <= 100.0);
        }

        #[test]
        fn test_used_memory_less_than_total() {
            let sysinfo = SystemInfo::new();
            
            assert!(sysinfo.used_memory() <= sysinfo.total_memory());
        }

        #[test]
        fn test_disk_info_not_empty() {
            let sysinfo = SystemInfo::new();
            let disks = sysinfo.disk_info();
            
            // Should have at least one disk
            assert!(!disks.is_empty());
        }

        #[test]
        fn test_disk_usage_percent() {
            let sysinfo = SystemInfo::new();
            
            for disk in sysinfo.disk_info() {
                let usage = disk.usage_percent();
                assert!(usage >= 0.0);
                assert!(usage <= 100.0);
            }
        }

        #[test]
        fn test_uptime() {
            let sysinfo = SystemInfo::new();
            let uptime = sysinfo.uptime();
            
            // Uptime should be positive
            assert!(uptime > 0);
        }

        #[test]
        fn test_load_average() {
            let sysinfo = SystemInfo::new();
            let (l1, l5, l15) = sysinfo.load_average();
            
            // Load averages should be non-negative
            assert!(l1 >= 0.0);
            assert!(l5 >= 0.0);
            assert!(l15 >= 0.0);
        }

        #[test]
        fn test_hostname() {
            let sysinfo = SystemInfo::new();
            let hostname = sysinfo.hostname();
            
            assert!(!hostname.is_empty());
        }

        #[test]
        fn test_top_processes() {
            let sysinfo = SystemInfo::new();
            let procs = sysinfo.top_processes_by_cpu(5);
            
            // Should return at most 5 processes
            assert!(procs.len() <= 5);
        }
    }
}
