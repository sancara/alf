use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::AlfError;
use crate::fsops::Fs;

/// `.alf/manifest.toml` — the project's declared skills (intent). Version
/// requirements are loose; the exact resolved version lives in the lock.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub skills: BTreeMap<String, String>,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Manifest, AlfError> {
        if !path.exists() {
            return Ok(Manifest::default());
        }
        let text = std::fs::read_to_string(path).map_err(|e| AlfError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        toml::from_str(&text).map_err(|e| AlfError::TomlDe {
            path: path.to_path_buf(),
            source: e,
        })
    }

    pub fn declare(&mut self, name: &str, requirement: &str) {
        self.skills.insert(name.to_string(), requirement.to_string());
    }

    pub fn save(&self, fs: &mut Fs, path: &Path) -> Result<(), AlfError> {
        let body = toml::to_string_pretty(self).map_err(|e| AlfError::TomlSer { source: e })?;
        let text = format!(
            "# alf manifest — skills this project declares.\n# Edit freely, or use `alf add`. \"*\" means track the catalog's latest.\n\n{body}"
        );
        fs.write(path, &text)
    }
}
