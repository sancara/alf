use std::fs;
use std::path::Path;

use alf::catalog::Catalog;
use alf::commands;
use alf::project::Project;

fn make_catalog(root: &Path) {
    write_skill(root, "understand-the-problem", "0.2.0", "Use at the start of any task.");
    write_skill(root, "execution-plan",          "0.2.0", "Use before modifying code.");
    write_skill(root, "quality-reviewer",        "0.1.0", "Use whenever code changes.");
    write_skill(root, "backend-expert",          "0.1.0", "Use for server logic and data.");
    write_skill(root, "frontend-expert",         "0.1.0", "Use for UI.");
    write_skill(root, "security-expert",         "0.1.0", "Use for auth and sensitive data.");
}

fn write_skill(catalog_root: &Path, name: &str, version: &str, description: &str) {
    let dir = catalog_root.join("skills").join(name);
    fs::create_dir_all(&dir).unwrap();
    let body = format!("---\nname: {name}\nversion: {version}\ndescription: {description}\n---\n\n# {name}\n\nBody.\n");
    fs::write(dir.join("SKILL.md"), body).unwrap();
}

// Helper: make a fake git repo so .git/info/exclude exists
fn make_git_repo(root: &Path) {
    git(root, &["init", "-q"]);
    // ensure info/ dir exists
    fs::create_dir_all(root.join(".git").join("info")).unwrap();
    // git init already creates the exclude file, but ensure it exists
    let exclude = root.join(".git").join("info").join("exclude");
    if !exclude.exists() {
        fs::write(&exclude, "# git ls-files --others --exclude-from=.git/info/exclude\n").unwrap();
    }
}

#[test]
fn catalog_lists_skills_sorted() {
    let tmp = tempfile::tempdir().unwrap();
    make_catalog(tmp.path());

    let catalog = Catalog::open(tmp.path()).unwrap();
    let skills = catalog.skills().unwrap();
    let names: Vec<_> = skills.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec![
        "backend-expert", "execution-plan", "frontend-expert",
        "quality-reviewer", "security-expert", "understand-the-problem"
    ]);
}

