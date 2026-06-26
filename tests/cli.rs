use std::fs;
use std::path::Path;

use alf::catalog::Catalog;
use alf::commands;

/// Build a throwaway catalog with a couple of skills and return its root.
fn make_catalog(root: &Path) {
    write_skill(
        root,
        "understand-the-problem",
        "0.2.0",
        "Use at the start of any task.",
    );
    write_skill(root, "execution-plan", "0.2.0", "Use before modifying code.");
    write_skill(
        root,
        "quality-reviewer",
        "0.1.0",
        "Use whenever code changes.",
    );
    write_skill(
        root,
        "backend-expert",
        "0.1.0",
        "Use for server logic and data.",
    );
    write_skill(root, "frontend-expert", "0.1.0", "Use for UI.");
    write_skill(
        root,
        "security-expert",
        "0.1.0",
        "Use for auth and sensitive data.",
    );
}

fn write_skill(catalog_root: &Path, name: &str, version: &str, description: &str) {
    let dir = catalog_root.join("skills").join(name);
    fs::create_dir_all(&dir).unwrap();
    let body = format!(
        "---\nname: {name}\nversion: {version}\ndescription: {description}\n---\n\n# {name}\n\nBody.\n"
    );
    fs::write(dir.join("SKILL.md"), body).unwrap();
}

#[test]
fn catalog_lists_skills_sorted() {
    let tmp = tempfile::tempdir().unwrap();
    make_catalog(tmp.path());

    let catalog = Catalog::open(tmp.path()).unwrap();
    let skills = catalog.skills().unwrap();
    let names: Vec<_> = skills.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "backend-expert",
            "execution-plan",
            "frontend-expert",
            "quality-reviewer",
            "security-expert",
            "understand-the-problem"
        ]
    );
}

