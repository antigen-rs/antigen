//! Build camp — the recursive seed campsite.
//!
//! Camp's first campsite is "build camp itself." This module declares the
//! discipline-claim: when both tekgy and team-lead have attested, camp is
//! considered done (in the sense of: the camp crate exists, the antigens
//! compile, the first campsite — this one — has its sidecar wired up, and
//! `cargo antigen audit --root camp` reports Execution tier on this
//! campsite's `done()` function).
//!
//! ## The recursion
//!
//! Camp is dogfood. Camp's first work-unit is camp's own existence. When
//! `cargo antigen audit --root camp` reports this campsite at Execution
//! tier, that's the proof-of-concept that the dogfood pattern works at
//! all — the same audit machinery adopters will run on their own crates
//! is the machinery that validates camp itself.
//!
//! ## Required signers
//!
//! - **tekgy**: this is load-bearing-for-user (deciding camp's shape
//!   is a relationship-defining design call); Tekgy's attestation is
//!   "yes, this is the camp we want."
//! - **team-lead** (parent Claude / main-thread): integration check
//!   between camp design and the wider antigen project's architecture;
//!   "yes, this slots cleanly into how the antigen team works."
//!
//! Smaller required-set than later campsites (which typically include
//! navigator + pathmaker + observer) because this campsite is for-us-
//! together work, not team-dispatched. Once camp exists, future
//! campsites will use the full team-role signer set.
//!
//! ## What "done" means here
//!
//! Specifically: the camp crate at `R:/antigen/camp/` compiles (`cargo
//! check`), `cargo antigen scan --root camp` finds the `CampsiteOpen`
//! antigen declaration + this `done()` function as a presentation,
//! `cargo antigen audit --root camp` evaluates the substrate-witness
//! predicate and reports the right tier per the sidecar state, and
//! both signers have attested via `cargo antigen attest sign`.
//!
//! ## What "done" does NOT mean
//!
//! - That camp is feature-complete (it isn't; the SKILL.md flags many
//!   v0.2+ extensions)
//! - That the camp CLI wrapper exists (it doesn't yet; `cargo antigen`
//!   primitives ARE the CLI for v0.1)
//! - That other campsites have been created (none have yet; this is the
//!   seed)
//! - That the SKILL.md evolution to usage-spec is complete (it's still
//!   in design+usage hybrid form)

// Note: antigens.rs `CampsiteOpen` is referenced in `#[immune(CampsiteOpen, ...)]`
// macro args. The macro resolves it via name-lookup; no `use` import needed here.
//
// Same story for role names: antigen's `requires = signers(required = [...])`
// macro parses STRING LITERALS only, not Rust constants. So we write the role
// names inline (`"tekgy"`, `"team-lead"`) rather than referencing
// `roles::TEKGY` etc. The constants in `roles.rs` are the canonical-list
// reference for adopters; v0.2+ antigen macro enhancement might let us use
// the constants directly (found-friction logged in SKILL.md).

use antigen::immune;

/// The build-camp campsite.
///
/// Existence-marker for the campsite; doc-comment is the canonical
/// description of the work + required signers + what "done" means.
/// The signing happens on the `done()` function below.
pub struct BuildCamp;

/// Build-camp completion claim.
///
/// Immune to `CampsiteOpen` (presentation = "this campsite is not done
/// yet") when the required signers have attested. Required signers:
/// `tekgy` + `team-lead`.
///
/// To attest:
///
/// ```sh
/// cargo run --release --bin cargo-antigen -- antigen attest sign \
///   --sidecar camp/src/campsites/.attest/CampsiteOpen.json \
///   --item-path camp::campsites::build_camp::done \
///   --signer tekgy --role tekgy \
///   --fingerprint <from scan output>
/// ```
///
/// (Substitute `--signer team-lead --role team-lead` for the second
/// attestation.)
///
/// To check tier:
///
/// ```sh
/// cargo run --release --bin cargo-antigen -- antigen audit --root camp
/// ```
#[immune(
    CampsiteOpen,
    requires = all_of([
        signers(required = ["tekgy", "team-lead"]),
    ])
)]
pub fn done() {
    // Empty by design — the discipline-claim is the substrate.
    // When both required signers have attested in the sidecar, the
    // audit reports this function at Execution tier with
    // EvidenceKind::SubstrateState.
}
