use std::fs;
use std::path::Path;

/// First-pass, intentionally simple stack detection. Picks which vertical
/// personas `init` adds based on marker files. Refine as real repos teach us
/// where it guesses wrong — that's exactly what `glearn` is for.
pub fn detect_verticals(root: &Path) -> Vec<&'static str> {
    let has = |rel: &str| root.join(rel).exists();

    let frontend = has("package.json")
        || has("index.html")
        || has("angular.json")
        || has("vite.config.ts")
        || has("vite.config.js")
        || has_file_with_ext(root, &["tsx", "jsx", "vue", "svelte"]);

    let backend = has("Cargo.toml")
        || has("go.mod")
        || has("pyproject.toml")
        || has("requirements.txt")
        || has("pom.xml")
        || has("build.gradle")
        || has("composer.json");

    let devops = has("Dockerfile")
        || has("docker-compose.yml")
        || has("docker-compose.yaml")
        || has(".github/workflows")
        || has("Makefile")
        || has_file_with_ext(root, &["tf"]);

    let mut verticals = Vec::new();
    if frontend {
        verticals.push("frontend-expert");
    }
    if backend {
        verticals.push("backend-expert");
    }
    if devops {
        verticals.push("devops-expert");
    }
    // Security joins whenever there's a server-side or deployed surface.
    if backend || devops {
        verticals.push("security-expert");
    }
    verticals
}

/// Shallow, cheap search for any file with one of the given extensions,
/// skipping heavy/irrelevant directories. Bounded depth keeps `init` fast.
fn has_file_with_ext(root: &Path, exts: &[&str]) -> bool {
    fn walk(dir: &Path, exts: &[&str], depth: usize) -> bool {
        if depth == 0 {
            return false;
        }
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return false,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.is_dir() {
                if matches!(
                    name.as_ref(),
                    "node_modules" | "target" | ".git" | "dist" | "build"
                ) {
                    continue;
                }
                if walk(&path, exts, depth - 1) {
                    return true;
                }
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if exts.contains(&ext) {
                    return true;
                }
            }
        }
        false
    }
    walk(root, exts, 3)
}
