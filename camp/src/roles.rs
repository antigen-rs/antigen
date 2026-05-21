//! Role names used in `signers(required = [...])` predicates on campsites.
//!
//! These are STRING CONSTANTS, not types. Antigen's `signers` predicate
//! takes role names as opaque strings; the meaning of "navigator" or
//! "pathmaker" is defined by the team's convention, not by antigen.
//!
//! Adding a role: append a constant here + use it in campsite declarations.
//! Removing a role: search-and-replace across `src/campsites/**`, then
//! delete the constant.

/// Coordinator + de-facto team lead. Owns campsite stewardship
/// per camp discipline. Decides per-campsite required-signer sets +
/// "done" definitions when team-lead (parent Claude) doesn't.
pub const NAVIGATOR: &str = "navigator";

/// Synthesist + builder. Implements; commits when work feels whole.
/// Typically the required-signer for "impl" lanes.
pub const PATHMAKER: &str = "pathmaker";

/// Scientific conscience + peer-review. Lab notebook of what IS,
/// not what we hope. Required-signer for verification lanes.
pub const OBSERVER: &str = "observer";

/// First-principles deconstructor (Phase 1-8). Required-signer for
/// architectural decisions + ratification gates.
pub const ARISTOTLE: &str = "aristotle";

/// Noticer + mapmaker + biology grounding. Required-signer when work
/// touches the biological metaphor or surfaces structural rhymes.
pub const NATURALIST: &str = "naturalist";

/// Skeptic + stress-tester. Required-signer for security-adjacent
/// surfaces; attack-pass gates.
pub const ADVERSARIAL: &str = "adversarial";

/// Explorer + tangent-follower. Surfaces what focused work misses.
/// Not typically required-signer; signs as found-thing reporter.
pub const SCOUT: &str = "scout";

/// Naive-questioning specialist. Required-signer for adopter-facing
/// surfaces (catches insider-jargon before it ships).
pub const OUTSIDER: &str = "outsider";

/// Dependency-graphing specialist. Required-signer for sequencing +
/// critical-path decisions; not for substantive content.
pub const EXECUTOR: &str = "executor";

/// Validator + manuscript author (theory mode for antigen). Required-
/// signer for ADR text + whitepaper sections.
pub const SCIENTIST: &str = "scientist";

/// Tekgy (human collaborator). Required-signer for load-bearing-for-user
/// decisions: release timing, positioning, naming, anything that breaks
/// foundational ADRs.
pub const TEKGY: &str = "tekgy";

/// Main-thread Claude / team-lead. Required-signer when parent-context
/// integration is the discipline check (rare; typically defers to navigator).
pub const TEAM_LEAD: &str = "team-lead";
