use std::fs;
use std::path::Path;
use std::process::Command;

use crate::catalog::Catalog;
use crate::config::ProjectConfig;
use crate::error::AlfError;
use crate::fsops::Fs;
use crate::gitexclude;
use crate::lock::{CatalogRef, Lock, LockedSkill};
use crate::manifest::Manifest;
use crate::project::Project;
use crate::scaffold;
use crate::skill::{content_hash_of, Skill};
use crate::detect;
use crate::version::{bump_version, set_frontmatter_version, Bump};

/// Generic personas installed in every project.
pub const GENERIC_SKILLS: &[&str] = &[
    "understand-the-problem",
    "execution-plan",
    "quality-reviewer",
];

fn home_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
}

// ---------------- list ----------------

pub struct CatalogEntry {
    pub name: String,
    pub version: String,
    pub installed: bool,
}

#[derive(PartialEq)]
pub enum InstallStatus { Ok, Modified, Missing }

pub struct InstalledEntry {
    pub name: String,
    pub version: String,
    pub status: InstallStatus,
}

pub struct ListReport {
    pub catalog: Vec<CatalogEntry>,
    pub installed: Option<Vec<InstalledEntry>>,
}

pub fn list(catalog_root: &Path, project_root: Option<&Path>) -> Result<ListReport, AlfError> {
    let catalog = Catalog::open(catalog_root)?;
    let catalog_skills = catalog.skills()?;

    let installed = match project_root {
        Some(start) => match Project::find(start) {
            Ok(project) => Some(installed_status(&project)?),
            Err(_) => None,
        },
        None => None,
    };

    let installed_names: Vec<String> = installed.as_ref()
        .map(|v| v.iter().map(|e| e.name.clone()).collect())
        .unwrap_or_default();

    let catalog = catalog_skills.into_iter().map(|s| CatalogEntry {
        installed: installed_names.contains(&s.name),
        name: s.name,
        version: s.version,
    }).collect();

    Ok(ListReport { catalog, installed })
}

fn installed_status(project: &Project) -> Result<Vec<InstalledEntry>, AlfError> {
    let lock = Lock::load(&project.lock_path())?;
    let mut entries = Vec::new();
    for locked in &lock.skills {
        let dir = project.skills_dir().join(&locked.name);
        let status = if !dir.join("SKILL.md").is_file() {
            InstallStatus::Missing
        } else {
            match Skill::load(&dir).and_then(|s| s.content_hash()) {
                Ok(hash) if hash == locked.hash => InstallStatus::Ok,
                Ok(_) => InstallStatus::Modified,
                Err(_) => InstallStatus::Missing,
            }
        };
        entries.push(InstalledEntry {
            name: locked.name.clone(),
            version: locked.version.clone(),
            status,
        });
    }
    Ok(entries)
}

// ---------------- init ----------------

pub fn init(
    catalog_root: &Path,
    repo_dir: &Path,
    with: &[String],
    dry_run: bool,
) -> Result<Fs, AlfError> {
    // Fail early: if this isn't a git repo, write nothing at all — not global
    // skills, not the project dir. The .git/info/exclude step requires it anyway.
    crate::gitexclude::find_git_dir_pub(repo_dir).ok_or_else(|| {
        AlfError::Message(format!(
            "`{}` is not a git repository (no .git found here or in any parent). \
             Run `git init` first.",
            repo_dir.display()
        ))
    })?;

    let catalog = Catalog::open(catalog_root)?;
    let project = Project::for_repo(repo_dir);
    let home = home_dir();

    let push = |name: &str, names: &mut Vec<String>| {
        if !names.iter().any(|n| n == name) {
            names.push(name.to_string());
        }
    };

    let mut required: Vec<String> = Vec::new();
    for g in GENERIC_SKILLS { push(g, &mut required); }
    for w in with { push(w, &mut required); }

    let mut detected: Vec<String> = Vec::new();
    for v in detect::detect_verticals(repo_dir) {
        if !required.iter().any(|n| n == v) { push(v, &mut detected); }
    }

    let mut skills: Vec<Skill> = Vec::new();
    for name in &required { skills.push(catalog.get(name)?); }
    for name in &detected {
        if let Ok(skill) = catalog.get(name) { skills.push(skill); }
    }

    let (remote, commit) = catalog.provenance();
    let mut fs = Fs::new(dry_run);

    // 1. Create alf's project dir (outside the repo)
    fs.create_dir_all(&project.alf_dir())?;
    fs.create_dir_all(&project.skills_dir())?;

    // 2. Write context files into the alf project dir
    scaffold::write_context_files(&mut fs, &project.alf_dir())?;

    // 3. Install skills globally (Claude Code) + into alf's project dir
    for skill in &skills {
        scaffold::install_skill(&mut fs, &project.skills_dir(), &home, skill)?;
    }

    // 4. Write manifest + lock + config into the alf project dir
    let mut manifest = Manifest::default();
    let mut lock = Lock::new(remote, commit);
    for skill in &skills {
        manifest.declare(&skill.name, "*");
        lock.upsert(LockedSkill {
            name: skill.name.clone(),
            version: skill.version.clone(),
            hash: skill.content_hash()?,
            source: format!("skills/{}", skill.name),
        });
    }
    manifest.save(&mut fs, &project.manifest_path())?;
    lock.save(&mut fs, &project.lock_path())?;
    ProjectConfig::default().save(&mut fs, &project.config_path())?;

    // 5. Add alf patterns to .git/info/exclude — repo stays 100% clean
    gitexclude::ensure_git_exclude(&mut fs, repo_dir)?;

    Ok(fs)
}

