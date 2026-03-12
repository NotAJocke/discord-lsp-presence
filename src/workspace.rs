use std::path::PathBuf;
use url::Url;

fn find_vcs_root(path: &std::path::Path) -> Option<(PathBuf, VcsType)> {
    let mut current_dir = path.parent();

    while let Some(dir) = current_dir {
        let git_dir = dir.join(".git");
        let jj_dir = dir.join(".jj");

        if jj_dir.is_dir() {
            return Some((dir.to_path_buf(), VcsType::Jj));
        }
        if git_dir.exists() {
            return Some((dir.to_path_buf(), VcsType::Git));
        }
        current_dir = dir.parent();
    }

    None
}

#[derive(PartialEq)]
enum VcsType {
    Git,
    Jj,
}

pub fn detect_workspace_name(uri: &Url) -> Option<String> {
    let path = uri.to_file_path().ok()?;

    if let Some((root, _)) = find_vcs_root(&path) {
        return root
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string());
    }

    path.parent()
        .and_then(|dir| dir.file_name())
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
}

pub fn is_git_repo(uri: &Url) -> bool {
    let path = match uri.to_file_path() {
        Ok(p) => p,
        Err(_) => return false,
    };

    find_vcs_root(&path).is_some()
}

pub fn get_git_remote_url(uri: &Url) -> Option<String> {
    let path = match uri.to_file_path() {
        Ok(p) => p,
        Err(_) => return None,
    };

    let mut current_dir: Option<PathBuf> = Some(path.parent()?.to_path_buf());

    while let Some(dir) = current_dir {
        // Check for .git directory or .git file (colocated jj repo)
        let git_dir = dir.join(".git");

        if git_dir.is_dir() {
            // Normal git repo - look for .git/config
            let git_config = git_dir.join("config");
            if git_config.exists() {
                if let Ok(remote_url) = parse_git_remote(&git_config) {
                    return Some(remote_url);
                }
            }
        } else if git_dir.exists() {
            // Colocated repo (.git is a file) - try to find the actual git dir
            if let Ok(content) = std::fs::read_to_string(&git_dir) {
                let gitdir_path = content.trim().trim_start_matches("gitdir:");
                let gitdir_path = gitdir_path.trim();
                let actual_git_dir = if std::path::Path::new(gitdir_path).is_absolute() {
                    std::path::PathBuf::from(gitdir_path)
                } else {
                    dir.join(gitdir_path)
                };
                let git_config = actual_git_dir.join("config");
                if git_config.exists() {
                    if let Ok(remote_url) = parse_git_remote(&git_config) {
                        return Some(remote_url);
                    }
                }
            }
        }

        // Check for jj native repo
        let jj_dir = dir.join(".jj");
        if jj_dir.is_dir() {
            let jj_config = jj_dir.join("repo").join("config.toml");
            if jj_config.exists() {
                if let Ok(remote_url) = parse_jj_config(&jj_config) {
                    return Some(remote_url);
                }
            }
        }

        current_dir = dir.parent().map(|p| p.to_path_buf());
    }

    None
}

fn parse_git_remote(git_config_path: &PathBuf) -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(git_config_path)?;

    let mut in_remote_origin = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            in_remote_origin = line == "[remote \"origin\"]";
            continue;
        }

        if in_remote_origin && line.starts_with("url") {
            if let Some(url) = line.split('=').nth(1) {
                let url = url.trim();
                return Ok(convert_git_url_to_web(url));
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No remote origin found",
    ))
}

fn parse_jj_config(jj_config_path: &PathBuf) -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(jj_config_path)?;

    let mut in_remote_origin = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            in_remote_origin = line.starts_with("[remote \"");
            continue;
        }

        if in_remote_origin && line.starts_with("url") {
            if let Some(url) = line.split('=').nth(1) {
                let url = url.trim();
                return Ok(convert_git_url_to_web(url));
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No remote origin found in jj config",
    ))
}

fn convert_git_url_to_web(url: &str) -> String {
    if url.starts_with("https://") || url.starts_with("http://") {
        return url.to_string();
    }

    if url.starts_with("git@") {
        let url = url.trim_start_matches("git@");

        if let Some((host, path)) = url.split_once(':') {
            let path = path.trim_end_matches(".git");
            return format!("https://{}/{}", host, path);
        }
    }

    url.to_string()
}

pub fn get_filename_from_uri(uri: &Url) -> Option<String> {
    uri.path_segments()
        .and_then(|s| s.last())
        .map(|s| s.to_string())
}
