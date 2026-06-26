use std::path::PathBuf;
use thiserror::Error;

/// Library-level errors. The CLI layer wraps these with `anyhow` at the edges.
#[derive(Error, Debug)]
pub enum AlfError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("could not read skill file {path}: {source}")]
    SkillRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{path} has no valid frontmatter block (expected a leading `---` ... `---`)")]
    SkillFrontmatter { path: PathBuf },

    #[error("{path} frontmatter is missing required field `{field}`")]
    SkillField { path: PathBuf, field: &'static str },

    #[error("catalog not found at {path} (expected a `skills/` directory inside)")]
    CatalogNotFound { path: PathBuf },

    #[error("skill `{name}` not found in the catalog")]
    SkillNotFound { name: String },

    #[error("could not parse TOML at {path}: {source}")]
    TomlDe {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("could not serialize TOML: {source}")]
    TomlSer {
        #[source]
        source: toml::ser::Error,
    },

    #[error("not inside an alf project (no `.alf/` found at or above {path})")]
    NotAProject { path: PathBuf },

    #[error("git failed: {0}")]
    Git(String),

    #[error("`{0}` is designed but not implemented yet")]
    NotImplemented(&'static str),

    #[error("{0}")]
    Message(String),
}
