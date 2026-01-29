# MCP — Product Requirements Document (Tech + Architecture)

> Short version: build a local **Model Context Protocol (MCP) server** that safely gives LLMs limited, auditable control of your Windows machine — plus a sleek desktop app to prompt/control it. This PRD is ready to hand to an AI coding agent (Cursor / AntiGravity).

---

## 1 — TL;DR (keepin’ it real)

* Goal: a secure local service (MCP) that exposes *capability-limited tools* (file ops, safe commands, git, apps, containers, notes, browser automation, etc.) to LLMs.
* UI: Desktop app (prompt box + chat + activity log + confirmations + history + tools panel). Windows-first but cross-platform-ready.
* Safety: whitelists, sandboxed execution, dry-run, user confirmations, strict ACLs, audit logs, rollback/versioning.
* Tech picks (precise):

  * Desktop UI: **Tauri** + **React** + **TypeScript**
  * MCP core server: **Rust** (Tokio + tonic gRPC)
  * Agent/LLM bridge: **Node.js** (TypeScript) for SDK flexibility & existing agent libs + optional **Python** worker (for local models)
  * Local DB: **SQLite** (encrypted via SQLCipher)
  * Optional cloud DB: **Postgres** (for multi-device / team)
  * IPC: **gRPC** over loopback / Unix Domain Sockets or Windows Named Pipes; fallback WebSocket/JSON-RPC for UI integrations
  * Packaging: Tauri bundler -> Windows MSI / EXE
* Deliverables for the agent: server + agent-bridge + desktop app + test-suite + installer + docs.

---

## 2 — Who this helps

* You, the dev & founder: automates repetitive dev tasks, acts as a coding assistant that can run safe commands, open editors, scaffold apps, commit/push, run tests, manage containers, and draft founder docs — all controlled from a simple prompt UI.
* Use cases:

  * “Scaffold a Node + React app, open it in VSCode, start dev server”
  * “Search repo for TODOs and create a task list”
  * “Run tests, report failing tests, create issue with stack trace”
  * “Create pitch deck skeleton and save in founder folder”
  * “Deploy latest to staging via CLI wrapper (after approval)”

---

## 3 — Expanded tools list (full, developer-focused)

Each tool is an RPC surface with whitelisted commands + strict inputs.

1. **File Tool**

   * `create_file(path, content, mode)`
   * `read_file(path)`
   * `append_file(path, content)`
   * `move(path_from, path_to)`
   * `copy(path_from, path_to)`
   * `list_dir(path, options)`
   * `stat(path)`
   * *Safety:* max file size cap, allowed root folders.

2. **Folder / Project Tool**

   * `create_project(template, name, dest, options)` (templates: node-api, react-app, godot, python-cli)
   * `apply_template_vars(template_id, vars)`

3. **Command Runner (sandboxed)**

   * `run_command(command, args[], cwd, dry_run=true, allowed_commands[])`
   * *Only allow whitelisted binaries:* `git`, `npm`, `pnpm`, `yarn`, `python`, `dotnet`, `cargo`, `docker` (wrapped), `powershell` (restricted)
   * *Safety:* default `dry_run=true` — must ask user confirm for destructive flags.

4. **Git Tool (wrapped)**

   * `status(repo_path)`
   * `commit(repo_path, message, files[])`
   * `push(repo_path, remote, branch)`
   * `pull(repo_path)`
   * `create_branch(repo_path, branch)`
   * *Wrap git binary, validate repo path, disallow force-push by default.*

5. **App Control / Launcher**

   * `open_app(app_name, args[], cwd)`
   * `focus_window(title_match)`
   * `close_app(app_name)`
   * *Examples:* VS Code, Chrome (with profile), Explorer, Terminal.

6. **Container Manager**

   * `docker_build(tag, dockerfile_path)`
   * `docker_run(image, args, limits)`
   * `docker_ps()`
   * *Always run Docker containers with resource limits & network restrictions by default.*

7. **Test / CI Runner**

   * `run_tests(project_path, runner, timeout)`
   * `collect_coverage(project_path)`
   * `report_test_results()`

