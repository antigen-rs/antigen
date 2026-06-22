//! Oracle lifecycle example — declare → complete → attest → deprecate.
//! (ADR-021 oracle-as-artifact-class with stewardship + state machine.)
//!
//! # The story
//!
//! Your methodology depends on an external reference — Higham 2002 §6.3
//! ("Accuracy and Stability of Numerical Algorithms," 2nd edition,
//! Section 6.3 on triangular linear systems backward error). You want
//! signers to be able to attest "I reviewed my code against this exact
//! reference" with a structural pointer to the reference — not a free-text
//! URL that goes stale, not a copy-pasted excerpt that drifts, but a
//! first-class workspace artifact with stewardship and lifecycle.
//!
//! That's the oracle artifact-class (ADR-021 §D3 Model B):
//!
//! - **Oracle**: a typed reference (`LocalFile` / `Url` / `Doi` / `Arxiv`
//!   / `GitHubIssue` / `Other`) bound to a 5-state lifecycle
//!   (`Draft → Complete → Deprecated / Retired / Revoked`)
//! - **Stewards**: appointed reviewers who authorize state transitions.
//!   Minimum 2 per oracle (ATK-021-13 succession mitigation). Stewards are
//!   categorically distinct from signers — the reviewer who attests
//!   against an oracle is NOT necessarily its steward.
//! - **State machine**: monotonic. Backward transitions prohibited. Draft
//!   blocks signer attestation (`oracles_complete(...)` predicate rejects);
//!   Complete admits; Deprecated/Retired preserve prior attestations at
//!   `Execution` tier; Revoked with `invalidates_prior_attestations=true`
//!   retroactively demotes priors to `Reachability`.
//! - **Sign-time validity** (§D4): an attestation's tier is determined by
//!   the oracle's state AT SIGN TIME, not at audit time. Stewards changing
//!   state after attestation produce audit HINTS, not tier degradation
//!   (except `Revoked(invalidates_prior=true)`).
//!
//! # What this example demonstrates
//!
//! This file is mostly NARRATIVE — the oracle lifecycle is CLI-driven, so
//! the example is a walkable script of `cargo antigen oracle ...` invocations
//! with explanations of what each one means and what happens on disk. The
//! Rust code at the bottom is a small fixture that prints the workflow.
//!
//! Run:
//!
//! ```sh
//! cargo run --example oracle_lifecycle --package antigen
//!
//! # Then walk the lifecycle yourself in a tempdir:
//! mkdir /tmp/oracle-demo && cd /tmp/oracle-demo
//! cargo run --manifest-path R:/antigen/Cargo.toml --bin cargo-antigen -- antigen oracle list
//! ```

#![allow(dead_code, unused_imports)]

/// Walkthrough script. Each step lists a single `cargo antigen oracle`
/// invocation + what changes on disk + what the audit/scan would report.
const ORACLE_LIFECYCLE_SCRIPT: &str = r#"
ORACLE LIFECYCLE WALKTHROUGH
============================

The oracle artifact-class lives at workspace-level: `.antigen/oracles/<id>.oracle.json`.
Each step below mutates that file via a stewardship-authorized CLI verb.

──────────────────────────────────────────────────────────────────────────────
STEP 1: DECLARE — create an oracle in Draft state
──────────────────────────────────────────────────────────────────────────────

  cargo antigen oracle declare \
    --id higham-2002-section-6-3 \
    --kind doi \
    --reference "10.1137/1.9780898718027" \
    --steward alice \
    --steward bob \
    --rationale "Higham 2002 §6.3 (triangular system backward error) chosen as \
                 the authoritative methodology reference for our numeric linear \
                 solver. Alice + Bob appointed as stewards per team-lead 2026-03-01."

What happens:
  - File created at `.antigen/oracles/higham-2002-section-6-3.oracle.json`
  - state = "draft"
  - stewards = [{ name: "alice", authorization_basis: <rationale> },
                 { name: "bob",   authorization_basis: <rationale> }]
  - transitions = [] (creation event captured by Provenance, NOT as a transition)

Schema enforcement: < 2 stewards rejected at save time (ATK-021-13).
The CLI emits a warning for single-steward declares; save_oracle's
Oracle::validate() refuses to write.

──────────────────────────────────────────────────────────────────────────────
STEP 2: ATTEMPT EARLY ATTESTATION — should be REJECTED while Draft
──────────────────────────────────────────────────────────────────────────────

If a signer tries to attest against the DRAFT oracle via a predicate like:

  requires = oracles_complete(files = ["higham-2002-section-6-3"])

