//! Policy engine for MCP operations

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::Config;
use crate::error::{McpError, McpResult};

/// Policy decision result
#[derive(Debug, Clone)]
pub enum PolicyDecision {
    /// Action is allowed without approval
    Allow,
    /// Action requires user approval
    RequireApproval(String),
    /// Action is denied
    Deny(String),
}

/// Policy engine for checking and enforcing rules
pub struct PolicyEngine {
    config: Arc<RwLock<Config>>,
}

impl PolicyEngine {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self { config }
    }

    /// Check if a file operation is allowed
    pub async fn check_file_access(&self, path: &std::path::Path, write: bool) -> McpResult<PolicyDecision> {
        let config = self.config.read().await;

        // Check if path is within allowed paths
        if !config.is_path_allowed(path) {
            return Ok(PolicyDecision::Deny(format!(
                "Path '{}' is not within allowed directories",
                path.display()
            )));
        }

        // Write operations may require approval
        if write {
            // Check for sensitive paths
            let path_str = path.to_string_lossy().to_lowercase();
            if path_str.contains("system32") || path_str.contains("windows") || path_str.contains("/etc") {
                return Ok(PolicyDecision::Deny("Cannot write to system directories".to_string()));
            }

            // By default, file writes require approval unless auto-approved
            return Ok(PolicyDecision::RequireApproval(format!(
                "Write to '{}'",
                path.display()
            )));
        }

        Ok(PolicyDecision::Allow)
    }

    /// Check if a command execution is allowed
    pub async fn check_command(&self, command: &str, args: &[String]) -> McpResult<PolicyDecision> {
        let config = self.config.read().await;

        // Check if command is whitelisted
        if !config.is_command_whitelisted(command) {
            return Ok(PolicyDecision::Deny(format!(
                "Command '{}' is not whitelisted",
                command
            )));
        }

        // Build full command string for pattern matching
        let full_command = format!("{} {}", command, args.join(" "));

        // Check for auto-approve patterns
        for pattern in &config.auto_approve_patterns {
            if full_command.starts_with(pattern) {
                return Ok(PolicyDecision::Allow);
            }
        }

        // Check for sensitive patterns (always require approval)
        for pattern in &config.sensitive_patterns {
            if full_command.contains(pattern) {
                return Ok(PolicyDecision::RequireApproval(format!(
                    "Sensitive command detected: {}",
                    full_command
                )));
            }
        }

        // Default: require approval for commands
        Ok(PolicyDecision::RequireApproval(format!(
            "Execute command: {}",
            full_command
        )))
    }

    /// Check if a git operation is allowed
    pub async fn check_git_operation(&self, repo_path: &std::path::Path, operation: &str) -> McpResult<PolicyDecision> {
        let config = self.config.read().await;

        // Check if repo path is within allowed paths
        if !config.is_path_allowed(repo_path) {
            return Ok(PolicyDecision::Deny(format!(
                "Repository path '{}' is not within allowed directories",
                repo_path.display()
            )));
        }

        // Read operations are generally allowed
        match operation {
            "status" | "log" | "diff" | "branch" => Ok(PolicyDecision::Allow),
            "commit" | "push" | "pull" | "checkout" | "merge" => {
                Ok(PolicyDecision::RequireApproval(format!(
                    "Git {}: {}",
                    operation,
                    repo_path.display()
                )))
            }
            "push --force" | "reset --hard" => {
                Ok(PolicyDecision::Deny(format!(
                    "Dangerous git operation '{}' is blocked by default",
                    operation
                )))
            }
            _ => Ok(PolicyDecision::RequireApproval(format!(
                "Git {}: {}",
                operation,
                repo_path.display()
            ))),
        }
    }

    /// Validate an approval token
    pub async fn validate_approval(&self, token: &str) -> bool {
        // In a real implementation, this would check against stored approval tokens
        // For now, we accept any non-empty token
        !token.is_empty()
    }
}
