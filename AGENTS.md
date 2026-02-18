This project adds Discord Rich Presence support to the Helix editor by running a small, standalone Language Server written in Rust. The server uses the tower-lsp crate to communicate with Helix through the Language Server Protocol and listens to standard editor events such as opening and changing documents.

Based on these LSP events, the server infers the current editing context, including the active file and the workspace. It then publishes this information to Discord using the discord-presence crate, which communicates with the locally running Discord client via IPC.

The language server does not implement any language features like diagnostics, code completion, or formatting. It exists solely as an integration layer between Helix and Discord and remains active for the duration of the editor session. Because Helix does not support extensions, using an LSP server allows this integration without modifying Helix itself or replacing its built-in language servers.

## Architecture

The codebase is organized into modular components:

- **src/main.rs** - Main LSP server implementation, event handlers, and server initialization
- **src/state.rs** - FileState and WorkspaceState structs for tracking current file/workspace and timestamps
- **src/config.rs** - Configuration management with optional fields and hardcoded defaults
- **src/workspace.rs** - Workspace detection (looks for .git directory) and filename extraction from URIs
- **src/discord.rs** - Discord Rich Presence update helpers

## Configuration

Configuration is optional. The server uses hardcoded defaults if no config file exists.

Config location: `~/.config/discord-presence-lsp/config.toml`

### Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `application_id` | `u64` | `1470506076574187745` | Discord application ID |
| `time_tracking` | `"file"` or `"workspace"` | `"file"` | How to track elapsed time |
| `activity.details` | `string` | `"Editing: {filename}"` | Top line in Discord presence |
| `activity.state` | `string` | `"in {workspace}"` | Bottom line in Discord presence |
| `activity.large_image_key` | `string` | none | Large image asset name |
| `activity.large_image_text` | `string` | none | Large image hover text |
| `activity.small_image_key` | `string` | none | Small image asset name |
| `activity.small_image_text` | `string` | none | Small image hover text |

### Time Tracking Modes

- **`"file"`** (default): Timer resets each time a different file is opened
- **`"workspace"`**: Timer only resets when switching to a different workspace/project

### Example Config

```toml
application_id = 123456789012345678
time_tracking = "workspace"

[activity]
details = "Working on {filename}"
state = "Project: {workspace}"
large_image_key = "helix"
large_image_text = "Helix Editor"
```

### Placeholders

The `{filename}` and `{workspace}` placeholders can be used in `details`, `state`, and image text fields.

## Features

- **Workspace Detection**: Automatically detects the project/workspace name by walking up the directory tree looking for a `.git` folder, falling back to the immediate parent directory
- **File Tracking**: Tracks the currently open file and workspace with timestamps
- **Immediate Presence**: Sets Discord presence immediately when Helix opens with a file
- **Time Display**: Shows elapsed time in Discord (configurable: per-file or per-workspace)
- **Flexible Configuration**: All settings are optional with sensible defaults

## Current Limitations

- No `did_close` handler yet (presence persists when file is closed)
- No idle detection (timer keeps running even when not typing)
- No programming language detection or icons
