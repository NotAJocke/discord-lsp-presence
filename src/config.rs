use serde::Deserialize;
use std::env::home_dir;
use std::path::PathBuf;

pub fn get_config_dir() -> Option<PathBuf> {
    home_dir().and_then(|home| Some(home.join(".config").join("discord-presence-lsp")))
}

pub fn ensure_config() -> std::result::Result<PathBuf, &'static str> {
    let Some(path) = get_config_dir() else {
        return Err("Couldn't get config directory");
    };

    if !path.exists() {
        if std::fs::create_dir_all(&path).is_err() {
            return Err("Couldn't create config dir");
        }
    }

    let path = path.join("config.toml");

    if !path.exists() {
        if std::fs::write(&path, "foo = 'bar'").is_err() {
            return Err("Couldn't create config file");
        }
    }

    Ok(path)
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub foo: String,
}
