# Discipline Witnesses — Draft v2

> **Status**: draft, single-instance critical pass on 2026-05-18, ahead of
> any team deconstruction. v1 stays as substrate at
> [discipline-witnesses-v1.md](discipline-witnesses-v1.md); v2 sharpens
> load-bearing claims, resolves questions the single instance has a
> position on, and isolates what genuinely needs team time.
> Observational tone ("here's what fell out, sharpened"), not directive
> ("here's what ships").

> **Provenance**: v1 distilled from the 2026-05-18 design conversation
> ([capture](../captures/discipline-witnesses-2026-05-18.md)). v2 is a
> critical-pass review by single-instance Claude after that capture
> landed, before the team deconstruction pass v1's readiness-state
> section anticipated. The shifts are catalogued in the "What changed
> v1→v2" section immediately below.

---

## What changed v1→v2

**Strengthened (was assertion in v1; argument in v2)**:
- The four load-bearing items now name *what breaks* without each piece
  (§"What's load-bearing — strengthened")
- The substrate-witness tier-honesty mapping is reasoned, not just
  tabled (§"Tier-honesty mapping — sharpened")
- The biology break-point has both framings (clean break vs
  expanded-unit-of-analysis), so naturalist can choose rather than be
  handed a single conclusion (§"Biology grounding — break-point reframed")

**Positions taken (was open question in v1; my-take embedded in v2)**:
- Audit I/O surface: **filesystem-only in v0.1**; forge-API deferred
- Signing weight: **git-trust as default**; `Signer.signature` slot reserved
- Schema versioning: **`schema_version: 1` shipped**; migration CLI is
  skeleton until v2 lands
- `antigen-attestation` crate boundary: **separate crate**
- Predicate-language ceiling: **closed combinator grammar**, no
  user-defined functions, no conditionals
- Refactor tooling (`attest move`): **ship in v0.1**, deferred-with-error
  on edge cases
- Recognition-vs-design ratio: **healthy** — minimal design layer over
  extensive recognition substrate
- **ADR shape**: **one ADR** (substrate-witness predicate family),
  introducing predicate language + schema + CLI as one primitive;
  amendments for refinements

**Left for team (was open question in v1; remains so in v2 with sharpened framing)**:
- Macro syntax: `witness = adjacent_sidecar(...)` (A) vs `requires = ...`
  parallel parameter (B) — tier-honesty surface implications either way
- CODEOWNERS interop UX (literal vs role-resolved)
- (Naturalist call) Biology break-point: clean-break (framing A) vs
  expanded-unit-of-analysis (framing B)

**Unchanged**:
- The problem statement and the substrate-witness reframe
- The three-piece shape (predicate language + Ratification schema + CLI)
- Per-antigen-per-file granularity with `.attest/` subfolder
- Generated-code position (input-level discipline; "accept tradeoff" is
  expected default)

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

Tambear hit this trying to declare `#[immune(SignedZeroDiscipline,
witness = ?)]` for `sinh`/`cosh` numerics. None of the existing witness
types fit. The naive escape — phantom-type as stand-in for "agent attested"
— would be a tier-honesty violation per ADR-005 Amendment 3. The naive
extension — adding an "Attestation" tier below Reachability in `WitnessTier`
— creates a laundering vector: once `witness = doc_attested(...)` exists in
the enum, it gets used for things that should have had mechanical witnesses,
and "Attestation" becomes ambient cover for unverified claims.

The right reframe (recognition-not-design):

> Witnesses currently check *code-side* substrate. They should be extensible
> to check *non-code* substrate as well — ratified docs, sign-off records,
> signed git trailers, oracle-completion markers — *as long as the audit
> remains tier-honest about what was actually verified*.

Once witnesses can check non-code substrate, "discipline antigens" stop
being a special category and become *ordinary antigens whose witness
predicates evaluate against substrate that isn't `.rs` files*.

---

## Shape proposed

Three coupled pieces that ship together (per ADR-007 anti-YAGNI —
structurally-required for the discipline to be more than decorative):

### 1. Substrate-witness predicate family

A small declarative predicate language over typed substrate. Closed
combinator grammar (no Turing tarpit), small starting set of leaf
primitives that the audit knows how to evaluate against on-disk substrate.

**Leaf primitives (v0.1 shipping set)**:

| Primitive | Checks |
|---|---|
| `ratified_doc(path?, min_version?, anchor?)` | Doc exists at path; frontmatter version ≥ min; optional anchor present |
| `signers(required, roles?)` | Sidecar `signers[]` array includes all required names (optionally with matching roles) |
| `signed_trailer(key, role?, count?)` | Git log on touching commits has matching trailer (e.g., `Discipline-Verified-By:`) |
| `oracles_complete(files)` | Listed oracle files exist with `status: complete` markers |
| `fresh_within_days(n)` | Most recent signature/sidecar mtime within N days |

