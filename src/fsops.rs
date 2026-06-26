use std::fs;
use std::path::Path;

use crate::error::AlfError;

/// Filesystem facade that honors `--dry-run` and records every action it takes
/// (or would take). The recorded actions double as a human-readable log and as
/// an inspection point for tests.
pub struct Fs {
    pub dry_run: bool,
    pub actions: Vec<String>,
}

impl Fs {
    pub fn new(dry_run: bool) -> Self {
        Fs {
            dry_run,
            actions: Vec::new(),
        }
    }

    fn io(path: &Path, source: std::io::Error) -> AlfError {
        AlfError::Io {
            path: path.to_path_buf(),
            source,
        }
    }

    pub fn create_dir_all(&mut self, path: &Path) -> Result<(), AlfError> {
        self.actions.push(format!("mkdir -p {}", path.display()));
        if !self.dry_run {
            fs::create_dir_all(path).map_err(|e| Self::io(path, e))?;
        }
        Ok(())
    }

    pub fn write(&mut self, path: &Path, contents: &str) -> Result<(), AlfError> {
        self.actions.push(format!("write {}", path.display()));
        if !self.dry_run {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| Self::io(parent, e))?;
            }
            fs::write(path, contents).map_err(|e| Self::io(path, e))?;
        }
        Ok(())
    }

    /// Write only if the file does not already exist. Returns true if written.
    /// Used for files the developer may have edited (AGENTS.md, CLAUDE.md).
    pub fn write_if_absent(&mut self, path: &Path, contents: &str) -> Result<bool, AlfError> {
        if path.exists() {
            self.actions
                .push(format!("skip (already exists) {}", path.display()));
            return Ok(false);
        }
        self.write(path, contents)?;
        Ok(true)
    }

    pub fn remove_dir_all_if_exists(&mut self, path: &Path) -> Result<(), AlfError> {
        if path.exists() {
            self.actions.push(format!("rm -rf {}", path.display()));
            if !self.dry_run {
                fs::remove_dir_all(path).map_err(|e| Self::io(path, e))?;
            }
        }
        Ok(())
    }

    /// Recursively copy `src` into `dst`. Records one action; performs the full
    /// copy unless in dry-run mode.
    pub fn copy_dir(&mut self, src: &Path, dst: &Path) -> Result<(), AlfError> {
        self.actions
            .push(format!("copy {} -> {}", src.display(), dst.display()));
        if self.dry_run {
            return Ok(());
        }
        copy_dir_recursive(src, dst)
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), AlfError> {
    fs::create_dir_all(dst).map_err(|e| Fs::io(dst, e))?;
    for entry in fs::read_dir(src).map_err(|e| Fs::io(src, e))? {
        let entry = entry.map_err(|e| Fs::io(src, e))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let file_type = entry.file_type().map_err(|e| Fs::io(&from, e))?;
        if file_type.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| Fs::io(&from, e))?;
        }
    }
    Ok(())
}
