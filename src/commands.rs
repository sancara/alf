use std::fs;
use std::path::Path;
use std::process::Command;

use crate::catalog::Catalog;
use crate::config::ProjectConfig;
use crate::error::AlfError;
use crate::fsops::Fs;
use crate::lock::{CatalogRef, Lock, LockedSkill};
use crate::manifest::Manifest;
use crate::project::Project;
use crate::scaffold::{self, CANONICAL_SKILLS, CLAUDE_SKILLS};
use crate::skill::{content_hash_of, Skill};
use crate::detect;
use crate::version::{bump_version, set_frontmatter_version, Bump};

/// Generic personas installed in every project, regardless of stack.
pub const GENERIC_SKILLS: &[&str] = &[
    "understand-the-problem",
    "execution-plan",
    "quality-reviewer",
];

// ---------------- list ----------------

pub struct CatalogEntry {
    pub name: String,
    pub version: String,
    pub installed: bool,
}

#[derive(PartialEq)]
pub enum InstallStatus {
    Ok,
    Modified,
    Missing,
}

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

    let installed_names: Vec<String> = installed
        .as_ref()
        .map(|v| v.iter().map(|e| e.name.clone()).collect())
        .unwrap_or_default();

    let catalog = catalog_skills
        .into_iter()
        .map(|s| CatalogEntry {
            installed: installed_names.contains(&s.name),
            name: s.name,
            version: s.version,
        })
        .collect();

    Ok(ListReport { catalog, installed })
}

fn installed_status(project: &Project) -> Result<Vec<InstalledEntry>, AlfError> {
    let lock = Lock::load(&project.lock_path())?;
    let mut entries = Vec::new();
    for locked in &lock.skills {
        let dir = project.root.join(CANONICAL_SKILLS).join(&locked.name);
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
    project_dir: &Path,
    with: &[String],
    dry_run: bool,
) -> Result<Fs, AlfError> {
    let catalog = Catalog::open(catalog_root)?;
    let project = Project::at(project_dir);

    // Required = generic personas + explicit `--with` extras. Missing => error.
    // Detected = heuristic verticals. Missing from the catalog => silently skipped,
    // because detection is a guess, not a request.
    let push = |name: &str, names: &mut Vec<String>| {
        if !names.iter().any(|n| n == name) {
            names.push(name.to_string());
        }
    };

    let mut required: Vec<String> = Vec::new();
    for g in GENERIC_SKILLS {
        push(g, &mut required);
    }
    for w in with {
        push(w, &mut required);
    }

    let mut detected: Vec<String> = Vec::new();
    for v in detect::detect_verticals(project_dir) {
        if !required.iter().any(|n| n == v) {
            push(v, &mut detected);
        }
    }

    let mut skills: Vec<Skill> = Vec::new();
    for name in &required {
        skills.push(catalog.get(name)?);
    }
    for name in &detected {
        if let Ok(skill) = catalog.get(name) {
            skills.push(skill);
        }
    }

    let (remote, commit) = catalog.provenance();
    let mut fs = Fs::new(dry_run);

    scaffold::write_scaffold(&mut fs, &project.root, &skills)?;

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

    Ok(fs)
}

// ---------------- add ----------------

pub fn add(
    catalog_root: &Path,
    project_root: &Path,
    name: &str,
    dry_run: bool,
) -> Result<Fs, AlfError> {
    let catalog = Catalog::open(catalog_root)?;
    let skill = catalog.get(name)?;
    let project = Project::find(project_root)?;

    let mut fs = Fs::new(dry_run);
    scaffold::install_skill(&mut fs, &project.root, &skill)?;

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
    project_root: &Path,
    only: Option<&str>,
    dry_run: bool,
) -> Result<(Fs, Vec<UpdateEntry>), AlfError> {
    let catalog = Catalog::open(catalog_root)?;
    let project = Project::find(project_root)?;
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
            scaffold::install_skill(&mut fs, &project.root, &latest)?;
            lock.upsert(LockedSkill {
                name: latest.name.clone(),
                version: latest.version.clone(),
                hash: new_hash,
                source: format!("skills/{}", latest.name),
            });
        }
        report.push(UpdateEntry {
            name,
            from,
            to: latest.version.clone(),
            changed,
        });
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
            fs.create_dir_all(&path.join("skills"))?;
            fs.write(&path.join("skills").join(".gitkeep"), "")?;
            fs.write(&path.join("README.md"), CATALOG_README)?;
        }
    }
    Ok(fs)
}

