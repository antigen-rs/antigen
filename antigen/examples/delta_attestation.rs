//! Delta-attestation example — Fresh + chained Delta with anti-laundering safeguards.
//! (ADR-019 §M3 `SignerBasis` + adversarial T2-R three-layer safeguard set.)
//!
//! # The story
//!
//! Alice signs an attestation against `sinh` at fingerprint `fp-A`. Three
//! months later the function body changes — a minor refactor that doesn't
//! affect the signed-zero discipline. Alice could (a) re-review the diff
//! and sign Fresh against the new fingerprint `fp-B`, or (b) sign a Delta
//! that says "I reviewed the diff fp-A → fp-B and it preserves the
//! discipline invariant."
//!
//! Delta saves real time — re-reviewing the whole function for an
//! unrelated formatting change is wasted effort. But "trust me, the diff
//! is fine" is exactly how attestation discipline DEGRADES. Without
//! safeguards, a long chain of "trust me" deltas could drift arbitrarily
//! far from the originally-reviewed substance with no Fresh re-attestation
//! ever happening.
//!
//! ADR-019 §M3 ratifies three structural safeguards (adversarial T2-R):
//!
//! **Safeguard #1 — chain-depth cap**: the longest chain of consecutive
//! Delta entries since the last Fresh signature is capped (default 3,
//! configurable per workspace). At cap, audit reports
//! `discipline-substrate-delta-chain-near-cap`; over cap, schema validation
//! refuses to load the sidecar. The signer MUST do a Fresh re-attestation
//! after at most N deltas. Prevents long "trust-me" chains.
//!
//! **Safeguard #2 — cumulative-fingerprint tracking**: each Delta entry
//! records `cumulative_root_fingerprint` (the fingerprint of the LAST
//! Fresh signature in this chain). The audit verifies the cumulative diff
//! from root → current hasn't exceeded the workspace's threshold. Prevents
//! slow drift across many small-but-substantive deltas.
//!
//! **Safeguard #3 — required rationale**: every Delta entry MUST carry a
//! non-empty rationale of at least 20 characters (configurable hard floor
//! of 10). The CLI rejects empty/whitespace at write time; the schema
//! rejects at parse time. Prevents rubber-stamp "ok"/"lgtm" deltas.
//!
//! Together: short chains, anchored cumulative tracking, required
//! justification. Delta becomes a controlled relief valve, not a tier
//! erosion vector.
//!
//! # What this example demonstrates
//!
//! - Module-narrative on the three safeguards
//! - Walkable script of `attest sign` (Fresh) then `attest delta` (chained)
//!   invocations
//! - What each safeguard catches when violated
//!
//! Run:
//!
//! ```sh
//! cargo run --example delta_attestation --package antigen
//! ```

#![allow(dead_code, unused_imports)]

use antigen::{antigen, immune};

/// Witness function placeholder — the example's immune site references this.
/// The function itself is fixture; the discipline lives in the macro.
#[allow(dead_code)]
const fn discipline_invariant_holds_test() {
    // Real code would have a meaningful invariant test here.
}

/// Numeric-stability discipline for the example. A fixture antigen — the
/// real story is in the CLI workflow below, not the Rust types.
#[antigen(
    name = "numeric-stability-discipline",
    family = "forgotten-lesson",
    fingerprint = r#"all_of([item = fn, name = matches("stable_*")])"#,
    summary = "Functions whose names start with `stable_` are bound to a \
               numeric-stability discipline reviewed via signed sidecars. \
               Delta attestations carry chain-depth + cumulative-fingerprint \
               + rationale safeguards."
)]
pub struct NumericStabilityDiscipline;

/// Example site bound to the discipline.
///
/// The `requires` predicate names the audit-time contract: at least one
/// signer with role `numerics-reviewer`, and the most recent signature
/// within 365 days. Delta chains are governed by workspace-level
/// chain-depth cap (default 3) and rationale minimum (20).
#[immune(
    NumericStabilityDiscipline,
    requires = all_of([
        signers(required = ["numerics-reviewer"]),
        fresh_within_days(days = 365),
    ])
)]
pub fn stable_kahan_sum(values: &[f64]) -> f64 {
    let mut sum = 0.0_f64;
    let mut compensation = 0.0_f64;
    for &v in values {
        let y = v - compensation;
        let t = sum + y;
        compensation = (t - sum) - y;
        sum = t;
    }
    sum
}

/// Walkable CLI script demonstrating the Fresh → Delta workflow + safeguard
/// enforcement.
const DELTA_WALKTHROUGH: &str = r#"
DELTA-ATTESTATION WALKTHROUGH
=============================

──────────────────────────────────────────────────────────────────────────────
STEP 1: SCAFFOLD + FRESH SIGNATURE (the anchor)
──────────────────────────────────────────────────────────────────────────────

  cargo antigen attest scaffold \
    --antigen NumericStabilityDiscipline \
    --source-file antigen/examples/delta_attestation.rs \
    --item-path stable_kahan_sum \
    --fingerprint fp-A

  cargo antigen attest sign \
    --sidecar antigen/examples/.attest/NumericStabilityDiscipline.json \
    --item-path stable_kahan_sum \
    --signer alice --role numerics-reviewer \
    --fingerprint fp-A \
    --reasoning "reviewed Kahan summation implementation in full; compensation \
                 update and error-term tracking match Higham 2002 §4.3"

What writes to disk:
  signers = [{ name: "alice", basis: Fresh { reasoning: "..." }, ... }]

The Fresh basis anchors the chain. chain_depth = 0.

──────────────────────────────────────────────────────────────────────────────
STEP 2: FUNCTION BODY CHANGES — fp-A → fp-B (minor refactor)
──────────────────────────────────────────────────────────────────────────────