**Combinators (full set; closed)**: `all_of([...])`, `any_of([...])`,
`not(...)`. No `if`/`while`/user-defined-fn. If more expressiveness is
needed, use a regular `test_fn` witness — see §"Predicate-language
ceiling" below for the structural argument.

**Macro shape**:

```rust
#[immune(SignedZeroDiscipline, requires = all_of([
    signers(required = ["alice", "bob"]),
    oracles_complete(["docs/oracles/sinh-domain.md"]),
    ratified_doc(min_version = "1.0"),
    fresh_within_days(180),
]))]
pub fn sinh(x: f64) -> f64 { ... }
```

**Team-level open question (macro syntax)**: A vs B.
- **A**: keep `witness = ...` parameter; substrate-witness is a new
  witness type with `witness = adjacent_sidecar(requires = all_of([...]))`.
  Conservative; preserves witness vocabulary.
- **B**: new `requires = ...` parameter parallel to `witness`. Reads
  more naturally for discipline antigens, distinguishes discipline-class
  at the macro syntax level.

Tier-honesty implications either way; aristotle should weigh. v2 draft
shows B-style for readability without committing.

### 2. Ratification schema + JSON sidecars adjacent to source

The substrate-witness leaves read **typed JSON sidecars** living next to
the source file. Schema is a serde-derived Rust type — single source of
truth for audit (reads), CLI (writes), and editor support
(`JsonSchema` → `.schema.json` → autocomplete).

**Location convention**:

```
src/numerics.rs                         ← source + #[immune(...)] macros
src/numerics.attest/                    ← subfolder, only exists if any
                                          discipline antigens present in
                                          this file
  SignedZeroDiscipline.json             ← one sidecar per antigen
  AllSignersRequired.json
  NumericalStabilityDiscipline.json
```

Three-layer granularity:
- **Source file** ↔ `.attest/` folder
- **Antigen** ↔ `<AntigenName>.json` sidecar inside the folder
- **Item** (function/struct/impl) ↔ entry in the sidecar's `items[]`
  array

**Schema sketch**:

```rust
// antigen-attestation crate (separate so witness-provider crates can
// depend on it without taking all of antigen — see §"Open questions"
// resolution for crate-boundary argument)
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Ratification {
    pub schema_version: SchemaVersion,   // v0.1 ships v1; migration CLI
                                          // skeleton ready for v2+
    pub antigen: AntigenIdentifier,
    pub source_file: PathBuf,
    pub items: Vec<ItemRatification>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ItemRatification {
    pub item_path: String,             // crate::numerics::sinh
    pub current_fingerprint: String,   // reuses antigen_fingerprint::Fingerprint
                                        // (see §"Load-bearing" item 4)
    pub doc_ref: Option<DocRef>,       // overrides antigen's discipline_doc
    pub signers: Vec<Signer>,
    pub oracles: Vec<OracleRef>,
    pub fresh_through: Option<NaiveDate>,
    pub extensions: BTreeMap<String, Value>,  // forward-compat slot
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Signer {
    pub name: String,
    pub role: Option<String>,
    pub date: NaiveDate,
    pub signed_against_fingerprint: String,  // pin to fingerprint at
                                              // sign-time; affinity-
                                              // maturation rhyme
    pub signature: Option<Signature>,        // slot reserved per
                                              // ADR-007; git-trust is
                                              // the default
}
```

### 3. `cargo antigen attest` CLI family

Scaffolding tooling closes the adoption-friction gap. Without it, every
team writes their sidecars slightly differently → drift → tier-honesty
erodes back to where we started (see §"Load-bearing" item 5).

```sh
cargo antigen attest scaffold --file src/numerics.rs \
    --antigen SignedZeroDiscipline --item sinh
# creates src/numerics.attest/SignedZeroDiscipline.json (and folder if
# absent) with TODO markers for required fields; resolves antigen ID
# from project's antigens.rs

cargo antigen attest sign --file src/numerics.rs \
    --antigen SignedZeroDiscipline --item sinh --role math-researcher
# appends signer entry using git config user.name/email + UTC timestamp;
# pins signed_against_fingerprint to current; refuses if role already
# signed (idempotent); refuses if git config not set

cargo antigen attest oracle complete --file src/numerics.rs \
    --antigen SignedZeroDiscipline --item sinh \
    --oracle docs/oracles/sinh-domain.md
# updates oracle status, records marker

cargo antigen attest check [--file ... | --all]
# validates schema across all sidecars; reports per-leaf failures;
# exits nonzero if any predicate fails

cargo antigen attest list [--stale | --pending | --signed-by alice]
# workspace queries; surfaces sidecars needing re-attestation, items
# missing signers, oracles still pending

cargo antigen attest move --from src/numerics.rs \
    --to src/numerics/transcendental.rs
# rebases the .attest/ folder when source moves; deferred-with-error
# on edge cases (inline submodules, trait-impl item_path
# disambiguation) — ship in v0.1 per structural-guarantee argument
# (§"Open questions" item 9)

cargo antigen attest migrate --from v1 --to v2
# skeleton; no migrations defined until v2 schema lands. Slot reserved
# per ADR-007 (schema migration is structurally-guaranteed-needed even
# if specific migrations are not)

cargo antigen attest gc
# removes orphaned entries (items no longer presenting the antigen);
# deletes empty sidecars and empty folders
```

