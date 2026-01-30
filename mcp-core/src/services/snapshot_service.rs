//! Snapshot service implementation

use std::sync::Arc;
use std::path::PathBuf;
use tonic::{Request, Response, Status};

use crate::audit::AuditLogger;
use crate::snapshot::SnapshotManager;

pub use crate::snapshot_proto::*;

pub struct SnapshotServiceImpl {
    audit: Arc<AuditLogger>,
    snapshots: Arc<SnapshotManager>,
}

impl SnapshotServiceImpl {
    pub fn new(audit: Arc<AuditLogger>, snapshots: Arc<SnapshotManager>) -> Self {
        Self { audit, snapshots }
    }
}

#[tonic::async_trait]
impl snapshot_service_server::SnapshotService for SnapshotServiceImpl {
    async fn create(
        &self,
        request: Request<CreateSnapshotRequest>,
    ) -> Result<Response<CreateSnapshotResponse>, Status> {
        let req = request.into_inner();
        let paths: Vec<PathBuf> = req.paths.iter().map(PathBuf::from).collect();

        let snapshot = self.snapshots.create(&paths, &req.label)
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut entry = AuditLogger::create_entry("snapshot", "create");
        entry.details = format!("Created snapshot: {} - {}", snapshot.id, req.label);
        entry.result = "success".to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(CreateSnapshotResponse {
            snapshot_id: snapshot.id,
            created_at: snapshot.created_at.to_rfc3339(),
        }))
    }

    async fn restore(
        &self,
        request: Request<RestoreSnapshotRequest>,
    ) -> Result<Response<RestoreSnapshotResponse>, Status> {
        let req = request.into_inner();
        let target_paths: Option<Vec<PathBuf>> = if req.target_paths.is_empty() {
            None
        } else {
            Some(req.target_paths.iter().map(PathBuf::from).collect())
        };

        let restored = self.snapshots.restore(&req.snapshot_id, target_paths.as_deref())
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut entry = AuditLogger::create_entry("snapshot", "restore");
        entry.details = format!("Restored snapshot: {} ({} files)", req.snapshot_id, restored.len());
        entry.result = "success".to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(RestoreSnapshotResponse {
            success: true,
            restored_paths: restored.iter().map(|p| p.to_string_lossy().to_string()).collect(),
        }))
    }

    async fn list(
        &self,
        _request: Request<ListSnapshotsRequest>,
    ) -> Result<Response<ListSnapshotsResponse>, Status> {
        let snapshots = self.snapshots.list();

        let snapshot_infos: Vec<SnapshotInfo> = snapshots.iter().map(|s| {
            SnapshotInfo {
                id: s.id.clone(),
                label: s.label.clone(),
                created_at: s.created_at.to_rfc3339(),
                file_count: s.files.len() as u32,
            }
        }).collect();

        Ok(Response::new(ListSnapshotsResponse {
            snapshots: snapshot_infos,
        }))
    }

    async fn delete(
        &self,
        request: Request<DeleteSnapshotRequest>,
    ) -> Result<Response<DeleteSnapshotResponse>, Status> {
        let req = request.into_inner();

        self.snapshots.delete(&req.snapshot_id)
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut entry = AuditLogger::create_entry("snapshot", "delete");
        entry.details = format!("Deleted snapshot: {}", req.snapshot_id);
        entry.result = "success".to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(DeleteSnapshotResponse {
            success: true,
        }))
    }
}
