use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::AlfError;
use crate::skill::Skill;

/// The catalog: a git repo whose `skills/` directory is the single source of
/// truth. Resolution is filesystem + frontmatter — there is no index file.
pub struct Catalog {
    pub root: PathBuf,
}

impl Catalog {
    pub fn open(root: &Path) -> Result<Catalog, AlfError> {
        if !root.join("skills").is_dir() {
            return Err(AlfError::CatalogNotFound {
                path: root.to_path_buf(),
            });
        }
        Ok(Catalog {
            root: root.to_path_buf(),
        })
    }

    pub fn skills_dir(&self) -> PathBuf {
        self.root.join("skills")
    }

    /// Every skill in the catalog, sorted by name. Scans `skills/*/SKILL.md`.
    pub fn skills(&self) -> Result<Vec<Skill>, AlfError> {
        let dir = self.skills_dir();
        let mut skills = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| AlfError::Io {
            path: dir.clone(),
            source: e,
        })? {
            let entry = entry.map_err(|e| AlfError::Io {
                path: dir.clone(),
                source: e,
            })?;
            let path = entry.path();
            if path.is_dir() && path.join("SKILL.md").is_file() {
                skills.push(Skill::load(&path)?);
            }
        }
        skills.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(skills)
    }

    pub fn get(&self, name: &str) -> Result<Skill, AlfError> {
        self.skills()?
            .into_iter()
            .find(|s| s.name == name)
            .ok_or_else(|| AlfError::SkillNotFound {
                name: name.to_string(),
            })
    }

    /// Best-effort provenance for the lockfile: the catalog's current commit
    /// and origin remote. Returns "unknown" when git can't answer (e.g. the
    /// catalog isn't a git repo yet), rather than failing the command.
    pub fn provenance(&self) -> (String, String) {
        let remote = git_capture(&self.root, &["remote", "get-url", "origin"])
            .unwrap_or_else(|| "unknown".to_string());
        let commit = git_capture(&self.root, &["rev-parse", "HEAD"])
            .unwrap_or_else(|| "unknown".to_string());
        (remote, commit)
    }
}

fn git_capture(cwd: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git").arg("-C").arg(cwd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