// ---------------- add ----------------

pub fn add(
    catalog_root: &Path,
    repo_root: &Path,
    name: &str,
    dry_run: bool,
) -> Result<Fs, AlfError> {
    let catalog = Catalog::open(catalog_root)?;
    let skill = catalog.get(name)?;
    let project = Project::find(repo_root)?;
    let home = home_dir();

    let mut fs = Fs::new(dry_run);
    scaffold::install_skill(&mut fs, &project.skills_dir(), &home, &skill)?;

    let mut manifest = Manifest::load(&project.manifest_path())?;
    manifest.declare(&skill.name, "*");
    manifest.save(&mut fs, &project.manifest_path())?;

    let (remote, commit) = catalog.provenance();
    let mut lock = if project.lock_path().exists() {
        Lock::load(&project.lock_path())?
    } else {
        Lock::new(remote, commit)
    };
    lock.upsert(LockedSkill {
        name: skill.name.clone(),
        version: skill.version.clone(),
        hash: skill.content_hash()?,
        source: format!("skills/{}", skill.name),
    });
    lock.save(&mut fs, &project.lock_path())?;

    Ok(fs)
}

// ---------------- update ----------------

pub struct UpdateEntry {
    pub name: String,
    pub from: String,
    pub to: String,
    pub changed: bool,
}

pub fn update(
    catalog_root: &Path,
    repo_root: &Path,
    only: Option<&str>,
    dry_run: bool,
) -> Result<(Fs, Vec<UpdateEntry>), AlfError> {
    let catalog = Catalog::open(catalog_root)?;
    let project = Project::find(repo_root)?;
    let home = home_dir();
    let mut lock = Lock::load(&project.lock_path())?;

    let targets: Vec<String> = match only {
        Some(n) => vec![n.to_string()],
        None => lock.skills.iter().map(|s| s.name.clone()).collect(),
    };

    let mut fs = Fs::new(dry_run);
    let mut report = Vec::new();

    for name in targets {
        let latest = catalog.get(&name)?;
        let new_hash = latest.content_hash()?;
        let current = lock.skills.iter().find(|s| s.name == name);
        let changed = match current {
            Some(c) => c.hash != new_hash,
            None => true,
        };
        let from = current.map(|c| c.version.clone()).unwrap_or_default();

        if changed {
            scaffold::install_skill(&mut fs, &project.skills_dir(), &home, &latest)?;
            lock.upsert(LockedSkill {
                name: latest.name.clone(),
                version: latest.version.clone(),
                hash: new_hash,
                source: format!("skills/{}", latest.name),
            });
        }
        report.push(UpdateEntry { name, from, to: latest.version.clone(), changed });
    }

    if report.iter().any(|e| e.changed) {
        lock.save(&mut fs, &project.lock_path())?;
    }

    Ok((fs, report))
}

// ---------------- catalog init ----------------

