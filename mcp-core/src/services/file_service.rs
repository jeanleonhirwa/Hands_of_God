//! File service implementation

use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use sha2::{Sha256, Digest};

use crate::audit::{AuditLogger, AuditEntry};
use crate::config::Config;
use crate::policy::{PolicyEngine, PolicyDecision};
use crate::snapshot::SnapshotManager;
use crate::error::McpError;

// Re-export proto types
pub use crate::file_proto::*;

pub struct FileServiceImpl {
    config: Arc<RwLock<Config>>,
    audit: Arc<AuditLogger>,
    policy: Arc<PolicyEngine>,
    snapshots: Arc<SnapshotManager>,
}

impl FileServiceImpl {
    pub fn new(
        config: Arc<RwLock<Config>>,
        audit: Arc<AuditLogger>,
        policy: Arc<PolicyEngine>,
        snapshots: Arc<SnapshotManager>,
    ) -> Self {
        Self { config, audit, policy, snapshots }
    }

    fn compute_sha256(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())
    }
}

#[tonic::async_trait]
impl file_service_server::FileService for FileServiceImpl {
    async fn read_file(
        &self,
        request: Request<ReadFileRequest>,
    ) -> Result<Response<ReadFileResponse>, Status> {
        let req = request.into_inner();
        let path = PathBuf::from(&req.path);

        // Check policy
        match self.policy.check_file_access(&path, false).await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            _ => {}
        }

        // Read file
        let config = self.config.read().await;
        let metadata = std::fs::metadata(&path)
            .map_err(|e| Status::not_found(format!("File not found: {}", e)))?;

        if metadata.len() > config.max_file_size {
            return Err(Status::invalid_argument(format!(
                "File exceeds maximum size of {} bytes",
                config.max_file_size
            )));
        }

        let content = std::fs::read(&path)
            .map_err(|e| Status::internal(format!("Failed to read file: {}", e)))?;

        let sha256 = Self::compute_sha256(&content);