8. **Code Intelligence**

   * `search_code(query, paths[])` (fast, regex + repo index)
   * `refactor_file(path, edits[])` (apply via patch)
   * `generate_snippet(prompt, context_files[])` (uses LLM but only returns recommended code; file write requires confirmation)

9. **Notes & Docs**

   * `create_note(title, content, tags)`
   * `search_notes(query)`
   * `export_note_format(id, format)` (md/pdf)

10. **Browser Automation**

    * `open_url(url, options)`
    * `screenshot(url, selector?)`
    * `fill_form(url, actions[])`
    * *Uses Playwright under a restricted profile.*

11. **System Info Tool (read-only)**

    * `get_cpu()`, `get_ram()`, `get_disk()`, `get_processes()`

12. **Clipboard / Screenshot Tool**

    * `copy_to_clipboard(text)`
    * `screenshot(desktop|window|area)`

13. **Scheduler / Cron**

    * `schedule(task_id, cron_expr, action_rpc)`
    * `cancel_schedule(task_id)`

14. **Secrets & Keyring**

    * `get_secret(name)` (uses OS keyring)
    * `store_secret(name, value)` (encrypted)

15. **SSH / Remote Deploy**

    * `ssh_command(host, user, key_ref, command)`
    * `scp_put(host, path_local, path_remote)`

16. **Telemetry / Audit**

    * `get_audit_logs(filter)`
    * `clear_logs()` (admin only)

17. **Undo / Snapshot**

    * `snapshot(paths[]) -> snapshot_id`
    * `restore_snapshot(snapshot_id, target_paths[])`
    * *Implementation:* lightweight copy-on-write snapshots or Git-based versioning for text files.

18. **Config & Policy**

    * `get_config()`, `set_config()`
    * `get_policy()`, `set_policy()` (whitelists, approval rules)

---

## 4 — Platform (Desktop app) features

* Single-window app: left panel = tools & history; main area = chat/prompt + assistant responses; right panel = activity log, audit & approvals.
* Prompt box supports:

  * plain text prompts
  * action buttons (dry-run, exec, approve)
  * pre-built templates (scaffold, commit, run tests)
* Confirm modal for risky actions: shows command and diffs (for file edits), asks for YES (or configurable automatic approvals).
* Live output console for long-running tasks (tail logs).
* Snapshot manager (list snapshots, restore).
* Repo browser + in-app code editor (builtin quick file viewer).
* Integrations: GitHub OAuth, Docker Desktop detection, VSCode (open via `code` CLI).
* Settings: policies, allowed folders, keyring, LLM connectors (OpenAI key or local model path), telemetry toggle.
* Security-first defaults: all remote LLMs treated as untrusted — no auto-exec unless allowed.
* Activity/History: searchable, downloadable logs (for debugging & audits).

---

## 5 — Architecture (high level)

```
[Desktop UI (Tauri + React)]
          ↕ (gRPC / Named Pipe / WebSocket JSON-RPC)
[Local MCP Server (Rust)]
   ├─ Tool Handlers (File, Git, Command Runner, Docker Wrapper, Notes, Browser)
   ├─ Sandbox / Executor (Windows Job Objects / dedicated low-priv user / WSL / VM)
   ├─ Agent Bridge Connector (Node.js) <---> [LLM Provider(s): OpenAI/Local/Other]
   ├─ Policy Engine (whitelist, approvals)
   ├─ Audit Logger (SQLite encrypted)
   └─ Snapshot Service (file versioning)
```

* **Flow**: UI -> MCP (RPC) -> Policy check -> Sandbox -> Executor -> Return result -> MCP logs -> UI displays.
* **Agent mode**: LLM (remote or local) communicates with MCP via the Agent Bridge (LLM calls the MCP RPCs) or MCP sends LLM observations and LLM returns intents/commands. Two modes:

  * **Human-in-the-loop**: LLM suggests commands, MCP shows them with dry-run, user approves.
  * **Autonomous (opt-in)**: LLM triggers allowed commands directly, under strict policy & quota.

---

## 6 — Precise tech stack & why (final picks)

* **Desktop UI**: **Tauri** + **React** + **TypeScript**

  * tiny bundles vs Electron, native feel, easy auto-updates, security.
* **MCP Core**: **Rust** (Tokio async runtime + tonic gRPC)

  * safety, memory-safety, performance for handling child processes & sandboxing.