/// Run a git subcommand, recording it as an action and honoring dry-run.
fn run_git(fs: &mut Fs, cwd: Option<&Path>, args: &[&str]) -> Result<(), AlfError> {
    let mut shown = String::from("git");
    if let Some(dir) = cwd {
        shown.push_str(&format!(" -C {}", dir.display()));
    }
    shown.push(' ');
    shown.push_str(&args.join(" "));
    fs.actions.push(shown.clone());

    if fs.dry_run {
        return Ok(());
    }

    let mut command = Command::new("git");
    if let Some(dir) = cwd {
        command.arg("-C").arg(dir);
    }
    let output = command
        .args(args)
        .output()
        .map_err(|e| AlfError::Git(format!("could not run `{shown}`: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(AlfError::Git(format!("`{shown}` failed: {stderr}")));
    }
    Ok(())
}

// ---------------- plearn (project learn) ----------------

pub enum PlearnKind {
    ModifiedFromCatalog,
    NewLocal,
}

pub struct PlearnEntry {
    pub name: String,
    pub kind: PlearnKind,
    pub diff: Option<String>,
}

/// Reconcile local skill edits the agent made while working. Skills that diverged
/// from their locked hash are accepted as the new local truth (mirror regenerated,
/// lock hash updated); brand-new local skills get their mirror generated. The diff
/// shown is the project skill vs. the catalog version.
pub fn plearn(
    catalog_root: &Path,
    project_root: &Path,
    dry_run: bool,
) -> Result<(Fs, Vec<PlearnEntry>), AlfError> {
    let project = Project::find(project_root)?;
    let catalog = Catalog::open(catalog_root).ok(); // best-effort, only for diffs
    let mut lock = Lock::load(&project.lock_path())?;
    let canonical_root = project.root.join(CANONICAL_SKILLS);

    let mut fs = Fs::new(dry_run);
    let mut entries = Vec::new();
    let mut changed = false;

    // Catalog-sourced skills whose installed content drifted from the lock.
    let locked_names: Vec<String> = lock.skills.iter().map(|s| s.name.clone()).collect();
    for name in &locked_names {
        let dir = canonical_root.join(name);
        if !dir.join("SKILL.md").is_file() {
            continue;
        }
        let skill = Skill::load(&dir)?;
        let current_hash = skill.content_hash()?;
        let locked_hash = lock
            .skills
            .iter()
            .find(|s| &s.name == name)
            .map(|s| s.hash.clone())
            .unwrap_or_default();
        if current_hash == locked_hash {
            continue;
        }

        let diff = catalog.as_ref().and_then(|c| {
            git_diff_no_index(&c.skills_dir().join(name).join("SKILL.md"), &dir.join("SKILL.md"))
        });
        entries.push(PlearnEntry {
            name: name.clone(),
            kind: PlearnKind::ModifiedFromCatalog,
            diff,
        });

        scaffold::regenerate_mirror(&mut fs, &project.root, name)?;
        if let Some(s) = lock.skills.iter_mut().find(|s| &s.name == name) {
            s.hash = current_hash;
        }
        changed = true;
    }

    // Brand-new project-local skills (not from the catalog, not in the lock).
    if let Ok(read) = fs::read_dir(&canonical_root) {
        for entry in read.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir()
                && path.join("SKILL.md").is_file()
                && !locked_names.contains(&name)
            {
                entries.push(PlearnEntry {
                    name: name.clone(),
                    kind: PlearnKind::NewLocal,
                    diff: None,
                });
                scaffold::regenerate_mirror(&mut fs, &project.root, &name)?;
            }
        }
    }

    if changed {
        lock.save(&mut fs, &project.lock_path())?;
    }

    Ok((fs, entries))
}

// ---------------- glearn (global learn) ----------------

pub struct GlearnResult {
    pub name: String,
    pub from_version: Option<String>,
    pub to_version: String,
    pub diff: Option<String>,
    pub catalog_is_git: bool,
    pub committed: bool,
    pub pushed: bool,
}

/// Promote a project skill (locally diverged, or new) to the shared catalog:
/// bump its version, write it into the catalog, commit (and optionally push),
/// then re-track it in the project so it follows the promoted version.
pub fn glearn(
    catalog_root: &Path,
    project_root: &Path,
    name: &str,
    bump: Bump,
    explicit_version: Option<&str>,
    push: bool,
    dry_run: bool,
) -> Result<(Fs, GlearnResult), AlfError> {
    let project = Project::find(project_root)?;
    let catalog = Catalog::open(catalog_root)?;

    let proj_dir = project.root.join(CANONICAL_SKILLS).join(name);
    let proj_md = proj_dir.join("SKILL.md");
    if !proj_md.is_file() {
        return Err(AlfError::SkillNotFound {
            name: name.to_string(),
        });
    }
    let proj_content = fs::read_to_string(&proj_md).map_err(|e| AlfError::SkillRead {
        path: proj_md.clone(),
        source: e,
    })?;

    let cat_dir = catalog.skills_dir().join(name);
    let cat_md = cat_dir.join("SKILL.md");
    let from_version = catalog.get(name).ok().map(|s| s.version);

    let diff = if cat_md.is_file() {
        git_diff_no_index(&cat_md, &proj_md)
    } else {
        None
    };

    // New version: explicit wins, else bump the catalog's current (or the skill's
    // own) version. The developer can override; alf proposes a minor by default.
    let base = from_version
        .clone()
        .or_else(|| Skill::load(&proj_dir).ok().map(|s| s.version))
        .unwrap_or_else(|| "0.0.0".to_string());
    let to_version = match explicit_version {
        Some(v) => v.to_string(),
        None => bump_version(&base, bump).ok_or_else(|| {
            AlfError::Message(format!(
                "could not bump non-semver version `{base}`; pass an explicit --set-version"
            ))
        })?,
    };

    let new_content = set_frontmatter_version(&proj_content, &to_version);
    let new_hash = content_hash_of(new_content.as_bytes());

    let mut fs = Fs::new(dry_run);

    // 1) Write the promoted skill into the catalog. (v1 promotes SKILL.md content;
    //    bundling resources is a future refinement.)
    fs.create_dir_all(&cat_dir)?;
    fs.write(&cat_md, &new_content)?;

    // 2) Commit it in the catalog repo (with a fallback identity so it works
    //    even without a configured global git user). --push publishes it.
    let catalog_is_git = catalog.root.join(".git").is_dir();
    let mut committed = false;
    let mut pushed = false;
    if catalog_is_git {
        run_git(&mut fs, Some(&catalog.root), &["add", &format!("skills/{name}")])?;
        let msg = format!("glearn: {name} -> {to_version}");
        run_git(
            &mut fs,
            Some(&catalog.root),
            &[
                "-c",
                "user.name=alf",
                "-c",
                "user.email=alf@localhost",
                "commit",
                "-m",
                &msg,
            ],
        )?;
        committed = true;
        if push {
            run_git(&mut fs, Some(&catalog.root), &["push"])?;
            pushed = true;
        }
    }

    // 3) Re-track in the project: rewrite canonical + mirror to the promoted
    //    content and update the lock, so the project follows the new version.
    fs.write(&proj_md, &new_content)?;
    fs.write(
        &project.root.join(CLAUDE_SKILLS).join(name).join("SKILL.md"),
        &new_content,
    )?;

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

    Ok((
        fs,
        GlearnResult {
            name: name.to_string(),
            from_version,
            to_version,
            diff,
            catalog_is_git,
            committed,
            pushed,
        },
    ))
}

/// `git diff --no-index` between two files. Exit code 1 (differences) is success,
/// not an error. Returns None when there is no diff or git is unavailable.
fn git_diff_no_index(a: &Path, b: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["diff", "--no-index", "--"])
        .arg(a)
        .arg(b)
        .output()
        .ok()?;
    match output.status.code() {
        Some(0) | Some(1) => {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        }
        _ => None,
    }
}
