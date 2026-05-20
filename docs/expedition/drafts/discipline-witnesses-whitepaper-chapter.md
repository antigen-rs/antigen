# Discipline Witnesses — Whitepaper Chapter

> **Status**: Draft 2026-05-19 — scientist-authored whitepaper chapter for
> discipline-witnesses. Companion to [`docs/vision-pitch.md`](../vision-pitch.md).
> Audience: Rust ecosystem maintainers, library authors, tooling engineers.
> Covers basic / mid / advanced tier usage.
>
> **Scope**: v0.1-rc discipline-witnesses feature (ADR-019). Assumes reader has
> read [`docs/vision-pitch.md`](../vision-pitch.md); this chapter goes deeper on
> the discipline-witness primitive specifically.

---

## The Gap That Antigen's Core Macros Don't Fill

Antigen's core macros — `#[antigen]`, `#[immune]`, `#[presents]`, `#[descended_from]`
— handle failure-class memory where immunity is *mechanically verifiable*: a test
passes, a proof is valid, a phantom type compiles, a clippy lint is satisfied. For
these cases, `cargo antigen audit` can verify immunity by running the witness. The
audit is self-contained.

But many real failure-classes in real Rust projects don't have mechanically-verifiable
immunity. Consider:

- A mathematical library has a function whose signed-zero behavior under IEEE 754
  has been formally reviewed by the team's math researcher — a discipline applied
  through expert judgment, not a test that can pass or fail.
- A security library exports a function whose constant-time property has been
  verified against a specific algorithm document by two reviewers over two weeks.
  No automated tool can verify "the team reviewed this against the documented
  algorithm"; no test can produce that evidence.
- An oracle test file marks completion of a test scenario as `status: complete` —
  meaningful progress toward immunity, but not immunity itself. The audit should
  report this honestly.

These are **discipline failure-classes**: the antigen marks a code site as needing
a specific kind of discipline to be immune, and the witness for that discipline is
not a piece of code but a record — a ratified doc, a sign-off, a completed oracle,
a structured attestation that the right people applied the right review.

The naive response is: "Add this to a README or a PR description." The problem with
that response is drift. README entries and PR descriptions are invisible to
`cargo antigen audit`; they drift from the code; they don't produce machine-readable
tier reports; they don't alert CI when the discipline goes stale.

The discipline-witness primitive makes discipline failure-classes **structurally
first-class** in antigen's vocabulary — with the same tier-honest audit output,
the same fingerprint-pinning that detects drift, and the same composability with
existing witnesses.

---

## What Ships: Three Coupled Pieces

Discipline-witnesses ship as one primitive in v0.1-rc — three pieces that require
each other:

1. **A substrate-witness predicate language**: a small closed declarative language
   over on-disk substrate (docs, sidecars, git log, oracle files). The predicate
   lives in the `requires =` parameter on `#[immune]` or `#[antigen_tolerance]`.

2. **A ratification schema**: a `serde`-derived Rust type in the
   `antigen-attestation` crate. JSON sidecars adjacent to source files carry the
   structured attestation records — who signed, when, against which fingerprint,
   with what reasoning. The same schema serves both immunity claims and tolerance
   ratifications.

3. **A `cargo antigen attest` CLI family**: the only write-path to the sidecar.
   `scaffold` creates an empty sidecar; `sign` records a signer entry against the
   current fingerprint; `check` evaluates predicates; `list` shows pending or
   stale attestations. A parallel `cargo antigen tolerate` family handles tolerance
   ratifications.

Each piece alone delivers nothing. The predicate language with no sidecar to
evaluate against is empty syntax. The sidecar with no predicate language to express
requirements is an unreadable blob. The CLI with no schema to scaffold is an
unstructured note-taking tool. They ship together because the primitive is
the combination.

---

## Basic Tier: First-Contact Attestation

**Who this is for**: A team that has one function needing a discipline sign-off. Low
setup cost; immediate CI value.

**What they get**: machine-readable attestation that a human reviewed this code,
with automatic audit failure when the code drifts past the point of review.

### The Scenario

The `sinh` function in a mathematics library implements signed-zero behavior under
IEEE 754. The math researcher has reviewed the implementation against the team's
methodology document. Without discipline-witnesses, this review lives in a PR
comment or a private Slack message — invisible to future audits, invisible to CI.

