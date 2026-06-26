/// The seven built-in skills, embedded at compile time.
/// `catalog init` writes these into the catalog so it works offline and
/// without depending on any external repo.
pub const SEEDS: &[(&str, &str)] = &[
    (
        "understand-the-problem",
        include_str!("../skills/understand-the-problem/SKILL.md"),
    ),
    (
        "execution-plan",
        include_str!("../skills/execution-plan/SKILL.md"),
    ),
    (
        "quality-reviewer",
        include_str!("../skills/quality-reviewer/SKILL.md"),
    ),
    (
        "frontend-expert",
        include_str!("../skills/frontend-expert/SKILL.md"),
    ),
    (
        "backend-expert",
        include_str!("../skills/backend-expert/SKILL.md"),
    ),
    (
        "devops-expert",
        include_str!("../skills/devops-expert/SKILL.md"),
    ),
    (
        "security-expert",
        include_str!("../skills/security-expert/SKILL.md"),
    ),
];