        // Log action
        let mut entry = AuditLogger::create_entry("file", "read");
        entry.details = format!("Read file: {}", path.display());
        entry.result = "success".to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(ReadFileResponse {
            path: req.path,
            content: String::from_utf8_lossy(&content).to_string(),
            sha256,
            size: metadata.len(),
        }))
    }

    async fn create_file(
        &self,
        request: Request<CreateFileRequest>,
    ) -> Result<Response<CreateFileResponse>, Status> {
        let req = request.into_inner();
        let path = PathBuf::from(&req.path);

        // Check policy
        match self.policy.check_file_access(&path, true).await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            PolicyDecision::RequireApproval(reason) => {
                if req.approval_token.is_empty() {
                    return Err(Status::failed_precondition(format!(
                        "Approval required: {}",
                        reason
                    )));
                }
                if !self.policy.validate_approval(&req.approval_token).await {
                    return Err(Status::permission_denied("Invalid approval token"));
                }
            }
            PolicyDecision::Allow => {}
        }

        // Create snapshot before modification if file exists
        let snapshot_id = if path.exists() {
            Some(self.snapshots.create(&[path.clone()], "pre-create")?.id)
        } else {
            None
        };

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Status::internal(format!("Failed to create directories: {}", e)))?;
        }

        // Write file
        std::fs::write(&path, &req.content)
            .map_err(|e| Status::internal(format!("Failed to write file: {}", e)))?;

        let sha256 = Self::compute_sha256(req.content.as_bytes());

        // Log action
        let mut entry = AuditLogger::create_entry("file", "create");
        entry.details = format!("Created file: {}", path.display());
        entry.user_approved = !req.approval_token.is_empty();
        entry.approval_token = if req.approval_token.is_empty() { None } else { Some(req.approval_token) };
        entry.snapshot_id = snapshot_id.clone();
        entry.result = "success".to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(CreateFileResponse {
            success: true,
            path: req.path,
            sha256,
            snapshot_id: snapshot_id.unwrap_or_default(),
        }))
    }

    async fn append_file(
        &self,
        request: Request<AppendFileRequest>,
    ) -> Result<Response<AppendFileResponse>, Status> {
        let req = request.into_inner();
        let path = PathBuf::from(&req.path);

        // Check policy
        match self.policy.check_file_access(&path, true).await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            PolicyDecision::RequireApproval(reason) => {
                if req.approval_token.is_empty() {
                    return Err(Status::failed_precondition(format!(
                        "Approval required: {}",
                        reason
                    )));
                }
            }
            PolicyDecision::Allow => {}
        }

        // Create snapshot before modification
        let snapshot_id = if path.exists() {
            Some(self.snapshots.create(&[path.clone()], "pre-append")?.id)
        } else {
            None
        };

        // Append to file
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| Status::internal(format!("Failed to open file: {}", e)))?;

        file.write_all(req.content.as_bytes())
            .map_err(|e| Status::internal(format!("Failed to append to file: {}", e)))?;

        let metadata = std::fs::metadata(&path)
            .map_err(|e| Status::internal(format!("Failed to get metadata: {}", e)))?;

        // Log action
        let mut entry = AuditLogger::create_entry("file", "append");
        entry.details = format!("Appended to file: {}", path.display());
        entry.snapshot_id = snapshot_id.clone();
        entry.result = "success".to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(AppendFileResponse {
            success: true,
            new_size: metadata.len(),
            snapshot_id: snapshot_id.unwrap_or_default(),
        }))
    }

    async fn move_file(
        &self,
        request: Request<MoveFileRequest>,
    ) -> Result<Response<MoveFileResponse>, Status> {
        let req = request.into_inner();
        let from_path = PathBuf::from(&req.from_path);
        let to_path = PathBuf::from(&req.to_path);

        // Check policy for both paths
        match self.policy.check_file_access(&from_path, true).await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            PolicyDecision::RequireApproval(reason) => {
                if req.approval_token.is_empty() {
                    return Err(Status::failed_precondition(format!(
                        "Approval required: {}",
                        reason
                    )));
                }
            }
            PolicyDecision::Allow => {}
        }

        match self.policy.check_file_access(&to_path, true).await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            _ => {}
        }

        // Create snapshot
        let snapshot_id = self.snapshots.create(&[from_path.clone()], "pre-move")?.id;

        // Move file
        std::fs::rename(&from_path, &to_path)
            .map_err(|e| Status::internal(format!("Failed to move file: {}", e)))?;

        // Log action
        let mut entry = AuditLogger::create_entry("file", "move");
        entry.details = format!("Moved {} to {}", from_path.display(), to_path.display());
        entry.snapshot_id = Some(snapshot_id.clone());
        entry.result = "success".to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(MoveFileResponse {
            success: true,
            snapshot_id,
        }))
    }

    async fn copy_file(
        &self,
        request: Request<CopyFileRequest>,
    ) -> Result<Response<CopyFileResponse>, Status> {
        let req = request.into_inner();
        let from_path = PathBuf::from(&req.from_path);
        let to_path = PathBuf::from(&req.to_path);

        // Check policy
        match self.policy.check_file_access(&from_path, false).await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            _ => {}
        }

        match self.policy.check_file_access(&to_path, true).await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            PolicyDecision::RequireApproval(reason) => {
                if req.approval_token.is_empty() {
                    return Err(Status::failed_precondition(format!(
                        "Approval required: {}",
                        reason
                    )));
                }
            }
            PolicyDecision::Allow => {}
        }

        // Copy file
        std::fs::copy(&from_path, &to_path)
            .map_err(|e| Status::internal(format!("Failed to copy file: {}", e)))?;

        // Log action
        let mut entry = AuditLogger::create_entry("file", "copy");
        entry.details = format!("Copied {} to {}", from_path.display(), to_path.display());
        entry.result = "success".to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(CopyFileResponse {
            success: true,
        }))
    }

    async fn list_dir(
        &self,
        request: Request<ListDirRequest>,
    ) -> Result<Response<ListDirResponse>, Status> {
        let req = request.into_inner();
        let path = PathBuf::from(&req.path);

        // Check policy
        match self.policy.check_file_access(&path, false).await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            _ => {}
        }

        let entries = std::fs::read_dir(&path)
            .map_err(|e| Status::not_found(format!("Directory not found: {}", e)))?;

        let mut dir_entries = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| Status::internal(e.to_string()))?;
            let metadata = entry.metadata().map_err(|e| Status::internal(e.to_string()))?;
            
            dir_entries.push(DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                is_dir: metadata.is_dir(),
                is_file: metadata.is_file(),
                size: metadata.len(),
            });
        }

        Ok(Response::new(ListDirResponse {
            entries: dir_entries,
        }))
    }

    async fn stat(
        &self,
        request: Request<StatRequest>,
    ) -> Result<Response<StatResponse>, Status> {
        let req = request.into_inner();
        let path = PathBuf::from(&req.path);

        // Check policy
        match self.policy.check_file_access(&path, false).await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            _ => {}
        }

        let metadata = std::fs::metadata(&path)
            .map_err(|e| Status::not_found(format!("Path not found: {}", e)))?;

        use std::time::UNIX_EPOCH;
        let modified = metadata.modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let created = metadata.created()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(Response::new(StatResponse {
            exists: true,
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified_at: modified,
            created_at: created,
        }))
    }
}
