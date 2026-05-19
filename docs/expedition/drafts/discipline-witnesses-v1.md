# Discipline Witnesses — Draft v1

> **Status**: draft, pre-team-deconstruction. The shape below fell out of a
> design conversation on 2026-05-18 (see
> [captures/discipline-witnesses-2026-05-18.md](../captures/discipline-witnesses-2026-05-18.md)
> for the back-and-forth, the dead ends, and the user-driven moves). The team
> will deconstruct this; expect substantive change. Observational tone
> ("here's what fell out"), not directive ("here's what ships").

> **Provenance**: Tekgy + Claude design conversation. Tambear surfaced the
> gap (its v0.1-rc adoption attempt hit "no doc-attestation witness type");
> the antigen-project design conversation reframed the gap as "witnesses
> currently only check `.rs`-side substrate; the discipline gap is
> witnesses over *non-code* substrate." Three user-driven moves were
> load-bearing: (1) strict structural representation over free-form markdown,
> (2) code-locality over doc-locality for fulfillment substrate, (3)
> per-antigen-per-file sidecars in a `.attest/` subfolder.

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

Three coupled pieces that probably ship together (or the discipline falls
apart):

### 1. Substrate-witness predicate family

A small declarative predicate language over typed substrate. Closed
combinator grammar (no Turing tarpit), small starting set of leaf
primitives that the audit knows how to evaluate against on-disk substrate.

**Leaf primitives (proposed minimum for v0.1)**:

| Primitive | Checks |
|---|---|
| `ratified_doc(path?, min_version?, anchor?)` | Doc exists at path; frontmatter version ≥ min; optional anchor present |
| `signers(required, roles?)` | Sidecar `signers[]` array includes all required names (optionally with matching roles) |
| `signed_trailer(key, role?, count?)` | Git log on touching commits has matching trailer (e.g., `Discipline-Verified-By:`) |
| `oracles_complete(files)` | Listed oracle files exist with `status: complete` markers |
| `fresh_within_days(n)` | Most recent signature/sidecar mtime within N days |

**Combinators**: `all_of([...])`, `any_of([...])`, `not(...)`. That's the
full grammar. No `if`/`while`/user-defined-fn — if that's needed, use a
regular `test_fn` witness.

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

**Open question (A vs B for team)**:
- **A**: keep `witness = ...` parameter; substrate-witness is a new
  witness type with `witness = adjacent_sidecar(requires = all_of([...]))`.
  Conservative; preserves witness vocabulary.
- **B**: new `requires = ...` parameter parallel to `witness`. Reads
  more naturally for discipline antigens, distinguishes discipline-class
  at the macro syntax level.

Both work; B is more declarative; A is more conservative. The draft
shows B-style for readability but doesn't commit.

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
// depend on it without taking all of antigen)
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Ratification {
    pub schema_version: SchemaVersion,
    pub antigen: AntigenIdentifier,
    pub source_file: PathBuf,
    pub items: Vec<ItemRatification>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ItemRatification {
    pub item_path: String,             // crate::numerics::sinh
    pub current_fingerprint: String,   // sha256:...
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
                                              // sign-time
    pub signature: Option<Signature>,        // optional crypto; git-trust
                                              // is the default
}
```

### 3. `cargo antigen attest` CLI family

Scaffolding tooling closes the adoption-friction gap. Without it, every
team writes their sidecars slightly differently → drift → tier-honesty
erodes back to where we started.

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
# rebases the .attest/ folder when source moves

cargo antigen attest gc
# removes orphaned entries (items no longer presenting the antigen);
# deletes empty sidecars and empty folders
```

---

## Tier-honesty mapping

The audit reports the work it actually performed:

| State | Tier | Audit hint |
|---|---|---|
| No `.attest/` folder, or no sidecar for this antigen | `None` | `discipline-sidecar-missing` |
| Sidecar exists but malformed (schema-fail) | `None` | `discipline-sidecar-schema-invalid` |
| Sidecar exists, schema valid, but predicate fails (missing signer / pending oracle / expired freshness) | `None` | `discipline-predicate-failed` (with per-leaf details) |
| Sidecar exists, schema valid, predicate passes | `Reachability` | `discipline-substrate-validated` |
| Reachability + signature `signed_against_fingerprint` matches current | `Execution` | `discipline-substrate-current` |

Note: `Execution` requires fingerprint-currency on signatures, which
catches the affinity-maturation case — when the function body changes,
old signatures are still readable but no longer current; audit drops to
`Reachability` until re-sign. This is structural, not opt-in.

`FormalProof` is not reachable from substrate-witnesses; reserved for
phantom-type witnesses as before.

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
  base case
- Sidecar location has a sensible default; configurable for teams with
  strong opinions

The principle: *closed at load-bearing parts, open at the integration
edges*. Closed where openness would leak tier-honesty (schema shape,
combinator grammar, audit reporting). Open where closedness would block
adoption (extension fields, witness providers, signing weight).

---

## Biology grounding

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
automatically. Re-attestation discipline is structural, not opt-in.

**Per-cell antigen processing → code-locality, not doc-locality**:
in immunology, antigens are presented AT THE CELL where they're
processed. Recognition memory lives in the lymphocyte. Distributed
substrate, locally validated, per-presentation. There is no central
registry of "who's been vaccinated." Code-side sidecars (`.attest/`
adjacent to source) are the germinal-center pattern. Doc-side sidecars
(adjacent to discipline-doc) would have been the central-registry
pattern — wrong abstraction layer.

**Productive break-point**: biology's passive immunity (maternal IgG to
infant, monoclonal antibody therapy) still works through in-host
molecular machinery. Substrate-witnesses are more like "the immune
system trusts the medical record that says 'this person was
vaccinated.'" Biology doesn't have that because immune systems can't
read records. This is where the metaphor stops predicting and we're
inventing. Worth flagging in any ADR draft so the naturalist can
foreground the break-point rather than have it surface as a hostile
review later.

