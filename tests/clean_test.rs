//! Tests for the clean command functionality

use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

/// Create a mock cache structure for testing
fn create_mock_cache() -> TempDir {
    let temp = TempDir::new().unwrap();
    
    // Create cache subdirectories
    let cache_dir = temp.path().join(".cache");
    fs::create_dir_all(&cache_dir).unwrap();
    
    // Create some cache files
    let mut file = File::create(cache_dir.join("cache1.tmp")).unwrap();
    file.write_all(b"cached data 1").unwrap();
    
    let mut file = File::create(cache_dir.join("cache2.tmp")).unwrap();
    file.write_all(b"cached data 2").unwrap();
    
    // Create thumbnails directory
    let thumbs = cache_dir.join("thumbnails");
    fs::create_dir_all(&thumbs).unwrap();
    let mut file = File::create(thumbs.join("thumb1.png")).unwrap();
    file.write_all(&[0u8; 1000]).unwrap();
    
    temp
}

#[test]
fn test_mock_cache_structure() {
    let temp = create_mock_cache();
    
    assert!(temp.path().join(".cache").exists());
    assert!(temp.path().join(".cache/cache1.tmp").exists());
    assert!(temp.path().join(".cache/thumbnails").exists());
}

#[test]
fn test_cache_file_sizes() {
    let temp = create_mock_cache();
    
    let cache1 = fs::metadata(temp.path().join(".cache/cache1.tmp")).unwrap();
    assert_eq!(cache1.len(), 13); // "cached data 1"
    
    let thumb = fs::metadata(temp.path().join(".cache/thumbnails/thumb1.png")).unwrap();
    assert_eq!(thumb.len(), 1000);
}
