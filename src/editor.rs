use tower_lsp::lsp_types::InitializeParams;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorInfo {
    pub name: String,
    pub icon_key: String,
}

impl Default for EditorInfo {
    fn default() -> Self {
        Self {
            name: "Unknown Editor".to_string(),
            icon_key: "".to_string(),
        }
    }
}

impl EditorInfo {
    pub fn new(name: &str, icon_key: &str) -> Self {
        Self {
            name: name.to_string(),
            icon_key: icon_key.to_string(),
        }
    }
}

pub fn detect_editor(editor_name: &str) -> EditorInfo {
    let lower = editor_name.to_ascii_lowercase();

    if lower.contains("helix") {
        EditorInfo::new("Helix", "helix")
    } else if lower.contains("opencode") {
        EditorInfo::new("OpenCode", "opencode")
    } else if lower.contains("neovim") || lower.contains("nvim") {
        EditorInfo::new("Neovim", "neovim")
    } else if lower.contains("vim") || lower.contains("gvim") {
        EditorInfo::new("Vim", "vim")
    } else if lower.contains("vscode")
        || lower.contains("vs code")
        || lower.contains("visual studio code")
    {
        EditorInfo::new("VS Code", "vscode")
    } else if lower.contains("idea") || lower.contains("jetbrains") {
        EditorInfo::new("JetBrains IDE", "jetbrains")
    } else if lower.contains("emacs") {
        EditorInfo::new("Emacs", "emacs")
    } else if lower.contains("sublime") {
        EditorInfo::new("Sublime Text", "sublime")
    } else if lower.contains("atom") {
        EditorInfo::new("Atom", "atom")
    } else if lower.contains("zed") {
        EditorInfo::new("Zed", "zed")
    } else if lower.contains("kate") {
        EditorInfo::new("Kate", "kate")
    } else if lower.contains("notepad") {
        EditorInfo::new("Notepad++", "notepadpp")
    } else {
        EditorInfo::new(editor_name, "")
    }
}

pub fn extract_editor_from_init_options(params: &InitializeParams) -> Option<String> {
    let init_options = params.initialization_options.as_ref()?;

    if let Some(editor) = init_options.get("editor").and_then(|v| v.as_str()) {
        return Some(editor.to_string());
    }

    if let Some(name) = init_options.get("name").and_then(|v| v.as_str()) {
        return Some(name.to_string());
    }

    None
}

pub fn extract_editor_from_client_info(params: &InitializeParams) -> Option<String> {
    params.client_info.as_ref().map(|ci| ci.name.clone())
}

pub fn determine_editor_name(params: &InitializeParams, cli_editor: Option<String>) -> String {
    if let Some(editor) = cli_editor {
        return editor;
    }

    if let Some(editor) = extract_editor_from_init_options(params) {
        return editor;
    }

    if let Some(editor) = extract_editor_from_client_info(params) {
        return editor;
    }

    "Unknown Editor".to_string()
}
