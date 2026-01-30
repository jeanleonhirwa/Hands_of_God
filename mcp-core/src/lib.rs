//! MCP Core Library
//! 
//! This module exports the core functionality for use in tests and as a library.

pub mod audit;
pub mod config;
pub mod error;
pub mod policy;
pub mod sandbox;
pub mod snapshot;

pub use audit::{AuditLogger, AuditEntry};
pub use config::Config;
pub use error::{McpError, McpResult};
pub use policy::{PolicyEngine, PolicyDecision};
pub use sandbox::{SandboxExecutor, SandboxConfig, SandboxOutput};
pub use snapshot::{SnapshotManager, Snapshot};