…then `cargo antigen attest check` reports `discipline-predicate-failed`
at `WitnessTier::None`. Draft oracles BLOCK the `oracles_complete` leaf —
this is the load-bearing structural guarantee: no signer can attest
against an oracle whose stewards haven't authorized it as authoritative.

──────────────────────────────────────────────────────────────────────────────
STEP 3: COMPLETE — steward authorizes Draft → Complete
──────────────────────────────────────────────────────────────────────────────

  cargo antigen oracle complete \
    --id higham-2002-section-6-3 \
    --steward alice \
    --version "2nd-edition-2002" \
    --rationale "Reviewed §6.3 in full; methodology applies cleanly to our \
                 triangular solver implementation; bob co-reviewed and concurs."

What happens:
  - state = "complete"
  - transitions = [{ from: "draft", to: "complete", authorized_by: "alice", at: <today>,
                     rationale: <rationale> }]
  - version.pinned = "2nd-edition-2002" (load-bearing: stewards pin the
    version of the reference at the moment of completion; signers attest
    against this pinned snapshot, not the live reference)

Schema enforcement: authorized_by must appear in stewards[*].name (ATK-021-15).
A non-steward authorizing the transition is rejected at validation time.

──────────────────────────────────────────────────────────────────────────────
STEP 4: ATTEST AGAINST THE COMPLETE ORACLE
──────────────────────────────────────────────────────────────────────────────

Now signers can attest. The `oracles_complete` leaf accepts the Complete
oracle; the audit reports `discipline-predicate-passed-substrate-current`
at `WitnessTier::Execution` with `EvidenceKind::SubstrateState`.

In practice this is the same `cargo antigen attest scaffold/sign` flow
from substrate_witness.rs — the difference is the predicate includes
`oracles_complete(files = [<oracle-id>])` and the audit reads from the
canonical oracle store at `.antigen/oracles/`.

──────────────────────────────────────────────────────────────────────────────
STEP 5: DEPRECATE — Complete → Deprecated when superseded
──────────────────────────────────────────────────────────────────────────────

  cargo antigen oracle deprecate \
    --id higham-2002-section-6-3 \
    --steward bob \
    --superseded-by higham-2002-section-6-4 \
    --rationale "Section 6.4 (block-triangular solvers) covers our v0.2 algorithm \
                 cleanly; 6.3 is still correct for v0.1 but no longer the canonical \
                 reference for new work."

What happens:
  - state = "deprecated" with superseded_by = "higham-2002-section-6-4"
  - transitions = [{ from: "draft", to: "complete", ... },
                    { from: "complete", to: "deprecated", authorized_by: "bob", ... }]
  - SIGN-TIME VALIDITY (§D4): existing attestations against the Complete state
    REMAIN HONORED at Execution tier. The audit emits an informational hint
    ("oracle-deprecated") on those attestations but does NOT demote them.
    Signers cannot be held responsible for post-sign-time state changes.

The Deprecated/Retired/Revoked distinction matters:
  - Retired: oracle is gone but the discipline was correct. Use when an
    oracle is no longer maintained but the underlying methodology stands.
  - Deprecated(superseded_by): same-trust replacement available; soft handoff.
  - Revoked(invalidates_prior=true): oracle was INCORRECT or fraudulent.
    Retroactively demotes prior attestations to Reachability. Use only when
    the methodology itself was wrong.

──────────────────────────────────────────────────────────────────────────────
STEP 6: INSPECT VIA STATUS + LIST
──────────────────────────────────────────────────────────────────────────────

  cargo antigen oracle status --id higham-2002-section-6-3

Reports current state + steward list + transition log (append-only,
time-ordered). Each transition entry preserves WHO authorized + WHEN +
WHY — the full institutional memory of the discipline's evolution.

  cargo antigen oracle list [--format json]

Walks `.antigen/oracles/` and inventories every declared oracle with id +
current state + steward count. JSON format for scripting; human format
for terminals.

──────────────────────────────────────────────────────────────────────────────
SUMMARY
──────────────────────────────────────────────────────────────────────────────

Oracle as artifact-class gives discipline references the same structural
treatment that #[immune] gives function bodies: the WHY lives at a stable
address, stewardship is explicit, transitions are auditable, and signer
attestations are pinned to a specific reviewed version. The biology rhyme
that grounds this (per B-021-4): follicular dendritic cells (FDCs) are of
stromal origin distinct from B-cells — stewardship and attestation are
biology-predicted to be structurally separated, not just convention.
"#;

fn main() {
    println!("antigen oracle lifecycle example — walkable CLI script.");
    println!();
    println!("{ORACLE_LIFECYCLE_SCRIPT}");
}