const CATALOG_README: &str = "# alf catalog\n\nThis git repo is the source of truth for your alf skills.\nEach skill is a directory under `skills/<name>/` with a `SKILL.md`.\n";

pub fn catalog_init(path: &Path, remote: Option<&str>, dry_run: bool) -> Result<Fs, AlfError> {
    let mut fs = Fs::new(dry_run);

    match remote {
        Some(url) => {
            run_git(&mut fs, None, &["clone", url, &path.to_string_lossy()])?;
        }
        None => {
            fs.create_dir_all(path)?;
            run_git(&mut fs, Some(path), &["init"])?;
            fs.write(&path.join("README.md"), CATALOG_README)?;

            for (name, content) in crate::seeds::SEEDS {
                let skill_dir = path.join("skills").join(name);
                fs.create_dir_all(&skill_dir)?;
                fs.write(&skill_dir.join("SKILL.md"), content)?;
            }

            run_git(&mut fs, Some(path), &["add", "."])?;
            run_git(&mut fs, Some(path), &[
                "-c", "user.name=alf",
                "-c", "user.email=alf@localhost",
                "commit", "-m", "seed: 8 built-in personas",
            ])?;
        }
    }
    Ok(fs)
}

// ---------------- plearn ----------------

pub enum PlearnKind { ModifiedFromCatalog, NewLocal }

pub struct PlearnEntry {
    pub name: String,
    pub kind: PlearnKind,
    pub diff: Option<String>,
}

pub fn plearn(
    catalog_root: &Path,
    repo_root: &Path,
    dry_run: bool,
) -> Result<(Fs, Vec<PlearnEntry>), AlfError> {
    let project = Project::find(repo_root)?;
    let home = home_dir();
    let catalog = Catalog::open(catalog_root).ok();
    let mut lock = Lock::load(&project.lock_path())?;

    let mut fs = Fs::new(dry_run);
    let mut entries = Vec::new();
    let mut changed = false;

    let locked_names: Vec<String> = lock.skills.iter().map(|s| s.name.clone()).collect();
    for name in &locked_names {
        let dir = project.skills_dir().join(name);
        if !dir.join("SKILL.md").is_file() { continue; }
        let skill = Skill::load(&dir)?;
        let current_hash = skill.content_hash()?;
        let locked_hash = lock.skills.iter().find(|s| &s.name == name)
            .map(|s| s.hash.clone()).unwrap_or_default();
        if current_hash == locked_hash { continue; }

        let diff = catalog.as_ref().and_then(|c| {
            git_diff_no_index(
                &c.skills_dir().join(name).join("SKILL.md"),
                &dir.join("SKILL.md"),
            )
        });
        entries.push(PlearnEntry {
            name: name.clone(),
            kind: PlearnKind::ModifiedFromCatalog,
            diff,
        });

        scaffold::regenerate_claude_skill(&mut fs, &project.skills_dir(), &home, name)?;
        if let Some(s) = lock.skills.iter_mut().find(|s| &s.name == name) {
            s.hash = current_hash;
        }
        changed = true;
    }

    if let Ok(read) = fs::read_dir(&project.skills_dir()) {
        for entry in read.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() && path.join("SKILL.md").is_file() && !locked_names.contains(&name) {
                entries.push(PlearnEntry { name: name.clone(), kind: PlearnKind::NewLocal, diff: None });
                scaffold::regenerate_claude_skill(&mut fs, &project.skills_dir(), &home, &name)?;
            }
        }
    }

    if changed { lock.save(&mut fs, &project.lock_path())?; }

    Ok((fs, entries))
}

// ---------------- glearn ----------------

pub struct GlearnResult {
    pub name: String,
    pub from_version: Option<String>,
    pub to_version: String,
    pub diff: Option<String>,
    pub catalog_is_git: bool,
    pub committed: bool,
    pub pushed: bool,
}

