This project adds Discord Rich Presence support to LSP-compatible editors by running a small, standalone Language Server written in Rust. The server uses the tower-lsp crate to communicate with the editor through the Language Server Protocol and listens to standard editor events such as opening and changing documents.

Based on these LSP events, the server infers the current editing context, including the active file and the workspace. It then publishes this information to Discord using the discord-presence crate, which communicates with the locally running Discord client via IPC.

The language server does not implement any language features like diagnostics, code completion, or formatting. It exists solely as an integration layer between the editor and Discord and remains active for the duration of the editor session.

## Architecture

The codebase is organized into modular components:

- **src/main.rs** - Application entry point, initialization and server startup
- **src/server.rs** - Backend struct, LanguageServer trait implementation, and LSP event handlers
- **src/state.rs** - FileState and WorkspaceState structs for tracking current file/workspace and timestamps
- **src/config.rs** - Configuration management with optional fields and hardcoded defaults
- **src/workspace.rs** - Workspace detection (looks for .git directory) and filename extraction from URIs
- **src/discord.rs** - Discord Rich Presence update helpers and handler setup
- **src/language.rs** - Language detection from file extensions and Discord icon key mapping
- **src/logging.rs** - Logging initialization with file output and rotation
- **src/activity.rs** - Discord activity building helpers
- **src/editor.rs** - Editor detection, icon mapping, and LSP parameter parsing helpers

## Configuration

Configuration is optional. The server uses hardcoded defaults if no config file exists.

Config location: `~/.config/discord-presence-lsp/config.toml`

### Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `application_id` | `u64` | `1470506076574187745` | Discord application ID |
| `enabled` | `bool` | `true` | Whether to update Discord presence |
| `time_tracking` | `"file"` or `"workspace"` | `"file"` | How to track elapsed time |
| `activity.details` | `string` | `"Editing: {filename}"` | Top line in Discord presence |
| `activity.state` | `string` | `"in {workspace}"` | Bottom line in Discord presence |
| `activity.editor_image_key` | `string` | none | Large image asset key for editor icon |
| `activity.editor_image_text` | `string` | none | Large image hover text, supports placeholders |
| `activity.language_images` | `bool` | `true` | Whether to show detected language icon as small image |
| `activity.button_label` | `string` | `"View Repository"` | Button label (only shown if git remote URL is detected) |
| `activity.large_image_key` | `string` | none | Legacy fallback for editor large image key |
| `activity.large_image_text` | `string` | none | Legacy fallback for editor large image text |

### Editor Detection

The editor name is automatically detected from the LSP client. The detection priority is:

1. **CLI flag**: `--editor=<name>` (highest priority)
2. **`initialization_options`**: From LSP `initialize` request (field: `editor` or `name`)
3. **`client_info`**: From LSP handshake (e.g., "Helix 24.01")
4. **Fallback**: "Unknown Editor"

Example usage:
```bash
discord-lsp-presence --editor=helix
```

### Time Tracking Modes

- **`"file"`** (default): Timer resets each time a different file is opened
- **`"workspace"`**: Timer only resets when switching to a different workspace/project

### Example Config

```toml
application_id = 123456789012345678
enabled = true
time_tracking = "workspace"

[activity]
details = "Working on {filename} ({language})"
state = "Project: {workspace}"
editor_image_key = "helix"
editor_image_text = "{editor}"
language_images = true
```

### Placeholders

The `{filename}`, `{workspace}`, `{language}`, and `{editor}` placeholders can be used in `details`, `state`, and image text fields.

### Project-Level Configuration

You can override global settings per-project by creating `.discord-presence.toml` in your project root (where `.git` or `.jj` lives). Project config merges with global config - project values override global ones.

**Example:**

Global config (`~/.config/discord-presence-lsp/config.toml`):
```toml
[activity]
details = "Editing: {filename}"
state = "in {workspace}"
```

Project config (`.discord-presence.toml`):
```toml
[activity]
details = "Working on {language}"
state = ""  # Override to show nothing
```

**Unsetting Fields:**

Set empty string to override a field with nothing. This works for all string fields:

| Field | Empty string value |
|-------|-------------------|
| `state = ""` | Removes the state line |
| `details = ""` | Removes the details line |
| `button_label = ""` | Removes the repository button |
| `large_image_key = ""` | Falls back to auto-detected editor icon |
| `editor_image_text = ""` | Shows editor name (or nothing if also empty) |
| `language_images = false` | Disables language icons per-project |

## Logging

Logs are written to `~/.local/share/discord-lsp-presence/logs/app.log` with daily rotation. The logging system uses the `tracing` crate and outputs to both file and stderr.

Log output includes:
- Editor detection events
- Discord connection status
- Presence updates and errors
- File/workspace changes

## Features

- **Editor Auto-Detection**: Automatically detects the editor from LSP client info (CLI flag, initialization_options, or client_info) and selects the matching Discord asset icon
- **Workspace Detection**: Automatically detects the project/workspace name by walking up the directory tree looking for a `.git` or `.jj` folder, falling back to the immediate parent directory
- **File Tracking**: Tracks the currently open file and workspace with timestamps
- **Immediate Presence**: Sets Discord presence immediately when the editor opens with a file
- **Time Display**: Shows elapsed time in Discord (configurable: per-file or per-workspace)
- **Flexible Configuration**: All settings are optional with sensible defaults
- **Language Detection**: Detects language from file extension and can show language icon as the small image
- **Editor Icon Support**: Auto-detected editor icon as the large image (can be overridden via config)
- **View Repository Button**: Shows a clickable "View Repository" button when a git remote URL is detected (works with both Git and Jujutsu/JJ repositories)
- **Git/Jujutsu Support**: Detects both Git (`.git/`) and Jujutsu (`.jj/`) repositories
- **Project-Level Config**: Override global settings per-project with `.discord-presence.toml`

## Current Limitations

- No `did_close` handler yet (presence persists when file is closed)
- No idle detection (timer keeps running even when not typing)
- Buttons are not visible to the user who set the presence (Discord limitation - other users can see them)

## Discord Asset Requirements

Upload Discord application assets for any icons you reference:

**Editor icons** (auto-detected based on editor name):
- `helix`, `opencode`, `neovim`, `vim`, `vscode`, `jetbrains`, `emacs`, `sublime`, `atom`, `zed`, `kate`, `notepadpp`

**Language icons**:
- `rust`, `python`, `javascript`, `typescript`, `go`, `java`, `c`, `cpp`, `csharp`, `ruby`, `php`, `html`, `css`, `json`, `markdown`, `toml`, `yaml`, `shell`, `lua`, `kotlin`, `swift`, `zig`, `dart`, `elixir`, `erlang`, `scala`, `r`, `sql`

If an icon key is not uploaded in Discord, Discord simply omits that image. The editor icon can be overridden via `activity.editor_image_key` in config.
