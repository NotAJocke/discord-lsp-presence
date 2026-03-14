#[derive(Debug, Clone)]
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
