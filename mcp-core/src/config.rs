//! Configuration management for MCP Core

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{McpError, McpResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server listen address
    pub server_address: String,

    /// Allowed root paths for file operations
    pub allowed_paths: Vec<PathBuf>,

    /// Whitelisted commands that can be executed
    pub whitelisted_commands: Vec<String>,

    /// Path to audit database
    pub audit_db_path: PathBuf,

    /// Directory for snapshots
    pub snapshot_dir: PathBuf,

    /// Maximum file size for read operations (bytes)
    pub max_file_size: u64,

    /// Whether dry-run is enabled by default
    pub dry_run_default: bool,

    /// Auto-approve patterns (commands that don't need approval)
    pub auto_approve_patterns: Vec<String>,

    /// Sensitive action patterns (always require approval)
    pub sensitive_patterns: Vec<String>,

    /// Enable sandbox mode for command execution
    pub sandbox_enabled: bool,

    /// LLM provider configuration
    pub llm_config: LlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Provider type: "openai", "anthropic", "local", "mock"
    pub provider: String,

    /// API endpoint (for remote providers)
    pub endpoint: Option<String>,

    /// Model name
    pub model: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let mcp_dir = home.join(".mcp");

        Self {
            server_address: "127.0.0.1:50051".to_string(),
            allowed_paths: vec![
                home.join("projects"),
                home.join("Documents"),
                home.join("Desktop"),
            ],
            whitelisted_commands: vec![
                "git".to_string(),
                "npm".to_string(),
                "pnpm".to_string(),
                "yarn".to_string(),
                "node".to_string(),
                "python".to_string(),
                "python3".to_string(),
                "cargo".to_string(),
                "rustc".to_string(),
                "dotnet".to_string(),
                "code".to_string(),
                "docker".to_string(),
            ],
            audit_db_path: mcp_dir.join("audit.db"),
            snapshot_dir: mcp_dir.join("snapshots"),
            max_file_size: 10 * 1024 * 1024, // 10MB
            dry_run_default: true,
            auto_approve_patterns: vec![
                "git status".to_string(),
                "git log".to_string(),
                "git diff".to_string(),
                "npm list".to_string(),
            ],
            sensitive_patterns: vec![
                "rm -rf".to_string(),
                "del /s".to_string(),
                "format".to_string(),
                "shutdown".to_string(),
                "reboot".to_string(),
                "git push --force".to_string(),
            ],
            sandbox_enabled: true,
            llm_config: LlmConfig::default(),
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "mock".to_string(),
            endpoint: None,
            model: Some("gpt-4".to_string()),
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    pub fn load_or_default() -> McpResult<Self> {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| McpError::ConfigError(e.to_string()))?;
            serde_json::from_str(&content)
                .map_err(|e| McpError::ConfigError(e.to_string()))
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> McpResult<()> {
        let config_path = Self::config_path();
        
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| McpError::ConfigError(e.to_string()))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| McpError::ConfigError(e.to_string()))?;
        
        std::fs::write(&config_path, content)
            .map_err(|e| McpError::ConfigError(e.to_string()))?;

        Ok(())
    }

    /// Get the configuration file path
    fn config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".mcp").join("config.json")
    }

    /// Check if a path is within allowed paths
    pub fn is_path_allowed(&self, path: &std::path::Path) -> bool {
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return false,
        };

        self.allowed_paths.iter().any(|allowed| {
            if let Ok(allowed_canonical) = allowed.canonicalize() {
                canonical.starts_with(&allowed_canonical)
            } else {
                false
            }
        })
    }

    /// Check if a command is whitelisted
    pub fn is_command_whitelisted(&self, command: &str) -> bool {
        self.whitelisted_commands.iter().any(|c| c == command)
    }
}
