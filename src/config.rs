use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AlfError;
use crate::fsops::Fs;

// ---------- Project config: committed, applies to everyone in the repo ----------

/// `.alf/config.toml` — per-repo alf behavior. The stop-rule threshold here is
/// the value AGENTS.md points to.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(rename = "loop", default)]
    pub loop_cfg: LoopConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoopConfig {
    #[serde(default = "default_attempts")]
    pub stop_after_attempts: u32,
}

impl Default for LoopConfig {
    fn default() -> Self {
        LoopConfig {
            stop_after_attempts: default_attempts(),
        }
    }
}

fn default_attempts() -> u32 {
    3
}

impl ProjectConfig {
    pub fn save(&self, fs: &mut Fs, path: &Path) -> Result<(), AlfError> {
        let body = toml::to_string_pretty(self).map_err(|e| AlfError::TomlSer { source: e })?;
        let text =
            format!("# alf project config. Committed; applies to everyone in this repo.\n\n{body}");
        fs.write(path, &text)
    }
}

// ---------- Machine config: never committed, per-developer ----------

/// `~/.config/alf/config.toml` — where the catalog lives and the default target.
/// Holds no keys and no provider settings: alf never calls a model.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MachineConfig {
    #[serde(default)]
    pub catalog: MachineCatalog,
    #[serde(default)]
    pub defaults: Defaults,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MachineCatalog {
    #[serde(default)]
    pub remote: String,
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Defaults {
    #[serde(default)]
    pub target: String,
}

impl MachineConfig {
    pub fn config_path() -> PathBuf {
        config_home().join("alf").join("config.toml")
    }

    pub fn load() -> Result<MachineConfig, AlfError> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(MachineConfig::default());
        }
        let text = std::fs::read_to_string(&path).map_err(|e| AlfError::Io {
            path: path.clone(),
            source: e,
        })?;
        toml::from_str(&text).map_err(|e| AlfError::TomlDe { path, source: e })
    }

    /// Resolved catalog clone location: explicit config wins, else the default.
    pub fn catalog_path(&self) -> PathBuf {
        if self.catalog.path.is_empty() {
            default_catalog_path()
        } else {
            PathBuf::from(&self.catalog.path)
        }
    }
}

pub fn default_catalog_path() -> PathBuf {
    config_home().join("alf").join("catalog")
}

pub fn config_home() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config")
}
