use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CliError;

const DEFAULT_BASE_URL: &str = "https://api.bitbucket.org/2.0";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub username: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub current: String,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub dir: PathBuf,
    pub file: PathBuf,
}

fn default_base_url() -> String {
    DEFAULT_BASE_URL.to_string()
}

impl Config {
    pub fn set_profile_with_auth(
        &mut self,
        name: &str,
        username: &str,
        token: &str,
        base_url: Option<&str>,
    ) {
        let profile_name = if name.trim().is_empty() {
            "default"
        } else {
            name.trim()
        };
        let resolved_base_url = base_url
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_BASE_URL);

        self.profiles.insert(
            profile_name.to_string(),
            Profile {
                base_url: resolved_base_url.to_string(),
                token: token.trim().to_string(),
                username: username.trim().to_string(),
            },
        );
        self.current = profile_name.to_string();
    }

    pub fn active_profile(
        &self,
        override_name: Option<&str>,
    ) -> Result<(Profile, String), CliError> {
        let name = override_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(self.current.trim());

        if name.is_empty() {
            return Err(CliError::NotLoggedIn);
        }

        let mut profile = self
            .profiles
            .get(name)
            .cloned()
            .ok_or_else(|| CliError::Config(format!("profile \"{name}\" not found")))?;

        if profile.base_url.trim().is_empty() {
            profile.base_url = DEFAULT_BASE_URL.to_string();
        }

        Ok((profile, name.to_string()))
    }

    pub fn remove_profile(&mut self, name: Option<&str>) -> (String, bool) {
        let target = name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(self.current.trim())
            .to_string();

        if target.is_empty() {
            return (target, false);
        }

        if self.profiles.remove(&target).is_none() {
            return (target, false);
        }

        if self.current == target {
            self.current = self.profiles.keys().next().cloned().unwrap_or_default();
        }

        (target, true)
    }
}

pub fn config_paths() -> Result<ConfigPaths, CliError> {
    let file = if let Some(explicit) = explicit_config_path() {
        PathBuf::from(explicit)
    } else {
        let home = detect_home_dir()?;
        let xdg = env::var("XDG_CONFIG_HOME").ok().map(PathBuf::from);
        config_paths_from_home(&home, xdg.as_deref()).file
    };

    let dir = file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    Ok(ConfigPaths { dir, file })
}

fn config_paths_from_home(home: &Path, xdg_config_home: Option<&Path>) -> ConfigPaths {
    let base = xdg_config_home
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".config"));
    let file = base.join("bb").join("config.json");
    let dir = file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    ConfigPaths { dir, file }
}

pub fn load() -> Result<Config, CliError> {
    let paths = config_paths()?;
    if !paths.file.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&paths.file)
        .map_err(|error| CliError::Config(format!("read config: {error}")))?;
    serde_json::from_str(&content)
        .map_err(|error| CliError::Config(format!("decode config: {error}")))
}

pub fn save(config: &Config) -> Result<(), CliError> {
    let paths = config_paths()?;
    fs::create_dir_all(&paths.dir)
        .map_err(|error| CliError::Config(format!("create config directory: {error}")))?;

    let body = serde_json::to_string_pretty(config)
        .map_err(|error| CliError::Config(format!("encode config: {error}")))?;
    fs::write(&paths.file, format!("{body}\n"))
        .map_err(|error| CliError::Config(format!("write config: {error}")))?;
    Ok(())
}

fn explicit_config_path() -> Option<String> {
    env::var("BB_CONFIG_PATH")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn detect_home_dir() -> Result<PathBuf, CliError> {
    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home));
    }

    if let Some(home) = env::var_os("USERPROFILE") {
        return Ok(PathBuf::from(home));
    }

    match (env::var_os("HOMEDRIVE"), env::var_os("HOMEPATH")) {
        (Some(drive), Some(path)) => {
            let mut home = PathBuf::from(drive);
            home.push(path);
            Ok(home)
        }
        _ => Err(CliError::Config(
            "could not determine home directory".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_uses_home_dot_config() {
        let paths = config_paths_from_home(Path::new("/tmp/bb-home"), None);
        assert_eq!(
            paths.file,
            PathBuf::from("/tmp/bb-home/.config/bb/config.json")
        );
    }

    #[test]
    fn remove_profile_falls_back_to_first_remaining() {
        let mut config = Config::default();
        config.set_profile_with_auth("zeta", "", "z", None);
        config.set_profile_with_auth("alpha", "", "a", None);

        let (removed, ok) = config.remove_profile(None);
        assert!(ok);
        assert_eq!(removed, "alpha");
        assert_eq!(config.current, "zeta");
    }
}