---

## Tier-honesty mapping — sharpened

v1's mapping capped substrate-witnesses at Execution only when fingerprint
currency held, and put predicate-passes-without-currency at Reachability.
That's roughly right but the framing needs an explicit argument because
it touches a deeper question: *can substrate-witnesses reach Execution tier
at all*, given that Execution today means "code was exercised by a test
harness"?

**The argument for substrate-witness reaching Execution**:

WitnessTier names *depth of verification work the audit performed*, not
*kind of evidence the verification produces*. From `audit.rs`:
- `Reachability` = "Witness identifier resolves but no execution-level
  verification happened"
- `Execution` = "Witness was executed: a test or proptest function whose
  run was confirmed"

For test/proptest witnesses, the verifier is `cargo test` /
proptest harness. v0.1 audit doesn't invoke them, so reports Reachability
+ `test-attribute-present-not-invoked`. A4-A5 ships harness invocation;
those witnesses then earn Execution.

For substrate-witnesses, **the audit IS the verifier**. There's no
separate harness to invoke — the audit reads the sidecar, validates the
schema, evaluates the predicate against on-disk substrate, checks
fingerprint currency. When all of that completes with a passing answer,
the audit has done the full verification work the substrate-witness
encodes. That's Execution-tier work by direct analogy with the test case
*after* harness invocation lands.

The *kind* of evidence differs (attestation state vs code behavior), but
that's the audit_hint axis, not the tier axis. ADR-005 Amendment 3
Mechanics §2 explicitly establishes the tier/hint split: "Two witnesses
can carry the same `WitnessTier` but different `AuditHint`."

**Counter-argument considered and rejected**: capping substrate-witnesses
at Reachability would force every CI gate that wants tier-honest
discipline checking to either (a) add per-antigen-class exceptions to
its tier threshold, (b) silently fail every discipline-antigen claim, or
(c) maintain a parallel tier system. All three are worse than letting
substrate-witnesses earn Execution with a distinguishing hint. The
hint preserves consumer visibility into kind-of-evidence; the tier
preserves uniform consumer machinery.

**FormalProof remains unreachable from substrate-witnesses**. Substrate
is human/social attestation, not type-system proof or mathematical
verification. The biological cognate: B-cell receptors don't reach the
kind of certainty T-cell receptors get from sealed MHC presentation —
they remain probabilistic-recognition machinery. Reserved for
phantom-type witnesses as before.

**Revised mapping**:

| State | Tier | Audit hint |
|---|---|---|
| No `.attest/` folder, or no sidecar for this antigen | `None` | `discipline-sidecar-missing` |
| Sidecar exists but malformed (schema-fail) | `None` | `discipline-sidecar-schema-invalid` |
| Sidecar exists, schema valid, but predicate fails | `None` | `discipline-predicate-failed` (with per-leaf details) |
| Sidecar exists, schema valid, predicate passes, but ≥1 signature pinned to stale fingerprint | `Reachability` | `discipline-substrate-stale` |
| Sidecar exists, schema valid, predicate passes, all signatures pinned to current fingerprint | `Execution` | `discipline-substrate-validated-and-current` |

The `discipline-substrate-stale` state is the affinity-maturation case:
the substrate-witness *would* have passed against the prior code, but
the code drifted past the pin and the audit honestly reports the
verification work as Reachability (predicate would pass; substrate not
current). Re-attestation upgrades back to Execution.

**Connection to ADR-005 Am 3 OQ-1**: Am 3 Open Question 1 asks whether
tier-honesty generalizes to non-audit recognition mechanisms (scan-time
parsing, schema versioning at producer-consumer boundary, cross-crate
`descended_from` propagation) — "stays open until three independent
instances surface outside audit." Substrate-witnesses are
**audit-internal** — they're a new audit mechanism, not an outside-audit
instance. So they don't promote OQ-1's instance count. But they *do*
exercise Am 3 in new audit territory (verifying attestation state vs
code behavior), and the design choices here (Execution-when-current,
FormalProof-never-reachable, per-leaf resolution emitted) operationalize
Am 3's principle in this new sub-territory. This is an
*intra-audit-class extension*, not an outside-audit instance.

