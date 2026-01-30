//! Sandbox execution environment for safe command execution

use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use crate::error::{McpError, McpResult};

/// Sandbox configuration for command execution
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Working directory for the command
    pub cwd: Option<String>,
    /// Environment variables to set
    pub env: HashMap<String, String>,
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// Whether to capture stdout
    pub capture_stdout: bool,
    /// Whether to capture stderr
    pub capture_stderr: bool,
    /// Resource limits
    pub limits: ResourceLimits,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceLimits {
    /// Max memory in bytes (0 = unlimited)
    pub max_memory: u64,
    /// Max CPU time in seconds (0 = unlimited)
    pub max_cpu_time: u64,
    /// Max file size in bytes (0 = unlimited)
    pub max_file_size: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            cwd: None,
            env: HashMap::new(),
            timeout_secs: 300, // 5 minutes default
            capture_stdout: true,
            capture_stderr: true,
            limits: ResourceLimits::default(),
        }
    }
}

/// Execute a command in a sandboxed environment
pub struct SandboxExecutor;

impl SandboxExecutor {
    /// Execute a command with the given configuration
    pub fn execute(
        command: &str,
        args: &[String],
        config: &SandboxConfig,
    ) -> McpResult<SandboxOutput> {
        let mut cmd = Command::new(command);
        cmd.args(args);

        if let Some(cwd) = &config.cwd {
            cmd.current_dir(cwd);
        }

        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        if config.capture_stdout {
            cmd.stdout(Stdio::piped());
        }

        if config.capture_stderr {
            cmd.stderr(Stdio::piped());
        }

        // On Windows, we can use Job Objects for resource limiting
        // For now, we'll implement basic execution
        #[cfg(target_os = "windows")]
        {
            // TODO: Implement Windows Job Object sandboxing
        }

        let output = cmd.output()
            .map_err(|e| McpError::CommandError(format!("Failed to execute command: {}", e)))?;

        Ok(SandboxOutput::from_output(output))
    }

    /// Predict the effects of a command without executing it (dry-run)
    pub fn predict_effects(command: &str, args: &[String], cwd: Option<&Path>) -> Vec<String> {
        let mut effects = Vec::new();
        let full_cmd = format!("{} {}", command, args.join(" "));

        match command {
            "npm" | "pnpm" | "yarn" => {
                if args.first().map(|s| s.as_str()) == Some("install") {
                    effects.push("Will create/update node_modules folder".to_string());
                    effects.push("May update package-lock.json or yarn.lock".to_string());
                } else if args.first().map(|s| s.as_str()) == Some("run") {
                    effects.push(format!("Will run npm script: {}", args.get(1).unwrap_or(&"".to_string())));
                }
            }
            "git" => {
                match args.first().map(|s| s.as_str()) {
                    Some("commit") => effects.push("Will create a new git commit".to_string()),
                    Some("push") => effects.push("Will push commits to remote repository".to_string()),
                    Some("pull") => effects.push("Will fetch and merge changes from remote".to_string()),
                    Some("checkout") => effects.push("Will switch branches or restore files".to_string()),
                    _ => {}
                }
            }
            "cargo" => {
                if args.first().map(|s| s.as_str()) == Some("build") {
                    effects.push("Will compile Rust project".to_string());
                    effects.push("Will create/update target directory".to_string());
                }
            }
            "docker" => {
                match args.first().map(|s| s.as_str()) {
                    Some("build") => effects.push("Will build a Docker image".to_string()),
                    Some("run") => effects.push("Will start a Docker container".to_string()),
                    Some("stop") => effects.push("Will stop running container(s)".to_string()),
                    _ => {}
                }
            }
            _ => {
                effects.push(format!("Will execute: {}", full_cmd));
            }
        }

        if let Some(dir) = cwd {
            effects.push(format!("Working directory: {}", dir.display()));
        }

        effects
    }
}

#[derive(Debug)]
pub struct SandboxOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

impl SandboxOutput {
    fn from_output(output: Output) -> Self {
        Self {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        }
    }
}
