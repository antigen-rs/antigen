# Discipline Witnesses — Draft v3 (rolling current canonical)

> **Status**: rolling current draft. v1 and v2 stay as substrate at
> [discipline-witnesses-v1.md](discipline-witnesses-v1.md) and
> [discipline-witnesses-v2.md](discipline-witnesses-v2.md). v3 absorbs
> all 7 interim captures and rolls forward in place with in-session
> refinements. Captures preserve the substrate trail. Observational
> tone ("here's the consolidated current shape"), not directive
> ("here's what ships").

> **Provenance**:
> - **Original v3** (folded 5 captures): adversarial self-attack, naturalist self-pass, aristotle self-pass, aristotle team-pass, academic-research
> - **Amended 2026-05-19**: team-scout (tolerance-ratification as v0.1-rc structural addition; 4 new dimension candidates; cross-domain insights); team-adversarial (10 attacks → 6 land + refinements T1-R through T9-R)
>
> See [INDEX.md](../INDEX.md) captures/ for the substrate trail.

---

## What's in v3 right now

**Load-bearing structural additions (since v2)**:
- **`EvidenceKind` enum** as first-class audit-output axis parallel to `WitnessTier × AuditHint` (aristotle F8)
- **`scope` field** on antigen declarations (`site | file | package | workspace`) (aristotle F3)
- **`Signer.basis`** field discriminating fresh-attestation from `DeltaFrom(fingerprint)` carry-forward (aristotle F5)
- **`signature_strength`** field on audit output (`git-trust | crypto-signed`) (adversarial R-A5)
- **`evidence_provenance`** field on antigen declarations (`observed(n) | predicted(rationale)`) — encodes ADR-006's three-instances threshold as structured data (scout S2)
- **Tolerance-ratification mechanism** — `#[antigen_tolerance(X, sidecar = true)]` opt-in plugs the current tier-honesty gap in ADR-011; schema **isomorphic to immunity sidecars** (scout S1 — biggest v0.1-rc addition since v2)

**New CLI primitives**:
- **`cargo antigen attest delta`** with anti-laundering safeguards: chain-depth cap + cumulative-fingerprint tracking + required non-empty rationale per delta (adversarial T2-R)
- **`cargo antigen tolerate scaffold/sign/check/list`** — parallel CLI family for tolerance-ratification (scout S1 implication)

**Structural commitments made explicit**:
- **Ratchet-asymmetry property** — "audit reports lower bound; promotions require evidence; downgrades automatic when evidence falters" (aristotle F5 + R-Ar1)
- **Audit-of-audit recursion is bounded** by EvidenceKind monotonic decrease + inherited adoption-trust (aristotle F6)
- **Witness-provider-crate trust boundary on critical path** for v0.2+ — v0.1 sealed-leaf-set is structurally required; v0.2+ ADR MUST specify ACTUAL enforcement mechanism (WASM sandbox / `no_std`+restricted-deps / subprocess isolation), not just contract spec (aristotle F7 + adversarial T1-R)
- **Discipline-level vs machinery-level unification asymmetry** documented + **enforced via guardrails**: in-code comment block at unification points + adversarial schema-validation precision test (aristotle F1 + adversarial T5-R)
- **Closed-set tool boundary 4-point bright-line**: (1) binary named in leaf source, (2) own release process, (3) doesn't execute user-supplied code, (4) invocation args fixed except for declared substrate-parameters. Excludes `cargo build`/`cargo run`/`curl` with external URLs (adversarial T4-R; replaces aristotle F4's vague "ecosystem tools")

**Refinements absorbed**:
- **Doc-level ratification absorbed** into extended `ratified_doc` leaf with optional sibling JSON (aristotle F2 replaces R-Ar4)
- **Combinator-specific biology rhymes** (naturalist R-N2)
- **Vaccination-booster rhyme** for manual re-sign (naturalist R-N4 + framing-B lean)
- **Memory-cells rhyme** for sidecar persistence (naturalist R-N7)
- **`signers(against = "current"|"any")`** parameter (adversarial R-A3)
- **CLI sign discipline** — `attest sign` is ONLY write-path for `signed_against_fingerprint` (adversarial R-A4)
- **Schema rejects zero-leaf compositions** at parse-time (adversarial R-A6)
- **Per-consumer ratification for cross-crate** (adversarial R-A7)
- **Macro-invocation site as input layer** (adversarial R-A8)
- **Hint renamed**: `discipline-substrate-validated-and-current` → `discipline-predicate-passed-substrate-current` (adversarial T6-R; overclaim removal)
- **Reviewer-not-committer workflow documented**: v0.1 via `signed_trailer` leaf; v0.4+ via crypto-signing (adversarial T9-R)

**v0.2+ amendments named** (structurally guaranteed; not in v0.1):
- `signers(required_threshold = K, candidates = [...])` — TUF k-of-n threshold signatures with CAP-theorem framing (academic absorb + scout S4)
- `required_role` — CODEOWNERS-style role resolution (T2)
- cargo-vet imports pattern for cross-crate ratification (academic absorb)
- **Witness-provider-crate ADR** — with EXPLICIT enforcement-mechanism specification, not just contract spec (aristotle F7 + adversarial T1-R)
- PASETO version-discipline for combinator grammar (academic absorb)
- **`--prioritized` flag** for `attest list --pending` to mitigate annotation fatigue (scout S4 software-ergonomics literature)
- **Lifetime on discipline claims**: `permanent | temporal(cadence) | transitional(condition)` (scout S2)