* **Agent / LLM Bridge**: **Node.js (TypeScript)**

  * best ecosystem for LLM SDKs (OpenAI, Anthropic, LangChain.js, agent libs). Optionally provide Python microservice for local LLM (llama.cpp, vLLM) if needed.
* **IPC / API**: **gRPC** (proto definitions) over loopback (preferred) or named pipes on Windows. JSON-RPC fallback for browser devtools.
* **Local DB**: **SQLite** + **SQLCipher** encryption (single-user local state & audit). Cloud option: **Postgres**.
* **Package & Installer**: Tauri bundler -> Windows installer (MSI or EXE). Provide portable dev build for testers.
* **CI / Tests**: GitHub Actions. Unit tests in Rust + integration tests (spawn server, run sample commands). UI E2E with Playwright.
* **Secrets**: OS keyring integration (Windows Credential Manager). Use libs for encryption at rest.
* **Telemetry**: opt-in only, hashed identifiers, no sensitive data.

---

## 7 — Security model (concrete rules)

* **Principle**: least privilege, explicit approval for destructive actions, auditable history.

1. **Sandboxing**: run risky commands in a low-privilege user account or WSL container by default.
2. **Whitelists**: only allowed binaries and allowed path roots are executable. Use allowlist patterns in config.
3. **Dry-run default**: every command executed by LLM is first simulated or printed. Only after user approves does it run.
4. **Approvals & Policies**:

   * minor changes can be auto-approved (user choice) if they match policy.
   * sensitive actions (delete, move out of allowed folders, network changes) require explicit approval.
5. **Audit logs**: immutable append-only log (SQLite WAL + signed entries). UI shows diff and who approved.
6. **Rollback**: every file write produces a snapshot. Restore available from UI.
7. **Secrets handling**: LLM never receives raw secret values (unless explicitly revealed by user). Secret retrieval requires UI confirmation.
8. **Network controls**: option to run MCP offline-only. If online, restrict outbound hosts or use proxy.

---

## 8 — API / RPC contract (examples)

Use protobuf/gRPC for strong typing. Below are JSON-style samples to hand to the agent.

**File read example**

```json
# Request: FileService.Read
{
  "path": "C:\\Users\\you\\projects\\notexa\\README.md",
  "max_bytes": 65536
}

# Response
{
  "path": "C:\\...\\README.md",
  "content": "## Notexa\nThis is ...",
  "sha256": "abc123",
  "size": 1452
}
```

**Run command (dry-run)**

```json
# Request: CommandService.Run
{
  "command": "npm",
  "args": ["install"],
  "cwd": "C:\\Users\\you\\projects\\notexa",
  "dry_run": true,
  "timeout_secs": 120
}

# Response (dry-run)
{
  "dry_run": true,
  "command_line": "npm install",
  "predicted_effects": [
    "Will create node_modules folder",
    "May update package-lock.json"
  ],
  "estimated_time": "10-30s"
}
```

**Approve & Exec**

```json
# Request: CommandService.Run (after approval)
{
  "command": "npm",
  "args": ["install"],
  "cwd": "C:\\Users\\you\\projects\\notexa",
  "dry_run": false,
  "approval_token": "user-approval-uuid"
}
```

**Git commit**

```json
# Request: GitService.Commit
{
  "repo_path": "C:\\Users\\you\\projects\\notexa",
  "message": "fix(auth): update login flow",
  "files": ["src/auth.js", "package.json"]
}

# Response
{
  "success": true,
  "commit_hash": "abcd1234",
  "diff_summary": "...",
  "warnings": []
}
```

**Snapshot**

```json
# Request: SnapshotService.Create
{
  "paths": ["C:\\Users\\you\\projects\\notexa\\src"],
  "label": "pre-refactor-2026-01-29"
}

# Response
{
  "snapshot_id": "snap-0001",
  "created_at": "2026-01-29T09:10:00Z"
}
```

---

## 9 — Testing & QA (concrete)

