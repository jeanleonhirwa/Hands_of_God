//! Tauri commands for MCP Desktop

use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptResponse {
    pub message: String,
    pub tool_calls: Vec<ToolCall>,
    pub requires_approval: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    pub status: String,
    pub dry_run_result: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub action: String,
    pub service: String,
    pub details: String,
    pub result: String,
}

pub struct McpState {
    pub connected: Mutex<bool>,
    pub server_address: Mutex<String>,
}

impl Default for McpState {
    fn default() -> Self {
        Self {
            connected: Mutex::new(false),
            server_address: Mutex::new("localhost:50051".to_string()),
        }
    }
}

#[tauri::command]
pub async fn connect_mcp(
    address: Option<String>,
    state: State<'_, McpState>,
) -> Result<McpResponse, String> {
    let addr = address.unwrap_or_else(|| "localhost:50051".to_string());
    
    // Store address
    *state.server_address.lock().unwrap() = addr.clone();
    
    // TODO: Implement actual gRPC connection
    // For now, simulate connection
    *state.connected.lock().unwrap() = true;
    
    Ok(McpResponse {
        success: true,
        message: format!("Connected to MCP server at {}", addr),
        data: None,
    })
}

#[tauri::command]
pub async fn disconnect_mcp(state: State<'_, McpState>) -> Result<McpResponse, String> {
    *state.connected.lock().unwrap() = false;
    
    Ok(McpResponse {
        success: true,
        message: "Disconnected from MCP server".to_string(),
        data: None,
    })
}

#[tauri::command]
pub async fn send_prompt(
    prompt: String,
    state: State<'_, McpState>,
) -> Result<PromptResponse, String> {
    let connected = *state.connected.lock().unwrap();
    
    if !connected {
        return Err("Not connected to MCP server".to_string());
    }
    
    // TODO: Send to actual agent bridge
    // For now, return mock response
    
    // Simulate different responses based on prompt
    let prompt_lower = prompt.to_lowercase();
    
    if prompt_lower.contains("git status") {
        return Ok(PromptResponse {
            message: "I'll check the git status for you.".to_string(),
            tool_calls: vec![ToolCall {
                id: uuid::Uuid::new_v4().to_string(),
                name: "git_status".to_string(),
                arguments: serde_json::json!({ "repo_path": "." }),
                status: "pending".to_string(),
                dry_run_result: Some(serde_json::json!({
                    "branch": "main",
                    "modified_files": ["src/main.rs"],
                    "staged_files": [],
                    "untracked_files": ["new_file.txt"]
                })),
            }],
            requires_approval: false,
        });
    }
    
    if prompt_lower.contains("npm") || prompt_lower.contains("install") {
        return Ok(PromptResponse {
            message: "I'll run npm install. This requires your approval.".to_string(),
            tool_calls: vec![ToolCall {
                id: uuid::Uuid::new_v4().to_string(),
                name: "run_command".to_string(),
                arguments: serde_json::json!({
                    "command": "npm",
                    "args": ["install"],
                    "cwd": "."
                }),
                status: "pending".to_string(),
                dry_run_result: Some(serde_json::json!({
                    "predicted_effects": [
                        "Will create/update node_modules folder",
                        "May update package-lock.json"
                    ],
                    "estimated_time": "10-30s"
                })),
            }],
            requires_approval: true,
        });
    }
    
    // Default response
    Ok(PromptResponse {
        message: format!(
            "I understand you want to: \"{}\". Let me help you with that.",
            prompt
        ),
        tool_calls: vec![],
        requires_approval: false,
    })
}

#[tauri::command]
pub async fn approve_action(
    tool_call_id: String,
    state: State<'_, McpState>,
) -> Result<McpResponse, String> {
    let connected = *state.connected.lock().unwrap();
    
    if !connected {
        return Err("Not connected to MCP server".to_string());
    }
    
    // TODO: Execute the approved action through MCP
    
    Ok(McpResponse {
        success: true,
        message: format!("Action {} approved and executed", tool_call_id),
        data: Some(serde_json::json!({
            "tool_call_id": tool_call_id,
            "status": "executed",
            "result": "Success"
        })),
    })
}

#[tauri::command]
pub async fn reject_action(tool_call_id: String) -> Result<McpResponse, String> {
    Ok(McpResponse {
        success: true,
        message: format!("Action {} rejected", tool_call_id),
        data: None,
    })
}

#[tauri::command]
pub async fn get_audit_logs(
    limit: Option<i32>,
    state: State<'_, McpState>,
) -> Result<Vec<AuditEntry>, String> {
    let connected = *state.connected.lock().unwrap();
    
    if !connected {
        return Ok(vec![]);
    }
    
    // TODO: Fetch from actual MCP server
    // Return mock data for now
    Ok(vec![
        AuditEntry {
            id: "1".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            action: "read_file".to_string(),
            service: "file".to_string(),
            details: "Read README.md".to_string(),
            result: "success".to_string(),
        },
    ])
}

#[tauri::command]
pub async fn get_system_info() -> Result<serde_json::Value, String> {
    // TODO: Get actual system info from MCP
    Ok(serde_json::json!({
        "cpu_usage": 25.5,
        "memory_used": 8_000_000_000_u64,
        "memory_total": 16_000_000_000_u64,
    }))
}
