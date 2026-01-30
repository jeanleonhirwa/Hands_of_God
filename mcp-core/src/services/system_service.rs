//! System information service implementation

use std::sync::Arc;
use tonic::{Request, Response, Status};
use sysinfo::{System, Disks, Networks};

use crate::audit::AuditLogger;

pub use crate::system_proto::*;

pub struct SystemServiceImpl {
    audit: Arc<AuditLogger>,
}

impl SystemServiceImpl {
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self { audit }
    }
}

#[tonic::async_trait]
impl system_service_server::SystemService for SystemServiceImpl {
    async fn get_system_info(
        &self,
        _request: Request<GetSystemInfoRequest>,
    ) -> Result<Response<GetSystemInfoResponse>, Status> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_usage();
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();

        let disks = Disks::new_with_refreshed_list();
        let disk_infos: Vec<DiskInfo> = disks.iter().map(|d| {
            DiskInfo {
                name: d.name().to_string_lossy().to_string(),
                mount_point: d.mount_point().to_string_lossy().to_string(),
                total_space: d.total_space(),
                available_space: d.available_space(),
            }
        }).collect();

        Ok(Response::new(GetSystemInfoResponse {
            cpu_usage,
            total_memory,
            used_memory,
            disks: disk_infos,
        }))
    }

    async fn get_processes(
        &self,
        _request: Request<GetProcessesRequest>,
    ) -> Result<Response<GetProcessesResponse>, Status> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let processes: Vec<ProcessInfo> = sys.processes().iter().map(|(pid, proc)| {
            ProcessInfo {
                pid: pid.as_u32(),
                name: proc.name().to_string_lossy().to_string(),
                cpu_usage: proc.cpu_usage(),
                memory: proc.memory(),
            }
        }).collect();

        Ok(Response::new(GetProcessesResponse { processes }))
    }

    async fn get_audit_logs(
        &self,
        request: Request<GetAuditLogsRequest>,
    ) -> Result<Response<GetAuditLogsResponse>, Status> {
        let req = request.into_inner();
        let limit = if req.limit > 0 { req.limit as usize } else { 100 };

        let logs = self.audit.query(
            if req.service.is_empty() { None } else { Some(&req.service) },
            if req.action.is_empty() { None } else { Some(&req.action) },
            None,
            None,
            limit,
        ).map_err(|e| Status::internal(e.to_string()))?;

        let entries: Vec<AuditLogEntry> = logs.iter().map(|e| {
            AuditLogEntry {
                id: e.id.clone(),
                timestamp: e.timestamp.to_rfc3339(),
                action: e.action.clone(),
                service: e.service.clone(),
                details: e.details.clone(),
                result: e.result.clone(),
                snapshot_id: e.snapshot_id.clone().unwrap_or_default(),
            }
        }).collect();

        Ok(Response::new(GetAuditLogsResponse { entries }))
    }
}
