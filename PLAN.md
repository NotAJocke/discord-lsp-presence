# Future Implementation: Editor & Language Icons

## Overview

Add support for displaying editor icon (large image) and programming language icon (small image) in Discord Rich Presence, with configurable editor name and language detection from file extensions.

## Decisions

- **Asset source**: Upload language icons to Discord application assets
- **Editor name**: Configurable via config file

---

## Implementation Details

### 1. New File: `src/language.rs`

Create a language detection module:

```rust
pub struct LanguageInfo {
    pub name: String,      // Display name: "Rust", "Python", etc.
    pub icon_key: String,  // Discord asset key: "rust", "python", etc.
}

pub fn detect_language(filename: &str) -> LanguageInfo
```

- Map file extensions to language names and Discord asset keys
- Handle common languages: Rust, JavaScript, TypeScript, Python, Go, Java, C, C++, Ruby, PHP, etc.
- Fallback: use file extension as name, empty icon_key

#### Extension Mapping Examples

| Extension | Language Name | Icon Key |
|-----------|---------------|----------|
| `.rs` | Rust | `rust` |
| `.py` | Python | `python` |
| `.js` | JavaScript | `javascript` |
| `.ts` | TypeScript | `typescript` |
| `.go` | Go | `go` |
| `.java` | Java | `java` |
| `.c` | C | `c` |
| `.cpp` / `.cc` | C++ | `cpp` |
| `.rb` | Ruby | `ruby` |
| `.php` | PHP | `php` |
| `.html` | HTML | `html` |
| `.css` | CSS | `css` |
| `.json` | JSON | `json` |
| `.md` | Markdown | `markdown` |
| `.toml` | TOML | `toml` |
| `.yaml` / `.yml` | YAML | `yaml` |
| `.sh` | Shell | `shell` |
| `.lua` | Lua | `lua` |
| `.kt` | Kotlin | `kotlin` |
| `.swift` | Swift | `swift` |

---

### 2. Modify: `src/config.rs`

#### New Config Fields

```toml
# Top-level config
editor_name = "Helix"  # optional, default: "Helix"

[activity]
# Existing fields
details = "Editing {filename}"
state = "in {workspace}"

# New editor image fields (large image)
editor_image_key = "helix"           # optional, default: none
editor_image_text = "Helix Editor"   # optional, supports {editor} placeholder

# New language image field (small image)
language_images = true               # optional, default: true
```

#### New Placeholders

| Placeholder | Description |
|-------------|-------------|
| `{language}` | Detected language name (e.g., "Rust") |
| `{editor}` | Configured editor name (e.g., "Helix") |

#### Code Changes

```rust
#[derive(Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub application_id: Option<u64>,
    #[serde(default)]
    pub activity: Option<ActivityConfig>,
    #[serde(default)]
    pub time_tracking: Option<TimeTracking>,
    #[serde(default)]
    pub editor_name: Option<String>,  // NEW
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ActivityConfig {
    // Existing fields
    pub details: Option<String>,
    pub state: Option<String>,
    pub large_image_key: Option<String>,
    pub large_image_text: Option<String>,
    pub small_image_key: Option<String>,
    pub small_image_text: Option<String>,
    
    // New fields
    pub editor_image_key: Option<String>,   // NEW
    pub editor_image_text: Option<String>,  // NEW
    pub language_images: Option<bool>,      // NEW
}
```

#### New Methods

```rust
impl Config {
    pub fn get_editor_name(&self) -> &str {
        self.editor_name.as_deref().unwrap_or("Helix")
    }
    
    pub fn show_language_images(&self) -> bool {
        self.activity.as_ref().and_then(|a| a.language_images).unwrap_or(true)
    }
}
```

---

### 3. Modify: `src/main.rs`

#### Changes

1. Import language module:
   ```rust
   mod language;
   use language::detect_language;
   ```

