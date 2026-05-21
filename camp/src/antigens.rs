//! Antigen declarations for camp's team-coordination discipline.
//!
//! Currently one antigen: `CampsiteOpen` — presented at every campsite's
//! `done()` function, immune when the required signers have attested.
//!
//! Why a single antigen rather than per-state antigens (`CampsiteDraft`,
//! `CampsiteBlocked`, etc.)? Per the SKILL.md design decision: each
//! campsite's "doneness" is a single discipline-claim with a single
//! immunity predicate. State machinery lives in the per-campsite Oracle
//! (Draft → Complete → Deprecated / Retired / Revoked), not in parallel
//! antigens. One CampsiteOpen + variable `requires` per campsite is
//! simpler than five state-specific antigens.

use antigen::antigen;

/// A unit of team coordination work that has not yet been completed.
///
/// Every campsite in `src/campsites/` declares a `pub fn done() {}`
/// function and marks it with `#[immune(CampsiteOpen, requires = ...)]`
/// where the `requires` predicate names the signers + conditions that
/// must be satisfied before the campsite is considered done.
///
/// Audit (`cargo antigen audit --root camp`) reports:
///
/// | Sidecar state | Tier | Hint |
/// |---|---|---|
/// | No `.attest/CampsiteOpen.json` for the campsite | `None` | `discipline-sidecar-missing` |
/// | Predicate fails (missing signers, etc.) | `None` | `discipline-predicate-failed` |
/// | All required signers attested, current fingerprint | `Execution` | `discipline-predicate-passed-substrate-current` |
///
/// To gate team-readiness in CI:
///
/// ```sh
/// cd camp
/// cargo antigen audit --strict
/// ```
///
/// Exits nonzero if any campsite hasn't reached `Reachability` tier or
/// above.
///
/// ## Family
///
/// `coordination-discipline` — a meta-family for camp-specific antigens
/// distinct from antigen's own 8-class failure-class taxonomy. Camp
/// antigens don't defend against bugs; they encode "this work needs
/// these signers to be considered complete."
///
/// ## Fingerprint
///
/// Matches any `fn` named `done`. Every campsite module exports a
/// `pub fn done() {}` function — that's the structural marker.
/// Functions named `done` outside campsite modules would also match;
/// in practice this happens nowhere else in camp source.
///
/// ## Why this antigen exists at all (vs hardcoded campsite logic)
///
/// Camp could enforce "all campsites must have signers" via a custom
/// linter or build script. Using antigen primitives instead means
/// camp's discipline is verified by the SAME machinery that adopters
/// use for their own disciplines. Self-validating: if antigen's
/// substrate-witness predicates work for camp's coordination
/// discipline, they work for any team's disciplines.
#[antigen(
    name = "campsite-open",
    family = "coordination-discipline",
    fingerprint = r#"all_of([item = fn, name = matches("done")])"#,
    summary = "A unit of team coordination work that has not yet been completed. \
               Each campsite's `done()` function presents this antigen; the immunity \
               claim names which signers must attest before completion is recognized. \
               Camp's first internal-discipline antigen — the substrate antigen \
               itself uses for team coordination.",
    references = [
        "https://github.com/antigen-rs/antigen/blob/main/docs/roadmap.md",
    ],
)]
pub struct CampsiteOpen;