* **Unit tests**: each tool handler in Rust with mock filesystem & mock command runner.
* **Integration tests**: run real small commands in a sandboxed temp user, assert no privilege escalation.
* **E2E**: Tauri UI automation with Playwright or Puppeteer hitting local MCP server.
* **Security tests**: fuzz inputs, attempt `del`, `format`, escape attempts; ensure blocked.
* **Performance**: stress test parallel runs, memory leak tests.
* **Acceptance**: test suite must pass on CI for merge.

---

## 10 — CI / CD & packaging

* GitHub Actions pipelines:

  * `build:test` — Rust unit tests + Node tests + lint.
  * `integration` — spin temp VM/container to run integration tests.
  * `package` — build Tauri bundle + create Windows installer.
  * `release` — create GitHub release artifacts.
* Auto-update: Tauri’s updater pointing to releases; require signed releases.

---

## 11 — Roadmap & milestones (developer-friendly)

Step 0 — **MVP design & env**

* Repo skeleton (rust server, node agent, tauri ui)
* Proto contracts (gRPC) for File, Command, Git

Step 1 — **Core MVP**

* File tool (read/write/list)
* Command tool (dry-run + exec safe commands)
* Tauri UI: prompt + display responses + confirm modal
* Agent Bridge basic (connect to OpenAI or local mock)

Step 2 — **Dev tooling**

* Git wrapper (status, commit)
* Project generator (templates)
* Snapshot system (basic copy-based)
* Audit logs

Step 3 — **Polish & security**

* Sandbox low-priv user mode or WSL fallback
* Whitelist & policy engine
* Tests & CI

Step 4 — **Packaging & tests**

* Create Windows installer
* E2E tests + user testing
* Documentation + README + agent instructions

Phase 2

* Add Docker control, Playwright browser automation, advanced code refactor flows, GitHub integration.

---

## 12 — Acceptance criteria (for AI dev agent)

* Core RPCs implemented and gRPC proto docs included.
* Local tests: 80% coverage for critical tools (file, command, git).
* UI: prompt -> dry-run -> approval -> exec flow works end-to-end.
* Sandbox: commands that try to delete `C:\Windows` or escalate privileges are blocked.
* Snapshots: file snapshot & restore tested.
* Build: produce a functioning Windows installer with auto-update stub.
* Clear README + architecture diagrams (ascii or mermaid) + API docs.

---

## 13 — Handoff notes for Cursor 

* Repo layout:

  * `/mcp-core` (Rust)
  * `/agent-bridge` (Node TS)
  * `/desktop` (Tauri + React)
  * `/protos` (proto files)
  * `/templates` (project templates)
  * `/tests`
* Start by implementing **proto contracts** and **FileService**. Use that to validate IPC and UI.
* Make **dry-run** behavior a first-class response type (LLM-friendly).
* Provide a small local mock LLM endpoint so the UI can be tested offline.
* Prioritize security branches (policy & sandbox) before adding more potent tools.
* Deliver unit tests and integration tests for each tool handler.

---

## 14 — Dev tips & tradeoffs (real talk)

* Rust core = safe & fast but slower to iterate; keep agent in Node for rapid LLM SDK access. This split gives reliability + dev speed.
* Tauri saves bundle size; if you later want quick hacking, Electron is faster for prototyping but bigger.
* Snapshots can be heavy; start with Git-based snapshot for text + file copy for binaries. Later integrate volume-level snapshots (advanced).
* Local LLMs = privacy but heavy infra. Start with OpenAI API for early dev, add local model support as opt-in.

---

## 15 — Example user story + RPC exchange (fast)

User: “Create a Node + React skeleton for `hop-n-splat`, open it in VSCode, run dev server.”

1. UI sends `ProjectService.CreateProject(template=node-react, name=hop-n-splat, dest=...)`
2. MCP creates files, returns `snapshot(pre-create)`
3. MCP returns dry-run summary + estimated commands
4. User clicks **Approve**
5. MCP runs `npm install` then `npm run dev` in sandbox; streams output to UI
6. Audit log entry created with user approval token and snapshot id.

---

## 16 — Final notes (keep it chill)

* Start tiny: file ops + dry-run command runner + UI. That already makes life 10x smoother.
* Safety first: default everything to safe. Add auto-approve later when you trust it.
* This design is ready to give to an AI builder: it has APIs, tech stack, milestones, and acceptance criteria.
