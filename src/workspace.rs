use url::Url;

pub fn detect_workspace_name(uri: &Url) -> Option<String> {
    let path = uri.to_file_path().ok()?;
    let mut current_dir = path.parent()?;

    loop {
        if current_dir.join(".git").exists() {
            return current_dir
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string());
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent,
            None => break,
        }
    }

    path.parent()
        .and_then(|dir| dir.file_name())
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
}

pub fn get_filename_from_uri(uri: &Url) -> Option<String> {
    uri.path_segments()
        .and_then(|s| s.last())
        .map(|s| s.to_string())
}