**v0.3+/v0.4+ amendments named**:
- SARIF export adapter (v0.3+)
- **DSSE envelope** for `Signer.signature` activation; PAE non-obvious (v0.4+)
- **Sigstore identity-bound signatures** as the `Signer.signature` activation path with OIDC + transparency log (v0.4+; notary-institution 800-year design arc; scout S4)

**Left for team (open questions)**:
- **T1**: macro syntax `witness = adjacent_sidecar(...)` vs `requires = ...` parallel parameter
- **T2**: CODEOWNERS interop UX shape for v0.2
- **T3** (aristotle F9): does `discipline_doc` need separation into `canonical_reference` + `review_grounded`?
- **T4** (aristotle F8 + adversarial frontier): compound evidence overclaim surface — how to report behavioral test + substrate signatures on same antigen-site tier-honestly?
- **T5** (aristotle F7 + adversarial T1-R): leaf-contract enforcement mechanism specification (WASM/no_std/subprocess) for v0.2+ ADR
- **T6** (scout S2): substrate-grep on ADR-008 Am 1 — is severity-class already first-class in scan output? Action depends on result
- **T7** (adversarial frontier FA-2): fingerprint-scheme evolution causing false-stale across antigen version upgrades — needs cross-version migration story
- **T8** (adversarial frontier FA-5): `descended_from` predicate inheritance — can a consumer weaken the declaring crate's predicate and still claim `descended_from`?
- **(naturalist call)**: framing-A clean-break vs framing-B expanded-unit-of-analysis for biology. v3 leans framing-B; defers final naming

**Long-arc / design-preserve** (not v0.1; sidecar pattern should explicitly not rule out applying to these):
- **Fingerprint-ratification sidecars** — antibody-specificity-validation biology rhyme (scout S1)
- **Lineage-validation sidecars** for `#[descended_from]` — inheritance relationships are discipline claims (scout S1)
- Tolerance-ratification fully integrated with cross-crate semantics

**Unchanged from v2**:
- Problem statement and substrate-witness reframe
- Three-piece shape (predicate language + Ratification schema + CLI) — now extended to cover tolerance via isomorphic schema
- Per-antigen-per-file granularity with `.attest/` subfolder
- Generated-code position (input-level discipline; "accept tradeoff" is expected default)
- **ONE-ADR position** — reinforced by cargo-vet landscape precedent (distributed, in-tree, git-trust, declarative, custom-extensible-with-discipline at scale)

---

## Problem

Antigen's witness vocabulary currently covers code-side substrate only:
`test_fn`, `proptest!`, `clippy::lint`, `kani::proof`, `prusti::ensures`,
phantom-type proofs. Each verifies *the code itself* by running tests,
checking proofs, or pattern-matching the AST.

This is a structural gap for **discipline failure-classes** — failure-classes
where the antigen is presented at a code site but verification depends on
*something other than the code*: a ratified discipline doc, a team-member
sign-off, an oracle test fixture being marked complete, a PR review against
a canonical reference, etc.

The reframe (recognition-not-design):

> Witnesses currently check *code-side* substrate. They should be extensible
> to check **substrate other than the code being audited** as well —
> ratified docs, sign-off records, signed git trailers, oracle-completion
> markers — *as long as the audit remains tier-honest about what was
> actually verified*.

Note: v2 framed the axis as "code vs non-code substrate." v3 sharpens to
"this-code's-substrate vs other substrate" (per aristotle F1) because
cross-crate witnesses also evaluate against other substrate (the dep's
source code, which IS code, but not THIS code). The discipline-level
unification holds; the machinery-level unification does NOT.

Once witnesses can check substrate other than this code, "discipline
antigens" stop being a special category and become *ordinary antigens
whose witness predicates evaluate against substrate not under the audit's
direct mechanical reach*.

**Additional problem surfaced by team-scout (S1)**: `#[antigen_tolerance(...)]`
(ADR-011) is currently vibes-grade — inline rationale, no structured
attestation of who approved the tolerance, when, against what review.
The substrate-witness mechanism plugs this exactly: the tolerance schema
is **isomorphic** to the immunity sidecar schema. Shipping one schema
covers both use cases. Tolerance-without-attestation is a tier-honesty
gap in antigen TODAY; v3 addresses it via the same primitive.

---

## Shape proposed

Three coupled pieces shipping together in v0.1 (per ADR-007 anti-YAGNI;
each alone fails). The substrate-witness primitive covers BOTH immunity
claims AND tolerance ratifications (per scout S1):

### 1. Substrate-witness predicate family

A small declarative predicate language over typed substrate. Closed
combinator grammar, small starting set of leaf primitives the audit
evaluates against on-disk substrate.

**Leaf primitives (v0.1 shipping set)**:

| Primitive | Checks |
|---|---|
| `ratified_doc(path?, min_version?, anchor?, sibling_json?)` | Doc exists; frontmatter version ≥ min; anchor present; optional adjacent JSON for doc-level ratification (absorbs F2) |
| `signers(required, roles?, against?, signature_allow?, signature_prefer?)` | Sidecar `signers[]` includes required names (optionally with roles); `against = "current"\|"any"` (default `"current"`); `signature_allow` = categorical allow-set of `SignatureStrength` variants each signer must carry (ABO/Rh biology, B6); `signature_prefer` = preferred tier (hint fires if below, but does not fail predicate) |
| `signed_trailer(key, role?, count?)` | Git log on touching commits has matching trailer; uses `git interpret-trailers` for canonical parsing |
| `oracles_complete(files)` | Listed oracle files exist with `status: complete` markers |
| `fresh_within_days(n)` | Most recent current-fingerprint signer's structured `.date` field within N days (NFA-21: stale-fingerprint signer entries excluded from freshness computation; filesystem mtime is NOT used) |

**Combinators (full set; closed)**: `all_of([...])`, `any_of([...])`, `not(...)`.
Schema rejects zero-leaf compositions at parse-time.

**Biology rhymes per combinator** (naturalist R-N2):
- `all_of` ↔ co-stimulation (all signals required; missing → anergy)
- `any_of` ↔ redundant pathways (classical vs alternative complement)
- `not` ↔ inhibitory checkpoints (CTLA-4, PD-1, Tregs)

**Predicate-language ceiling** (refined per aristotle F4 + adversarial T4-R):
predicates must be (a) mechanically evaluatable by the audit, (b)
terminating, (c) **verifiable without invoking author-defined code**.
Closed-set ecosystem tool invocation is permitted under tier-honesty,
subject to the **4-point bright-line rule**:

1. **Binary named in leaf source** — no runtime tool-name resolution
2. **Has own release process** — package-managed, versioned, tagged
3. **Does NOT execute user-supplied code** — excludes `cargo build`/`cargo run` (build scripts; macro expansion), `bash`/`sh` of user-files, `curl` with external URLs
4. **Invocation args fixed in leaf source** except for declared substrate-parameters

`signed_trailer` invoking `git interpret-trailers` satisfies all four:
git is named, versioned, doesn't execute trailer content, args fixed.
A hypothetical `cargo_check_passes` leaf invoking `cargo check` does
NOT satisfy point 3 (cargo runs build scripts) and is REJECTED at
leaf-design review.

**Macro shape** (immunity claim):

```rust
#[immune(SignedZeroDiscipline, requires = all_of([
    signers(required = ["alice", "bob"], against = "current"),
    oracles_complete(["docs/oracles/sinh-domain.md"]),
    ratified_doc(min_version = "1.0"),
    fresh_within_days(180),
]))]
pub fn sinh(x: f64) -> f64 { ... }
```

**Macro shape** (tolerance ratification — NEW, scout S1):

```rust
#[antigen_tolerance(SignedZeroDiscipline, sidecar = true, requires = all_of([
    signers(required = ["alice"], against = "current"),
    ratified_doc(path = "docs/exceptions/legacy-sinh.md", min_version = "1.0"),
    fresh_within_days(90),  // shorter than immunity — tolerance is more accountable
]))]
pub fn legacy_sinh(x: f64) -> f64 { ... }
```

The schema is isomorphic. The same `Ratification` struct serves both;
the type discriminator is `RatificationKind::{Immunity, Tolerance}` —
small addition; reuses everything else.