#[test]
fn init_installs_skills_outside_repo() {
    let cat = tempfile::tempdir().unwrap();
    make_catalog(cat.path());
    let repo = tempfile::tempdir().unwrap();
    make_git_repo(repo.path());
    // Simulate a backend repo
    fs::write(repo.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();

    // Use a fake HOME so we don't touch the real ~/.claude
    let fake_home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", fake_home.path());
    std::env::set_var("XDG_CONFIG_HOME", fake_home.path().join(".config"));

    commands::init(cat.path(), repo.path(), &[], false).unwrap();

    let project = Project::for_repo(repo.path());

    // Skills live in alf's project dir, NOT in the repo
    let skill = project.skills_dir().join("understand-the-problem").join("SKILL.md");
    assert!(skill.is_file(), "skill should be in alf project dir");

    // Skills also copied to ~/.claude/skills/
    let claude_skill = fake_home.path()
        .join(".claude").join("skills")
        .join("understand-the-problem").join("SKILL.md");
    assert!(claude_skill.is_file(), "skill should be in ~/.claude/skills/");

    // alf metadata lives outside the repo too
    assert!(project.manifest_path().exists());
    assert!(project.lock_path().exists());

    // The repo itself has NO new files (except .git/info/exclude which is already there)
    let repo_files: Vec<_> = fs::read_dir(repo.path()).unwrap()
        .flatten()
        .filter(|e| e.file_name() != ".git" && e.file_name() != "Cargo.toml")
        .collect();
    assert!(repo_files.is_empty(), "repo should have no new files from alf: {:?}", 
        repo_files.iter().map(|e| e.file_name()).collect::<Vec<_>>());

    // .git/info/exclude has alf's patterns
    let exclude = fs::read_to_string(repo.path().join(".git/info/exclude")).unwrap();
    assert!(exclude.contains("# --- alf"), ".git/info/exclude should contain alf patterns");
    assert!(exclude.contains("CLAUDE.md"));
    assert!(exclude.contains("AGENTS.md"));
}

#[test]
fn dry_run_writes_nothing() {
    let cat = tempfile::tempdir().unwrap();
    make_catalog(cat.path());
    let repo = tempfile::tempdir().unwrap();
    make_git_repo(repo.path());

    let fake_home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", fake_home.path());
    std::env::set_var("XDG_CONFIG_HOME", fake_home.path().join(".config"));

    let fs_log = commands::init(cat.path(), repo.path(), &[], true).unwrap();

    assert!(!fs_log.actions.is_empty(), "dry run should still plan actions");
    // Nothing written to repo
    let repo_files: Vec<_> = fs::read_dir(repo.path()).unwrap()
        .flatten()
        .filter(|e| e.file_name() != ".git")
        .collect();
    assert!(repo_files.is_empty());
}

#[test]
fn add_then_list_reports_installed() {
    let cat = tempfile::tempdir().unwrap();
    make_catalog(cat.path());
    let repo = tempfile::tempdir().unwrap();
    make_git_repo(repo.path());

    let fake_home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", fake_home.path());
    std::env::set_var("XDG_CONFIG_HOME", fake_home.path().join(".config"));

    commands::init(cat.path(), repo.path(), &[], false).unwrap();
    commands::add(cat.path(), repo.path(), "backend-expert", false).unwrap();

    let report = commands::list(cat.path(), Some(repo.path())).unwrap();
    let installed = report.installed.expect("should be inside a project");
    assert!(installed.iter().any(|e| e.name == "backend-expert"));
}

#[test]
fn plearn_reconciles_local_edits() {
    let cat = tempfile::tempdir().unwrap();
    make_catalog(cat.path());
    let repo = tempfile::tempdir().unwrap();
    make_git_repo(repo.path());

    let fake_home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", fake_home.path());
    std::env::set_var("XDG_CONFIG_HOME", fake_home.path().join(".config"));

    commands::init(cat.path(), repo.path(), &[], false).unwrap();
    let project = Project::for_repo(repo.path());

    // Agent edits a skill in the alf project dir
    let skill = project.skills_dir().join("execution-plan").join("SKILL.md");
    let mut content = fs::read_to_string(&skill).unwrap();
    content.push_str("\nLearned: reset the DB mock between cases.\n");
    fs::write(&skill, &content).unwrap();

    // plearn reconciles
    let (_fs, entries) = commands::plearn(cat.path(), repo.path(), false).unwrap();
    assert!(entries.iter().any(|e| e.name == "execution-plan"));

    // ~/.claude/skills updated
    let mirror = fake_home.path()
        .join(".claude").join("skills")
        .join("execution-plan").join("SKILL.md");
    assert!(fs::read_to_string(&mirror).unwrap().contains("DB mock"));
}

#[test]
fn glearn_promotes_to_catalog_and_retracks() {
    let cat = tempfile::tempdir().unwrap();
    make_catalog(cat.path());
    git(cat.path(), &["init", "-q"]);
    git(cat.path(), &["add", "."]);
    git_commit(cat.path(), "seed");

    let repo = tempfile::tempdir().unwrap();
    make_git_repo(repo.path());

    let fake_home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", fake_home.path());
    std::env::set_var("XDG_CONFIG_HOME", fake_home.path().join(".config"));

    commands::init(cat.path(), repo.path(), &[], false).unwrap();
    let project = Project::for_repo(repo.path());

    let skill = project.skills_dir().join("quality-reviewer").join("SKILL.md");
    let mut content = fs::read_to_string(&skill).unwrap();
    content.push_str("\nAlways measure before optimizing.\n");
    fs::write(&skill, &content).unwrap();

    let (_fs, result) = commands::glearn(
        cat.path(), repo.path(), "quality-reviewer",
        alf::version::Bump::Minor, None, false, false,
    ).unwrap();
    assert_eq!(result.to_version, "0.2.0");
    assert!(result.committed);

    let cat_md = fs::read_to_string(
        cat.path().join("skills/quality-reviewer/SKILL.md")
    ).unwrap();
    assert!(cat_md.contains("version: 0.2.0"));
    assert!(cat_md.contains("measure before optimizing"));
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
