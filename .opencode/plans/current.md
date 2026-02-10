# Discord LSP Presence Implementation Plan

## Goal

Start the Discord client on LSP initialization and show a "hello world" presence when ready. The LSP should fail if Discord isn't running.

## Current State

- Basic LSP server with `tower-lsp`
- Discord client created but not started in `main`
- Discord app ID already configured

## Implementation Steps

### 1. Move DiscordClient into Backend struct

**File:** `src/main.rs`

- Import `Arc` and `Mutex` for shared state
- Add DiscordClient as field in Backend struct
- Initialize DiscordClient in LspService::new callback

### 2. Configure Discord ready handler

**File:** `src/main.rs`

- Add on_ready callback to Discord client
- Set Activity with "hello world" state
- This will be one-time setup when Discord connects

### 3. Start Discord client in initialized()

**File:** `src/main.rs`

- Call `discord.start()` in the `initialized()` method
- Handle errors - if Discord fails, log and potentially fail LSP

### 4. Make LSP fail if Discord unavailable

**File:** `src/main.rs`

- Since Discord start is async in initialized(), we need to handle the case where it fails
- Could panic or use process exit on failure

## Code Changes Preview

### Imports to add

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
```

### Backend struct change

```rust
struct Backend {
    client: Client,
    discord: Arc<Mutex<DiscordClient>>,
}
```

### Main function changes

```rust
let discord = Arc::new(Mutex::new(DiscordClient::new(DISCORD_APPLICATION_ID)));

// Configure on_ready handler
discord.on_ready({
    let discord = Arc::clone(&discord);
    move |data| {
        discord.set_activity(Activity::new().state("hello world"));
    }
});

let (service, socket) = LspService::new(move |client| Backend { client, discord });
```

### Initialized method

```rust
async fn initialized(&self, _: InitializedParams) {
    let discord = self.discord.lock().await;
    discord.start().expect("Failed to connect to Discord");
    self.client.log_message(MessageType::INFO, "Discord client started").await;
}
```

## Testing

1. Start Discord desktop app
2. Run the LSP
3. Check Discord presence shows "hello world"
4. Test without Discord running - LSP should fail
