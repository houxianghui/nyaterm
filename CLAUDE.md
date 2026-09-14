# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development commands

### Root app
- `pnpm install` — install JS dependencies (runs `postinstall`: patches xterm-webgl via `scripts/patch-xterm-webgl-clear-model.cjs`)
- `pnpm dev` — run the Vite frontend only
- `pnpm tauri dev` — run the full desktop app in Tauri dev mode (dev server on port `1420`, HMR on `1421`)
- `pnpm build` — build the MCP sidecar, then run `tsc` and build the frontend with Vite
- `pnpm build:mcp-sidecar` — build the MCP sidecar binary (`scripts/build-mcp-sidecar.mjs`)
- `pnpm tauri build` — build the production desktop bundle
- `pnpm test` — run frontend unit tests with Vitest (configured inline in `vite.config.ts`; setup file `src/test/setup.ts`)
- `pnpm test <path>` — run a single frontend test file, e.g. `pnpm test src/lib/appSessionFactory.test.ts`
- `pnpm lint` — run `scripts/check-no-console.mjs` then `biome lint src/`
- `pnpm format` — `biome check --write src/` (Biome lint + format, writes changes)
- `pnpm format:check` — `biome check src/` (Biome lint + format, no writes)
- `pnpm fmt` — format the whole repo: Prettier (frontend) + `cargo fmt` (Rust). Use this for cross-language formatting.
- `pnpm fmt:frontend` / `pnpm fmt:rust` / `pnpm fmt:check` — run individual formatters or checks
- `pnpm i18n:check` — check locale JSON formatting (Prettier)
- `pnpm i18n:fix` — rewrite locale JSON formatting (Prettier)
- `pnpm version-sync` — sync version numbers across app files
- `pnpm release` — version sync (with `--commit`) + Tauri build (production release)

