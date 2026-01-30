//! MCP Core Server
//! 
//! A secure local Model Context Protocol server that exposes capability-limited
//! tools to LLMs with audit logging and policy enforcement.

mod proto;
mod services;
mod policy;
mod audit;
mod sandbox;
mod snapshot;
mod error;
mod config;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Server;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::audit::AuditLogger;
use crate::config::Config;
use crate::policy::PolicyEngine;
use crate::services::{
    file_service::FileServiceImpl,
    command_service::CommandServiceImpl,
    git_service::GitServiceImpl,
    snapshot_service::SnapshotServiceImpl,
    system_service::SystemServiceImpl,
};

pub mod file_proto {
    include!("proto/file_service.rs");
}

pub mod command_proto {
    include!("proto/command_service.rs");
}

pub mod git_proto {
    include!("proto/git_service.rs");
}

pub mod snapshot_proto {
    include!("proto/snapshot_service.rs");
}

pub mod system_proto {
    include!("proto/system_service.rs");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .pretty()
        .init();

    info!("Starting MCP Core Server...");

    // Load configuration
    let config = Config::load_or_default()?;
    let config = Arc::new(RwLock::new(config));

    // Initialize audit logger
    let audit_logger = Arc::new(AuditLogger::new(&config.read().await.audit_db_path)?);

    // Initialize policy engine
    let policy_engine = Arc::new(PolicyEngine::new(config.clone()));

    // Initialize snapshot service
    let snapshot_service = Arc::new(snapshot::SnapshotManager::new(
        &config.read().await.snapshot_dir,
    )?);

    // Create service implementations
    let file_service = FileServiceImpl::new(
        config.clone(),
        audit_logger.clone(),
        policy_engine.clone(),
        snapshot_service.clone(),
    );

    let command_service = CommandServiceImpl::new(
        config.clone(),
        audit_logger.clone(),
        policy_engine.clone(),
    );

    let git_service = GitServiceImpl::new(
        config.clone(),
        audit_logger.clone(),
        policy_engine.clone(),
    );

    let snapshot_svc = SnapshotServiceImpl::new(
        audit_logger.clone(),
        snapshot_service.clone(),
    );

    let system_service = SystemServiceImpl::new(
        audit_logger.clone(),
    );

    // Configure server address
    let addr: SocketAddr = config.read().await.server_address.parse()?;
    info!("MCP Server listening on {}", addr);

    // Start gRPC server
    Server::builder()
        .add_service(file_proto::file_service_server::FileServiceServer::new(file_service))
        .add_service(command_proto::command_service_server::CommandServiceServer::new(command_service))
        .add_service(git_proto::git_service_server::GitServiceServer::new(git_service))
        .add_service(snapshot_proto::snapshot_service_server::SnapshotServiceServer::new(snapshot_svc))
        .add_service(system_proto::system_service_server::SystemServiceServer::new(system_service))
        .serve(addr)
        .await?;

    Ok(())
}
