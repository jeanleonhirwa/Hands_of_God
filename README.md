# MCP - Model Context Protocol

> A secure local service that exposes capability-limited tools to LLMs with a sleek desktop interface.

![MCP Architecture](docs/architecture.png)

## Overview

MCP (Model Context Protocol) is a local development assistant that safely gives AI/LLM models limited, auditable control of your machine. It features:

- **🔒 Security First**: Whitelists, sandboxed execution, dry-run mode, user confirmations, strict ACLs, audit logs
- **🛠️ Rich Tool Set**: File operations, git control, command execution, snapshots, and more
- **💻 Beautiful Desktop UI**: Native app built with Tauri + React following Apple HIG design
- **🤖 LLM Integration**: Connect to OpenAI, Anthropic, or local models

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Desktop UI (Tauri + React)                    │
│         Prompt Box │ Chat │ Activity Log │ Approvals             │
└─────────────────────────────┬───────────────────────────────────┘
                              │ gRPC / WebSocket
┌─────────────────────────────▼───────────────────────────────────┐
│                    MCP Core Server (Rust)                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌────────────┐ │
│  │ File Tool   │ │ Git Tool    │ │ Command     │ │ Snapshot   │ │
│  │             │ │             │ │ Runner      │ │ Service    │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └────────────┘ │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │ Policy      │ │ Audit       │ │ Sandbox     │               │
│  │ Engine      │ │ Logger      │ │ Executor    │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────────┐
│                  Agent Bridge (Node.js)                          │
│              LLM Provider Abstraction Layer                      │
│         OpenAI │ Anthropic │ Local Models │ Mock                 │
└─────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
mcp/
├── mcp-core/           # Rust core server
│   ├── src/
│   │   ├── services/   # Tool implementations
│   │   ├── policy.rs   # Policy engine
│   │   ├── audit.rs    # Audit logging
│   │   └── sandbox.rs  # Sandboxed execution
│   └── Cargo.toml
├── agent-bridge/       # Node.js LLM bridge
│   ├── src/
│   │   ├── providers/  # LLM provider implementations
│   │   └── client.ts   # MCP gRPC client
│   └── package.json
├── desktop/            # Tauri + React desktop app
│   ├── src/            # React frontend
│   ├── src-tauri/      # Tauri backend
│   └── package.json
├── protos/             # Protocol buffer definitions
└── templates/          # Project templates
```

## Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 18+
- pnpm or npm

### Development

1. **Start MCP Core Server**
   ```bash
   cd mcp-core
   cargo run
   ```

2. **Start Agent Bridge**
   ```bash
   cd agent-bridge
   npm install
   npm run dev
   ```

3. **Start Desktop App**
   ```bash
   cd desktop
   npm install
   npm run tauri:dev
   ```

### Building for Production

```bash
# Build everything
cd desktop
npm run tauri:build
```

## Available Tools

| Tool | Description | Approval Required |
|------|-------------|-------------------|
| `read_file` | Read file contents | No |
| `create_file` | Create or overwrite files | Yes |
| `list_dir` | List directory contents | No |
| `run_command` | Execute whitelisted commands | Yes |
| `git_status` | Get repository status | No |
| `git_commit` | Create commits | Yes |
| `create_snapshot` | Backup files | No |
| `restore_snapshot` | Restore from backup | Yes |

## Security Model

1. **Whitelists**: Only allowed binaries and paths are accessible
2. **Dry-run Default**: Commands are simulated first, then require approval
3. **Audit Logs**: All actions are logged with user approval tokens
4. **Snapshots**: Automatic backups before file modifications
5. **Sandbox**: Commands run in restricted environments

## Configuration

Configuration is stored in `~/.mcp/config.json`:

```json
{
  "server_address": "127.0.0.1:50051",
  "allowed_paths": ["~/projects", "~/Documents"],
  "whitelisted_commands": ["git", "npm", "cargo", "python"],
  "dry_run_default": true,
  "sandbox_enabled": true
}
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_SERVER_ADDRESS` | Core server address | `localhost:50051` |
| `LLM_PROVIDER` | LLM provider (openai/anthropic/mock) | `mock` |
| `LLM_API_KEY` | API key for LLM provider | - |
| `LLM_MODEL` | Model name | `gpt-4` |

## License

MIT License - See [LICENSE](LICENSE) for details.