**v0.2 amendments**:
- `signers(required_threshold = K, candidates = [...])` — TUF k-of-n
  (academic absorb #6 + scout S4 CAP-theorem framing)
- `required_role` parameter for CODEOWNERS-style resolution (T2)

**Team-level T1 (carried)**: macro syntax `witness = adjacent_sidecar(...)`
vs `requires = ...` parallel parameter.

### 2. Ratification schema + JSON sidecars adjacent to source

Schema is a serde-derived Rust type — single source of truth for audit
(reads), CLI (writes), and editor support.

**Location convention**: per-antigen-per-file under `.attest/` subfolder.
Granularity follows antigen presentation scope (aristotle F3 + R-A9).

**Schema sketch** (v0.1, supporting both immunity + tolerance):

```rust
// antigen-attestation crate (separate per v2 R4 — precedent:
// antigen-fingerprint already factored out)
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Ratification {
    pub schema_version: SchemaVersion,
    pub kind: RatificationKind,        // NEW (scout S1): Immunity | Tolerance
    pub antigen: AntigenIdentifier,
    pub source_file: PathBuf,
    pub items: Vec<ItemRatification>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum RatificationKind {
    Immunity,                          // for #[immune] claims
    Tolerance,                         // for #[antigen_tolerance] claims (scout S1)
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ItemRatification {
    pub item_path: String,
    pub current_fingerprint: String,   // reuses antigen_fingerprint::Fingerprint
    pub doc_ref: Option<DocRef>,
    pub signers: Vec<Signer>,
    pub oracles: Vec<OracleRef>,
    pub fresh_through: Option<NaiveDate>,
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Signer {
    pub name: String,
    pub role: Option<String>,
    pub date: NaiveDate,
    pub signed_against_fingerprint: String,
    pub basis: SignerBasis,            // Fresh | DeltaFrom(...)
    pub signature: Option<Signature>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum SignerBasis {
    Fresh,
    DeltaFrom {
        prior_fingerprint: String,
        cumulative_root_fingerprint: String,  // (T2-R) — what was the LAST
                                              // Fresh-basis signature for this
                                              // signer; chain-tracking prevents
                                              // laundering
        chain_depth: u32,                     // (T2-R) — counts deltas since
                                              // last Fresh; hard cap enforced
        rationale: String,                    // (T2-R) — non-empty required;
                                              // schema rejects empty strings
    },
}
```

**Delta-chain anti-laundering safeguards** (adversarial T2-R):
- **Hard cap**: `chain_depth <= 3` (configurable in workspace TOML;
  default 3). Beyond this, signer MUST do a Fresh re-attestation.
- **Cumulative tracking**: `cumulative_root_fingerprint` always references
  the LAST `Fresh` basis for this signer, not the immediate previous
  delta. Audit verifies the cumulative-diff (root → current) hasn't
  grown beyond a configurable threshold (default: 200 lines or 25%
  of item, whichever is smaller).
- **Required non-empty rationale**: schema-enforced. Empty or whitespace-
  only rationale is schema-rejected at sidecar-parse time.
- **Audit hint when caps approach**: `discipline-substrate-delta-chain-near-cap`
  warns when chain_depth >= cap-1 (e.g., 2 of 3); informs the signer
  next delta won't be acceptable.

These three together close T2's attack surface: a 4-step laundering
chain is blocked by chain-depth-cap; a slow drift across many small
deltas is caught by cumulative-diff threshold; an unjustified
rubber-stamp delta is blocked by required-rationale.

**New on antigen declaration** (per aristotle F3 + F9 + scout S2):

```rust
#[antigen(
    SignedZeroDiscipline,
    scope = "site",                    // (F3): site | file | package | workspace
    discipline_doc = "docs/sinh.md",   // existing field
    evidence_provenance = observed(7), // NEW (scout S2): observed(n) | predicted(rationale)
                                        // encodes ADR-006's three-instances threshold
                                        // as structured data; audit can verify N ≥ 3
                                        // for stdlib promotion eligibility
    canonical_reference = "...",       // T3 (open): does this need separation
                                        // from discipline_doc?
)]
pub struct SignedZeroDiscipline;
```

### 3. `cargo antigen attest` + `cargo antigen tolerate` CLI families

**Immunity attestation** (existing surface, extended):

```sh
cargo antigen attest scaffold --file src/numerics.rs \
    --antigen SignedZeroDiscipline --item sinh

cargo antigen attest sign --file src/numerics.rs \
    --antigen SignedZeroDiscipline --item sinh --role math-researcher
# THE ONLY write-path for signed_against_fingerprint (per R-A4); only
# signer's own entry; basis = Fresh; identity from git config; refuses
# if unset (no anonymous bypass).

cargo antigen attest delta --file src/numerics.rs \
    --antigen SignedZeroDiscipline --item sinh \
    --from <old-fingerprint> --as math-researcher \
    --rationale "Reviewed diff; sinh body unchanged in invariant-preserving way"
# (T2-R) records signer entry with basis = DeltaFrom{...}.
# Refuses if: rationale empty; chain_depth would exceed cap;
# cumulative-diff threshold would be exceeded.

cargo antigen attest oracle complete ...
cargo antigen attest check [--file ... | --all]
cargo antigen attest list [--stale | --pending | --signed-by alice | --delta-chain-near-cap]
cargo antigen attest move --from <src> --to <dst>
cargo antigen attest migrate --from v1 --to v2
cargo antigen attest gc [--dry-run]    # default dry-run; --commit required
```

**Tolerance ratification** (NEW per scout S1):

```sh
cargo antigen tolerate scaffold --file src/legacy.rs \
    --antigen SignedZeroDiscipline --item legacy_sinh
# creates src/legacy.attest/SignedZeroDiscipline.json with kind = Tolerance

cargo antigen tolerate sign --file src/legacy.rs \
    --antigen SignedZeroDiscipline --item legacy_sinh --role math-researcher
# same discipline as attest sign; same Fresh/Delta semantics; same anti-laundering

cargo antigen tolerate list [--expiring-soon | --signed-by alice]
# tolerance-specific queries
```

**Reviewer-not-committer workflow** (adversarial T9-R):

Standard PR workflows where alice reviews but bob commits aren't covered
by `attest sign` directly (sign uses committer's git identity). Two paths:

- **v0.1**: use `signed_trailer` predicate leaf. Alice puts
  `Discipline-Verified-By: alice@example.com` in commit trailer; bob
  commits the code (his git identity); the `signed_trailer` predicate
  satisfies on alice's trailer presence. The sidecar entry can be added
  by either party (bob commits; predicate verifies alice's trailer
  separately).
- **v0.4+**: crypto-signing via DSSE envelope. Alice's signature on the
  sidecar entry is verifiable regardless of who committed the file.
  `Signer.signature` populated with DSSE-PAE-encoded signature; audit
  verifies cryptographically rather than via committer-identity.

**No `--on-behalf-of` flag in v0.1** (T9-R explicitly): such a flag
would reintroduce the bypass risk (anyone could sign as anyone). The
v0.1 workaround forces the cryptographic identity (git trailer = signed
commit) or the git-trust ceremony (committer signs as themselves).

**Witness-provider crates explicitly deferred to v0.2+ ADR** (per
aristotle F7 + adversarial T1-R):

v0.1 ships SEALED leaf set. The v0.2+ leaf-provider ADR MUST specify an
ACTUAL enforcement mechanism, not just contract documentation:

- **Option A: WASM sandboxing** — robust isolation, expensive at audit
  invocation (cold-start + per-leaf evaluation overhead). The "right
  answer" for security-critical adoption.
- **Option B: `no_std` + restricted-dep-check at build time** —
  pre-screens leaf code; lighter weight at audit time. Doesn't catch
  all attacks (build-time checks can be bypassed) but raises the bar.
- **Option C: subprocess isolation with timeout + memory cap** —
  runtime, medium cost, OS-level isolation. Compromise between A and B.

ADR-019 names the v0.2+ leaf-provider ADR on the critical path with
explicit enforcement-mechanism-specification as scope. **Documentation-
trust alone is not sufficient** (per T1-R) — adversarial review confirmed
that leaf-contract documentation without enforcement does not prevent
non-compliant implementations.

---

## Tier-honesty mapping — three-axis (sharpened)

Per aristotle F8, the honest model has THREE axes:

| Axis | Values | Source |
|---|---|---|
| `WitnessTier` | `None | Reachability | Execution | FormalProof` | existing (W7 enum) |
| `AuditHint` | per-case verification-work disambiguation | existing (ADR-005 Am 3) |
| `EvidenceKind` | `None | TypeSystemProof | Behavioral | SubstrateState` | NEW (F8) |

**Per-EvidenceKind ceiling**:
- `None` → reaches `None` tier (no substrate consulted; used for vibes-grade tolerance states and error states where no evaluation occurred)
- `TypeSystemProof` → reaches `FormalProof`
- `Behavioral` → reaches `Execution` (after harness invocation; Reachability in v0.1)
- `SubstrateState` → reaches `Execution` (when predicate passes + currency holds); cannot reach `FormalProof`

**Substrate-witness state mapping** (v0.1, immunity claims):

| State | EvidenceKind | WitnessTier | AuditHint | Signature strength |
|---|---|---|---|---|
| No `.attest/`, or no sidecar for this antigen | SubstrateState | None | `discipline-sidecar-missing` | n/a |
| Sidecar exists but schema-invalid | SubstrateState | None | `discipline-sidecar-schema-invalid` | n/a |
| Sidecar exists, predicate fails | SubstrateState | None | `discipline-predicate-failed` (per-leaf details) | n/a |
| Predicate passes, ≥1 signature stale AND `against = "current"` | SubstrateState | Reachability | `discipline-substrate-stale` | git-trust |
| Predicate passes, all current, some delta-chain near cap | SubstrateState | Execution | `discipline-substrate-delta-chain-near-cap` | git-trust |
| Predicate passes, all current, some basis = DeltaFrom (within caps) | SubstrateState | Execution | `discipline-predicate-passed-via-delta-chain` | git-trust |
| Predicate passes, all current, all basis = Fresh | SubstrateState | Execution | `discipline-predicate-passed-substrate-current` | git-trust |

**Note**: hint name `discipline-predicate-passed-substrate-current`
replaces v2's `discipline-substrate-validated-and-current` per
adversarial T6-R. The earlier name overclaimed — what was validated is
the predicate against the sidecar, not the discipline itself. The new
name is precise: the predicate passed; the substrate is current; what
that means for "discipline" is consumer judgment.

**Substrate-witness state mapping for tolerance claims** (v0.1, NEW per scout S1):

| State | EvidenceKind | WitnessTier | AuditHint |
|---|---|---|---|
| `#[antigen_tolerance(X)]` with no `sidecar = true` opt-in | **`None`** | None | `tolerance-vibes-grade` |
| `#[antigen_tolerance(X, sidecar = true)]` but no sidecar exists | SubstrateState | None | `tolerance-sidecar-missing` |
| Tolerance sidecar exists, predicate fails | SubstrateState | None | `tolerance-predicate-failed` |
| Tolerance sidecar exists, predicate passes, all signers current Fresh | SubstrateState | Execution | `tolerance-predicate-passed-substrate-current` |

**Key tier-honesty win** (scout S1 + adversarial-confirmed): the
`tolerance-vibes-grade` hint with `EvidenceKind::None` makes
**unattested tolerance visible in audit output**. Today, antigen has no
way to distinguish "this code is intentionally non-compliant with full
team buy-in" from "developer added `#[antigen_tolerance]` to silence
the audit." With this addition, the audit reports the distinction; CI
gates can refuse `None`-tier tolerance claims while accepting
`Execution`-tier ones.

**Ratchet-asymmetry property** (aristotle F5 + R-Ar1 named explicitly):
audit reports **lower bound** of verification work; never upper bound.
Promotions require evidence; downgrades are automatic when evidence
falters (fingerprint drift, expiry, signer removal, chain-depth cap
hit). This is what makes tier-honesty a discipline rather than a
preference.

**Audit-of-audit bounded-recursion** (aristotle F6): the audit's own
implementation is verifiable in a bounded chain via EvidenceKind
monotonic decrease + inherited adoption-trust of cargo-test + peer
review. Finite, not infinite.

**Connection to ADR-005 Am 3 OQ-1**: substrate-witnesses are
audit-internal; don't promote OQ-1's "outside-audit instances" count.
The EvidenceKind axis (F8) IS a generalization that potentially
applies to non-audit contexts but doesn't promote OQ-1 by itself.

---

## What doesn't unify (carried + enforcement-strengthened)

Substrate-witnesses and cross-crate witnesses share **discipline**-level
unification (both follow tier-honesty; both have SubstrateState evidence
kind; both cap at Execution; both use audit_hint for disambiguation).

They do NOT share **machinery**-level unification (different parsers;
different attack surfaces; different recovery semantics).

**Guardrails to prevent future drift** (adversarial T5-R):

The verbal documentation alone (this section + aristotle F1 capture)
isn't enough — a maintainer in a refactor could read "discipline
unification" and try to share parser code, with silent precision-
degradation passing all existing tests. v3 adds:

1. **In-code comment block** at every unification point in `audit.rs`
   and `scan.rs` naming the discipline-vs-machinery asymmetry, citing
   ADR-019 + aristotle F1 + adversarial T5-R, with explicit "DO NOT
   share parser code between sidecar-JSON and Rust-AST paths" warning.
2. **Adversarial schema-validation precision test** in
   `antigen/tests/atk_a3_unification_guardrail.rs`: synthetic test
   that constructs a near-collision case where shared parser code would
   silently mis-classify a JSON-formed-payload as Rust-AST-shaped (or
   vice versa); verifies the audit correctly distinguishes. If a future
   maintainer wires up shared parsing, this test FAILS, preventing the
   silent drift.

The unification is at the principle layer; implementation stays separate
per substrate type; the test ENFORCES the boundary.

---

## Posture: opinionated-with-flexibility (unchanged)

ADR-002 (compose-don't-compete) applied here:

**Closed** (no negotiation): schema is Rust type; combinator grammar closed;
tier-honesty mandatory; sidecar substrate-currency is JSON-against-schema;
CLI scaffolds canonical shape; leaf set sealed at use-site.

**Open** (integration surface): `extensions` BTreeMap slot; witness-provider
crates at adoption-time per v0.2+ ADR (with enforcement-mechanism spec);
format adapter trait deferred; `Signer.signature` field reserved (v0.4+
via DSSE); `required_role` opt-in (v0.2); sidecar location configurable.

---

## Biology grounding — sharpened (naturalist refinements folded)

### Core rhymes (load-bearing, unchanged from prior v3)

MHC presentation → typed sidecar; T-cell+B-cell co-stimulation → `all_of`;
redundant pathways → `any_of`; inhibitory checkpoints → `not`; affinity
maturation → fingerprint-pinned signatures (structural); vaccination
boosters → manual re-sign (clinical-infrastructure layer); per-cell
processing → code-locality; clonal selection + peripheral tolerance →
trust extension layering; memory cells → sidecar persistence; innate vs
adaptive immunity at machinery level → EvidenceKind axis.

### NEW rhyme reserved for long-arc (scout S1)

**Antibody specificity validation → fingerprint-ratification sidecars**:
in immunology, an antibody is validated not just by "does it bind?"
but "does it bind ONLY what we want?" (no cross-reactivity). Fingerprint
specificity validation is the same — the fingerprint must match the
right sites and not the wrong ones. This is a long-arc design-preserve;
NOT v0.1. But the sidecar pattern should explicitly not rule out
applying to fingerprint declarations when the team is ready.

### Productive break-point — leaning framing-B (naturalist call still open)

v2 offered framings A (clean break) and B (expanded unit-of-analysis).
Single-instance naturalist + cross-domain scout work both support
framing-B: vaccination boosters rhyme with manual re-sign; immunization
registries are literal central registries of substrate attestations;
school vaccination requirements ARE substrate-witnesses in deployment;
notary institutions (scout S4) provide an 800-year design arc that
predicts the substrate-witness escalation path (git-trust → OIDC +
transparency log).

v3 leans framing-B; team-naturalist's final call still open.

---

## What's load-bearing — strengthened (now 11 items)

### 1. Audit reads non-`.rs` substrate (carried)
### 2. Witness expressions need parse + serialize + replay (carried)
### 3. `discipline_doc` field on antigen declaration (carried; T3 open)
### 4. Fingerprint reuse (carried)
### 5. CLI scaffolding (elevated from "tooling" to "load-bearing")
### 6. EvidenceKind axis (aristotle F8)
### 7. Scope as first-class field on antigen declaration (aristotle F3)
### 8. Witness-provider-crate trust boundary specification on critical path with ENFORCEMENT MECHANISM (aristotle F7 + adversarial T1-R)
### 9. Ratchet-asymmetry property explicitly named (aristotle F5 + R-Ar1)
### 10. Tolerance-ratification mechanism (NEW — scout S1)

**What breaks without it**: antigen has a tier-honesty gap TODAY
(`#[antigen_tolerance(...)]` is vibes-grade). The substrate-witness
primitive plugs it exactly. Without this addition, v0.1-rc ships with
a known tier-honesty gap that the substrate-witness work could have
closed at zero additional schema cost (it's isomorphic). Adding the
isomorphic schema extension + parallel CLI family is the
minimum-additional-complexity move with maximum tier-honesty payoff.

### 11. Anti-laundering safeguards on delta-attestation (NEW — adversarial T2-R)

**What breaks without it**: `attest delta` becomes a laundering escape
hatch. A 4-step chain of individually-small carry-forwards smuggles a
substantive semantic change through; the audit reports Execution-tier
throughout despite no signer having actually reviewed the cumulative
change. This is exactly the failure mode tier-honesty exists to
prevent. The three safeguards (chain-depth cap; cumulative-fingerprint
tracking; required non-empty rationale) close the surface together;
removing any one re-opens it.

---

## Open questions — sorted

### Team-needs (genuinely open)

**T1**: macro syntax A vs B (carried; team call)
**T2**: CODEOWNERS interop UX shape for v0.2
**T3**: `discipline_doc` dual-jobs separation (aristotle F9; needs adoption substrate)
**T4**: compound evidence (behavioral test + substrate signatures on same site) overclaim surface — how to report tier-honestly?
**T5**: leaf-contract ENFORCEMENT MECHANISM specification (WASM/no_std-build-check/subprocess) for v0.2+ ADR; this is the scope of that ADR's content
**T6** (NEW — scout S2): substrate-grep on ADR-008 Am 1 — is severity-class already first-class in scan output? Action depends on result (if yes: just expose in audit; if no: new dimension worth adding)
**T7** (NEW — adversarial frontier FA-2): fingerprint-scheme evolution across antigen version upgrades → false-stale; needs cross-version migration story
**T8** (NEW — adversarial frontier FA-5): `descended_from` predicate inheritance — can consumer weaken declaring crate's predicate and still claim descended_from? Tier-honesty implications

**(naturalist call)**: framing-A clean-break vs framing-B expanded-unit-of-analysis

### My-take-embedded (resolved by single-instance + team work)

All v2 my-takes (R1-R8) survive with extensions; see v2 for details.

---

## Where this work fits in the broader antigen story

**Multi-component immunity framing** (scout S5): discipline-witnesses
are **Component 1.5 — attestation-mediated-judgment**, the bridge
between C1 (developer-judgment-as-immunity) and C2 (passive
scan/tools-as-immunity) in the multi-component-immunity taxonomy. This
missing slot was identified by scout pass and is worth naming
explicitly in `multi-component-immunity.md` when v3 graduates to ADR.

**Scan/audit asymmetry flag** (scout S6): discipline-witnesses extend
the audit's substrate reading (audit now walks JSON sidecars + git log
in addition to Rust source). A parallel structural question exists for
the scan: should the scan also read non-Rust substrate (Cargo.toml for
optional-dependency-implicit-feature antigens per stdlib-seed-antigens.md
Antigen 10; build.rs; etc.)? v3 does NOT design this; it explicitly
flags the structural parallel so it doesn't get lost when the team
considers what's in v0.1-rc vs later.

---

## Readiness state — full-team launch ready

All 4 team passes complete or in-substrate:
- **aristotle team-pass**: Phase 1-8 on load-bearing principles; 8 F-findings; ratify/replace mapping to self-pass
- **adversarial team-pass**: 10 attacks; 6 land + 6 refinements (T1-R through T9-R) absorbed
- **naturalist work**: single-instance self-pass with 7 refinements (R-N1 through R-N7); team-naturalist next-pass scope flagged
- **academic-research**: 14-system landscape; cargo-vet closest analog; 9-item absorb / 6-item don't-absorb
- **scout**: F2 absorption-pattern reach (tolerance-ratification key finding); F3 4 new dimensions; F8 reach; 4 cross-domain insights

**Full-team next-pass substrate**:
- All agents: v3 (this doc) + INDEX
- team-aristotle: T3 frontier (discipline_doc dual-jobs); EvidenceKind enum closure; doc-level absorption with adoption substrate
- team-adversarial: attack v3 directly; T4/T5/T7/T8 open attack surfaces; 6 frontier-attacks (FA-1...FA-6) from prior pass for context
- team-naturalist: framing-A vs framing-B final call; F3 scope biology; F8 evidence-kind biology validation
- team-academic-researcher: cargo-vet adoption-maturity primary sourcing; leaf-contract specs in other plugin systems (WASM/Lua/NPM); compound-evidence reporting in adjacent literatures
- team-scout: long-arc design-preserves (fingerprint-ratification, lineage-validation); scan-side asymmetry; lifetime-on-claims

---

## What this is NOT

- **Not a replacement** for `#[immune(X, witness = test_fn)]`: code-side witnesses remain; substrate-witnesses are additive.
- **Not a special category**: discipline-antigens are ordinary antigens with non-this-code-substrate witnesses.
- **Not vibes-grade attestation**: signing requires real substrate (git config, fingerprint pin, structured sidecar entry, required-rationale on deltas).
- **Not a doc-management system**: sidecar carries compliance state, not doc content. Doc-level ratification absorbed into extended `ratified_doc` leaf via optional sibling JSON.
- **Not a CI gate by default**: `cargo antigen audit` reports state; CI gates choose to fail on hints (including new `tolerance-vibes-grade` hint).
- **Not three ADRs**: ONE ADR (substrate-witness predicate family) introducing predicate language + schema + CLI as one primitive covering BOTH immunity and tolerance.
- **Not an outside-audit instance of ADR-005 Am 3 OQ-1**: substrate-witnesses are audit-internal.
- **Not a generator-output coverage system**: generated `.rs` files out of scope; discipline antigens for generator output belong at input layer.
- **Not machinery-shared with cross-crate witnesses**: unification is at discipline level (tier-honesty); machinery stays separate; **boundary enforced via in-code comments + adversarial precision test** (T5-R).
- **Not a leaf-plugin system in v0.1**: sealed leaf set; v0.2+ ADR specifies leaf-contract + enforcement-mechanism (WASM/no_std/subprocess) + default-cap + workspace-opt-in (per F7 + T1-R).
- **Not single-axis tier reporting**: WitnessTier × AuditHint × EvidenceKind is three-axis; WitnessTier alone is projection-for-coarse-gating.
- **Not a vibes-grade tolerance escape valve** (NEW — scout S1): `#[antigen_tolerance(X)]` without `sidecar = true` opt-in is REPORTED as `EvidenceKind::None` + `tolerance-vibes-grade` hint, making the tier-honesty gap visible. v0.1 ships with this discipline; teams can opt in to attested tolerance via `sidecar = true`.
- **Not a delta-audit laundering channel** (NEW — adversarial T2-R): `attest delta` carries chain-depth cap + cumulative-fingerprint tracking + required-rationale. A chain of small deltas cannot smuggle a substantive cumulative change without explicit Fresh re-attestation.
- **Not an `--on-behalf-of` signing mechanism** (NEW — adversarial T9-R): such a flag would reintroduce bypass risk. v0.1 PR workflows use `signed_trailer` predicate leaf for reviewer-not-committer case; v0.4+ crypto-signing decouples from committer identity.
- **Not a permissive leaf-tool-invocation framework**: leaves invoking external tools must satisfy the 4-point bright-line rule (T4-R); `cargo build`/`cargo run`/`curl-with-external-URLs` are explicitly excluded.

---

## ADR-019 citation map

ADR-019 — Substrate-witness predicate family — cites:

- **ADR-002** (compose-don't-compete): cargo-vet landscape precedent; closed leaves + named compositions
- **ADR-004** (implicit-to-explicit elevation): EvidenceKind axis, scope field, evidence_provenance — all elevation moves
- **ADR-005 sub-clause F**: predicate evaluation IS trust-boundary check; witness-provider-crate boundary deferred to v0.2+ ADR with explicit enforcement-mechanism scope per F7 + T1-R
- **ADR-005 Amendment 3** (audit-tier-honesty): extends to substrate-witness recognition surface; ratchet-asymmetry named; EvidenceKind adds third orthogonal field; bounded-recursion of audit-of-audit named explicitly
- **ADR-006** (recognition-not-design): each leaf primitive recognizes existing substrate; evidence_provenance field encodes the three-instances threshold as structured data
- **ADR-007** (anti-YAGNI structurally-guaranteed): predicate-language + schema + CLI all forced; tolerance-ratification isomorphic addition; v0.2+ amendments named now
- **ADR-008** (Amendment 1 surface for severity-class — pending T6 substrate-grep)
- **ADR-011** (substrate-witness primitive RESOLVES the tolerance vibes-grade gap via isomorphic schema extension per scout S1)
- **Notary-institution design-arc** (scout S4): name the accountability-escalation path explicitly — git-trust → OIDC + transparency log (Sigstore) as the principled escalation; 800 years of notary practice predicts the arc

ADR-019 amendments at ratification + later:
- v0.2 Am 1: CODEOWNERS interop UX (T2)
- v0.2 Am 2: TUF k-of-n threshold signatures (`signers(required_threshold = K, candidates = [...])`)
- v0.2 ADR-N: witness-provider-crate leaf-contract + ENFORCEMENT MECHANISM (separate ADR per F7 + T1-R critical-path)
- v0.2 Am 3: `--prioritized` flag for annotation-fatigue mitigation
- v0.2+ Am 4: lifetime on discipline claims (permanent/temporal/transitional)
- v0.3+ Am 5: forge-API integration; SARIF export adapter
- v0.4+ Am 6: DSSE envelope + Sigstore identity-bound signatures for `Signer.signature` activation
- Future Am 7: combinator grammar versioning (PASETO discipline)
- Future amendment: T3 discipline_doc dual-jobs separation; T7 fingerprint-scheme evolution; T8 descended_from predicate inheritance
- Future amendment: fingerprint-ratification sidecars (long-arc; antibody-specificity rhyme); lineage-validation sidecars (long-arc)

All amendments land WITHOUT splitting ADR-019. The ONE-ADR position
reflects that all extensions are *refinements of the same primitive*
(substrate-witness over substrate-other-than-this-code, with
discipline-level unification including tolerance via isomorphic
schema), not new primitives.
