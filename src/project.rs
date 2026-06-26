use std::path::{Path, PathBuf};

use crate::error::AlfError;

/// Paths inside an alf project, rooted at the directory that contains `.alf/`.
pub struct Project {
    pub root: PathBuf,
}

impl Project {
    /// Treat `dir` as the project root (used by `init`, which creates `.alf/`).
    pub fn at(dir: &Path) -> Project {
        Project {
            root: dir.to_path_buf(),
        }
    }

    /// Find the nearest ancestor (including `start`) that contains `.alf/`.
    pub fn find(start: &Path) -> Result<Project, AlfError> {
        let mut current = Some(start);
        while let Some(dir) = current {
            if dir.join(".alf").is_dir() {
                return Ok(Project {
                    root: dir.to_path_buf(),
                });
            }
            current = dir.parent();
        }
        Err(AlfError::NotAProject {
            path: start.to_path_buf(),
        })
    }

    pub fn alf_dir(&self) -> PathBuf {
        self.root.join(".alf")
    }
    pub fn manifest_path(&self) -> PathBuf {
        self.alf_dir().join("manifest.toml")
    }
    pub fn lock_path(&self) -> PathBuf {
        self.alf_dir().join("lock.toml")
    }
    pub fn config_path(&self) -> PathBuf {
        self.alf_dir().join("config.toml")
    }
}
