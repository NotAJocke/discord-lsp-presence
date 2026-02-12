This project adds Discord Rich Presence support to the Helix editor by running a small, standalone Language Server written in Rust. The server uses the tower-lsp crate to communicate with Helix through the Language Server Protocol and listens to standard editor events such as opening and changing documents.

Based on these LSP events, the server infers the current editing context, including the active file and the workspace. It then publishes this information to Discord using the discord-presence crate, which communicates with the locally running Discord client via IPC.

The language server does not implement any language features like diagnostics, code completion, or formatting. It exists solely as an integration layer between Helix and Discord and remains active for the duration of the editor session. Because Helix does not support extensions, using an LSP server allows this integration without modifying Helix itself or replacing its built-in language servers.

## Architecture

The codebase is organized into modular components:

- **src/main.rs** - Main LSP server implementation, event handlers, and server initialization
- **src/state.rs** - FileState struct for tracking current file, workspace, and editing start time
- **src/config.rs** - Configuration file management (creates config in ~/.config/discord-presence-lsp/)
- **src/workspace.rs** - Workspace detection (looks for .git directory) and filename extraction from URIs
- **src/discord.rs** - Discord Rich Presence update helpers

## Features

- **Workspace Detection**: Automatically detects the project/workspace name by walking up the directory tree looking for a `.git` folder, falling back to the immediate parent directory
- **File Tracking**: Tracks the currently open file and workspace with timestamps
- **Immediate Presence**: Sets Discord presence immediately when Helix opens with a file (no editing required)
- **Time Display**: Shows elapsed time since opening each file in Discord

## Discord Display Format

- **Details** (top line): `Editing: {filename}`
- **State** (bottom line): `in {workspace_name}`
- **Timestamp**: Shows elapsed time since file was opened

## Current Limitations

- No `did_close` handler yet (presence persists when file is closed)
- No idle detection (timer keeps running even when not typing)
- No programming language detection or icons
- Configuration system is minimal (placeholder only)