pub fn glearn(
    catalog_root: &Path,
    repo_root: &Path,
    name: &str,
    bump: Bump,
    explicit_version: Option<&str>,
    push: bool,
    dry_run: bool,
) -> Result<(Fs, GlearnResult), AlfError> {
    let project = Project::find(repo_root)?;
    let catalog = Catalog::open(catalog_root)?;
    let home = home_dir();

    let proj_dir = project.skills_dir().join(name);
    let proj_md = proj_dir.join("SKILL.md");
    if !proj_md.is_file() {
        return Err(AlfError::SkillNotFound { name: name.to_string() });
    }
    let proj_content = fs::read_to_string(&proj_md).map_err(|e| AlfError::SkillRead {
        path: proj_md.clone(), source: e,
    })?;

    let cat_dir = catalog.skills_dir().join(name);
    let cat_md = cat_dir.join("SKILL.md");
    let from_version = catalog.get(name).ok().map(|s| s.version);

    let diff = if cat_md.is_file() {
        git_diff_no_index(&cat_md, &proj_md)
    } else { None };

    let base = from_version.clone()
        .or_else(|| Skill::load(&proj_dir).ok().map(|s| s.version))
        .unwrap_or_else(|| "0.0.0".to_string());
    let to_version = match explicit_version {
        Some(v) => v.to_string(),
        None => bump_version(&base, bump).ok_or_else(|| {
            AlfError::Message(format!(
                "could not bump non-semver version `{base}`; pass --set-version"
            ))
        })?,
    };

    let new_content = set_frontmatter_version(&proj_content, &to_version);
    let new_hash = content_hash_of(new_content.as_bytes());

    let mut fs = Fs::new(dry_run);

    fs.create_dir_all(&cat_dir)?;
    fs.write(&cat_md, &new_content)?;

    let catalog_is_git = catalog.root.join(".git").is_dir();
    let mut committed = false;
    let mut pushed = false;
    if catalog_is_git {
        run_git(&mut fs, Some(&catalog.root), &["add", &format!("skills/{name}")])?;
        let msg = format!("glearn: {name} -> {to_version}");
        run_git(&mut fs, Some(&catalog.root), &[
            "-c", "user.name=alf", "-c", "user.email=alf@localhost",
            "commit", "-m", &msg,
        ])?;
        committed = true;
        if push {
            run_git(&mut fs, Some(&catalog.root), &["push"])?;
            pushed = true;
        }
    }

    // Re-track in the project
    fs.write(&proj_md, &new_content)?;
    scaffold::regenerate_claude_skill(&mut fs, &project.skills_dir(), &home, name)?;

    let (remote, commit) = catalog.provenance();
    let mut lock = if project.lock_path().exists() {
        Lock::load(&project.lock_path())?
    } else {
        Lock::new(remote.clone(), commit.clone())
    };
    lock.catalog = CatalogRef { remote, commit };
    lock.upsert(LockedSkill {
        name: name.to_string(),
        version: to_version.clone(),
        hash: new_hash,
        source: format!("skills/{name}"),
    });
    lock.save(&mut fs, &project.lock_path())?;

    Ok((fs, GlearnResult {
        name: name.to_string(), from_version, to_version, diff,
        catalog_is_git, committed, pushed,
    }))
}

fn git_diff_no_index(a: &Path, b: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["diff", "--no-index", "--"]).arg(a).arg(b)
        .output().ok()?;
    match output.status.code() {
        Some(0) | Some(1) => {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            if text.trim().is_empty() { None } else { Some(text) }
        }
        _ => None,
    }
}