#[test]
fn init_lays_down_the_full_scaffold() {
    let cat = tempfile::tempdir().unwrap();
    make_catalog(cat.path());
    let proj = tempfile::tempdir().unwrap();
    // Make it look like a backend repo so a vertical is detected.
    fs::write(proj.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();

    commands::init(cat.path(), proj.path(), &[], false).unwrap();

    // Canonical + mirror skill copies exist.
    let canonical = proj
        .path()
        .join(".agents/skills/understand-the-problem/SKILL.md");
    let mirror = proj
        .path()
        .join(".claude/skills/understand-the-problem/SKILL.md");
    assert!(canonical.is_file(), "canonical skill copy missing");
    assert!(mirror.is_file(), "claude mirror copy missing");

    // Instruction files and project metadata exist.
    assert!(proj.path().join("AGENTS.md").is_file());
    assert!(proj.path().join("CLAUDE.md").is_file());
    assert!(proj.path().join(".alf/manifest.toml").is_file());
    assert!(proj.path().join(".alf/lock.toml").is_file());
    assert!(proj.path().join(".alf/config.toml").is_file());

    // Backend repo => security-expert joins too; frontend does not.
    assert!(proj
        .path()
        .join(".agents/skills/backend-expert/SKILL.md")
        .is_file());
    assert!(proj
        .path()
        .join(".agents/skills/security-expert/SKILL.md")
        .is_file());
    assert!(!proj
        .path()
        .join(".agents/skills/frontend-expert")
        .exists());

    // Lock records the resolved version.
    let lock = fs::read_to_string(proj.path().join(".alf/lock.toml")).unwrap();
    assert!(lock.contains("understand-the-problem"));
    assert!(lock.contains("0.2.0"));
}

#[test]
fn dry_run_writes_nothing() {
    let cat = tempfile::tempdir().unwrap();
    make_catalog(cat.path());
    let proj = tempfile::tempdir().unwrap();

    let fs_log = commands::init(cat.path(), proj.path(), &[], true).unwrap();

    assert!(!fs_log.actions.is_empty(), "dry run should still plan actions");
    assert!(!proj.path().join("AGENTS.md").exists());
    assert!(!proj.path().join(".alf").exists());
}

#[test]
fn add_then_list_reports_installed() {
    let cat = tempfile::tempdir().unwrap();
    make_catalog(cat.path());
    let proj = tempfile::tempdir().unwrap();

    commands::init(cat.path(), proj.path(), &[], false).unwrap();
    // backend-expert is added explicitly here (empty repo wouldn't detect it).
    commands::add(cat.path(), proj.path(), "backend-expert", false).unwrap();

    let report = commands::list(cat.path(), Some(proj.path())).unwrap();
    let installed = report.installed.expect("should be inside a project");
    assert!(installed.iter().any(|e| e.name == "backend-expert"));
}

#[test]
fn plearn_reconciles_local_edits() {
    let cat = tempfile::tempdir().unwrap();
    make_catalog(cat.path());
    let proj = tempfile::tempdir().unwrap();
    commands::init(cat.path(), proj.path(), &[], false).unwrap();

    // Agent edits a copied skill while working.
    let skill = proj.path().join(".agents/skills/execution-plan/SKILL.md");
    let mut content = fs::read_to_string(&skill).unwrap();
    content.push_str("\nLearned: the DB mock must be reset between cases.\n");
    fs::write(&skill, &content).unwrap();

    // Before plearn: list flags it as modified.
    let before = commands::list(cat.path(), Some(proj.path())).unwrap();
    let entry = before.installed.unwrap().into_iter().find(|e| e.name == "execution-plan").unwrap();
    assert!(entry.status == commands::InstallStatus::Modified);

    // plearn reconciles: mirror updated, lock hash refreshed.
    let (_fs, entries) = commands::plearn(cat.path(), proj.path(), false).unwrap();
    assert!(entries.iter().any(|e| e.name == "execution-plan"));
    let mirror = proj.path().join(".claude/skills/execution-plan/SKILL.md");
    assert!(fs::read_to_string(&mirror).unwrap().contains("DB mock"));

    // After plearn: no longer flagged as modified.
    let after = commands::list(cat.path(), Some(proj.path())).unwrap();
    let entry = after.installed.unwrap().into_iter().find(|e| e.name == "execution-plan").unwrap();
    assert!(entry.status == commands::InstallStatus::Ok);
}

#[test]
fn glearn_promotes_to_catalog_and_retracks() {
    let cat = tempfile::tempdir().unwrap();
    make_catalog(cat.path());
    // The catalog must be a git repo for glearn to commit.
    git(cat.path(), &["init", "-q"]);
    git(cat.path(), &["add", "."]);
    git_commit(cat.path(), "seed");

    let proj = tempfile::tempdir().unwrap();
    commands::init(cat.path(), proj.path(), &[], false).unwrap();

    // A generalizable edit in the project.
    let skill = proj.path().join(".agents/skills/quality-reviewer/SKILL.md");
    let mut content = fs::read_to_string(&skill).unwrap();
    content.push_str("\nAlways measure before optimizing.\n");
    fs::write(&skill, &content).unwrap();

    // Promote it. Default bump is minor: 0.1.0 -> 0.2.0.
    let (_fs, result) = commands::glearn(
        cat.path(), proj.path(), "quality-reviewer",
        alf::version::Bump::Minor, None, false, false,
    ).unwrap();
    assert_eq!(result.to_version, "0.2.0");
    assert!(result.committed);

    // Catalog now carries the promoted content at the new version.
    let cat_md = fs::read_to_string(cat.path().join("skills/quality-reviewer/SKILL.md")).unwrap();
    assert!(cat_md.contains("version: 0.2.0"));
    assert!(cat_md.contains("measure before optimizing"));

    // Project re-tracks: lock records the new version, no longer drifting.
    let after = commands::list(cat.path(), Some(proj.path())).unwrap();
    let entry = after.installed.unwrap().into_iter().find(|e| e.name == "quality-reviewer").unwrap();
    assert_eq!(entry.version, "0.2.0");
    assert!(entry.status == commands::InstallStatus::Ok);
}

fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git").arg("-C").arg(dir).args(args).status().unwrap();
    assert!(status.success(), "git {args:?} failed");
}

fn git_commit(dir: &Path, msg: &str) {
    let status = std::process::Command::new("git").arg("-C").arg(dir)
        .args(["-c","user.name=test","-c","user.email=test@localhost","commit","-q","-m",msg])
        .status().unwrap();
    assert!(status.success(), "git commit failed");
}