The function changes (whitespace, variable rename, etc.) but the
discipline-relevant substance doesn't. Alice could re-review and sign
Fresh, or she can sign a Delta.

  cargo antigen attest delta \
    --sidecar antigen/examples/.attest/NumericStabilityDiscipline.json \
    --item-path stable_kahan_sum \
    --signer alice \
    --fingerprint fp-B \
    --prior-fingerprint fp-A \
    --rationale "renamed variables for clarity; control flow unchanged; \
                 compensation arithmetic identical; invariant preserved"

What writes to disk:
  signers = [
    { name: "alice", basis: Fresh { ... },                            ... fp-A },
    { name: "alice", basis: DeltaFrom { prior: "fp-A", root: "fp-A",
                                        chain_depth: 1, rationale: "..." }, fp-B },
  ]

The DeltaFrom basis records:
  - prior_fingerprint = "fp-A"  (the immediately preceding signature)
  - cumulative_root_fingerprint = "fp-A"  (the last Fresh in this chain;
    at chain_depth = 1, these must be identical — NFA-12 invariant)
  - chain_depth = 1
  - rationale = <required non-empty, min 20 chars>

──────────────────────────────────────────────────────────────────────────────
STEP 3: ANOTHER MINOR CHANGE — fp-B → fp-C (Delta chain extends)
──────────────────────────────────────────────────────────────────────────────

  cargo antigen attest delta \
    --sidecar ... --item-path stable_kahan_sum --signer alice \
    --fingerprint fp-C --prior-fingerprint fp-B \
    --rationale "extracted helper function; arithmetic identical; \
                 invariant trivially preserved"

What writes to disk:
  signers = [
    { Fresh,                                                          fp-A },
    { DeltaFrom { prior: fp-A, root: fp-A, chain_depth: 1, ... },     fp-B },
    { DeltaFrom { prior: fp-B, root: fp-A, chain_depth: 2, ... },     fp-C },
  ]

Note: cumulative_root_fingerprint stays "fp-A" — anchored to the last
Fresh, not to the immediate prior. This is Safeguard #2 — the audit can
verify the CUMULATIVE diff from fp-A → fp-C, not just the local fp-B → fp-C
diff. A chain of small diffs cannot launder past a single-diff threshold.

──────────────────────────────────────────────────────────────────────────────
STEP 4: AT CHAIN-DEPTH CAP — audit emits warning
──────────────────────────────────────────────────────────────────────────────

With default cap = 3, after Alice's third Delta:

  signers = [Fresh fp-A, Delta fp-B, Delta fp-C, Delta fp-D (chain_depth=3)]

`cargo antigen attest check` reports:
  audit_hint = discipline-substrate-delta-chain-near-cap
  witness_tier = Execution  (still passing, but warned)

The hint surfaces to operator: "you're at the chain-depth cap; the next
signature MUST be Fresh, not Delta." The audit hasn't refused yet, but
CI gates can be configured to refuse at this hint.

──────────────────────────────────────────────────────────────────────────────
STEP 5: ATTEMPT TO EXCEED CAP — schema validation refuses
──────────────────────────────────────────────────────────────────────────────

If a sidecar is hand-edited to chain_depth = 4 (over the default cap = 3),
or if the workspace config tries to raise the cap above the hard floor
HARD_DELTA_CHAIN_CAP_MAX = 10:

  ValidationError::ChainDepthExceeded { chain_depth: 4, cap: 3, ... }
  → audit_hint = discipline-sidecar-schema-invalid

This is Safeguard #1 — the operator MUST do a Fresh re-attestation
(re-review the entire current state against the discipline) after at
most cap deltas. The workspace can configure cap downward (tighter) but
NOT above the hard floor.

──────────────────────────────────────────────────────────────────────────────
STEP 6: RUBBER-STAMP RATIONALE — schema validation refuses
──────────────────────────────────────────────────────────────────────────────

Trying to sign a Delta with rationale = "ok" or rationale = "lgtm":

  cargo antigen attest delta ... --rationale "ok"

  → CLI rejects at write time (rationale too short, min 20 chars)

Or if a sidecar is hand-edited with rationale = "":

  ValidationError::EmptyDeltaRationale { ... }
  → audit_hint = discipline-sidecar-schema-invalid

This is Safeguard #3 — Delta entries cannot be rubber-stamped. Every
Delta MUST carry actual signal in its rationale field (at minimum 20
chars by default, hard floor 10).

──────────────────────────────────────────────────────────────────────────────
SUMMARY
──────────────────────────────────────────────────────────────────────────────

The Delta basis is a controlled relief valve. It saves real time when
a diff genuinely doesn't affect the discipline (formatting, variable
renames, refactors that preserve the invariant). The three safeguards
together prevent it from becoming a tier-erosion vector:

  #1 — chain-depth cap stops long "trust-me" chains
  #2 — cumulative-fingerprint tracking stops slow drift
  #3 — required rationale stops rubber-stamping

Biology rhyme: somatic hypermutation in B-cell germinal centers can
mutate antibody specificity across many rounds, but the lineage is
recorded; affinity-matured cells get re-selected against the antigen,
not trusted blindly. Delta with safeguards = mutation with selection,
not mutation with amnesia.
"#;

fn main() {
    println!("antigen delta-attestation example — Fresh + chained Delta with safeguards.");
    println!();
    println!("{DELTA_WALKTHROUGH}");

    // Exercise the example function so it's not pure dead-code.
    let sum = stable_kahan_sum(&[1.0e10, 1.0, -1.0e10, 1.0]);
    println!("stable_kahan_sum example: {sum}");
}