### Step 1: Declare the Antigen

```rust
#[antigen(
    SignedZeroDiscipline,
    scope = "site",
    discipline_doc = "docs/methodology/signed-zero.md",
)]
pub struct SignedZeroDiscipline;
```

This already existed for other reasons (antigen's core vocabulary). Discipline-
witnesses don't require a new kind of antigen — they extend the existing witness
vocabulary.

### Step 2: Express the Immunity Predicate

```rust
#[immune(SignedZeroDiscipline, requires = all_of([
    ratified_doc(path = "docs/methodology/signed-zero.md", min_version = "1.0"),
    signers(required = ["math-researcher"]),
]))]
pub fn sinh(x: f64) -> f64 {
    // ...
}
```

The predicate says: for `sinh` to be immune, the methodology doc must exist at
version 1.0 or later, and the sidecar must record a sign-off from `math-researcher`.

### Step 3: Create the Sidecar

```sh
cargo antigen attest scaffold-anchor \
    --file src/numerics.rs \
    --antigen SignedZeroDiscipline \
    --item sinh \
    --signer math-researcher \
    --doc docs/methodology/signed-zero.md \
    --reasoning "Reviewed sinh implementation against signed-zero methodology §3.2. \
                 Verified: (a) sinh(0.0) returns +0.0; (b) sinh(-0.0) returns -0.0; \
                 (c) limit behavior at ±∞ is correct per IEEE 754-2019 §9.2.1."
```

This creates `src/numerics.attest/SignedZeroDiscipline.json` adjacent to the source
file. It's a regular file; it appears in PR diffs; it's machine-readable.

### Step 4: What `cargo antigen audit` Now Reports

```
[SubstrateState] SignedZeroDiscipline on sinh — Execution tier
  AuditHint: discipline-predicate-passed-substrate-current
  Signature: git-trust (math-researcher, 2026-05-19, against fingerprint abc123)
```

If the `sinh` body changes and the sidecar fingerprint drifts:

```
[SubstrateState] SignedZeroDiscipline on sinh — None tier
  AuditHint: discipline-substrate-stale
  Note: signed_against_fingerprint abc123 no longer matches current fingerprint def456.
        Run: cargo antigen attest check --file src/numerics.rs
```

CI fails. The team sees: this function has drifted past its sign-off point. Someone
must re-review and re-sign.

### Basic Tier Summary

| What you write | `requires = signers(...)` predicate + `scaffold-anchor` invocation |
| What CI sees | Machine-readable tier report; fails on code drift |
| What reviewer sees | Adjacent JSON sidecar in the PR diff with reasoning |
| Escalation path | Add `fresh_within_days` for periodic re-review; add more signers |

**Adoption cost**: one macro parameter, one CLI invocation, one JSON file. The
`reasoning` field records the review in the sidecar; the fingerprint pins it.
When the code changes, the audit fails automatically.

---

## Mid Tier: Team Workflows, Predicate Composition, Tolerance Ratification

**Who this is for**: A team that uses pull-request review workflows, has multiple
domain-expert signers, and maintains formal discipline documents that evolve over time.
They want the attestation to reflect real team process, not just individual sign-offs.

### Scenario: Multi-Signer with Freshness

A cryptographic library has a constant-time comparison function. Immunity requires
two independent reviews: a cryptography reviewer and a Rust-safety reviewer. The
review must be refreshed every 180 days to account for changes in compiler
optimization behavior and new tooling.

```rust
#[immune(ConstantTimeComparison, requires = all_of([
    ratified_doc(
        path = "docs/disciplines/constant-time.md",
        min_version = "2.0",
        anchor = "comparison-functions",
    ),
    signers(required = ["crypto-reviewer", "rust-safety-reviewer"], against = "current"),
    fresh_within_days(180),
]))]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    // ...
}
```

The `against = "current"` parameter means both signers must have signed against
the *current* fingerprint — not any historical fingerprint. If one signer is
stale, the audit drops to `WitnessTier::Reachability` with
`discipline-substrate-stale`.

The `anchor = "comparison-functions"` parameter means the methodology doc must
contain that specific heading. If the section is renamed, the predicate fails and
the team is alerted.

### Scenario: Oracle Completion Tracking

A numerics library has a set of oracle test fixtures that represent known-good
outputs for edge-case inputs. Immunity requires not just that the tests run, but
that the oracle fixtures are manually marked complete by a domain expert.

```rust
#[immune(SignedZeroDiscipline, requires = all_of([
    signers(required = ["math-researcher"]),
    oracles_complete([
        "tests/oracles/sinh-signed-zero.md",
        "tests/oracles/cosh-signed-zero.md",
    ]),
]))]
pub fn sinh(x: f64) -> f64 { /* ... */ }
```

When the math researcher completes their oracle verification:

```sh
cargo antigen attest oracle complete \
    --file tests/oracles/sinh-signed-zero.md
```

This sets `status: complete` in the oracle file's frontmatter. The audit now checks
this as part of the predicate. Oracle files that haven't been completed produce
`discipline-predicate-failed` with per-leaf details.

### Scenario: PR Reviewer Who Didn't Commit

Standard pull-request workflows often have a subject-matter expert review code
that a different person commits. Alice reviews; Bob commits. The git committer
is Bob; the reviewer is Alice.

The `signed_trailer` leaf handles this by checking git commit trailers rather
than the committer identity:

```rust
#[immune(ConstantTimeComparison, requires = all_of([
    signed_trailer(key = "Crypto-Reviewed-By", role = "crypto-reviewer", count = 1),
    ratified_doc(path = "docs/disciplines/constant-time.md", min_version = "2.0"),
]))]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool { /* ... */ }
```

Alice adds `Crypto-Reviewed-By: alice@example.com` to the commit trailer when
reviewing Bob's commits. The `signed_trailer` leaf invokes `git interpret-trailers`
against the git log for commits touching this item. The reviewer's identity is
captured in the git log; the predicate verifies its presence.

In v0.4+, crypto-signing via DSSE envelopes will decouple signer identity from
committer identity entirely.

### Scenario: Tolerance Ratification

Not every deviation from an antigen's fingerprint is a bug. Sometimes a team has
reviewed a known non-compliant site and decided the tradeoff is acceptable for
documented reasons. Antigen's `#[antigen_tolerance]` macro records this decision.

Without discipline-witnesses, tolerance is vibes-grade: the rationale is an inline
string that no audit can verify was reviewed by anyone. In v0.1-rc, tolerance
ratification uses the same sidecar schema as immunity claims.

```rust
#[antigen_tolerance(SignedZeroDiscipline, sidecar = true, requires = all_of([
    signers(required = ["math-lead"], against = "current"),
    ratified_doc(
        path = "docs/exceptions/legacy-sinh.md",
        min_version = "1.0",
    ),
    fresh_within_days(90),  // tolerance is more accountable than immunity — shorter window
]))]
pub fn legacy_sinh(x: f64) -> f64 {
    // Legacy implementation maintained for API compatibility.
    // Tolerance rationale: docs/exceptions/legacy-sinh.md
}
```

The tolerance sidecar carries the same fingerprint-pinning and sign-off structure
as immunity. If the deprecated function changes significantly, the tolerance-stale
hint fires, prompting re-review.

Without `sidecar = true`, the audit reports:
```
[None] SignedZeroDiscipline on legacy_sinh — None tier
  AuditHint: tolerance-vibes-grade
  Note: #[antigen_tolerance] without sidecar = true is unattested.
        Add sidecar = true and run: cargo antigen tolerate scaffold ...
```

This makes the distinction between "intentionally tolerated with team sign-off"
and "tolerance added to silence the audit" machine-readable for the first time.

### Mid Tier Summary

| Workflow | Substrate-witness mechanism |
|---|---|
| Multi-signer team review | `signers(required = [...], against = "current")` |
| Living discipline documents | `ratified_doc(min_version = "N.0", anchor = "...")` |
| Oracle fixture completion | `oracles_complete(["..."])` |
| Reviewer ≠ committer | `signed_trailer(key = "...", role = "...")` |
| Periodic re-review | `fresh_within_days(N)` |
| Attested tolerance | `#[antigen_tolerance(..., sidecar = true)]` |

---

## Advanced Tier: Delta-Attestation, Cross-Crate Predicates, and the Long Arc

**Who this is for**: Large teams, security-critical libraries, and teams building
antigen into their permanent development discipline. Covers anti-laundering
safeguards, cross-crate predicate inheritance, and the v0.4+ escalation path.

### Delta-Attestation and Anti-Laundering

In long-lived projects, a function's body changes incrementally. A signer who
reviewed the function at fingerprint `abc123` shouldn't have to re-review the
entire function if only a comment was changed. The `attest delta` command records
a carry-forward attestation with explicit rationale:

```sh
cargo antigen attest delta \
    --file src/numerics.rs \
    --antigen SignedZeroDiscipline \
    --item sinh \
    --from abc123 \
    --as math-researcher \
    --rationale "Comment addition only; invariant-preserving. Reviewed diff; \
                 sinh body unchanged. No impact on signed-zero discipline."
```

However, a chain of individually-small deltas can accumulate into a substantively
different function — with each individual delta appearing reasonable. This is the
delta-laundering attack: evading the freshness requirement through many small changes
that individually don't trigger re-review but cumulatively represent a complete
rewrite.

The v0.1-rc anti-laundering safeguards prevent this through three layered mechanisms:

**Chain-depth cap** (default 3, workspace-configurable): after three delta-attestations,
the signer must do a Fresh re-attestation (`attest sign`). The cap prevents indefinite
carry-forward.

**Cumulative-diff tracking**: the schema records `cumulative_root_fingerprint` — the
fingerprint at the last Fresh attestation. The audit verifies that the cumulative
change (root fingerprint → current fingerprint) hasn't grown beyond a configurable
threshold (default: 200 lines or 25% of the item). If the cumulative diff exceeds
the threshold, the next delta is refused: the signer must do Fresh.

**Non-empty rationale**: the schema rejects empty or whitespace-only rationale
on `SignerBasis::DeltaFrom`. Every carry-forward must articulate why the carry-forward
is appropriate.

These three together prevent the laundering attack. Removing any one re-opens the
surface.

### Predicate Inheritance via `descended_from`

When a new function is structurally descended from an already-immune function, the
`#[descended_from]` macro propagates the antigen marker. The substrate-witness
predicate propagates with the descent.

A key discipline: a consumer that redeclares a substrate-witness predicate for a
descended antigen cannot silently weaken it. If `descendant_sinh` weakens the
predicate (fewer required signers, shorter freshness window), it must declare this
explicitly:

```json
// In the sidecar for the descendant antigen:
{
  "weakened_from": "signed-zero-discipline",
  "weakening_rationale": "Consumer context: research prototype; \
                          single-signer review acceptable per team lead approval"
}
```

Without the explicit weakening declaration, the audit emits
`discipline-predicate-weakening-undeclared`. This is Eiffel's variance rule applied
to substrate-witness predicates: weakening preconditions on immunity claims requires
explicit declaration.

### Signing Tier Escalation — Following the Notary Arc

The v0.1-rc attestation uses **git-trust**: signer identity comes from
`git config user.name` and `user.email`; the attestation is tied to the git
commit history; accountability is workspace-bounded.

This is the right floor for most teams. The value is not cryptographic non-repudiation
but *accountable attestation within a known audience*: the workspace's developers
know who signed; the git log preserves the record; the fingerprint pin detects drift.

This maps onto a 900-year institutional precedent: the **civic notary** of medieval
Italian city courts. A properly drafted notarial instrument was treated by these
courts as near-self-authenticating proof — the court didn't re-investigate a
transaction if a notary had attested it. The value was audit-time savings at a
known confidence level, not cryptographic certainty. The accountability structure
(guild membership, licensed examination, documentary trail) underwrote that trust
within a bounded audience.

The escalation path for antigen mirrors the historical escalation from civic notary
to notary public:

**v0.1: Git-trust** — workspace-bounded audit-time savings. Signer identity from
git config. Fingerprint pinning via sidecar. Audience: the workspace's own developers
and CI.

**v0.2: CODEOWNERS-role verification** — institutional roles declared in the
workspace's CODEOWNERS file as signing authorities. `required_role` parameter on
the `signers` leaf. The role declaration is itself a substrate-read (CODEOWNERS
is a file; the audit can verify role membership). Audience: teams where role-based
sign-off is already institutionalized.

**v0.4+: DSSE + Sigstore** — cryptographic identity bound to OIDC authentication.
The signer's signature is a DSSE-PAE-encoded entry; Sigstore's Rekor transparency
log provides an append-only record visible outside the workspace. Audience:
cross-organization consumers who weren't party to the internal workflow.

**The escalation is not "stronger signing" but "wider audience for whom audit-time
savings hold."** A workspace's own developers trust git-trust attestation for the
same reason Genoa's merchants trusted the local notary: the accountability structure
exists within the audience. Cross-organizational consumers need the OIDC-bound
signature for the same reason an out-of-city creditor needed the notary-public
with papal license: the accountability must be portable.

Teams choose the tier that matches their audience, not the tier they aspire to.
A research prototype needs git-trust. A crate published to crates.io for security-
critical use cases should be moving toward DSSE + Sigstore as the ecosystem matures.

### Three-Axis Audit Output — Reading the Report

Advanced-tier users need to understand what `cargo antigen audit` actually measures
and what it doesn't. The three-axis output:

```
[SubstrateState / Execution / discipline-predicate-passed-substrate-current]
  SignedZeroDiscipline on sinh — git-trust
  Signers: math-researcher (Fresh, 2026-05-19), alice (DeltaFrom, 2026-05-22, depth=1)
```

The three axes are orthogonal:

- **`EvidenceKind`** (`TypeSystemProof` / `Behavioral` / `SubstrateState`):
  *what kind of evidence is this?* SubstrateState means the audit verified on-disk
  substrate (sidecar JSON, git log, doc presence). It cannot be promoted to
  TypeSystemProof; it cannot be promoted to FormalProof tier.

- **`WitnessTier`** (`FormalProof` / `Execution` / `Reachability` / `None`):
  *how much verification work was done?* SubstrateState evidence can reach Execution
  when the predicate passes and currency holds. It cannot reach FormalProof — that
  tier is reserved for TypeSystemProof evidence (phantom-type proofs, kani
  verification).

- **`AuditHint`**: *which specific state triggered this result?* The hint provides
  disambiguation within a tier. `discipline-predicate-passed-substrate-current`
  means the predicate passed and all signatures are current. `discipline-substrate-stale`
  means the predicate passed but at least one signature is stale.

**What this does NOT mean**: "SubstrateState / Execution" does not mean "the
discipline was applied correctly." It means "the predicate passed; the sidecar is
current; the audit time-saved by trusting the recorded attestation." The discipline
quality depends on who signed, what they reviewed, and whether their reasoning is
sound. The audit can verify the *structure* of attestation; it cannot verify the
*quality* of the review.

This is why the audit hint says `discipline-predicate-passed-substrate-current`,
not `discipline-validated`. What was validated is the predicate against the sidecar.
What the discipline quality is remains consumer judgment — auditable, accountable,
machine-recorded, but not machine-certifiable.

### Advanced Tier Summary

| Capability | Mechanism |
|---|---|
| Safe carry-forward attestation | `attest delta` with chain-depth cap + cumulative tracking + required rationale |
| Predicate inheritance | `#[descended_from]` + `weakened_from` explicit declaration |
| Signing tier escalation | v0.1 git-trust → v0.2 CODEOWNERS-role → v0.4 DSSE+Sigstore |
| Understand what audit reports | WitnessTier × AuditHint × EvidenceKind three-axis |
| What audit cannot certify | Review quality, domain correctness, expert judgment |

---

## What Discipline-Witnesses Don't Do

**Not a replacement for code witnesses.** `#[immune(X, witness = proptest!{...})]`
remains the highest-confidence witness for mechanically-verifiable properties.
Substrate-witnesses extend the vocabulary to cover discipline properties; they don't
replace the code-side witness mechanism.

**Not vibes-grade attestation.** Signing requires real substrate: `git config`
identity (no anonymous bypass), fingerprint pin (drift detection), structured
sidecar entry (schema-validated, not free-text). The attestation is accountable,
not decorative.

**Not a documentation system.** The sidecar carries compliance state and signing
history, not document content. Documentation lives in the discipline doc and the
`reasoning` field; the sidecar carries the structured record of who applied the
discipline and when.

**Not a CI gate by default.** `cargo antigen audit` reports state; CI gates
configure which hint levels trigger failures. A team can start reporting without
gating and graduate to gating as discipline matures.

**Not a single-tier system.** Basic users get value from git-trust attestation.
Advanced teams can layer in CODEOWNERS-role verification and eventually DSSE+Sigstore.
The tiers compose with the same predicate language.

---

## Composing with Code Witnesses

The most powerful use of discipline-witnesses is in combination with code witnesses.
A function can require both a mechanical test and a discipline sign-off:

```rust
#[immune(SignedZeroDiscipline, requires = all_of([
    signers(required = ["math-researcher"]),
    oracles_complete(["tests/oracles/sinh-signed-zero.md"]),
]))]
#[immune(SignedZeroDiscipline, witness = proptest! {
    #[test]
    fn sinh_returns_correct_sign(x in -100f64..100f64) {
        let result = sinh(x);
        prop_assert_eq!(result.signum(), x.signum());
    }
})]
pub fn sinh(x: f64) -> f64 { /* ... */ }
```

The proptest witness produces `EvidenceKind::Behavioral / WitnessTier::Execution`
when the test passes. The substrate-witness produces
`EvidenceKind::SubstrateState / WitnessTier::Execution` when the sidecar is current.
Both appear in the audit output; both must satisfy their respective predicates for
full coverage.

A CI gate configured with `--min-tier execution` requires both. A team can also
require specific `EvidenceKind` coverage — ensuring that behavioral evidence and
substrate-state evidence are both present — for the highest-discipline functions.

---

## Adoption Gradient

Antigen's adoption gradient (ADR-009) applies to discipline-witnesses:

**Floor (any team can reach immediately)**: Declare the antigen, add the `requires =`
predicate, run `scaffold-anchor` with a reasoning string. One CLI invocation;
one JSON file; immediate CI feedback on drift.

**Mid (teams with formal review processes)**: Add `fresh_within_days(N)` for
periodic re-review. Add multiple required signers. Align predicate with team's
existing review workflows.

**Advanced (security-critical, external consumers)**: Delta-attestation for
incremental change tracking. Predicate inheritance with explicit weakening
declarations. Eventual migration to DSSE+Sigstore as ecosystem matures.

Each phase delivers value independently. A team that never leaves the floor tier
still gets: machine-readable record of who signed this, against which fingerprint,
with what reasoning; automatic CI failure on code drift.

---

## The Architecture Behind the Primitive

The design of discipline-witnesses was shaped by two biological observations that
serve as architectural anchors — not metaphor for its own sake, but architecture
derived from biological mechanism.

**Observation 1 (EvidenceKind ceilings)**: in immunology, three evidence mechanisms
exist: germline-encoded (hardwired, immutable), epigenetically-trained (triggered,
fades), and somatically-adapted (currency-dependent, drifts). These correspond
to antigen's TypeSystemProof / Behavioral / SubstrateState EvidenceKinds. The
Execution-tier ceiling for SubstrateState is not an arbitrary cap — it follows from
the biology: adaptive-substrate evidence is inherently currency-dependent, requiring
ongoing verification. It cannot achieve the certainty of germline encoding (phantom-
type proofs). A future argument that "attested discipline should reach FormalProof"
fails at this biological ground: the evidence kind is wrong for that tier.

**Observation 2 (Role-distinction)**: immunology distinguishes the recognition
machinery (B-cell receptor, specificity-encoded) from the evidence of past activation
(secreted antibody, circulation record). Both exist; they must be in separate
structures because they have different lifetimes, different update semantics, and
different visibility properties. Antigen inherits this: the `requires =` predicate
encodes recognition specificity (what does immunity require?); the `.attest/` sidecar
encodes evidence of attestation (who verified this, when, against what). These are
not interchangeable; collapsing them loses the structural clarity that makes each
useful independently.

These observations aren't window-dressing. They are architectural constraints with
design implications. The tier ceiling is enforced. The role-distinction is enforced.
Teams that understand the biology understand why these design decisions are not
negotiable conventions but structural requirements.

---

*For implementation details and the full ADR, see
[`docs/expedition/`](expedition/) and the ADR-019 decision substrate at
[`campsites/antigen-discipline-witnesses/.../pathmaker/notebooks/adr-019-substrate-witness-predicate-family-draft.md`](expedition/drafts/discipline-witnesses-v3.md).*
