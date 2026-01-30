//! Command service implementation with dry-run and sandbox support

use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::audit::{AuditLogger, AuditEntry};
use crate::config::Config;
use crate::policy::{PolicyEngine, PolicyDecision};
use crate::sandbox::{SandboxExecutor, SandboxConfig};

pub use crate::command_proto::*;

pub struct CommandServiceImpl {
    config: Arc<RwLock<Config>>,
    audit: Arc<AuditLogger>,
    policy: Arc<PolicyEngine>,
}

impl CommandServiceImpl {
    pub fn new(
        config: Arc<RwLock<Config>>,
        audit: Arc<AuditLogger>,
        policy: Arc<PolicyEngine>,
    ) -> Self {
        Self { config, audit, policy }
    }
}

#[tonic::async_trait]
impl command_service_server::CommandService for CommandServiceImpl {
    async fn run(
        &self,
        request: Request<RunCommandRequest>,
    ) -> Result<Response<RunCommandResponse>, Status> {
        let req = request.into_inner();

        // Check policy
        match self.policy.check_command(&req.command, &req.args).await? {
            PolicyDecision::Deny(reason) => return Err(Status::permission_denied(reason)),
            PolicyDecision::RequireApproval(reason) => {
                // If dry_run, we don't need approval
                if !req.dry_run && req.approval_token.is_empty() {
                    return Err(Status::failed_precondition(format!(
                        "Approval required: {}. Use dry_run=true to preview, or provide approval_token.",
                        reason
                    )));
                }
            }
            PolicyDecision::Allow => {}
        }

        let cwd = if req.cwd.is_empty() { None } else { Some(PathBuf::from(&req.cwd)) };

        // Dry-run mode: predict effects without executing
        if req.dry_run {
            let effects = SandboxExecutor::predict_effects(
                &req.command,
                &req.args,
                cwd.as_deref(),
            );

            let command_line = format!("{} {}", req.command, req.args.join(" "));

            // Log dry-run
            let mut entry = AuditLogger::create_entry("command", "dry_run");
            entry.details = format!("Dry-run: {}", command_line);
            entry.result = "simulated".to_string();
            let _ = self.audit.log(entry);

            return Ok(Response::new(RunCommandResponse {
                dry_run: true,
                command_line,
                predicted_effects: effects,
                estimated_time: "varies".to_string(),
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }));
        }

        // Validate approval token for actual execution
        if !req.approval_token.is_empty() && !self.policy.validate_approval(&req.approval_token).await {
            return Err(Status::permission_denied("Invalid approval token"));
        }

        // Execute command in sandbox
        let sandbox_config = SandboxConfig {
            cwd: cwd.map(|p| p.to_string_lossy().to_string()),
            timeout_secs: if req.timeout_secs > 0 { req.timeout_secs as u64 } else { 300 },
            ..Default::default()
        };

        let output = SandboxExecutor::execute(&req.command, &req.args, &sandbox_config)
            .map_err(|e| Status::internal(e.to_string()))?;

        let command_line = format!("{} {}", req.command, req.args.join(" "));

        // Log execution
        let mut entry = AuditLogger::create_entry("command", "execute");
        entry.details = format!("Executed: {} (exit: {})", command_line, output.exit_code);
        entry.user_approved = !req.approval_token.is_empty();
        entry.approval_token = if req.approval_token.is_empty() { None } else { Some(req.approval_token) };
        entry.result = if output.success { "success" } else { "failed" }.to_string();
        let _ = self.audit.log(entry);

        Ok(Response::new(RunCommandResponse {
            dry_run: false,
            command_line,
            predicted_effects: vec![],
            estimated_time: String::new(),
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            success: output.success,
        }))
    }

    async fn list_whitelisted(
        &self,
        _request: Request<ListWhitelistedRequest>,
    ) -> Result<Response<ListWhitelistedResponse>, Status> {
        let config = self.config.read().await;
        
        Ok(Response::new(ListWhitelistedResponse {
            commands: config.whitelisted_commands.clone(),
        }))
    }
}
