use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::config::config_home;
use crate::error::AlfError;

/// Paths for an alf project. Everything lives under `~/.config/alf/projects/<hash>/`
/// so the host repo is never touched. The repo_root is used only to:
///   1. compute the hash (identity of the project)
///   2. find `.git/info/exclude` (for gitexclude)
///   3. detect the stack (for vertical personas)
pub struct Project {
    /// The actual git repo the developer is working in (never written to by alf).
    pub repo_root: PathBuf,
    /// Where alf stores its state for this project (~/.config/alf/projects/<hash>/).
    pub alf_root: PathBuf,
}

impl Project {
    pub fn for_repo(repo_root: &Path) -> Project {
        let hash = path_hash(repo_root);
        let alf_root = alf_projects_dir().join(hash);
        Project {
            repo_root: repo_root.to_path_buf(),
            alf_root,
        }
    }

    /// Find the project by walking up from `start` to find a git repo root,
    /// then derive the alf_root from its hash.
    pub fn find(start: &Path) -> Result<Project, AlfError> {
        let repo_root = find_repo_root(start).ok_or_else(|| AlfError::NotAProject {
            path: start.to_path_buf(),
        })?;
        Ok(Project::for_repo(&repo_root))
    }

    // alf state paths — all under ~/.config/alf/projects/<hash>/
    pub fn alf_dir(&self) -> PathBuf { self.alf_root.clone() }
    pub fn manifest_path(&self) -> PathBuf { self.alf_root.join("manifest.toml") }
    pub fn lock_path(&self) -> PathBuf { self.alf_root.join("lock.toml") }
    pub fn config_path(&self) -> PathBuf { self.alf_root.join("config.toml") }

    // Skills installed for this project — also outside the repo.
    pub fn skills_dir(&self) -> PathBuf { self.alf_root.join("skills") }
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        if dir.join(".git").is_dir() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn alf_projects_dir() -> PathBuf {
    config_home().join("alf").join("projects")
}

fn path_hash(path: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
