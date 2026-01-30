//! Git service implementation

use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use git2::{Repository, Signature};

use crate::audit::AuditLogger;
use crate::config::Config;
use crate::policy::{PolicyEngine, PolicyDecision};

pub use crate::git_proto::*;

pub struct GitServiceImpl {
    config: Arc<RwLock<Config>>,
    audit: Arc<AuditLogger>,
    policy: Arc<PolicyEngine>,
}

impl GitServiceImpl {
    pub fn new(
        config: Arc<RwLock<Config>>,
        audit: Arc<AuditLogger>,
        policy: Arc<PolicyEngine>,
    ) -> Self {
        Self { config, audit, policy }
    }
}

#[tonic::async_trait]
impl git_service_server::GitService for GitServiceImpl {
    async fn status(
        &self,
        request: Request<GitStatusRequest>,
    ) -> Result<Response<GitStatusResponse>, Status> {
        let req = request.into_inner();
        let repo_path = PathBuf::from(&req.repo_path);

        // Check policy
        match self.policy.check_git_operation(&repo_path, "status").await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            _ => {}
        }

        let repo = Repository::open(&repo_path)
            .map_err(|e| Status::not_found(format!("Not a git repository: {}", e)))?;

        let statuses = repo.statuses(None)
            .map_err(|e| Status::internal(format!("Failed to get status: {}", e)))?;

        let mut modified = Vec::new();
        let mut staged = Vec::new();
        let mut untracked = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();

            if status.is_wt_modified() || status.is_wt_deleted() {
                modified.push(path.clone());
            }
            if status.is_index_new() || status.is_index_modified() {
                staged.push(path.clone());
            }
            if status.is_wt_new() {
                untracked.push(path);
            }
        }

        let branch = repo.head()
            .ok()
            .and_then(|h| h.shorthand().map(|s| s.to_string()))
            .unwrap_or_else(|| "HEAD".to_string());

        // Log action
        let mut entry = AuditLogger::create_entry("git", "status");
        entry.details = format!("Git status: {}", repo_path.display());
        entry.result = "success".to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(GitStatusResponse {
            branch,
            modified_files: modified,
            staged_files: staged,
            untracked_files: untracked,
        }))
    }

    async fn commit(
        &self,
        request: Request<GitCommitRequest>,
    ) -> Result<Response<GitCommitResponse>, Status> {
        let req = request.into_inner();
        let repo_path = PathBuf::from(&req.repo_path);

        // Check policy
        match self.policy.check_git_operation(&repo_path, "commit").await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            PolicyDecision::RequireApproval(reason) => {
                if req.approval_token.is_empty() {
                    return Err(Status::failed_precondition(format!(
                        "Approval required: {}", reason
                    )));
                }
            }
            PolicyDecision::Allow => {}
        }

        let repo = Repository::open(&repo_path)
            .map_err(|e| Status::not_found(format!("Not a git repository: {}", e)))?;

        // Stage specified files
        let mut index = repo.index()
            .map_err(|e| Status::internal(format!("Failed to get index: {}", e)))?;

        for file in &req.files {
            index.add_path(std::path::Path::new(file))
                .map_err(|e| Status::internal(format!("Failed to stage {}: {}", file, e)))?;
        }
        index.write()
            .map_err(|e| Status::internal(format!("Failed to write index: {}", e)))?;

        let tree_id = index.write_tree()
            .map_err(|e| Status::internal(format!("Failed to write tree: {}", e)))?;
        let tree = repo.find_tree(tree_id)
            .map_err(|e| Status::internal(format!("Failed to find tree: {}", e)))?;

        let sig = Signature::now("MCP User", "mcp@local")
            .map_err(|e| Status::internal(format!("Failed to create signature: {}", e)))?;

        let parent = repo.head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok());

        let parents: Vec<&git2::Commit> = parent.iter().collect();

        let commit_id = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &req.message,
            &tree,
            &parents,
        ).map_err(|e| Status::internal(format!("Failed to commit: {}", e)))?;

        // Log action
        let mut entry = AuditLogger::create_entry("git", "commit");
        entry.details = format!("Git commit: {} - {}", commit_id, req.message);
        entry.user_approved = !req.approval_token.is_empty();
        entry.result = "success".to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(GitCommitResponse {
            success: true,
            commit_hash: commit_id.to_string(),
            diff_summary: format!("{} files changed", req.files.len()),
            warnings: vec![],
        }))
    }

    async fn create_branch(
        &self,
        request: Request<CreateBranchRequest>,
    ) -> Result<Response<CreateBranchResponse>, Status> {
        let req = request.into_inner();
        let repo_path = PathBuf::from(&req.repo_path);

        match self.policy.check_git_operation(&repo_path, "branch").await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            _ => {}
        }

        let repo = Repository::open(&repo_path)
            .map_err(|e| Status::not_found(format!("Not a git repository: {}", e)))?;

        let head = repo.head()
            .map_err(|e| Status::internal(format!("Failed to get HEAD: {}", e)))?;
        let commit = head.peel_to_commit()
            .map_err(|e| Status::internal(format!("Failed to get commit: {}", e)))?;

        repo.branch(&req.branch_name, &commit, false)
            .map_err(|e| Status::internal(format!("Failed to create branch: {}", e)))?;

        let mut entry = AuditLogger::create_entry("git", "create_branch");
        entry.details = format!("Created branch: {}", req.branch_name);
        entry.result = "success".to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(CreateBranchResponse {
            success: true,
            branch_name: req.branch_name,
        }))
    }
}
