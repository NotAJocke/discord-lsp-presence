# Discord LSP Presence

A Discord Rich Presence integration for the Helix editor via a standalone Language Server.

## Overview

This project enables Discord Rich Presence support in Helix by running a small Language Server that listens to editor events (opening/changing documents) and publishes the current editing context to Discord.

Since Helix doesn't support extensions, this LSP-based approach allows integration without modifying the editor itself.

## Installation

### From Source

```bash
git clone https://github.com/yourusername/discord-lsp-presence.git
cd discord-lsp-presence
cargo build --release
```

The binary will be at `target/release/discord-lsp-presence`.

## Configuration

Add to your Helix languages.toml:

```toml
[language-server.discord-presence]
command = "/path/to/discord-lsp-presence"
```

Then add the LSP to any language you want tracked:

```toml
[[language]]
name = "rust"
language-servers = ["rust-analyzer", "discord-presence"]
```

### Optional Config File

Create `~/.config/discord-presence-lsp/config.toml`:

```toml
# Discord application ID (optional, uses default)
application_id = 123456789012345678

# Time tracking mode: "file" (default) or "workspace"
time_tracking = "workspace"

[activity]
details = "Working on {filename}"
state = "Project: {workspace}"
large_image_key = "helix"
large_image_text = "Helix Editor"
```

#### Available Options

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `application_id` | `u64` | `1470506076574187745` | Discord application ID |
| `time_tracking` | `"file"` / `"workspace"` | `"file"` | Timer reset behavior |
| `activity.details` | `string` | `"Editing: {filename}"` | Top line |
| `activity.state` | `string` | `"in {workspace}"` | Bottom line |
| `activity.large_image_key` | `string` | none | Large image asset |
| `activity.large_image_text` | `string` | none | Large image text |
| `activity.small_image_key` | `string` | none | Small image asset |
| `activity.small_image_text` | `string` | none | Small image text |

#### Time Tracking

- **`file`** (default): Timer resets when switching files
- **`workspace`**: Timer resets only when switching projects

#### Placeholders

Use `{filename}` and `{workspace}` in text fields.

## Features

- Automatic workspace detection via `.git` directory
- Shows elapsed time in Discord
- Configurable presence text
- No config file required

## Limitations

- No idle detection
- No `did_close` handling
- No language-specific icons

## License

MIT
