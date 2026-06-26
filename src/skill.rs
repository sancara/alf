use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::AlfError;

/// A skill as it lives on disk: a directory containing a `SKILL.md` whose
/// frontmatter carries `name`, `version`, and `description`.
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub version: String,
    pub description: String,
    pub dir: PathBuf,
}

impl Skill {
    pub fn load(dir: &Path) -> Result<Skill, AlfError> {
        let md_path = dir.join("SKILL.md");
        let content = fs::read_to_string(&md_path).map_err(|e| AlfError::SkillRead {
            path: md_path.clone(),
            source: e,
        })?;

        let fields =
            parse_frontmatter(&content).ok_or_else(|| AlfError::SkillFrontmatter {
                path: md_path.clone(),
            })?;

        let name = fields
            .get("name")
            .cloned()
            .ok_or(AlfError::SkillField {
                path: md_path.clone(),
                field: "name",
            })?;
        let version = fields
            .get("version")
            .cloned()
            .ok_or(AlfError::SkillField {
                path: md_path.clone(),
                field: "version",
            })?;
        let description = fields.get("description").cloned().unwrap_or_default();

        Ok(Skill {
            name,
            version,
            description,
            dir: dir.to_path_buf(),
        })
    }

    /// Content hash of the skill, used in the lockfile to detect local drift
    /// (a `glearn` candidate) and tampering. v1 hashes SKILL.md; resources can
    /// be folded in later without changing the lock format.
    pub fn content_hash(&self) -> Result<String, AlfError> {
        let md_path = self.dir.join("SKILL.md");
        let bytes = fs::read(&md_path).map_err(|e| AlfError::SkillRead {
            path: md_path.clone(),
            source: e,
        })?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(format!("sha256:{:x}", hasher.finalize()))
    }
}

/// Content hash of arbitrary bytes, in the same format as `Skill::content_hash`.
pub fn content_hash_of(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

/// Parse the leading `--- ... ---` YAML-ish frontmatter into key/value pairs.
/// We only need flat `key: value` lines, so we avoid a full YAML dependency.
fn parse_frontmatter(content: &str) -> Option<BTreeMap<String, String>> {
    let rest = content.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    let block = &rest[..end];

    let mut fields = BTreeMap::new();
    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            fields.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    Some(fields)
}
