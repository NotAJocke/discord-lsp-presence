use std::path::Path;

#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub name: String,
    pub icon_key: String,
}

impl LanguageInfo {
    fn new(name: &str, icon_key: &str) -> Self {
        Self {
            name: name.to_string(),
            icon_key: icon_key.to_string(),
        }
    }
}

pub fn detect_language(filename: &str) -> LanguageInfo {
    let extension = Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    match extension.as_deref() {
        Some("rs") => LanguageInfo::new("Rust", "rust"),
        Some("py") => LanguageInfo::new("Python", "python"),
        Some("js") | Some("mjs") | Some("cjs") => LanguageInfo::new("JavaScript", "javascript"),
        Some("ts") | Some("mts") | Some("cts") => LanguageInfo::new("TypeScript", "typescript"),
        Some("tsx") => LanguageInfo::new("TypeScript", "typescript"),
        Some("jsx") => LanguageInfo::new("JavaScript", "javascript"),
        Some("go") => LanguageInfo::new("Go", "go"),
        Some("java") => LanguageInfo::new("Java", "java"),
        Some("c") => LanguageInfo::new("C", "c"),
        Some("cc") | Some("cpp") | Some("cxx") | Some("hpp") | Some("hh") | Some("hxx") => {
            LanguageInfo::new("C++", "cpp")
        }
        Some("rb") => LanguageInfo::new("Ruby", "ruby"),
        Some("php") => LanguageInfo::new("PHP", "php"),
        Some("html") | Some("htm") => LanguageInfo::new("HTML", "html"),
        Some("css") => LanguageInfo::new("CSS", "css"),
        Some("json") => LanguageInfo::new("JSON", "json"),
        Some("md") | Some("markdown") => LanguageInfo::new("Markdown", "markdown"),
        Some("toml") => LanguageInfo::new("TOML", "toml"),
        Some("yaml") | Some("yml") => LanguageInfo::new("YAML", "yaml"),
        Some("sh") | Some("bash") | Some("zsh") => LanguageInfo::new("Shell", "shell"),
        Some("lua") => LanguageInfo::new("Lua", "lua"),
        Some("kt") | Some("kts") => LanguageInfo::new("Kotlin", "kotlin"),
        Some("swift") => LanguageInfo::new("Swift", "swift"),
        Some("cs") => LanguageInfo::new("C#", "csharp"),
        Some("zig") => LanguageInfo::new("Zig", "zig"),
        Some("dart") => LanguageInfo::new("Dart", "dart"),
        Some("ex") | Some("exs") => LanguageInfo::new("Elixir", "elixir"),
        Some("erl") | Some("hrl") => LanguageInfo::new("Erlang", "erlang"),
        Some("scala") | Some("sc") => LanguageInfo::new("Scala", "scala"),
        Some("r") => LanguageInfo::new("R", "r"),
        Some("sql") => LanguageInfo::new("SQL", "sql"),
        Some(ext) if !ext.is_empty() => LanguageInfo::new(ext, ""),
        _ => LanguageInfo::new("Unknown", ""),
    }
}

#[cfg(test)]
mod tests {
    use super::detect_language;

    #[test]
    fn detects_rust() {
        let lang = detect_language("main.rs");
        assert_eq!(lang.name, "Rust");
        assert_eq!(lang.icon_key, "rust");
    }

    #[test]
    fn falls_back_to_extension_without_icon() {
        let lang = detect_language("sample.foobar");
        assert_eq!(lang.name, "foobar");
        assert!(lang.icon_key.is_empty());
    }
}