fn run_git(fs: &mut Fs, cwd: Option<&Path>, args: &[&str]) -> Result<(), AlfError> {
    let mut shown = String::from("git");
    if let Some(dir) = cwd { shown.push_str(&format!(" -C {}", dir.display())); }
    shown.push(' ');
    shown.push_str(&args.join(" "));
    fs.actions.push(shown.clone());

    if fs.dry_run { return Ok(()); }

    let mut command = Command::new("git");
    if let Some(dir) = cwd { command.arg("-C").arg(dir); }
    let output = command.args(args).output()
        .map_err(|e| AlfError::Git(format!("could not run `{shown}`: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(AlfError::Git(format!("`{shown}` failed: {stderr}")));
    }
    Ok(())
}

// ---------------- memory install ----------------

pub struct MemoryInstallResult {
    pub already_installed: bool,
    pub installed_ok: bool,
    pub exclude_updated: bool,
    pub message: String,
}

/// Install codebase-memory-mcp (if not already present) and add its
/// data directory to .git/info/exclude so it never pollutes git status.
pub fn memory_install(catalog_root: &Path, repo_root: &Path, dry_run: bool) -> Result<(Fs, MemoryInstallResult), AlfError> {
    let mut fs = Fs::new(dry_run);

    // Check if already installed
    let already_installed = is_cbm_installed();

    if already_installed {
        fs.actions.push("codebase-memory-mcp already installed — skipping download".to_string());
    } else {
        // Invoke the official installer via sh
        let script = "curl -fsSL https://raw.githubusercontent.com/DeusData/codebase-memory-mcp/main/install.sh | bash";
        fs.actions.push(format!("sh -c \"{}\"", script));
        if !dry_run {
            let status = std::process::Command::new("sh")
                .arg("-c")
                .arg(script)
                .status()
                .map_err(|e| AlfError::Message(format!(
                    "could not run codebase-memory-mcp installer: {e}"
                )))?;
            if !status.success() {
                return Ok((fs, MemoryInstallResult {
                    already_installed: false,
                    installed_ok: false,
                    exclude_updated: false,
                    message: "codebase-memory-mcp installer failed. Check your network connection and try again, or install manually: https://github.com/DeusData/codebase-memory-mcp".to_string(),
                }));
            }
        }
    }

    // Add .codebase-memory/ to .git/info/exclude
    let exclude_updated = add_cbm_to_exclude(&mut fs, repo_root)?;

    // Install the skill that teaches the agent how to use the graph.
    // Best-effort: if the catalog or the skill is missing, we skip silently
    // rather than failing the whole command.
    if !dry_run {
        if let Ok(catalog) = Catalog::open(catalog_root) {
            if let Ok(skill) = catalog.get("codebase-navigator") {
                if let Ok(project) = Project::find(repo_root) {
                    let home = home_dir();
                    let _ = scaffold::install_skill(&mut fs, &project.skills_dir(), &home, &skill);
                }
            }
        }
    }

    let message = if dry_run {
        "dry run: nothing was installed. Without --dry-run, alf would install \
         codebase-memory-mcp and add .codebase-memory/ to .git/info/exclude."
            .to_string()
    } else if already_installed {
        "codebase-memory-mcp already installed. .codebase-memory/ added to .git/info/exclude."
            .to_string()
    } else {
        "codebase-memory-mcp installed. Restart your agent and say \"Index this project\" \
         to build the knowledge graph."
            .to_string()
    };

    Ok((fs, MemoryInstallResult {
        already_installed,
        installed_ok: true,
        exclude_updated,
        message,
    }))
}

fn is_cbm_installed() -> bool {
    // Check common install locations
    let home = home_dir();
    let candidates = [
        home.join(".local/bin/codebase-memory-mcp"),
        std::path::PathBuf::from("/usr/local/bin/codebase-memory-mcp"),
    ];
    candidates.iter().any(|p| p.is_file())
        || std::process::Command::new("which")
            .arg("codebase-memory-mcp")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

const CBM_EXCLUDE_PATTERNS: &[&str] = &[
    "# --- codebase-memory-mcp (added by alf) ---",
    ".codebase-memory/",
    "# --- end codebase-memory-mcp ---",
];
const CBM_MARKER: &str = "# --- codebase-memory-mcp";

fn add_cbm_to_exclude(fs: &mut Fs, repo_root: &Path) -> Result<bool, AlfError> {
    let git_dir = crate::gitexclude::find_git_dir_pub(repo_root)
        .ok_or_else(|| AlfError::Message(format!(
            "no .git directory found at or above {}",
            repo_root.display()
        )))?;

    let exclude_path = git_dir.join("info").join("exclude");
    let current = if exclude_path.exists() {
        std::fs::read_to_string(&exclude_path).map_err(|e| AlfError::Io {
            path: exclude_path.clone(),
            source: e,
        })?
    } else {
        String::new()
    };

    if current.contains(CBM_MARKER) {
        fs.actions.push(format!("skip (already present) {}", exclude_path.display()));
        return Ok(false);
    }

    let addition = format!("\n{}\n", CBM_EXCLUDE_PATTERNS.join("\n"));
    let new_content = format!("{}{}", current.trim_end(), addition);
    fs.write(&exclude_path, &new_content)?;
    Ok(true)
}
