//! Snapshot management for file versioning and rollback

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::error::{McpError, McpResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub paths: Vec<PathBuf>,
    pub files: HashMap<PathBuf, FileSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    pub original_path: PathBuf,
    pub snapshot_path: PathBuf,
    pub sha256: String,
    pub size: u64,
}

pub struct SnapshotManager {
    base_dir: PathBuf,
    snapshots: Mutex<HashMap<String, Snapshot>>,
}

impl SnapshotManager {
    pub fn new(base_dir: &Path) -> McpResult<Self> {
        fs::create_dir_all(base_dir)
            .map_err(|e| McpError::SnapshotError(e.to_string()))?;

        let manager = Self {
            base_dir: base_dir.to_path_buf(),
            snapshots: Mutex::new(HashMap::new()),
        };

        // Load existing snapshots
        manager.load_snapshots()?;
        Ok(manager)
    }

    fn load_snapshots(&self) -> McpResult<()> {
        let index_path = self.base_dir.join("index.json");
        if index_path.exists() {
            let content = fs::read_to_string(&index_path)
                .map_err(|e| McpError::SnapshotError(e.to_string()))?;
            let snapshots: HashMap<String, Snapshot> = serde_json::from_str(&content)
                .map_err(|e| McpError::SnapshotError(e.to_string()))?;
            *self.snapshots.lock().unwrap() = snapshots;
        }
        Ok(())
    }

    fn save_index(&self) -> McpResult<()> {
        let index_path = self.base_dir.join("index.json");
        let snapshots = self.snapshots.lock().unwrap();
        let content = serde_json::to_string_pretty(&*snapshots)
            .map_err(|e| McpError::SnapshotError(e.to_string()))?;
        fs::write(&index_path, content)
            .map_err(|e| McpError::SnapshotError(e.to_string()))?;
        Ok(())
    }

    pub fn create(&self, paths: &[PathBuf], label: &str) -> McpResult<Snapshot> {
        let id = Uuid::new_v4().to_string();
        let snapshot_dir = self.base_dir.join(&id);
        fs::create_dir_all(&snapshot_dir)
            .map_err(|e| McpError::SnapshotError(e.to_string()))?;

        let mut files = HashMap::new();

        for path in paths {
            if path.is_file() {
                let file_snapshot = self.snapshot_file(path, &snapshot_dir)?;
                files.insert(path.clone(), file_snapshot);
            } else if path.is_dir() {
                for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() {
                        let file_path = entry.path().to_path_buf();
                        let file_snapshot = self.snapshot_file(&file_path, &snapshot_dir)?;
                        files.insert(file_path, file_snapshot);
                    }
                }
            }
        }

        let snapshot = Snapshot {
            id: id.clone(),
            label: label.to_string(),
            created_at: Utc::now(),
            paths: paths.to_vec(),
            files,
        };

        self.snapshots.lock().unwrap().insert(id, snapshot.clone());
        self.save_index()?;

        Ok(snapshot)
    }

    fn snapshot_file(&self, path: &Path, snapshot_dir: &Path) -> McpResult<FileSnapshot> {
        let content = fs::read(path)
            .map_err(|e| McpError::SnapshotError(e.to_string()))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let sha256 = hex::encode(hasher.finalize());

        let snapshot_path = snapshot_dir.join(&sha256);
        if !snapshot_path.exists() {
            fs::write(&snapshot_path, &content)
                .map_err(|e| McpError::SnapshotError(e.to_string()))?;
        }

        Ok(FileSnapshot {
            original_path: path.to_path_buf(),
            snapshot_path,
            sha256,
            size: content.len() as u64,
        })
    }

    pub fn restore(&self, snapshot_id: &str, target_paths: Option<&[PathBuf]>) -> McpResult<Vec<PathBuf>> {
        let snapshots = self.snapshots.lock().unwrap();
        let snapshot = snapshots.get(snapshot_id)
            .ok_or_else(|| McpError::NotFound(format!("Snapshot '{}' not found", snapshot_id)))?;

        let mut restored = Vec::new();

        for (original_path, file_snapshot) in &snapshot.files {
            let should_restore = target_paths
                .map(|targets| targets.iter().any(|t| original_path.starts_with(t)))
                .unwrap_or(true);

            if should_restore {
                let content = fs::read(&file_snapshot.snapshot_path)
                    .map_err(|e| McpError::SnapshotError(e.to_string()))?;
                
                if let Some(parent) = original_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| McpError::SnapshotError(e.to_string()))?;
                }

                fs::write(original_path, content)
                    .map_err(|e| McpError::SnapshotError(e.to_string()))?;
                restored.push(original_path.clone());
            }
        }

        Ok(restored)
    }

    pub fn list(&self) -> Vec<Snapshot> {
        let snapshots = self.snapshots.lock().unwrap();
        snapshots.values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<Snapshot> {
        let snapshots = self.snapshots.lock().unwrap();
        snapshots.get(id).cloned()
    }

    pub fn delete(&self, id: &str) -> McpResult<()> {
        let mut snapshots = self.snapshots.lock().unwrap();
        if snapshots.remove(id).is_none() {
            return Err(McpError::NotFound(format!("Snapshot '{}' not found", id)));
        }

        let snapshot_dir = self.base_dir.join(id);
        if snapshot_dir.exists() {
            fs::remove_dir_all(&snapshot_dir)
                .map_err(|e| McpError::SnapshotError(e.to_string()))?;
        }

        drop(snapshots);
        self.save_index()?;
        Ok(())
    }
}
