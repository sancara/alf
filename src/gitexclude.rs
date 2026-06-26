use std::fs;
use std::path::Path;

use crate::error::AlfError;
use crate::fsops::Fs;

/// Patterns alf writes into `.git/info/exclude` so none of its files
/// appear in `git status` of the host repo. The file is per-clone and
/// never committed — invisible to teammates.
const ALF_EXCLUDE_PATTERNS: &[&str] = &[
    "# --- alf (added by `alf init`, safe to remove if you uninstall alf) ---",
    // Claude Code
    "CLAUDE.md",
    "CLAUDE.local.md",
    ".claude/",
    // Cursor
    ".cursor/",
    ".cursorignore",
    ".cursorindexingignore",
    // Gemini Antigravity
    "GEMINI.md",
    "GEMINI.local.md",
    ".gemini/",
    // GitHub Copilot
    ".github/copilot-instructions.md",
    // Codex / OpenAI
    "AGENTS.md",
    ".codex/",
    // Windsurf
    ".windsurf/",
    "WINDSURF.md",
    // alf metadata
    ".alf/",
    ".agents/",
    // end marker
    "# --- end alf ---",
];

const MARKER: &str = "# --- alf";

/// Append alf's exclude patterns to `.git/info/exclude` if they aren't
/// already present. Idempotent: safe to call on every `alf init`.
pub fn ensure_git_exclude(fs: &mut Fs, repo_root: &Path) -> Result<(), AlfError> {
    let git_dir = find_git_dir(repo_root).ok_or_else(|| AlfError::Message(format!(
        "no .git directory found at or above {}; is this a git repo?",
        repo_root.display()
    )))?;

    let exclude_path = git_dir.join("info").join("exclude");

    // Read current content (the file always exists in a fresh git repo).
    let current = if exclude_path.exists() {
        fs::read_to_string(&exclude_path).map_err(|e| AlfError::Io {
            path: exclude_path.clone(),
            source: e,
        })?
    } else {
        String::new()
    };

    // Already present — idempotent.
    if current.contains(MARKER) {
        fs.actions.push(format!(
            "skip (already present) {}",
            exclude_path.display()
        ));
        return Ok(());
    }

    let addition = format!("\n{}\n", ALF_EXCLUDE_PATTERNS.join("\n"));
    let new_content = format!("{}{}", current.trim_end(), addition);

    fs.write(&exclude_path, &new_content)?;
    Ok(())
}

/// Walk up from `start` until we find a `.git` directory.
fn find_git_dir(start: &Path) -> Option<std::path::PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let candidate = dir.join(".git");
        if candidate.is_dir() {
            return Some(candidate);
        }
        current = dir.parent();
    }
    None
}