### Rust / Tauri backend
- `cargo fmt --manifest-path src-tauri/Cargo.toml` — format Rust code
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets` — lint Rust code
- `cargo test --manifest-path src-tauri/Cargo.toml` — run backend Rust tests
- `cargo test --manifest-path src-tauri/Cargo.toml <test_name>` — run a single backend Rust test
- `cargo test --manifest-path src-tauri/crates/otp/Cargo.toml` — run OTP crate tests
- `cargo test --manifest-path src-tauri/crates/otp/Cargo.toml <test_name>` — run a single OTP crate test
- `cargo test --manifest-path src-tauri/crates/nyaterm-mcp/Cargo.toml` — run MCP crate tests
- `cargo test --manifest-path src-tauri/crates/nyaterm-mcp-protocol/Cargo.toml` — run MCP protocol crate tests

Example single-test command:
- `cargo test --manifest-path src-tauri/Cargo.toml normalizes_trailing_slashes_without_breaking_roots`

### Docs site
- `pnpm --dir docs-site start` — build and serve the docs site locally for all locales (`/` and `/en/`)
- `pnpm --dir docs-site start:zh` — run the zh-CN docs dev server with hot reload
- `pnpm --dir docs-site start:en` — run the English docs dev server with hot reload
- `pnpm --dir docs-site start:ko` — run the Korean docs dev server with hot reload
- `pnpm --dir docs-site build` — build the docs site

## Big-picture architecture

- This is a Tauri 2 desktop app: React/TypeScript frontend in `src/`, Rust backend in `src-tauri/src/`, and IPC between them via Tauri commands/events.
- The frontend should call Rust through the typed wrapper in `src/lib/invoke.ts`, not raw scattered `invoke()` calls where a shared wrapper already exists.
- Tauri commands are registered centrally in `src-tauri/src/lib.rs` and grouped by concern under `src-tauri/src/cmd/`. Current command modules include `session`, `sftp`, `connection`, `credential`, `settings`, `watcher`, `translate`, `stats`, `tunnel`, `proxy`, `otp`, `importer`, `ai`, `mcp`, `app`, `backup`, `clipboard`, `cloud_sync`, `log`, `docker`, `rdp`, `vnc`, `note`, `process`, `gpu`, `ascend_npu`, `external_open`, `local_fs`, `ssh_config`, `updater`, and `macos_menu`.

### Window model
- `src/main.tsx` decides between two boot paths:
  - main window: `AppProvider` + `App.tsx`
  - child windows: `ChildAppProvider` + `ChildWindowRouter`
- Child windows are opened from `src/lib/windowManager.ts` using `?window=` query params. Current child-window flows include settings, new-session, quick-command, and per-file auto-upload dialogs.
- Modal child-window focus/enable state is managed in `windowManager.ts` plus `src/ChildWindowRouter.tsx`; changes here affect cross-window UX.

### Frontend state model
- `src/context/AppContext.tsx` is the main state container. It owns:
  - tab/session workspace state
  - persisted UI/app settings
  - saved connections/groups loading and refresh
  - startup session restoration from persisted `ui.open_tabs`
- `src/context/ChildAppProvider.tsx` is a lightweight provider for child windows. It only loads/saves settings and emits cross-window events; it does not manage the full tab/session workspace.
- `src/context/TransferContext.tsx` separately tracks file transfer progress from backend `transfer-event` notifications.

### Workspace / terminal model
- The terminal workspace has two distinct layers that are easy to confuse:
  - `src/lib/workspaceTabs.ts` manages the persistent logical tab model. Each tab owns a recursive pane tree (`leaf` session panes and `split` panes). This is what gets serialized into `ui.open_tabs` for startup restore.
  - `src/lib/tabWindows.ts` manages the runtime terminal window layout: which tabs live in each split window leaf, active tab per leaf, and window split ratios for the multi-tab/multi-split UI.
- `src/App.tsx` is the shell that composes activity bars, left/right panels, the terminal workspace, quick command / serial send bottom panels, OTP dialogs, transfer UI, recording, and lock screen.
- `src/components/terminal/XTerminal.tsx` is the xterm.js integration point. It wires Fit/Search/WebLinks addons, shell integration, command suggestions, reconnect hooks, and listens to per-session backend events.

### Backend runtime model
- `src-tauri/src/lib.rs` constructs and stores the shared backend managers in Tauri state:
  - `SessionManager` (central registry for active sessions, command routing, command history, fuzzy history search, session lifecycle events)
  - `TunnelManager`, `RecordingManager`, `QuickCommandsStore`, `CloudSyncManager`
  - `PendingAuthManager`, `PendingSshAuthManager`, `PendingSshAgentAuthManager` (keyboard-interactive / OTP / ssh-agent flows)
  - `HostKeyVerifyManager` (TOFU `known_hosts` verification)
  - `AgentApprovalManager` (gates AI agent command execution)
  - `McpManager` (Model Context Protocol server lifecycle)
  - `CodexAppServerManager`, `ClaudeCodeRuntime` (AI provider runtimes)
  - `RdpSessionManager`, `VncSessionManager` (remote desktop)
  - `TransferDuplicateManager`, `DockerSudoManager`, `RemoteStatsSampler`
- `src-tauri/src/core/session.rs` is the central session registry (see `SessionManager` above).
- Session and capability implementations live under `src-tauri/src/core/`:
  - `ssh/` for SSH transport, auth, OSC/CWD tracking, tunnels, and SFTP
  - `pty.rs` / `terminal_session/` for local terminal sessions
  - `telnet.rs`, `serial.rs` (in `session.rs` routing) for Telnet / serial sessions
  - `recording/` for terminal recording, `watcher/` for file-watch driven flows
  - `importer/` for Xshell / MobaXterm / WindTerm import
  - `cloud_sync/` for sync/backup runtime and conflict events, `portable_snapshot/` for what sync/backup payloads include
  - `ai/` for provider calls, streaming responses, structured command cards, agent execution/approval, prompt redaction, audit/history, Codex/Claude Code runtimes
  - `mcp/` for Model Context Protocol server management
  - `sftp/` for remote file operations and transfer progress
  - `remote_desktop/`, `rdp.rs`, `vnc.rs` for remote desktop sessions
  - `capabilities/`, `capture/`, `history/`, `monitoring/`, `network/`, `quick_commands/`, `translate/`, `zmodem/` for supporting subsystems
- Backend session I/O is event-driven. The Rust side emits session-specific and app-wide events such as `terminal-output-{id}`, `cwd-changed-{id}`, `session-closed-{id}`, `sessions-changed`, `connections-changed`, `command-history-changed`, `transfer-event`, `otp-request`, `cloud-sync-status-changed`, `cloud-sync-history-changed`, and `cloud-sync-conflict`.

### SSH / auth / transfer details
- SSH logic is split across `src-tauri/src/core/ssh/`:
  - `client.rs` handles russh client setup, keepalive config, proxy-aware connection setup, and TOFU-style `known_hosts` verification (host-key prompts are coordinated through `HostKeyVerifyManager`)
  - `auth.rs` handles saved auth loading plus keyboard-interactive / OTP flows through `PendingAuthManager` and the `otp-request` event
  - `io.rs` streams terminal output and emits CWD updates
  - `sftp.rs` implements remote file operations and emits transfer progress events consumed by `TransferContext`
  - `tunnel.rs` manages local / remote / dynamic SSH tunnel behavior
- SFTP commands exposed to the frontend are in `src-tauri/src/cmd/sftp.rs`; the file explorer and transfer UI sit on top of these commands and events.
- File watcher / auto-upload flows bridge backend and child windows: remote files are downloaded locally, watched by `src-tauri/src/core/watcher.rs`, then uploaded back through the auto-upload UI flow.

### Persistence model
- App data lives under `~/.nyaterm/`, but the primary store is now `~/.nyaterm/nyaterm.redb` rather than a set of standalone JSON files.
- Important redb JSON documents include `settings`, `sessions`, `keys`, `passwords`, `otp`, `quick-command`, `tunnels`, `proxies`, `history`, `cloud-sync`, `cloud-sync-state`, `ai-history`, and `ai-audit`.
- Important text documents stored through the same layer include `known_hosts` and `master.key`.
- Sensitive values are encrypted before being written; cloud-sync credentials and other secret-bearing config need to stay within the existing crypto/storage helpers.
- When changing settings or workspace persistence, update both the frontend defaults (`AppContext` / `ChildAppProvider`) and the Rust persistence/migration code (`src-tauri/src/config/settings/mod.rs`, `src-tauri/src/config/ui.rs`, and related `src-tauri/src/storage.rs` / cloud-sync snapshot code when applicable).
- The app also contains legacy Dragonfly migration paths; if you change persistence formats, check that import/migration behavior still makes sense for `~/.dragonfly/` data.

## Project-specific guidance

- If a task touches UI, prefer shadcn/ui patterns and components. shadcn is configured in `components.json`, and shared UI components live in `src/components/ui/`. Add new shadcn components via `pnpm dlx shadcn@latest add <component>`.
- When changing user-facing UI text, update all locale files:
  - `src/i18n/locales/en.json`
  - `src/i18n/locales/zh-CN.json`
  - `src/i18n/locales/zh-TW.json`
  - `src/i18n/locales/ko.json`
- Frontend unit tests run on Vitest (`pnpm test`); setup lives in `src/test/setup.ts`. Backend tests are Rust tests under `src-tauri/` and `src-tauri/crates/{otp,nyaterm-mcp,nyaterm-mcp-protocol}/`.
- Frontend linting includes a no-`console` check before Biome (`pnpm lint` runs `scripts/check-no-console.mjs` then `biome lint src/`).
- Vite uses the `@` alias for `src/`.
- Tauri dev/build behavior is configured in `src-tauri/tauri.conf.json`.
- There is a separate Docusaurus docs app in `docs-site/`. The most useful repo docs for implementation context are in `docs-site/docs/development/` (`architecture.md`, `frontend.md`, `backend.md`, `setup.md`, `contributing.md`).
- Repo docs and recent history use Conventional Commits (`feat:`, `fix:`, `perf:`, `chore:`).