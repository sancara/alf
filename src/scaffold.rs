use std::path::Path;

use crate::error::AlfError;
use crate::fsops::Fs;
use crate::skill::Skill;

/// The AGENTS.md template, embedded at compile time.
pub const AGENTS_TEMPLATE: &str = include_str!("../templates/AGENTS.md");

/// The CLAUDE.md pointer for the claude target.
pub const CLAUDE_POINTER: &str = include_str!("../templates/CLAUDE.md");

/// Claude Code's global skills directory — read across ALL repos.
pub fn claude_global_skills(home: &Path) -> std::path::PathBuf {
    home.join(".claude").join("skills")
}

/// Install one skill into Claude Code's global skills dir and into the
/// alf project skills dir (used by plearn/glearn and list).
pub fn install_skill(
    fs: &mut Fs,
    alf_skills_dir: &Path,
    home: &Path,
    skill: &Skill,
) -> Result<(), AlfError> {
    // 1. alf's own copy (source of truth for plearn/glearn/lock)
    let alf_dest = alf_skills_dir.join(&skill.name);
    fs.remove_dir_all_if_exists(&alf_dest)?;
    fs.copy_dir(&skill.dir, &alf_dest)?;

    // 2. Claude Code global skills (auto-discovered in every repo)
    let claude_dest = claude_global_skills(home).join(&skill.name);
    fs.remove_dir_all_if_exists(&claude_dest)?;
    fs.copy_dir(&skill.dir, &claude_dest)?;

    Ok(())
}

/// Regenerate Claude Code's global mirror from alf's copy (used after plearn).
pub fn regenerate_claude_skill(
    fs: &mut Fs,
    alf_skills_dir: &Path,
    home: &Path,
    name: &str,
) -> Result<(), AlfError> {
    let src = alf_skills_dir.join(name);
    let dst = claude_global_skills(home).join(name);
    fs.remove_dir_all_if_exists(&dst)?;
    fs.copy_dir(&src, &dst)?;
    Ok(())
}

/// Write the AGENTS.md and CLAUDE.local.md context files into the alf
/// project dir (not the repo). These point the agent at the skills.
pub fn write_context_files(
    fs: &mut Fs,
    alf_project_dir: &Path,
) -> Result<(), AlfError> {
    fs.write_if_absent(&alf_project_dir.join("AGENTS.md"), AGENTS_TEMPLATE)?;
    fs.write_if_absent(&alf_project_dir.join("CLAUDE.local.md"), CLAUDE_POINTER)?;
    Ok(())
}
