use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

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
        // Canonicalize so that `alf init .`, `alf init /abs/path`, and a subsequent
        // `alf list` from inside the repo all derive the SAME project hash.
        // Without this, `init` hashes the literal "." while every other command
        // hashes the absolute path, so the state written by init is never found.
        let canonical = canonicalize_best_effort(repo_root);
        let hash = path_hash(&canonical);
        let alf_root = alf_projects_dir().join(hash);
        Project {
            repo_root: canonical,
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

/// Canonicalize a path to its absolute, symlink-resolved form.
/// Falls back to the original path if it doesn't exist yet (e.g. during init
/// before the directory is created).
fn canonicalize_best_effort(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Stable, cross-version hash using SHA-256.
/// `DefaultHasher` is explicitly documented as not stable between Rust versions,
/// which would silently "lose" all projects on each alf upgrade. SHA-256 is
/// deterministic forever. We use the first 16 hex chars (64 bits) — enough to
/// avoid collisions between repos on one machine.
fn path_hash(path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    format!("{:x}", digest)[..16].to_string()
}