2. Update `handle_file_event`:
   ```rust
   async fn handle_file_event(&self, uri: &Url) {
       let filename = get_filename_from_uri(uri);
       let workspace_name = detect_workspace_name(uri);
       
       if let Some(filename) = filename {
           let workspace = workspace_name.unwrap_or_else(|| "unknown workspace".to_string());
           let language = detect_language(&filename);  // NEW
           
           // ... timestamp logic ...
           
           if DiscordClient::is_ready() {
               discord::update_presence(
                   &self.discord,
                   &self.client,
                   &self.config,
                   &filename,
                   &workspace,
                   &language,  // NEW
                   start_timestamp,
               ).await;
           }
       }
   }
   ```

---

### 4. Modify: `src/discord.rs`

#### Update Function Signature

```rust
pub async fn update_presence(
    discord: &Arc<Mutex<DiscordClient>>,
    client: &Client,
    config: &Config,
    filename: &str,
    workspace: &str,
    language: &LanguageInfo,  // NEW
    start_timestamp: Option<u64>,
)
```

#### Placeholder Replacement Helper

```rust
fn replace_placeholders(text: &str, filename: &str, workspace: &str, language: &LanguageInfo, editor: &str) -> String {
    text.replace("{filename}", filename)
        .replace("{workspace}", workspace)
        .replace("{language}", &language.name)
        .replace("{editor}", editor)
}
```

#### Activity Building Logic

```rust
// Build details/state with all placeholders
let editor_name = config.get_editor_name();
let details = replace_placeholders(&activity_config.details.unwrap_or_default(), filename, workspace, language, editor_name);
let state = replace_placeholders(&activity_config.state.unwrap_or_default(), filename, workspace, language, editor_name);

// Large image: Editor icon
let large_image_key = activity_config.editor_image_key.clone();
let large_image_text = activity_config.editor_image_text.as_ref().map(|t| {
    replace_placeholders(t, filename, workspace, language, editor_name)
});

// Small image: Language icon (only if enabled and icon exists)
let small_image_key = if config.show_language_images() && !language.icon_key.is_empty() {
    Some(language.icon_key.clone())
} else {
    None
};
let small_image_text = Some(language.name.clone());
```

---

### 5. Update: `AGENTS.md`

Add documentation for:
- New config options
- New placeholders
- Required Discord assets
- Language detection behavior

---

## Required Discord Application Assets

Upload these images to your Discord application (https://discord.com/developers/applications):

### Editor Icons
- `helix` - Helix logo (or your preferred editor icon)

### Language Icons
Upload icons for languages you use:
- `rust`, `python`, `javascript`, `typescript`, `go`, `java`, `c`, `cpp`, `ruby`, `php`, `html`, `css`, `json`, `markdown`, `toml`, `yaml`, `shell`, `lua`, `kotlin`, `swift`

Recommended sources for language icons:
- https://github.com/denoland/deno/tree/main/docs/images (various)
- https://github.com/devicons/devicon (comprehensive set)
- Simple colored squares with language initials

---

## Default Behavior

| Scenario | Behavior |
|----------|----------|
| `editor_image_key` not set | No large image displayed |
| `language_images = false` | No small image, language only in text if placeholders used |
| Language icon doesn't exist in Discord assets | Small image won't display, no error |
| Unknown file extension | Language name = extension, no icon |

---

## Example Configurations

### Minimal (uses defaults)
```toml
# Just enable editor image
[activity]
editor_image_key = "helix"
editor_image_text = "Helix Editor"
```

### Full Configuration
```toml
application_id = 123456789012345678
editor_name = "Helix"
time_tracking = "workspace"

[activity]
details = "Editing {filename} ({language})"
state = "in {workspace}"
editor_image_key = "helix"
editor_image_text = "{editor}"
language_images = true
```

### Disable Language Icons
```toml
[activity]
editor_image_key = "helix"
editor_image_text = "Helix Editor"
language_images = false
```

---

## Implementation Order

1. Create `src/language.rs` with extension mapping
2. Update `src/config.rs` with new fields and methods
3. Update `src/discord.rs` with new signature and placeholder handling
4. Update `src/main.rs` to detect language and pass to update_presence
5. Update `AGENTS.md` with documentation
6. Test with various file types
