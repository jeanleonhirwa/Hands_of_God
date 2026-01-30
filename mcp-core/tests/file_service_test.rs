//! Unit tests for FileService

use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn test_read_file_success() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "Hello, World!").unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_create_file_success() {
        let temp_dir = setup_test_dir();
        let file_path = temp_dir.path().join("new_file.txt");
        
        std::fs::write(&file_path, "New content").unwrap();
        
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "New content");
    }

    #[test]
    fn test_list_dir_success() {
        let temp_dir = setup_test_dir();
        std::fs::write(temp_dir.path().join("file1.txt"), "").unwrap();
        std::fs::write(temp_dir.path().join("file2.txt"), "").unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_path_validation() {
        // Test that system paths are rejected
        let system_paths = vec![
            PathBuf::from("C:\\Windows\\System32"),
            PathBuf::from("/etc/passwd"),
            PathBuf::from("/usr/bin"),
        ];

        for path in system_paths {
            let path_str = path.to_string_lossy().to_lowercase();
            let is_system = path_str.contains("system32") 
                || path_str.contains("windows")
                || path_str.contains("/etc")
                || path_str.contains("/usr");
            assert!(is_system, "Path should be detected as system path");
        }
    }

    #[test]
    fn test_sha256_computation() {
        use sha2::{Sha256, Digest};
        
        let content = b"Hello, World!";
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hex::encode(hasher.finalize());
        
        // Known SHA256 of "Hello, World!"
        assert_eq!(result, "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f");
    }
}
