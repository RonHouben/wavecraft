# User Stories: CLI Start Command (`wavecraft start`)

## Overview

Currently, after scaffolding a new plugin with `wavecraft create my_plugin`, users are told to run `cargo xtask dev` to start development servers. This requires knowledge of the xtask pattern and isn't consistent with the CLI's naming convention.

This feature adds a `wavecraft start` command that:
1. Detects if `npm install` needs to run (missing `node_modules`)
2. Starts the development servers (Vite + WebSocket backend)
3. Provides a unified, discoverable experience through the CLI

## Version

**Target Version:** `0.8.0` (minor bump from `0.7.2`)

**Rationale:** Adding a new CLI command is a significant feature that improves developer experience. This warrants a minor version bump per coding standards.

---

## User Story 1: Start Development Servers

**As a** plugin developer  
**I want** to run `wavecraft start` from my plugin project  
**So that** I can quickly start the development environment without remembering internal tooling commands

### Acceptance Criteria
- [ ] Running `wavecraft start` in a Wavecraft plugin project starts both:
  - Vite dev server (default port 5173)
  - WebSocket backend server (default port 9000)
- [ ] Command fails gracefully with a helpful message if not run from a Wavecraft project directory
- [ ] Both servers run in foreground, Ctrl+C stops both
- [ ] Console output shows clear status: "Starting dev servers..." and accessible URLs

### Notes
- Detection of a Wavecraft project: presence of `ui/` and `engine/` directories, or a specific marker file
- Must handle graceful shutdown of both processes

---

## User Story 2: Automatic Dependency Installation

**As a** plugin developer  
**I want** the CLI to detect missing dependencies and install them  
**So that** I don't have to remember to run `npm install` manually after scaffolding

### Acceptance Criteria
- [ ] Before starting servers, check if `ui/node_modules` exists
- [ ] If missing, prompt user: "Dependencies not installed. Run npm install? (Y/n)"
- [ ] If user confirms, run `npm install` in the `ui/` directory
- [ ] If user declines, abort with helpful message
- [ ] Show installation progress
- [ ] Option `--no-install` to skip the prompt and fail if dependencies missing
- [ ] Option `--install` to auto-install without prompting

### Notes
- Use `npm install` rather than yarn/pnpm for simplicity (matches template)
- Consider also checking if `cargo build` has been run / dependencies are there

---

## User Story 3: Custom Port Configuration

**As a** plugin developer  
**I want** to specify custom ports for the development servers  
**So that** I can avoid conflicts with other services running on my machine

### Acceptance Criteria
- [ ] `--port <PORT>` or `-p <PORT>` sets the WebSocket backend port (default: 9000)
- [ ] `--ui-port <PORT>` sets the Vite dev server port (default: 5173)
- [ ] Invalid port values show helpful error messages
- [ ] Port conflicts are detected before starting servers (nice-to-have)

### Notes
- Ports must be passed to both Vite and the standalone server
- Consider environment variables as alternative (WAVECRAFT_PORT, WAVECRAFT_UI_PORT)

---

## User Story 4: Update "Next Steps" Output

**As a** plugin developer  
**I want** the `wavecraft create` command to show `wavecraft start` in next steps  
**So that** I have a consistent CLI experience

### Acceptance Criteria
- [ ] After running `wavecraft create my_plugin`, next steps show:
  ```
  Next steps:
    cd my_plugin
    wavecraft start    # Start development servers
  ```
- [ ] Optional: Show help text like `wavecraft start --help` for options

### Notes
- Simple text change in the `new` command output
- Keep documentation link

---

## Out of Scope

- Hot reload of Rust DSP code (requires full rebuild)
- Production build command (separate future feature)
- Multiple project support (monorepo)

---

## Technical Notes

### Project Detection

The command should detect a Wavecraft project by checking for:
1. Presence of `ui/` directory with `package.json`
2. Presence of `engine/` directory with `Cargo.toml`
3. (Optional) A marker like `wavecraft.toml` or section in `Cargo.toml`

### Implementation Approach

The CLI should:
1. Validate current directory is a Wavecraft project
2. Check for `ui/node_modules`, prompt to install if missing
3. Spawn two processes:
   - `npm run dev` in `ui/` (or direct vite command)
   - `cargo run -p standalone -- --dev-server --port <PORT>` in `engine/`
4. Handle SIGINT/SIGTERM to gracefully stop both

### Dependencies

- May need `tokio` or similar for process management
- Signal handling for graceful shutdown