---

## Posture: opinionated-with-flexibility

Already named by ADR-002 (compose-don't-compete). Applied here:

**Closed (the posture, no negotiation):**
- Schema is a Rust type; no inventing new top-level fields
- Combinator grammar is closed; no `if`/`while`/user-fn
- Tier-honesty is mandatory; audit reports actual verification strength
- Sidecar substrate-currency is JSON-against-schema; not "pick your own
  representation"
- CLI scaffolds the canonical shape

**Open (the integration surface):**
- `extensions: BTreeMap<String, Value>` slot on each `ItemRatification`
  for project-specific fields
- Witness provider crates contribute new leaf primitives without
  forking the audit (Tier 3 ambition; design extension point now, ship
  later)
- Format adapter trait for teams with strong TOML/YAML/frontmatter
  conventions (deferred — extension point only in v0.1)
- `Signer.signature` field for crypto-signing teams; git-trust as
  default
- CODEOWNERS interop is opt-in via `required_role`; literal names as
  base case (team-level UX call)
- Sidecar location has a sensible default; configurable for teams with
  strong opinions

The principle: *closed at load-bearing parts, open at the integration
edges*. Closed where openness would leak tier-honesty (schema shape,
combinator grammar, audit reporting). Open where closedness would block
adoption (extension fields, witness providers, signing weight).

---

## Biology grounding — break-point reframed

The biology earned its keep across multiple structural decisions:

**MHC presentation → typed sidecar substrate-currency**: antigens
present in structured frames (MHC class I/II grooves) that the immune
system can recognize. The frame is constrained, not arbitrary. Free-form
markdown is unstructured antigen the audit can't recognize without
interpretation; serde-validated JSON is MHC-class-style presentation
discipline.

**T-cell + B-cell co-stimulation → compound witnesses require all
signals**: single-signal antigen exposure produces *anergy* (tolerance),
not immunity. The compound-witness pattern rhymes — partial-immunity
where one leaf resolves and another doesn't shouldn't behave like
"partial immunity," it should structurally fail. The audit reporting
"unresolved until all leaves pass" is co-stim-style. Biology predicts the
strict-all-of default.

**Affinity maturation → fingerprint-pinned signatures**: antibodies are
selected against specific antigen variants; when the antigen mutates, old
antibodies may no longer bind and the germinal center has to refine.
Signatures pinned to `signed_against_fingerprint` rebind on code change
automatically. Re-attestation discipline is structural, not opt-in. This
is also where the `discipline-substrate-stale` audit hint lives —
predicate-passes-but-pin-stale is the affinity-maturation case caught at
the audit surface.

**B-cell vs T-cell certainty asymmetry → substrate-witness tier cap at
Execution, not FormalProof**: T-cell receptors bind MHC-presented
peptides with a kind of sealed-presentation certainty B-cell receptors
don't get from their free-floating antigen recognition. Phantom-type
witnesses are the sealed-MHC analog (constructor sealing + type-system
proof = FormalProof). Substrate-witnesses are the B-cell-style
probabilistic-recognition analog (predicate-evaluation against substrate
that humans created = Execution, not FormalProof). Biology *predicts*
the tier cap, doesn't just permit it.

**Per-cell antigen processing → code-locality, not doc-locality**:
in immunology, antigens are presented AT THE CELL where they're
processed. Recognition memory lives in the lymphocyte. Distributed
substrate, locally validated, per-presentation. There is no central
registry of "who's been vaccinated." Code-side sidecars (`.attest/`
adjacent to source) are the germinal-center pattern. Doc-side sidecars
(adjacent to discipline-doc) would have been the central-registry
pattern — wrong abstraction layer.

### Productive break-point — two framings

v1 framed substrate-witnesses as the place where biology stops
predicting because "immune systems can't read records." That's defensible
but might be too clean — depends on where we draw the unit-of-analysis
boundary. Two framings; naturalist call which one the discipline keeps.

**Framing A (clean break)**: The metaphor productively breaks at the
substrate-witness extension because biology-the-immune-system-in-isolation
has no mechanism for recognizing externally-attested state. Affinity
maturation works through molecular machinery; antibodies recognize
antigens directly. There is no biological structure that says "I trust
X is true because record Y says so." Substrate-witnesses are a
software-engineering extension into territory biology doesn't occupy —
and that's the *right* breakpoint, because antigen-the-project is a
discipline tool for human practice, and human practice includes
record-trust. The metaphor's silence here is the data: it correctly
declines to predict what it can't predict.

**Framing B (expanded unit-of-analysis)**: The break-point is less clean
when we expand the unit-of-analysis. Biology-narrow doesn't have
record-trust; biology-as-clinically-embedded does (vaccination
protocols, medical records, immunization registries, herd-immunity
surveillance, school admissions requirements). The substrate-witness
extension is consistent with the broader biological-clinical system
humans actually operate as. Predictive rhyme: school vaccination
requirements ARE substrate-witnesses — the audit (school admissions)
doesn't verify immunity directly; it verifies the medical record
asserting vaccination, and rejects when records are missing, expired,
or against the wrong vaccine. The metaphor continues to predict at this
expanded unit of analysis.

Both framings preserve the structural commitments. **What's load-bearing
either way**: the substrate-witness IS where the metaphor stops doing
predictive work in its biology-narrow form. Either the boundary is real
and clean (framing A — the metaphor honestly declines), or the boundary
moves outward to encompass clinical-immunity infrastructure (framing B
— the metaphor's reach is wider than v1 acknowledged).

The discipline-question (per memory note
`feedback_metaphor_silence_at_boundary_is_the_evidence.md`): a metaphor
that produces dense predictions in its domain AND clean silence at the
boundary is operating as instrument. Substrate-witnesses are exactly the
boundary case to apply this test. Naturalist owns the call between A and
B — the wrong move is forcing one over the other without doing the
substrate work.

---

## What's load-bearing — strengthened

v1 named four items without arguing them. Each below names *what
breaks* without the item, so aristotle's Phase 1-8 has something
substantive to push on rather than rederive.

### 1. Audit reads non-`.rs` substrate

**What breaks without it**: sidecars become decorative. They exist on
disk but the audit can't validate them, so they're indistinguishable
from no-sidecar from the audit's perspective. Tier-honesty (ADR-005 Am
3) then requires reporting `None` tier for every discipline-antigen
claim because the audit didn't verify anything. The substrate-witness
reframe collapses — "witnesses over non-code substrate" requires the
audit to actually read non-code substrate.

**Implementation impact**: today the audit walks AST and validates
witness identifiers resolve to functions. Adding substrate-witnesses
requires: read JSON sidecars (serde_json already a transitive dep),
validate against schema (schemars or hand-derived JsonSchema), evaluate
content predicates (closed-set leaves; pure functions), check
fingerprint currency (reuses `antigen_fingerprint::Fingerprint`),
optionally read git log for `signed_trailer` (probably `gix`).

### 2. Witness expressions need parse + serialize + replay

**What breaks without it**: reviewers see "audit failed" with no
discrimination between "missing signer alice" / "freshness expired" /
"doc version below pinned minimum". CI gates can only fail-binary;
no actionable detail. The audit becomes a black box and tier-honesty at
the reporting surface degrades — `discipline-predicate-failed` collapses
into a single hint when reviewers need per-leaf failure breakdown to
respond. This is structurally parallel to AuditHint being load-bearing
for code-witnesses (per ADR-005 Am 3 Mechanics §2): without
per-case disambiguation, tier-honesty at the reporting surface fails.

**Implementation impact**: JSON output emits the full expression tree
with per-leaf resolution status. Reviewers see
`all_of([✓ ratified_doc, ✗ signers (missing: carol), ✓ fresh_within_days])`.
Tier-honest by construction. Audit text output renders this as a tree
in human-readable form.

### 3. `discipline_doc` field on antigen declaration

**What breaks without it**: every sidecar repeats the discipline doc
path; doc moves require per-sidecar updates; per-sidecar `doc_ref`
becomes the canonical source which fragments the "antigen has one
canonical reference doc" property. The antigen declaration loses its
role as anchor for the discipline.

**Why this isn't speculative**: antigen declarations already carry
`references = [...]` (parse.rs:111). Adding `discipline_doc` is
making *implicit* structure (the canonical doc was always in
`references`; we just didn't pin one as canonical) *explicit* — exactly
ADR-004's elevation discipline. Per-sidecar `doc_ref` remains as
override for the rare case where a specific item ratifies against a
different doc, but the antigen-level default is the source of truth.

### 4. Fingerprint reuse

**What breaks without it**: substrate-witnesses invent their own
fingerprint scheme; two ways to hash "the same" code (scan's vs
attest's); guaranteed divergence at edge cases (whitespace handling,
comment treatment, attribute order, doc-comment handling). The
affinity-maturation rhyme breaks at the substrate-witness boundary —
signatures pinned to one scheme cannot recognize code change detected
by the other scheme.

**Why this isn't a hypothetical concern**: `antigen_fingerprint` is
*already* factored as a separate crate (per scan.rs:1389 —
`antigen_fingerprint::Fingerprint::parse`). The scan crate explicitly
established the fingerprint type as a stable boundary. `antigen-attestation`
takes the same dependency. Single source of truth for "is this code
changed" determination. No new scheme; no possibility of drift.

### 5. CLI scaffolding (the implicit fifth load-bearing item)

**What breaks without it**: every team writes their sidecars slightly
differently. Drift erodes the schema-as-contract that tier-honesty
depends on. Manual-JSON-authoring fails on adoption — humans
mis-quote, omit required fields, copy-paste with stale fingerprints,
forget `signed_against_fingerprint` pins. The closed-schema discipline
becomes theatrical because the substrate that should be schema-conformant
is hand-edited.

**Why elevate this from "tooling" to "load-bearing"**: opinionated-with-
flexibility (ADR-002) requires the opinionated-half to be enforced *at
the substrate-creation point*, not just at the substrate-reading point.
Audit-side validation catches drift after it happens; CLI-side
scaffolding prevents drift from happening. The discipline lives at both
boundaries.

---

## Open questions — sorted

v1 listed 10 undifferentiated. Sorted into three buckets below: team-needs
(genuinely needs multi-perspective deconstruction), my-take-embedded
(single-instance critical pass formed a position; surfaced for team to
push back if they want), and naturalist-judgment (the biology-side call
already covered in §"Biology grounding — break-point reframed").

### Team-needs (genuinely open after v2)

**T1. Macro syntax: A vs B** —
- **A**: `witness = adjacent_sidecar(requires = all_of([...]))` —
  conservative; preserves witness-vocabulary
- **B**: `requires = ...` parallel parameter — reads more naturally for
  discipline antigens; distinguishes discipline-class at macro syntax

Tier-honesty implications either way. v2 draft shows B-style for
readability without committing. Aristotle's Phase 1-8 should weigh.

**T2. CODEOWNERS interop UX** —
- Literal names in `signers.required` (simple; matches sidecar contents)
- Auto-resolve from CODEOWNERS via `required_role` (matches how real
  teams operate; but couples to forge-specific patterns)

v0.1 ships literal-only; `required_role` slot reserved for v0.2 per
ADR-007. Question is what the v0.2 UX looks like.

### My-take-embedded (resolved by single-instance critical pass)

**R1. Audit I/O surface** (was open Q2). **Position: filesystem-only in
v0.1; forge-API deferred to v0.2+.**

*Argument*: ship what cargo already supports (filesystem + git log).
Forge-API gates on network in CI, gates on org/repo permissions, brings
in API authentication — all v0.2+ work. The predicate-leaf
`signed_trailer` can already approximate "CODEOWNERS-approved" by
checking git trailer presence on touching commits. Full forge-API is
incremental on top of filesystem-substrate baseline.

**R2. Signing weight** (was open Q3). **Position: git-trust as default;
`Signer.signature` slot reserved per ADR-007.**

*Argument*: signing-via-git-config is what the substrate already
supports — git commits already carry author identity, signed trailers
already exist. Crypto-signing is opt-in for teams with stronger
compliance needs. ADR-007 anti-YAGNI: ship the slot (we KNOW some teams
will need this; structurally-required), don't ship the implementation
(no concrete-team implementation requirement yet).

**R3. Schema versioning + migration** (was open Q4). **Position: ship
`schema_version: 1` now; CLI accepts only v1; migration commands are
skeleton until v2 lands.**

*Argument*: ADR-007 anti-YAGNI for the slot AND for the migration
machinery (we KNOW we'll need migration; we don't yet know what
migrations). Ship `cargo antigen attest migrate --from v1 --to v2`
as a command that errors with "no migrations defined" until v2 ships.
Structurally-required-feature is the slot AND the migration interface,
not the migrations themselves.

**R4. `antigen-attestation` crate boundary** (was open Q5). **Position:
separate crate.**

*Argument*: three reasons.
- (a) Tier-3 witness-provider crates need the schema without taking all
  of antigen-core
- (b) Existing scan/audit already factored `antigen_fingerprint` as a
  separate crate — precedent for "shared schema = separate crate"
- (c) `antigen` core stays schema-lean — `serde_json` is already a
  transitive dep, but `JsonSchema`/`schemars` would be a new one; keep
  the cost where the consumers are

Three workspace members after this ships: `antigen` (re-exports +
audit), `antigen-fingerprint` (existing), `antigen-attestation` (new).

**R5. Predicate-language ceiling** (was open Q7). **Position: closed
combinator grammar.** `all_of` / `any_of` / `not` are the only
combinators; leaves are a typed enum (sealed) shipped with v0.1; no
user-defined-fn; no conditionals.

*Argument*: hard ceiling = predicates must be *declarative over
substrate*. Anything else is a code-witness. If you need conditional
logic, write a `test_fn` witness that does whatever you want.

*Biology rhyme*: TCR diversity comes from somatic recombination of FIXED
gene segments (V/D/J), not arbitrary code generation. The *recognition
repertoire* is vast (~10^11 unique TCRs) but the *recognition machinery*
is tightly bounded. Predicate-language follows the same pattern: vast
expressive power from compositions of fixed primitives, zero ability to
invent new primitives at use-site.

**R6. Refactor tooling: `attest move`** (was open Q9). **Position: ship
in v0.1; deferred-with-error on edge cases.**

*Argument*: ADR-007 structurally-guaranteed — file moves WILL happen;
without `attest move`, every move silently breaks sidecar linkage; that's
an adoption-killer. Inline submodule edge cases (where `item_path` needs
disambiguation across `mod foo { fn bar() }` vs `fn foo_bar()`) can
report structured-error in v0.1 ("move couldn't disambiguate items A and
B; edit manually") rather than block ship. Structural-guarantee for the
*command*, not for *perfect handling*.

**R7. Recognition-vs-design ratio** (was open Q10). **Position: healthy
ratio.**

*Argument*:
- *Recognition*: each leaf primitive recognizes existing substrate
  (markdown frontmatter, git trailers, JSON schema, file existence,
  signed-off-by trailers, CODEOWNERS files, mtime). All existing
  substrate; no invention.
- *Design*: the combinator language is invented (small DSL of
  `all_of`/`any_of`/`not`); the schema shape is invented (Ratification
  struct fields); the CLI shape is invented (commands and flags).
- *Ratio*: the design pieces are SMALL (three combinators, ~6 schema
  field types, ~7 CLI subcommands). The recognition pieces are vast
  (everything the leaves read).

The right ratio for recognition-not-design (ADR-006): extensive
recognition over a minimal design layer. The combinator layer is
design-of-Boolean-algebra; that's a 100-year-old design space; the
"invention" is choosing the closed subset, not creating new combinators.

Naturalist owns the final call but the substrate strongly suggests
healthy.

**R8. ADR shape: one ADR vs three** (was open Q6). **Position: ONE
ADR.**

*Argument*: existing precedent in `decisions.md`:
- ADR-001: 1 amendment with 8 changes — primitives ship in one ADR,
  refined via amendments
- ADR-005: 3 amendments (each tightly scoped)
- ADR-010: 5 amendments

The pattern is consistent: introduce primitives in one ADR; amend with
refinements. The three pieces here (predicate language + schema + CLI)
are *tightly coupled* — not three independent primitives but three faces
of one primitive (substrate-witnesses). The predicate language assumes
the schema; the schema assumes the CLI scaffolds it; the CLI assumes the
predicate language gives it shape to scaffold for. They're load-bearing
*on each other*.

**Position: ADR-019 — Substrate-witness predicate family** introduces
all three coupled pieces as one primitive. Amendments handle:
- v0.2 Am 1: CODEOWNERS interop UX
- v0.3+ Am 2: Forge-API integration
- v0.4+ Am 3: Crypto-signing field activation if real adoption requires
- Future Am N: Tier-3 witness-provider crate convention

ADR-019 cites:
- ADR-002 (compose-don't-compete: substrate-witnesses compose with
  markdown, git, JSON, filesystem as substrate)
- ADR-005 sub-clause F (predicate evaluation IS the trust-boundary
  check for substrate-witness claims; explicit per ADR-005's
  "every trust boundary requires validation" discipline)
- ADR-005 Amendment 3 (extends tier-honesty discipline to
  substrate-witness recognition surface; new audit_hint values; this is
  intra-audit-class extension, NOT the outside-audit instance Am 3
  OQ-1 was waiting for)
- ADR-006 (recognition-not-design: each leaf primitive recognizes
  existing substrate; combinator language is the small design layer)
- ADR-007 (anti-YAGNI structurally-guaranteed: predicate-language +
  schema + CLI all forced by adoption; full primitives ship in v0.1
  even if Tier-3 witness providers defer)

---

## Readiness state — revised

v1 said "dense enough for team deconstruction" and named a recommended
five-agent pass (aristotle / naturalist / adversarial /
academic-researcher / scout).

v2 sharpens: most of the would-be team pass has been done by
single-instance critical pass. What remains genuinely team-needing:

**Team-pass scope after v2**:
- **aristotle**: Phase 1-8 deconstruction of the macro syntax A-vs-B
  call (T1); verify the strengthened load-bearing arguments hold up
  to first-principles push; sanity-check the predicate-language ceiling
  doesn't have a hidden expressiveness escape; sanity-check the ONE-ADR
  position against the process implications
- **naturalist**: the productive break-point A-vs-B call (clean-break
  vs expanded-unit-of-analysis); verify the strengthened biology rhymes
  (especially B-cell-vs-T-cell certainty asymmetry → tier cap argument)
  hold against actual immunology; pull outside-domain inspiration for
  patterns the predicate language might mirror or fail to mirror
- **adversarial**: attack the seams — what attestation-claim shapes
  bypass the closed combinator grammar? what sidecar drift escapes
  the schema-as-contract? what happens at edge cases of `attest move`
  deferred-with-error? what does `attest gc` false-positive look like?
- **academic-researcher**: how have other tools framed similar
  trade-offs (cargo-deny, clippy-config, dependabot ratifications,
  Sigstore, in-toto attestations, Salsa provenance, SLSA framework
  attestation predicates) — informs the predicate-language design
  space without committing to anyone else's framing
- **scout**: where does this rhyme into other antigen surfaces we
  haven't visited (multi-component immunity, descended_from
  inheritance, autoimmunity prevention) — and the inverse, what does
  scout-mode reveal that single-instance critical pass missed

**Team-pass scope BEFORE v2 was** essentially everything above plus
re-derivation of the my-take-embedded items. v2 reduces team time by
catching what single-instance critical pass can catch.

After team-pass: draft graduates into **ADR-019 Substrate-witness
predicate family** (one ADR, with cited cross-references to ADR-002 /
ADR-005 / ADR-005 Am 3 / ADR-006 / ADR-007 per §"ADR shape" above).

---

## What this is NOT

- **Not a replacement for `#[immune(X, witness = test_fn)]`**: mechanical
  antigens still use code-side witnesses. Substrate-witnesses are
  *additive* to the existing witness family, not substitutive.
- **Not a special category of antigen**: discipline-antigens are
  ordinary antigens whose witness predicates evaluate against non-`.rs`
  substrate. No new antigen declaration syntax beyond the optional
  `discipline_doc` default field.
- **Not a vibes-grade attestation system**: signing requires real
  substrate (git config, fingerprint pin, structured sidecar entry).
  No `witness = trust_me` escape hatch.
- **Not a doc-management system**: the sidecar carries compliance state,
  not doc content. Discipline docs live wherever they already live
  (in-repo, on a wiki, in a PR, as a published paper). The sidecar
  references them, doesn't replace them.
- **Not a CI gate by default**: `cargo antigen audit` reports state.
  CI gates are downstream consumers that choose to fail on
  `discipline-predicate-failed` audit hints. Teams adopt at their own
  pace.
- **Not three ADRs**: per §"Open questions" R8, this is ONE ADR
  introducing one primitive (substrate-witness predicate family) with
  three tightly-coupled pieces (predicate language + schema + CLI).
  Existing decisions.md precedent is "primitives in one ADR; refinements
  as amendments." Three independent ADRs would suggest three
  independent primitives that ship-and-evolve separately; that's not
  the structure here.
- **Not an outside-audit instance of ADR-005 Am 3 OQ-1**: substrate-
  witnesses are audit-internal. They exercise Am 3 in new sub-territory
  (verifying attestation state vs code behavior) but don't promote
  Am 3's open-question instance count for non-audit recognition
  mechanisms.
- **Not a generator-output coverage system**: generated `.rs` files
  (build-script output in `OUT_DIR`, proc-macro expansion, etc.) are
  out of scope for `.attest/` sidecars in v0.1. Generated code can
  still *present* antigens via emitted macros; the audit scans them
  like any other code. Discipline antigens that should cover generator
  output belong at the generator's **input layer** — the `.proto`
  schema, the IDL file, the `build.rs`, the macro invocation site —
  where the author actually exercises the discipline. Generated output
  is mechanical; the discipline lives where judgment is exercised. If
  a generator doesn't expose authorable input, three cases:
  **(a) Existing trusted generator** — accept the tradeoff. This is
  the expected default, not a regrettable gap: adopters of an
  existing generator extended trust at adoption time (the generator's
  output is *what they're using the generator for*), and per-output
  attestation would re-attest something already attested at a coarser
  trust boundary. Aligns with ADR-005 sub-clause F: trust boundaries
  live where trust was extended, not at every downstream presentation.
  **(b) Generator could expose authorable input but doesn't yet** —
  upstream contribution or vendoring is the move; once input is
  authorable, input-level discipline applies.
  **(c) New generator being written** — build antigen attestation
  into the generator from the start, emitting input-level antigen
  declarations alongside output.
  **No generator-plugin hook subsystem is planned** unless real
  adopters surface specific use cases that none of (a)/(b)/(c)
  covers. Compose-don't-compete (ADR-002) — discipline applies at the
  right layer, not by building output-coverage machinery.

  Biology rhyme: **clonal selection at thymic education** — trust in
  self-antigens established once during T-cell development; TCRs that
  pass negative selection aren't re-checked against every self-antigen
  presentation. Trust boundary lives at the developmental layer, not
  per-encounter. Generator adoption is the thymic-education equivalent.
