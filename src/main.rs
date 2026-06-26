use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use alf::commands::{self, InstallStatus};
use alf::config::{default_catalog_path, MachineConfig};
use alf::version::Bump;

#[derive(Parser)]
#[command(
    name = "alf",
    version,
    about = "Scaffold spec-driven, agentic development environments with zero friction"
)]
struct Cli {
    /// Show what would happen without writing anything.
    #[arg(long, global = true)]
    dry_run: bool,

    /// Path to the alf catalog (overrides machine config).
    #[arg(long, global = true)]
    catalog: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage the skill catalog.
    Catalog {
        #[command(subcommand)]
        action: CatalogCommands,
    },
    /// Initialize the current repo: scaffold + install skills.
    Init {
        /// Project directory (defaults to the current directory).
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Extra skills to install beyond the generic + detected set.
        #[arg(long = "with")]
        with: Vec<String>,
    },
    /// Add a catalog skill to the current project.
    Add { name: String },
    /// Update installed skills to the catalog's current version.
    Update { name: Option<String> },
    /// List installed and available skills.
    List,
    /// Reconcile local skill edits into this project (project learn).
    Plearn,
    /// Promote a project skill to the shared catalog (global learn).
    Glearn {
        /// The skill to promote.
        name: String,
        /// Version bump when none is given explicitly: major | minor | patch.
        #[arg(long, default_value = "minor")]
        bump: String,
        /// Set an explicit version instead of bumping.
        #[arg(long = "set-version")]
        set_version: Option<String>,
        /// Also push the catalog repo after committing.
        #[arg(long)]
        push: bool,
    },
}

#[derive(Subcommand)]
enum CatalogCommands {
    /// Create or clone the master catalog (seeds 7 built-in personas).
    Init {
        /// Where to create or clone the catalog (defaults to ~/.config/alf/catalog).
        path: Option<PathBuf>,
        /// Clone from this git remote instead of creating a fresh catalog.
        #[arg(long)]
        remote: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let catalog_path = resolve_catalog_path(cli.catalog)?;
    let cwd = std::env::current_dir().context("could not read the current directory")?;

    match cli.command {
        Commands::Catalog { action } => match action {
            CatalogCommands::Init { path, remote } => {
                let target = path.unwrap_or_else(default_catalog_path);
                let fs = commands::catalog_init(&target, remote.as_deref(), cli.dry_run)?;
                print_actions(&fs.actions, cli.dry_run);
                println!("\nCatalog ready at {}", target.display());
            }
        },

        Commands::Init { path, with } => {
            let fs = commands::init(&catalog_path, &path, &with, cli.dry_run)?;
            print_actions(&fs.actions, cli.dry_run);
            println!(
                "\nProject scaffolded at {}. Open your agent and complete AGENTS.md.",
                path.display()
            );
        }

        Commands::Add { name } => {
            let fs = commands::add(&catalog_path, &cwd, &name, cli.dry_run)?;
            print_actions(&fs.actions, cli.dry_run);
            println!("\nAdded `{name}`.");
        }

        Commands::Update { name } => {
            let (fs, report) =
                commands::update(&catalog_path, &cwd, name.as_deref(), cli.dry_run)?;
            print_actions(&fs.actions, cli.dry_run);
            println!();
            for entry in &report {
                if entry.changed {
                    println!("  {} {} -> {}", entry.name, entry.from, entry.to);
                } else {
                    println!("  {} {} (up to date)", entry.name, entry.to);
                }
            }
        }

        Commands::List => {
            let report = commands::list(&catalog_path, Some(&cwd))?;
            if let Some(installed) = &report.installed {
                println!("Installed in this project:");
                for e in installed {
                    let status = match e.status {
                        InstallStatus::Ok => "",
                        InstallStatus::Modified => "  (modified — glearn candidate)",
                        InstallStatus::Missing => "  (missing)",
                    };
                    println!("  {} {}{}", e.name, e.version, status);
                }
                println!();
            }
            println!("Available in the catalog:");
            for e in &report.catalog {
                let mark = if e.installed { "*" } else { " " };
                println!("  {} {} {}", mark, e.name, e.version);
            }
        }

        Commands::Plearn => {
            let (fs, entries) = commands::plearn(&catalog_path, &cwd, cli.dry_run)?;
            if entries.is_empty() {
                println!("Nothing to reconcile — no local skill edits found.");
            } else {
                for entry in &entries {
                    match entry.kind {
                        commands::PlearnKind::ModifiedFromCatalog => {
                            println!("~ {} (modified)", entry.name)
                        }
                        commands::PlearnKind::NewLocal => println!("+ {} (new local)", entry.name),
                    }
                    if let Some(diff) = &entry.diff {
                        println!("{diff}");
                    }
                }
                print_actions(&fs.actions, cli.dry_run);
            }
        }

        Commands::Glearn {
            name,
            bump,
            set_version,
            push,
        } => {
            let bump = parse_bump(&bump)?;
            let (fs, result) = commands::glearn(
                &catalog_path,
                &cwd,
                &name,
                bump,
                set_version.as_deref(),
                push,
                cli.dry_run,
            )?;
            if let Some(diff) = &result.diff {
                println!("{diff}");
            }
            let from = result.from_version.as_deref().unwrap_or("(new)");
            println!("\n{} {} -> {}", result.name, from, result.to_version);
            print_actions(&fs.actions, cli.dry_run);
            if !result.catalog_is_git {
                println!("\nNote: the catalog is not a git repo, so nothing was committed.");
            } else if result.pushed {
                println!("\nCommitted and pushed to the catalog.");
            } else if result.committed {
                println!("\nCommitted to the catalog (use --push to publish).");
            }
        }
    }

    Ok(())
}

fn parse_bump(s: &str) -> Result<Bump> {
    match s {
        "major" => Ok(Bump::Major),
        "minor" => Ok(Bump::Minor),
        "patch" => Ok(Bump::Patch),
        other => anyhow::bail!("invalid --bump `{other}` (expected major, minor, or patch)"),
    }
}

fn resolve_catalog_path(flag: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = flag {
        return Ok(p);
    }
    let machine = MachineConfig::load().context("could not load machine config")?;
    Ok(machine.catalog_path())
}

fn print_actions(actions: &[String], dry_run: bool) {
    if dry_run {
        println!("dry run — no changes written:");
    }
    for action in actions {
        println!("  {action}");
    }
}