---

## What's load-bearing for this to work

- **Audit reads non-`.rs` substrate.** Today it walks AST and validates
  witness identifiers resolve to functions. New: reads JSON sidecars,
  validates against schema, validates content predicates. Adds
  dependencies (probably `serde_json` already, plus a `chrono` for
  dates, plus optionally `gix` for git-log reading if `signed_trailer`
  ships in v0.1).
- **Witness expressions need parse + serialize + replay.** Audit JSON
  output emits the full expression tree with per-leaf resolution
  status. Reviewers see
  `all_of([✓ ratified_doc, ✗ signers (missing: carol)])`. Tier-honest
  by construction.
- **`discipline_doc` field on antigen declaration** (optional default;
  per-item `doc_ref` in sidecar overrides). If the antigen declares its
  canonical doc, sidecars don't have to repeat it.
- **Fingerprint reuse**: don't invent a new fingerprint scheme for
  signature-pinning; reuse antigen's existing scan fingerprint (whatever
  it currently hashes). The audit already computes fingerprints; the
  sidecar pins to them.

---

## Open questions for team deconstruction

1. **Macro syntax**: `witness = adjacent_sidecar(requires = ...)` vs
   `requires = ...` parallel parameter. Tier-honesty implications either
   way. Need aristotle's first-principles pass.

2. **Audit I/O surface**: filesystem-only (read JSON sidecars + git log
   for trailers) vs forge-API (resolve CODEOWNERS, fetch PR approval
   state). Forge-API is bigger commitment, gates on network in CI, but
   is what "team-signed" really needs at scale. Adoption-gradient
   question.

3. **Signing weight**: git-trust (name + email + sha) as default; crypto
   signatures (Sigstore-style) as opt-in via `Signer.signature` field.
   Probably ship git-trust in v0.1 with the field reserved; ADR-007
   anti-YAGNI argues for reserving the slot now.

4. **Schema versioning + migration**: `schema_version` is first-class.
   CLI ships migration commands as schema evolves. Same posture as
   `cargo migrate-resolver`. How aggressive should migration be in
   v0.1?

5. **`antigen-attestation` as separate crate vs `antigen-core::attestation`
   module**: separate crate enables witness-provider crates (Tier 3) to
   depend on the schema without taking all of antigen. Probably worth
   the boundary even pre-Tier-3.

6. **Does this need its own ADR family or amend ADR-001 / ADR-002?**
   Current bet: new ADR ("ADR-019: Substrate-witness predicate family")
   that *cites* ADR-002 (composition — we're composing with markdown,
   git, filesystem as substrate tools) and *amends* ADR-005 Am 3
   (audit-tier-honesty — defining how substrate-witnesses fit the
   four-tier model). Open to recombination by aristotle/navigator.

7. **What's the failure mode for "too expressive" predicate language?**
   Turing tarpit. The hard ceiling is: combinators stay closed-set; no
   user-defined-fn; substrate-witnesses are *declarative predicate over
   substrate*, period. If you need more, you write a regular `test_fn`
   witness. Naturalist should sanity-check this ceiling against biology.

8. **CODEOWNERS interop**: literal names in `signers.required` (simple,
   matches sidecar contents) vs auto-resolve from CODEOWNERS (matches
   how real teams operate). Probably ship literal in v0.1; design
   `required_role` for v0.2 CODEOWNERS interop.

9. **Refactor tooling**: `cargo antigen attest move` to follow file
   moves. Inline submodules and trait impls present `item_path`
   disambiguation challenges. Probably tractable; needs an executor
   pass for the dependency graph.

10. **Recognition vs design check**: are we recognizing or designing?
    My take: the predicate language is design (we're inventing a small
    DSL); each leaf primitive recognizes existing substrate (markdown
    docs, signed-off-by trailers, CODEOWNERS, all already exist).
    Design layer is thin; recognition layer is the substrate the leaves
    read. Healthy ratio. Naturalist owns this check.

---

## Readiness state

The substrate is dense enough for team deconstruction. Recommended pass:

- **aristotle**: deconstruct the assumptions (substrate-currency, MHC
  presentation as architectural justification, the predicate language
  ceiling, the closed-grammar commitment)
- **naturalist**: check the biology rhymes (MHC, co-stim, affinity
  maturation, germinal centers) and the productive break-point (passive
  immunity / record-trust gap); pull outside-domain inspiration
- **adversarial**: attack the seams (template drift despite CLI; the
  forge-API gap; sidecar-vs-PR-review boundary; what happens at the
  end of `cargo antigen attest gc`'s false-positive case)
- **academic-researcher**: how have other tools framed similar
  trade-offs (cargo-deny, clippy-config, dependabot ratifications,
  Sigstore, in-toto attestations, Salsa provenance)
- **scout**: where does this rhyme into other antigen surfaces we
  haven't visited (multi-component immunity, descended_from
  inheritance, autoimmunity prevention)

After deconstruction, the draft can graduate into:
- **ADR-019 substrate-witness predicate family** (the predicate language)
- **ADR-020 Ratification schema + sidecar convention** (the substrate-
  currency)
- **ADR-021 `cargo antigen attest` CLI** (the tooling)

…or recombined into a single ADR-019 with three changes, depending on
process discipline preferences.

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
