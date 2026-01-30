//! Error types for MCP Core

use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Path not allowed: {0}")]
    PathNotAllowed(String),

    #[error("Command not whitelisted: {0}")]
    CommandNotWhitelisted(String),

    #[error("Approval required for action: {0}")]
    ApprovalRequired(String),

    #[error("File operation failed: {0}")]
    FileError(String),

    #[error("Git operation failed: {0}")]
    GitError(String),

    #[error("Command execution failed: {0}")]
    CommandError(String),

    #[error("Snapshot error: {0}")]
    SnapshotError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<McpError> for tonic::Status {
    fn from(err: McpError) -> Self {
        match err {
            McpError::PolicyViolation(msg) => tonic::Status::permission_denied(msg),
            McpError::PathNotAllowed(msg) => tonic::Status::permission_denied(msg),
            McpError::CommandNotWhitelisted(msg) => tonic::Status::permission_denied(msg),
            McpError::ApprovalRequired(msg) => tonic::Status::failed_precondition(msg),
            McpError::InvalidArgument(msg) => tonic::Status::invalid_argument(msg),
            McpError::NotFound(msg) => tonic::Status::not_found(msg),
            _ => tonic::Status::internal(err.to_string()),
        }
    }
}

pub type McpResult<T> = Result<T, McpError>;
