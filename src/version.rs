//! Tiny semver helpers for `glearn`. Skills use plain `MAJOR.MINOR.PATCH`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bump {
    Major,
    Minor,
    Patch,
}

/// Bump a `MAJOR.MINOR.PATCH` string. Returns None if it isn't three numbers.
pub fn bump_version(version: &str, bump: Bump) -> Option<String> {
    let mut parts = version.trim().split('.');
    let major: u64 = parts.next()?.parse().ok()?;
    let minor: u64 = parts.next()?.parse().ok()?;
    let patch: u64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let (major, minor, patch) = match bump {
        Bump::Major => (major + 1, 0, 0),
        Bump::Minor => (major, minor + 1, 0),
        Bump::Patch => (major, minor, patch + 1),
    };
    Some(format!("{major}.{minor}.{patch}"))
}

/// Replace the `version:` line inside the leading `---` frontmatter block.
/// Leaves the content untouched if no frontmatter version line is found.
pub fn set_frontmatter_version(content: &str, version: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut in_frontmatter = false;
    let mut seen_open = false;
    let mut replaced = false;

    for line in content.lines() {
        if line.trim() == "---" {
            if !seen_open {
                seen_open = true;
                in_frontmatter = true;
            } else if in_frontmatter {
                in_frontmatter = false;
            }
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_frontmatter && !replaced && line.trim_start().starts_with("version:") {
            out.push_str(&format!("version: {version}"));
            out.push('\n');
            replaced = true;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
