use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidebarPosition {
    Left,
    #[default]
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub sidebar_position: SidebarPosition,
    pub sidebar_visible: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sidebar_position: SidebarPosition::Right,
            sidebar_visible: true,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    let data_dir = dirs_fallback();
    Some(data_dir.join("config.json"))
}

fn dirs_fallback() -> PathBuf {
    if let Ok(state_home) = std::env::var("XDG_STATE_HOME") {
        PathBuf::from(state_home).join("omawrite-rs")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("omawrite-rs")
    } else {
        std::env::temp_dir().join("omawrite-rs")
    }
}

pub fn load_config() -> AppConfig {
    if let Some(path) = config_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) {
    if let Some(path) = config_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(config) {
            let _ = fs::write(path, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.sidebar_position, SidebarPosition::Right);
        assert!(config.sidebar_visible);
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = AppConfig {
            sidebar_position: SidebarPosition::Left,
            sidebar_visible: false,
        };

        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: AppConfig = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.sidebar_position, SidebarPosition::Left);
        assert!(!deserialized.sidebar_visible);
    }
}
