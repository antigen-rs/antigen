# Antigen — Examples Guide

> **v0.2 idiom note**: Some examples use the v0.1 `#[immune(...)]` API (particularly
> `broken_witness.rs`, which is preserved intentionally to demonstrate the deprecated form's
> failure mode). For v0.2, prefer `#[defended_by(X)]` (code-tier) or `#[presents(X, requires=...)]`
> (substrate-tier). See [`macros.md`](macros.md) for the current vocabulary.

> Curated walkthrough of the example files in `antigen/examples/`,
> ordered for progressive learning. Each lesson builds on the prior.
>
> Lessons 1–5 cover the core vocabulary (`#[antigen]` / `#[presents]` /
> `#[defended_by]` / `#[antigen_tolerance]` / `#[descended_from]`) + the
> witness model. Lessons 6–9 cover the substrate-witness pipeline
> (ADR-019), Oracle artifact lifecycle (ADR-021), delta-chained
> attestations, and attested-vs-vibes-grade tolerance.
>
> For the full tutorial narrative, see [`tutorial.md`](tutorial.md).
> For pattern recipes, see [`usage-patterns.md`](usage-patterns.md).

---

## How to use this guide

Each example is a complete, runnable Rust file in
`antigen/examples/`. Run any of them with:

```sh
cargo run --example <name> --package antigen
```

Or scan + audit them together to see how the tools interact:

```sh
cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

The examples are ordered below for progressive learning. Each lesson
adds one new concept to what the previous lessons established.

---

## Lesson 1 — `basic.rs`: declare, present, defend

**File**: [`antigen/examples/basic.rs`](../antigen/examples/basic.rs)

**Concept introduced**: the three core moves — declare a failure-class,
mark a vulnerable site, register a defense. Immunity is *observed* by
audit from the evidence, never claimed at the site (ADR-029).

**What's in the file**:
- `#[antigen]` declaration of `PanickingInDrop`
- A `VulnerableType` whose `Drop` impl could panic (marked
  `#[presents(PanickingInDrop)]`)
- A `SafeType` whose `Drop` impl is panic-free — also marked
  `#[presents(PanickingInDrop)]` (the site presents the shape; it does
  not claim to be immune)
- A witness function `safe_type_drop_no_panic_test` carrying
  `#[defended_by(PanickingInDrop)]` — a code-tier witness that exercises
  the safe drop paths. The defense lives on the witness, not the site.

**What to learn**:
- The three-verb structure: `build` an antigen (declare), `give` an
  antigen (`#[presents]`), `find` defenses (`#[defended_by]` witnesses
  + `cargo antigen audit` observing them)
- Antigen declarations are unit structs
- Defense evidence lives at the witness site, not the vulnerable site —
  immunity is observed, not declared
- (The v0.1 `#[immune(witness=)]` API is deprecated in favor of
  `#[defended_by]` / `#[presents(requires=)]`)

**Try this**:
```sh
cargo run --example basic --package antigen
cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

Look at the scan output. Notice how `basic.rs` declares
`PanickingInDrop` and shows you both a vulnerable site
(`VulnerableType`, unaddressed) and a defended site (`SafeType`, marked
`#[presents]` with a `#[defended_by]` witness). `audit` observes the
witness and reports `SafeType`'s defense; the vulnerable site appears as
an unaddressed presentation.

---

## Lesson 2 — `broken_witness.rs`: what happens when the witness doesn't exist

**File**: [`antigen/examples/broken_witness.rs`](../antigen/examples/broken_witness.rs)

**Concept introduced**: audit-tier-honesty. The audit reports the
*actual* verification strength, never a stronger one. When a witness
doesn't resolve, the audit names the gap honestly.

**What's in the file**:
- A `DemoBrokenWitness` antigen (with a deliberately minimal
  fingerprint: `name = matches("Looks*")`)
- A `LooksImmuneButIsnt` type that claims `#[immune(DemoBrokenWitness,
  witness = nonexistent_test)]` — but `nonexistent_test` doesn't exist

**What to learn**:
- The audit is your first line of defense against theatrical witnesses
- Broken witnesses are reported at `None` tier with
  `WitnessNotFound` hint
- This is *honest reporting*, not failure — the audit's job is to
  surface gaps

**Try this**:
```sh
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

Look for the `broken_witness.rs:38` entry in the audit output. You'll
see `tier = None, hint = NoneApplicable` with the diagnostic: *no
function named `nonexistent_test` found in any .rs file under the
scan root*.

The structural memory says "this site is immune." The audit says
"actually it isn't — the witness is broken." That's the
audit-tier-honesty discipline operating (ADR-005 Amendment 3).

---

## Lesson 3 — `antigen_tolerance.rs`: explicit tolerance with required rationale

**File**: [`antigen/examples/antigen_tolerance.rs`](../antigen/examples/antigen_tolerance.rs)

**Concept introduced**: explicit tolerance. When a fingerprint catches
a site you've reviewed and *intentionally* want to keep, you mark it
with `#[antigen_tolerance]` and a required rationale.

**What's in the file**:
- An `IntentionalPanicAntigen` declaration
- A function `intentional_panic_site` that *deliberately* exhibits the
  pattern (it's test scaffolding that wants to panic)
- An `#[antigen_tolerance(IntentionalPanicAntigen, rationale = "...")]`
  acknowledging the match is intentional

**What to learn**:
- `rationale` is required at parse time (ADR-011)
- Tolerance ≠ immunity. Tolerance says "the failure-class is present
  and we accept it"; immunity says "the failure-class is structurally
  prevented"
- The `until` field can mark expected expiry, but isn't enforced
  automatically in v0.1
- Tolerance scales: a fixture deliberately constructing a panic case
  doesn't need to be refactored; it needs to be acknowledged

**Try this**:
```sh
cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
```

In the scan output, the tolerated site appears in the `tolerated
sites` count, not in the unaddressed-presentations list. The audit
reports tolerance status separately from immunity status.

---

## Lesson 4 — `descended_from.rs`: inheritance and propagation

**File**: [`antigen/examples/descended_from.rs`](../antigen/examples/descended_from.rs)

**Concept introduced**: failure-class taxonomy through inheritance.
`#[descended_from]` declares structural inheritance between
failure-classes; presentations propagate from parent to descendant
through the inheritance chain.

**What's in the file**:
- A parent `MemoryUnsafetyClass` antigen
- A child `UseAfterFreeClass` antigen with
  `#[descended_from(MemoryUnsafetyClass)]`
- Code marked with `#[presents(UseAfterFreeClass)]` — which also
  structurally presents `MemoryUnsafetyClass` via inheritance
- An immunity claim on the child antigen with explicit re-attestation

**What to learn**:
- Inheritance does NOT transitively claim immunity (per ADR-005
  sub-clause F) — descendants must *re-attest*
- Cycle detection guards `#[descended_from]` chains (ATK-A3-002)
- Diamond inheritance is dedup'd correctly (ADR-018 ProvenanceEntry)
- This is how a failure-class taxonomy grows: name the family, then
  name the specific variants

**Try this**:
```sh
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

Look for the `inherited-presentation-not-re-attested` audit hint —
this is the discipline catching sites that inherited a presentation
via descended_from without their own immunity claim.

---

## Lesson 5 — `phantom_witness.rs`: FormalProof tier through the type system

**File**: [`antigen/examples/phantom_witness.rs`](../antigen/examples/phantom_witness.rs)

**Concept introduced**: phantom-type witnesses. The strongest witness
tier antigen recognizes in v0.1.0-rc.1 — proofs encoded in the type
system itself, with the *type structure* serving as the witness.

**What's in the file**:
- A `DropPanicClass` antigen
- A `NonPanickingProof<T>` phantom-type with a private `_seal` field
  and a sealed `verified()` constructor
- A `PhantomVerifiedDropImpl` type marked
  `#[presents(DropPanicClass, proof = NonPanickingProof::<PhantomVerifiedDropImpl>::verified)]`
  (the phantom-type proof rides on `#[presents(..., proof=)]` per ADR-029)

**What to learn**:
- The turbofish syntax (`Foo::<T>::ctor`) is what antigen recognizes
  as a phantom-type witness
- Audit reports FormalProof tier with `PhantomTypeShapeRecognized`
  hint
- The proof is structural: if the code compiles, the proof holds
  (the sealed constructor cannot be bypassed)
- This is the most rigorous form of antigen-recognized witness in
  v0.1 (Execution tier reserved for A4-A5 harness invocation;
  external-tool witnesses sit at Reachability with disambiguating
  hints until A4-A5 lands)

**Try this**:
```sh
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

Look at the audit's confirmed-claims section. The phantom-type
witness appears at `tier = FormalProof, hint =
PhantomTypeShapeRecognized` — the structural memory says immune; the
audit confirms the proof structure is recognized.

---

## Lesson 6 — `substrate_witness.rs`: discipline-witness via sidecar

**File**: [`antigen/examples/substrate_witness.rs`](../antigen/examples/substrate_witness.rs)

**Concept introduced**: substrate-witness predicates. Some disciplines (signed-zero preservation across odd functions, structural invariants reviewed against external math literature) can't be witnessed by a single in-tree function — the verification lives in a *human review* recorded as a sidecar file. The substrate-witness pipeline (ADR-019) makes that review checkable at audit time.

**What's in the file** (the story is in the docstring; read it):
- A `SignedZeroDiscipline` antigen for the class "every odd function must preserve sign at signed zero"
- Two implementations: `signed_zero_preserving_sinh` (correct) and `naive_sinh_loses_sign_at_zero` (the bug)
- A `#[presents(SignedZeroDiscipline, requires = all_of([signers(required = [...]), fresh_within_days(180)]))]` site — the substrate-witness predicate rides on `#[presents]` (ADR-029); audit observes whether the sidecar satisfies it
- The `requires` predicate names what the sidecar file must contain for the defense to be credited at audit time

**What to learn**:
- Substrate-witness leaves: `signers(required = [...])`, `fresh_within_days(N)`, `ratified_doc(path = ...)`, `oracles_complete(files = [...])`, `signed_trailer(...)`
- Combinators: `all_of`, `any_of`, `not`
- The sidecar lives at `.attest/<AntigenName>.json` co-located with the declaration
- Audit tier climbs from `None` → `Reachability` → `Execution` as the sidecar gets scaffolded, then signed **by the required signer** (here `alice`, in the `math-researcher` role) — signing as anyone else leaves the gap open *by design* (see "Try this" below)
- This is how to verify disciplines that have NO in-tree witness function (e.g., "I reviewed this against Higham §6.3")

**Try this** — sign as yourself first and watch the gap *persist*, then sign as the required reviewer and watch it close:
```sh
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
# Scaffold the sidecar (auto-fills the fingerprint from the scan; prints the exact sign command):
cargo run --bin cargo-antigen -- antigen attest scaffold --antigen SignedZeroDiscipline --source-file antigen/examples/substrate_witness.rs --item-path signed_zero_preserving_sinh

# 1) Sign as yourself — then re-audit: the gap is STILL open.
cargo run --bin cargo-antigen -- antigen attest sign --sidecar antigen/examples/.attest/SignedZeroDiscipline.json --item-path signed_zero_preserving_sinh --signer you --fingerprint <fingerprint-from-scaffold-output>
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
#   → substrate_witness.rs:44  SignedZeroDiscipline — substrate-gap   (signing as `you` does NOT satisfy it)

# 2) Sign as the *required* reviewer — alice, in the math-researcher role — then re-audit: now it climbs.
cargo run --bin cargo-antigen -- antigen attest sign --sidecar antigen/examples/.attest/SignedZeroDiscipline.json --item-path signed_zero_preserving_sinh --signer alice --role math-researcher --fingerprint <fingerprint-from-scaffold-output>
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
#   → substrate_witness.rs:44  SignedZeroDiscipline — defended at Execution
```

> **Why the gap persists under `--signer you`**: the `requires` predicate is
> `all_of([signers(required = ["alice"], roles = {alice = "math-researcher"}), ratified_doc("docs/disciplines/ieee754-odd-functions.md", min_version = "1.0"), fresh_within_days(180)])`.
> An arbitrary signer satisfies *none* of the `signers` leaf — antigen refuses to credit
> self-attestation by a non-required reviewer. The defense reaches `Execution` only when the
> *named* reviewer (`alice`, a `math-researcher`) signs against the *ratified* discipline doc
> within the freshness window. That refusal is the whole point of a substrate-witness: the gap
> states exactly what real review it demands, and no shortcut closes it.

---

## Lesson 7 — `oracle_lifecycle.rs`: Oracle 5-state artifact lifecycle

**File**: [`antigen/examples/oracle_lifecycle.rs`](../antigen/examples/oracle_lifecycle.rs)

**Concept introduced**: Oracle artifacts. When your discipline depends on an *external reference* (a published paper, a ratified ADR, a versioned spec), you want signers to attest against that exact reference — not a free-text URL that goes stale. Oracle records first-class the reference + its stewardship + its lifecycle state.

**What's in the file**:
- Oracle declared via `cargo antigen oracle declare ...` with steward + provenance
- A presents-site with `#[presents(..., requires = oracles_complete(files = ["higham-2002-section-6-3"]))]` (the Oracle-backed substrate-witness predicate; ADR-029 idiom)
- Lifecycle transitions: Draft → Complete (signers attest the Oracle's content matches the reference) → Deprecated (the reference still exists but newer guidance supersedes) → Retired / Revoked

**What to learn**:
- Oracle 5-state machine (Draft / Complete / Deprecated / Retired / Revoked + Reopened)
- `oracles_complete(...)` predicate checks Oracle state at audit time
- Oracle records have signers (who attested), stewards (who maintains the reference link), and provenance
- The audit treats `Complete` as the load-bearing tier; `Deprecated` triggers a "use with caution" hint; `Retired`/`Revoked` fail audit
- This closes the "URLs go stale" problem at the substrate level

**Try this**:
```sh
# Run the example to read the lifecycle walkthrough narrative:
cargo run --example oracle_lifecycle --package antigen
# List any oracle records in the workspace (starts empty; see the example's walkthrough for declare → complete → status):
cargo run --bin cargo-antigen -- antigen oracle list --root antigen/examples
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

---

## Lesson 8 — `delta_attestation.rs`: chained signatures with anti-laundering

**File**: [`antigen/examples/delta_attestation.rs`](../antigen/examples/delta_attestation.rs)

**Concept introduced**: delta-chained attestations. When a function body changes (e.g., refactor that preserves the signed-zero discipline), the reviewer can sign a `Delta` saying "I reviewed fp-A → fp-B and it preserves the invariant" — avoiding a full re-review while keeping the signature chain auditable. ADR-019 §M3 + adversarial T2-R safeguards prevent laundering.

**What's in the file**:
- `NumericStabilityDiscipline` antigen on `stable_kahan_sum`
- Fresh signature against `fp-A`; refactor produces `fp-B`; reviewer signs a Delta(`fp-A`, `fp-B`)
- Three-layer anti-laundering safeguards (per adversarial T2-R)
- Demonstrates that the audit collapses the chain to current-state signers but preserves the provenance trail

**What to learn**:
- `SignerBasis` distinguishes Fresh vs Delta in the sidecar
- Delta has anti-laundering safeguards: bounded chain length, fingerprint-pinning, signer-identity-binding
- When the chain breaks (e.g., body changes substantively), the next signature MUST be Fresh
- The audit reports `chain_depth` as part of `SignatureStrength`
- This solves the "small refactor invalidates all my signatures" problem WITHOUT enabling rubber-stamp laundering

**Try this**:
```sh
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
# Look for the chain_depth field on NumericStabilityDiscipline's audit entry
```

---

## Lesson 9 — `tolerance_attested.rs`: vibes-grade vs sidecar-attested tolerance

**File**: [`antigen/examples/tolerance_attested.rs`](../antigen/examples/tolerance_attested.rs)

**Concept introduced**: attested tolerance. Tolerance comes in tiers. A one-line `rationale = "fine for now"` is vibes-grade. The same antigen tolerated with a sidecar capturing an hour-long review with a math expert is qualitatively stronger evidence. ADR-019 §tolerance tier surfaces this distinction.

**What's in the file**:
- An `UncheckedRecursion` antigen
- `walk_config_tree_vibes_grade` — tolerated with one-line rationale
- `newton_iterate_sidecar_attested` — tolerated with a sidecar capturing the review
- Side-by-side comparison in audit output

**What to learn**:
- Both forms are tolerance (both opt out of immunity); audit treats them differently
- Vibes-grade tolerance reports at `Reachability` tier with `RationaleOnly` hint
- Sidecar-attested tolerance reports at `Execution` tier (or higher with signer attestation)
- The discipline scales: teams can require sidecar-attested tolerance for certain antigens via workspace config
- This closes the "tolerance is a back door" problem — tolerance can be as strong as immunity when the evidence justifies it

**Try this**:
```sh
cargo run --bin cargo-antigen -- antigen tolerate list --root antigen/examples
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

---

## Lesson 10 — `supply_chain_content_hash.rs`: proactive content-hash attestation workflow

**File**: [`antigen/examples/supply_chain_content_hash.rs`](../antigen/examples/supply_chain_content_hash.rs)

**Concept introduced**: supply-chain defense via substrate-witness. The `ContentHashMismatch`
antigen defends against the 2025 chalk/debug/eslint-config attack: content replacement at
a fixed Cargo.lock version. The defense requires *proactive first-attestation* — you must
record the expected hash before it can detect divergence.

**What's in the file**:
- `ContentHashMismatch` stdlib antigen import from `antigen::stdlib::supply_chain`
- A service function marked `#[presents(ContentHashMismatch)]` (vulnerable: uses a dep
  that hasn't been content-hash attested yet)
- An immune version with `requires = content_hash_matches("serde", "1.0.200")` substrate
  witness — claims immunity once the first-attestation sidecar exists
- Comments walking through the workflow: `cargo antigen verify content-hash record` → 
  first-attestation sidecar created → audit passes → `cargo antigen verify content-hash`
  for subsequent checks

**What to learn**:
- Cargo.lock pins VERSION but not CONTENT-HASH; lockfile pinning alone doesn't prevent
  this attack class
- The `content_hash_matches(crate, version)` substrate-witness leaf backs `ContentHashMismatch` immunity
- `content-hash-no-attestation` hint fires before first-attestation; `content-hash-mismatch`
  fires if the sidecar hash and current Cargo.lock diverge
- Named limitation: v0.2 hash-source is the Cargo.lock checksum; crates.io tarball
  verification is v0.3+

---

## Lesson 11 — `supply_chain_unpinned.rs`: exact-pin enforcement

**File**: [`antigen/examples/supply_chain_unpinned.rs`](../antigen/examples/supply_chain_unpinned.rs)

**Concept introduced**: substrate-witness over Cargo.toml dep specs. The `UnpinnedDependency`
antigen fires on any dep without `=X.Y.Z` exact-pin specifier.

**What's in the file**:
- A service with range-pinned deps (`^1.x`) marked `#[presents(UnpinnedDependency)]`
- An immune version with `requires = dep_pinned()` substrate witness
- The NARROW `UnpinnedTransitiveDependency` definition demonstrated:
  CORRECT = "direct dep with `*/?` for its own deps" (fires here);
  INCORRECT = "any transitive dep with non-exact pins" (would 100% false-positive)

**What to learn**:
- `dep_pinned()` leaf checks all deps; `dep_pinned("serde")` checks a single dep
- The NARROW definition of `UnpinnedTransitiveDependency` is load-bearing — the wide
  definition has ~100% false-positive rate (Cargo.lock resolution makes most transitive
  deps stable despite non-exact upstream specs)
- `cargo antigen verify dep-pin` pins unpinned deps in one sweep

---

## Lesson 12 — `convergent_diagnostic.rs`: multi-modality independence discipline

**File**: [`antigen/examples/convergent_diagnostic.rs`](../antigen/examples/convergent_diagnostic.rs)

**Concept introduced**: convergent evidence via `#[diagnostic]`. Multiple *independent*
witness classes converge on a defense claim — independence means distinct `WitnessClass`
CATEGORIES, not raw witness count.

**What's in the file**:
- A `#[diagnostic(modalities = [WitnessClass::PropertyTest, WitnessClass::FormalVerification], min_independent = 2)]`
  annotation asserting two-category convergent evidence
- A class-collapse case: `[WitnessClass::StaticAnalysis, WitnessClass::StaticAnalysis]` with
  `min_independent = 2` — two witnesses of the same class don't satisfy 2-class independence
  (compile error at parse time, per ADR-024 C1)
- The six `WitnessClass` variants and when to use each

**What to learn**:
- `min_independent` counts distinct CATEGORIES, not witnesses.
  Running clippy three times doesn't add evidence — it's still one `StaticAnalysis` class.
- Parse-time error prevents vacuously-unsatisfiable claims
- Audit hint `diagnostic-modalities-class-collapsed` fires on pre-compiled code that
  bypassed parse-time enforcement (defense in depth)

---

## Lesson 13 — `convergent_clonal.rs`: iterated witness with non-deterministic seed

**File**: [`antigen/examples/convergent_clonal.rs`](../antigen/examples/convergent_clonal.rs)

**Concept introduced**: `#[clonal]` for iterated witness evaluation with explicit
iteration count and non-deterministic seed discipline.

**What's in the file**:
- `#[clonal(witness = property_test_fn, iterations = 1_000, seed = SeedKind::Random)]` —
  claims 1000 independent randomized iterations of a property test
- A rejected case: `seed = SeedKind::Fixed(42)` → COMPILE ERROR (fixed seed makes
  "independent iterations" a contradiction; per ADR-024 C2)
- `SeedKind` variants explained: `Random`, `EntropyFromCi`, `TimestampSeeded`, `Fixed(u64)`

**What to learn**:
- `SeedKind::Fixed(_)` is rejected at parse time (same mechanism as `#[immunosuppress]`
  duration-cap enforcement)
- `iterations` is required and must be > 0
- The `clonal-iterations-below-threshold` audit hint fires when `iterations` is below
  the workspace floor (default 100)

---

## After all the lessons

By now you've encountered the core vocabulary, four witness tiers, substrate-witness
pipeline, Oracle lifecycle, delta-chained signatures, tolerance tiers, and the full
v0.2 family surface:

| Lesson | Concept |
|---|---|
| 1 — basic | declare, present, immune (three core moves) |
| 2 — broken_witness | audit-tier-honesty + None tier |
| 3 — antigen_tolerance | explicit tolerance + required rationale |
| 4 — descended_from | inheritance + re-attestation discipline |
| 5 — phantom_witness | FormalProof tier via type-system proof |
| 6 — substrate_witness | substrate-witness predicates + sidecar pipeline |
| 7 — oracle_lifecycle | Oracle 5-state artifact lifecycle + stewardship |
| 8 — delta_attestation | chained signatures + anti-laundering safeguards |
| 9 — tolerance_attested | sidecar-attested vs vibes-grade tolerance |
| 10 — supply_chain_content_hash | proactive content-hash attestation workflow (ADR-025) |
| 11 — supply_chain_unpinned | exact-pin enforcement + NARROW transitive definition (ADR-025) |
| 12 — convergent_diagnostic | multi-modality independence + WitnessClass discipline (ADR-024) |
| 13 — convergent_clonal | iterated witness + SeedKind::Fixed rejection (ADR-024) |
| 14 — deferred_defense_orient | `#[orient]` + `#[triage_commit]` speech-act contrast (ADR-023/026) |
| 15 — recurrent_emergence | failure-class return through structural similarity (ADR-022) |
| 16 — mucosal_boundary | boundary defense + delegate centralization (ADR-027) |
| 17 — vcs_info_loss | git history as immune substrate — four erasure patterns (ADR-026) |
| 18 — agentic_coordination | session/agent boundary SubstrateAlignment failures (ADR-028) |
| 19 — antigen_category | SubstrateAlignment vs FunctionalCorrectness taxonomy (ADR-028) |
| 20 — triage_commit | decisional rollback — 5-color scale + orient contrast (ADR-026) |
| 21 — prescriptive_board | work-orchestration — four-valued `WorkVerdict` board, *code IS the board* (ADR-033) |

**Where to go next**:

- [`tutorial.md`](tutorial.md) — the full first-15-minutes guided
  walkthrough; covers the same concepts but in a single narrative
  flow with one specific failure-class
- [`usage-patterns.md`](usage-patterns.md) — pattern recipes for
  applying these concepts to real failure-classes (composition-
  boundary antigens, version-boundary recognition, cross-crate
  patterns, etc.)
- [`anti-patterns.md`](anti-patterns.md) — common mistakes when
  adopting antigen, with the structural reason each is wrong
- [`macros.md`](macros.md) — full reference for all five macros
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint
  DSL for authoring your own antigens
- [`witness-tiers.md`](witness-tiers.md) — deeper treatment of all
  four witness tiers + audit hint enumeration

---

## What the examples deliberately don't cover

Per the recognition-not-design discipline (ADR-006), the examples
demonstrate concepts using **substrate-grounded failure-classes** —
shapes that have actually appeared in real Rust codebases. They don't
demonstrate speculative or hypothetical patterns.

If you're looking for examples of failure-classes specific to your
domain, the right move is to *recognize* them in your own codebase
first (or its dependencies), then declare antigens that match what
you've encountered.

The examples in `antigen/examples/` are minimal-but-real. They aren't
comprehensive — they're the floor that demonstrates the vocabulary.
Your team's antigens will look different in surface; the shape of
declaration + presentation + immunity is the same.

---

## A note on the basic.rs cross-reactivity

If you run `cargo antigen scan --root antigen/examples`, you'll see
that `DemoBrokenWitness`'s fingerprint (`name = matches("Looks*")`)
fires fingerprint matches in `broken_witness.rs` deliberately, but
doesn't widely cross-react across the other example files because
its fingerprint is narrowly scoped to names starting with "Looks".

This is by design. Earlier versions of `broken_witness.rs` had a
broader fingerprint (`name = matches("*")`) that cross-reacted with
sites in `basic.rs` — demonstrating exactly the *recall-tuned filter*
property (per ADR-010 Amendment 4). False positives from broad
fingerprints are expected; the discipline is to narrow fingerprints
or tolerate matches explicitly.

The example was tightened to avoid confusing newcomers. If you want
to see the broad-fingerprint behavior, change `matches("Looks*")` to
`matches("*")` and re-run scan.

---

## Running everything

To see the full picture:

```sh
# Scan all examples at once
cargo run --bin cargo-antigen -- antigen scan --root antigen/examples

# Audit all examples at once
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

You'll see all five examples' antigens, presentations, immunities,
tolerances, fingerprint matches, and lineage edges in one report.
That's a small-scale version of what you'd see running scan against
a real codebase with declared antigens.

---

## Deferred-Defense Family (v0.2.0-alpha.1) — ADR-023

These four examples demonstrate the loudness-as-discipline family: primitives
for intentional non-immunity. Each has a structurally distinct posture with
parse-time enforcement.

### Lesson: `#[anergy]` — time-bounded deferral with co-stimulation trigger

**File**: `antigen/examples/deferred_defense_anergy.rs`

`#[anergy]` is for "I know this failure-class applies, and I cannot address it
right now, and here is my time-bound and the trigger that will re-engage." The
`until` field is REQUIRED — anergy without a time-bound is silent tolerance.

Key enforcement: `reason` minimum 20 characters; `until` required (A5 absorbed);
past `until`: hint escalates to `anergy-co-stimulation-not-arrived` / `anergy-stale`.

```sh
cargo run --example deferred_defense_anergy --package antigen
```

### Lesson: `#[immunosuppress]` — surgical silencing with parse-time duration cap

**File**: `antigen/examples/deferred_defense_immunosuppress.rs`

`#[immunosuppress]` is for "I am deliberately muting this check family for a
bounded duration." Duration cap enforced at **parse time** — compile error if
`until - since > duration_cap` (A4 absorbed). Default cap 90d; `duration_cap = N`
overrides per-site.

Key enforcement: `rationale` minimum 20 characters; `until` required; compile error
on cap violation.

```sh
cargo run --example deferred_defense_immunosuppress --package antigen
```

### Lesson: `#[poxparty]` — controlled exposure with structural isolation

**File**: `antigen/examples/deferred_defense_poxparty.rs`

`#[poxparty]` is for chaos tests, fault injection, red-team exercises. Structural
isolation via `antigen-poxparty` Cargo feature — items inside inactive
`#[cfg(feature = "antigen-poxparty")]` blocks never reach macro expansion. Feature
MUST NOT be in default set.

Key enforcement: `exercise_type` minimum 20 characters; `until` required;
`#[cfg]` gate is primary isolation; `CARGO_FEATURE_ANTIGEN_POXPARTY` env var
check is secondary (best-effort; A3 absorbed).

```sh
# Production (structural isolation active):
cargo run --example deferred_defense_poxparty --package antigen
# Exercises visible:
cargo run --example deferred_defense_poxparty --package antigen --features antigen-poxparty
```

### Lesson: `#[orient]` — acknowledged orientation with explicit path-out

**File**: `antigen/examples/deferred_defense_orient.rs`

`#[orient]` is for "I acknowledge this gap. I will close it by `<date>`." Both
`learning_path` (the explicit path-out) and `until` (the horizon) are **REQUIRED**
per ADR-023 Option-A — a bare `#[orient]` with no arguments is a compile error.
An orientation without a path-out and time-bound is silent deferred non-immunity,
which is just tolerance; `#[orient]` exists to be loud about it.

For decisional rollback sites, use `#[triage_commit]` (ADR-026), not `#[orient]`.
The distinction: orient = deferral; triage_commit = decision. Both are shown as
Form 3 (orient) and Form 4 (triage_commit) in `deferred_defense_orient.rs`.

```sh
cargo run --example deferred_defense_orient --package antigen
```

---

## Recurrent-Emergence Family (v0.2.0-alpha) — ADR-022

### Lesson: `recurrent_emergence` — the return of solved problems

**File**: `antigen/examples/recurrent_emergence.rs`

Recurrent failures are failure-classes that return after being solved — because the
defense is re-introduced in a refactored form, or because the structural condition
that produced them recurs. `#[descended_from]` propagates the failure-class memory
through structural similarity; if a new type is flagged as descended from a type
that `#[presents]` a known failure-class, the new type is automatically scanned.

Key concept: **the immune system's memory must survive refactors**. If a function
that presents `SignedOverflow` is rewritten into a new struct, the new struct
carries the same failure risk — but without `#[descended_from]`, the connection
is lost. This is the structural root of re-introduced bugs.

```sh
cargo run --example recurrent_emergence --package antigen
```

---

## Mucosal-Boundary Family (v0.2.0-alpha) — ADR-027

### Lesson: `mucosal_boundary` — defense at the boundary, not the interior

**File**: `antigen/examples/mucosal_boundary.rs`

Mucosal discipline says: sanitize at the boundary, not inside. The `#[mucosal]`
marker declares that a function is a boundary function that sanitizes inputs.
Functions inside the boundary don't need to re-sanitize; functions OUTSIDE the
boundary must never be called without going through the mucosal function first.

`#[mucosal_delegate]` centralizes delegation: if you have a mucosal handler for
`XssInHtmlOutput`, all delegate sites point to a single handler that owns the
discipline. Cross-crate delegate targets that exist but can't be reached by the
intra-crate index are a known limitation (see `DelegateCrossCrateResolutionGap`
in `agentic_coordination.rs`).

```sh
cargo run --example mucosal_boundary --package antigen
```

---

## VCS-Information-Loss Family (v0.2.0-alpha) — ADR-026

### Lesson: `vcs_info_loss` — git history as immune substrate

**File**: `antigen/examples/vcs_info_loss.rs`

Git operations that rewrite or erase history remove the structural memory of WHY
decisions were made. `git reset --hard`, `git push --force`, squash-merges, and
unrecorded rollbacks are the most common vectors. These are `SubstrateAlignment`
failures: the git-history representation diverges from the actual state of
why-this-was-done.

The four patterns covered:

| Pattern | Antigen | Defense |
|---------|---------|---------|
| Force-reset without record | `RollbackWithoutTriageCommit` | `#[triage_commit]` + `Triage-Decision:` trailer |
| Force-push erasing history | `ForcePushErasingHistory` | `Force-Push-Attestation:` trailer |
| Refactor losing context | `RefactorWithoutPreservationOfWhy` | `Preserves-Why:` trailer |
| Squash-merge losing trail | `SquashMergeLosingIntermediateState` | preserve-branch or merge commit |

The substrate-witness for all three trailer patterns is `signed_trailer(key = "...")` —
the `requires =` predicate that evaluates whether the named trailer is present.

The biology cognate for `ForcePushErasingHistory` is immune amnesia (Mina et al. 2015,
Science) — measles infects memory lymphocytes, erasing immunological memory. Force-push
erases commit memory. The structural rhyme is the central insight of the family.

```sh
cargo run --example vcs_info_loss --package antigen
```

---

## Agentic-Coordination Family (v0.2.0-alpha) — ADR-028

### Lesson: `agentic_coordination` — failures at session and agent boundaries

**File**: `antigen/examples/agentic_coordination.rs`

Multi-session, multi-agent, and human-LLM-collaboration workflows produce
`SubstrateAlignment` failures that are rare in single-developer, single-session
work. Two patterns:

**`AgentWakeWithoutSubstrateDeltaInjection`**: an agent that resumes from a context
snapshot without first reading the substrate delta (git log, camp status, pending
work) will route stale claims. The fix is the `camp wake` + `git log` discipline,
enforced via `ratified_doc(path = "docs/agentic-wake-protocol.md")`.

**`DelegateCrossCrateResolutionGap`**: a mucosal audit that resolves delegate
handlers using an intra-crate index silently produces false `MucosalDiscipline
DelegateTargetMissing` for cross-crate handlers that exist but aren't reachable
by the index.

Both are `SubstrateAlignment` category: the failure is in what the agent BELIEVES
is true, not what the code COMPUTES. The computation is correct given its inputs;
the inputs are stale.

```sh
cargo run --example agentic_coordination --package antigen
```

---

## Antigen-Category (v0.2.0-alpha) — ADR-028

### Lesson: `antigen_category` — SubstrateAlignment vs FunctionalCorrectness

**File**: `antigen/examples/antigen_category.rs`

Every antigen has a `category` field that classifies HOW the failure-class fires.
The two categories shape witness type, audit layer, and responder role:

**`FunctionalCorrectness`**: the verb produces the wrong output. Evidence is
behavioral — a test, proptest, formal proof, or lint exercises the verb. Use
`witness =`. Example: `NanInCleanedOutput`.

**`SubstrateAlignment`**: a representation diverges from actual state. Evidence
lives outside the code — a sign-off, a ratified doc, an un-reviewed record. Use
`requires =`. Example: `UnsignedSecurityPolicy`.

The quick test: *can a test exercise the thing you're defending?* If yes →
`FunctionalCorrectness` + `witness =`. If no → `SubstrateAlignment` + `requires =`.

```sh
cargo run --example antigen_category --package antigen
```

---

## Triage-Commit (v0.2.0-alpha) — ADR-026

### Lesson: `triage_commit` — decisional rollback as a speech-act

**File**: `antigen/examples/triage_commit.rs`

`#[triage_commit]` is the speech-act that turns a rollback function into a chart
entry. It carries five required fields (all compile-time enforced):

| Field | What it records |
|-------|----------------|
| `triage_decision` | Color (Black/Red/Yellow/Green/White) |
| `rollback_target` | SHA of last-known-good snapshot |
| `triaged_by` | Identity of the person/role who diagnosed |
| `rationale` | Chart-documentation (>= 20 chars) |
| `rollback_due_within_minutes` | Bounded time window (> 0) |

The 5-color scale: Black = system-down; Red = vital-metric regression; Yellow =
decision pending; Green = no regression (non-action documented); White = out of
scope (non-action documented). All five are valid outcomes — Green and White make
invisible non-action decisions visible in the git substrate.

```sh
cargo run --example triage_commit --package antigen
```

---

---

## Prescriptive / Work-Orchestration Family (v0.3) — ADR-033

### Lesson: `prescriptive_board` — code IS the Asana board

**File**: [`antigen/examples/prescriptive_board.rs`](../antigen/examples/prescriptive_board.rs)

**Concept introduced**: the prescriptive family expresses code-site-local
*work-needs* directly in the type system, and `cargo antigen audit` renders them
as a live board section. A `// TODO` rots; a `#[panel(needs = [...], filled_by =
[...], due = "...")]` stays current or emits a loud verdict when it doesn't.
*Code IS the board* (ADR-033).

The family is eight clinical-named macros over four structural shapes:

| Shape | Macros | Models |
|---|---|---|
| S1 Role-workflow | `#[panel]`, `#[rx]`, `#[refer]`, `#[biopsy]` | ordered who-steps to fill + review |
| S2 Elimination | `#[ddx]` | alternatives to rule out one by one |
| S3 Ordering | `#[triage]` | a re-validatable priority order over code sites |
| S4 Frame-only | `#[culture]`, `#[quarantine]` | a temporal window with an expiry |

**What's in the file**: a tiny config-parser module whose real items carry the
work-needs. The macros are chosen so each verdict is **deterministic and stable
across calendar time** — no row silently flips as the clock advances (the file's
closing comment explains how). The audit board shows:

- **`Overdue`** (loud, sorted to top) — a `#[quarantine]` whose `until` is in the
  past with no release attestation. Frame-expiry alone never fulfills a
  quarantine (the positive-closure guard, ATK-PRES-13).
- **`Pending`** (the expected, quiet state) — a `#[culture]` with no frame (an
  open observation window) and a `#[triage]` over resolvable code sites awaiting
  a `triaged_by` attestation.
- **`OutOfFrame`** (advisory — un-evaluable, *not* an alarm), three rows, each a
  *different* typed sub-cause with its own remedy:
  - `#[refer]` to a who-ref with no signed sidecar → `unknown-who-ref`
  - `#[panel]` with no who-step declared → `missing-work-step`
  - `#[quarantine]` with an unparseable `until` string → `unparseable-frame`

**What to learn**:
- The four-valued `WorkVerdict` is the defense tri-state with the unsatisfied
  cell *temporally split* by the frame (`undefended` → `Pending` within frame +
  `Overdue` past it; `substrate-gap` → `OutOfFrame`). One evaluator, one
  substrate read, a frame-aware projection — no parallel mechanism (ADR-033
  §Decision 3).
- `Overdue` (late, but gradable) and `OutOfFrame` (un-evaluable) are **never
  collapsed** (the three-valued-logic gem, ATK-PRES-8). An un-evaluable need
  routes a different fix than a late one — hence the typed sub-cause + remedy.
- The board is a **live projection** (ADR-034): recomputed every run, never
  stored, so it cannot drift the way a `// TODO` or an external tracker does.
- `#[triage]` orders **code-site references**, not camp campsites (anchor #3 —
  the audit never reads camp). The locality test is the antigen ↔ camp boundary:
  *if this code site vanished, does the work-need vanish with it?* Yes → antigen;
  no → camp.

**Try this**:
```sh
cargo run --example prescriptive_board --package antigen
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

In the audit output, find the `── Work board (ADR-033)` section. The count line
reports `1 overdue, 3 out-of-frame, 2 pending, 0 fulfilled`; the `OVERDUE` row is
marked `‼` and sorted first. Each `out-of-frame` row names its `cause:` and the
matching `remedy:`. The `#[triage]` row shows `✓ priority_order:` for each
resolvable code site — change one entry to a function that does not exist and
re-run: the whole triage drops to `out-of-frame` / `unresolvable-ref`, never
silently satisfied.

**Reaching `Fulfilled`**: the example documents (but does not ship) how a
work-need closes — add the closing who-steps (`filled_by` / `reviewed_by`),
scaffold + sign the `.attest/<item>.json` sidecar (the same pipeline defense
attestation uses), and the row flips to `fulfilled`. Satisfaction is
fingerprint-pinned (NFA-21): edit the item's body and the row drops back to
`pending` because the signed review is now stale — which is the silent-wrong-review
bug this family exists to surface.

For the full per-macro argument reference, see [`macros.md`](macros.md) — the
prescriptive family section.

---

## beta.2 Failure-Class Families (v0.3.0-beta.2) — ADR-039/040/041

These seven examples each demonstrate one **beta.2 stdlib failure-class family**:
a real Rust footgun, declared as a `#[antigen]`, with an **affinity-pair** exhibit
— a `BAD` site that *binds* the fingerprint next to a `GOOD` sibling the
fingerprint does **not** bind. Read the two side by side in the *source* to see the
structural difference. For the catalog view (every member's tier + fingerprint +
what it catches), see [`stdlib-families.md`](stdlib-families.md).

> **One honest caveat about what the console shows** (so the examples don't
> surprise you). These examples put `#[presents(...)]` on **both** the bad and the
> good sibling — deliberately, to teach the affinity-pair (the `basic.rs` Lesson-1
> model: a site *presents* the shape, it doesn't claim immunity). An explicit
> `#[presents]` is an author declaration that surfaces in `scan`/`audit`
> **regardless of whether the fingerprint matched** — so in the audit output you
> see *both* siblings listed, not the good one vanishing. "Spared" in these lessons
> means *the fingerprint doesn't bind it* (true, and visible in the source), **not**
> *it disappears from the console* (which only happens for an **un-marked** sibling
> — e.g. `drop_panic`'s `NotReallyDrop`). Each lesson's "Try this" tells you what
> the console actually shows.
>
> **To *watch the fingerprint itself separate clean from dirty*, read the guard
> tests** — they're clearer than the examples, because each family's `_binds_` and
> `_spares_` cases sit side by side with plain-English rationale:
> [`antigen/tests/stdlib_family_fingerprints.rs`](../antigen/tests/stdlib_family_fingerprints.rs)
> has a pair for *every* family — e.g.
> `unbounded_deserialization_binds_from_reader_call` (the bad shape matches) beside
> `unbounded_deserialization_spares_from_slice_namesake` (the clean namesake
> doesn't) — and
> [`antigen/tests/spares_namesake_contract.rs`](../antigen/tests/spares_namesake_contract.rs)
> pins the harder namesake cases. Running an example shows *class-level* defense
> (both siblings present); **reading** these tests shows the *fingerprint-level*
> bind/spare distinction the console can't (`cargo test` only confirms they hold).

A recurring beta.2 idea worth holding onto as you read: **the tier is the honesty
dial.** A `named` member promises "if it doesn't fire, you're covered"; a
`suspected` member is a *correlator* that may also fire on idiomatic-correct code
(a labeled recall hole, by design). Several examples below show *why* a member
sits where it does — usually a **clean-sibling rule** (a needle that would fire on
the recommended fix is dropped or demoted, never shipped at named).

---

## Lesson 22 — `deserialization.rs`: the trust-boundary deep tier (two affinity-pairs)

**File**: [`antigen/examples/deserialization.rs`](../antigen/examples/deserialization.rs)

**Concept introduced**: the deepest mucosal trust surface — untrusted bytes
crossing into typed structs. Two members, two shapes: an attribute presence-AND-
absence tell and a streaming-call tell.

**What's in the file**:
- `DeserializeWithoutDenyUnknownFields` (**suspected**) with a `LenientConfig`
  that derives `Deserialize` *without* `#[serde(deny_unknown_fields)]` (binds —
  unknown fields silently dropped) next to a `StrictConfig` that sets it (safe sibling — fingerprint-spared in source, still listed)
- `UnboundedDeserialization` (**named**) with `load_unbounded` calling
  `from_reader` with no bound (binds, unaddressed) next to `load_bounded` that
  caps the reader with `.take(1 << 20)` — and a `#[defended_by]` witness proving
  the cap

**What to learn**:
- The **surface-flag / witness-proof split** (design principle): `load_bounded`
  *still presents* the `from_reader` surface (the risky surface is present), so the
  intent is that the *witness* proves the defense at audit, **not** that the
  fingerprint spares the bounded form. A `not(take)` guard would silently suppress
  real DoS sites whenever an unrelated `Iterator::take` appeared — a silent
  false-negative the named tier cannot allow.
- **What audit actually shows here** (be honest with the console): both
  `load_unbounded` *and* `load_bounded` report `✓ defended`, both credited to the
  single `#[defended_by]` witness — because audit credits a witness at the
  **antigen-type** granularity (one witness for `UnboundedDeserialization` marks
  every presents-site of that type defended), not per-site. So this example does
  **not** visibly separate the two sites; the split above is the durable principle,
  and `stdlib-families.md`'s `UnboundedDeserialization` box tracks the per-site
  visibility gap for the tool/example crew.
- Why `from_slice` is *not* in the fingerprint: a slice is bounded, so it isn't an
  unbounded vector — and it fired on the bounded-slice fix itself (ADR-039 §C
  Amd-1, the clean-sibling rule).
- The example crate has no `serde` dependency — the derives are commented
  stand-ins; the scanner reads the `#[derive(...)]` / `from_reader` *token shape*
  syntactically.

**Try this**:
```sh
cargo run --example deserialization --package antigen
cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```
In `audit`, the two `DeserializeWithoutDenyUnknownFields` sites (`LenientConfig`,
`StrictConfig`) both report **undefended** — neither has a witness — so this pair
teaches the *fingerprint* difference in the source, not a console split. The two
`UnboundedDeserialization` sites (`load_unbounded`, `load_bounded`) both report
`✓ defended`, both credited to the one witness (the antigen-type-granular
crediting described above). To see a fingerprint genuinely *spare* a site at the
console, look at `drop_panic`'s un-marked `NotReallyDrop` (Lesson 24): no
`#[presents]`, no match, simply absent. And to *read* this member's bind/spare side
by side, see
[`stdlib_family_fingerprints.rs`](../antigen/tests/stdlib_family_fingerprints.rs) —
`unbounded_deserialization_binds_from_reader_call` (the bad shape matches) right
beside `unbounded_deserialization_spares_from_slice_namesake` (the clean namesake
doesn't).

---

## Lesson 23 — `time_ordering.rs`: the silent-in-tests / panic-in-prod clock footgun

**File**: [`antigen/examples/time_ordering.rs`](../antigen/examples/time_ordering.rs)

**Concept introduced**: a failure-class the test suite *structurally cannot
reach*. The system clock can run backwards (NTP, manual set, VM pause), so
`SystemTime::duration_since(...).unwrap()` panics in production — but never in
tests, because test machines don't NTP-skew mid-test.

**What's in the file**:
- `SystemTimeUnwrapPanic` (**suspected**)
- `age_since_panicking` — reads the clock and `.unwrap()`s the `Result` (binds)
- `age_since_safe` — the same clock read, but `.unwrap_or(Duration::ZERO)` handles
  the backwards-clock case (safe sibling — fingerprint-spared in source, still listed)

**What to learn**:
- Why this is **suspected**, not named: the shipped grammar has no method-chain
  leaf, so the fingerprint is the *co-occurrence* `all_of([duration_since,
  any_of([unwrap, expect])])`. Co-occurrence correlates with the panic-chain but
  doesn't prove it (the `unwrap` could guard an unrelated `Result`), and the
  infallible `Instant::duration_since` shares the name (a receiver-type-only FP
  scan can't resolve). Honest within-tier recall noise.
- Why `elapsed` is **excluded**: it would fire on `Instant::now().elapsed()` —
  but `Instant` is monotonic and `Instant::elapsed()` can't panic-on-skew. That's
  the *"use `Instant` instead of `SystemTime`"* fix — the member's own clean
  sibling. A needle that fires on the fix is dropped at every tier.

**Try this**:
```sh
cargo run --example time_ordering --package antigen
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```
Both `age_since_panicking` and `age_since_safe` appear as `✗ undefended`
`SystemTimeUnwrapPanic` presentations (both `#[presents]`-marked) — the
*fingerprint* difference (the `unwrap` present vs the `.unwrap_or(...)` handled
form) is in the source, per the section caveat above.

---

## Lesson 24 — `drop_panic.rs`: a real `Drop` that can panic (and the v2 precision)

**File**: [`antigen/examples/drop_panic.rs`](../antigen/examples/drop_panic.rs)

**Concept introduced**: panic-during-unwind aborts the process. A panic in `Drop`
while another panic is unwinding skips the destructor's cleanup → leaked resources
even on `panic=unwind`. This is the v2 of `basic.rs`'s `PanickingInDrop`.

**What's in the file**:
- `PanicInDrop` (**named**)
- `PanickyGuard` — a real `impl Drop` that `.unwrap()`s in teardown (binds — the
  *call-shaped* panic the macro-only `PanickingInDrop` missed)
- `SafeGuard` — a real `impl Drop` with a panic-free teardown (safe sibling — fingerprint-spared in source, still listed)
- `NotReallyDrop` — an **inherent** method merely *named* `drop` that `.unwrap()`s
  (spared by `impl_of_trait("Drop")` — this is not the `Drop` trait)

**What to learn**:
- `impl_of_trait("Drop")` is the precision the old fingerprint lacked: it matches
  the *real* `Drop` trait, so `NotReallyDrop`'s inherent `drop` is correctly
  spared (a panic there is an ordinary method panic, not an unwind-abort).
- The fingerprint covers **both** panic shapes — call-shaped (`unwrap`/`expect`)
  AND macro-shaped (`panic!`/`unreachable!`/`todo!`/`unimplemented!`) — because
  `.unwrap()` is the more common teardown panic and a macro-only tell misses it.

**Try this**:
```sh
cargo run --example drop_panic --package antigen
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```
In `audit`, **both** `PanickyGuard` *and* `SafeGuard` appear as
`✗ undefended` `PanicInDrop` presentations — because both are `#[presents]`-marked
(they teach the affinity-pair; neither has a witness). The genuinely *spared* site
is **`NotReallyDrop`**: it has no `#[presents]`, and `impl_of_trait("Drop")`
doesn't bind its inherent `drop`, so it never appears at all — the one place this
family lets you *watch* the fingerprint spare a site at the console. (To see the
SafeGuard/PanickyGuard difference, read the *source* — only `PanickyGuard`'s body
reaches a panic source.)

---

## Lesson 25 — `panic_on_index.rs`: `get_unchecked` — UB, not a panic

**File**: [`antigen/examples/panic_on_index.rs`](../antigen/examples/panic_on_index.rs)

**Concept introduced**: not all out-of-bounds is a clean crash. `get_unchecked` /
`get_unchecked_mut` skip the bounds check, so an out-of-bounds index is
**Undefined Behavior** — silent memory corruption, a soundness hole.

**What's in the file**:
- `GetUncheckedWithoutProof` (**named**)
- `first_unchecked` — reads through a `get_unchecked` call (binds)
- `first_checked` — the bounds-checked `.get(i)` returning `None` (safe sibling — fingerprint-spared in source, still listed)

**What to learn**:
- `get_unchecked` / `get_unchecked_mut` are slice/`Vec`-specific method names with
  no stdlib collision — a clean call-shape, which is why this is **named**.
- The witness is a `// SAFETY:` proof the index is in-bounds + a miri run, OR the
  checked `.get(i)`.
- The **biosafety pattern**: the workspace forbids `unsafe` (`-F unsafe-code`) and
  real `get_unchecked` is `unsafe`, so the example uses a *safe toy* with a method
  *named* `get_unchecked`. The fingerprint anchors on the call *token*, so the
  call-shape is exhibited faithfully without invoking real unchecked indexing.

**Try this**:
```sh
cargo run --example panic_on_index --package antigen
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```
Both `first_unchecked` and `first_checked` appear as `✗ undefended`
`GetUncheckedWithoutProof` presentations (both `#[presents]`-marked, no witness) —
the *fingerprint* difference (`get_unchecked` present vs the checked `.get(i)`)
lives in the source, per the section caveat above.

---

## Lesson 26 — `resource_lifecycle.rs`: explicit leaks that skip `Drop`

**File**: [`antigen/examples/resource_lifecycle.rs`](../antigen/examples/resource_lifecycle.rs)

**Concept introduced**: the *other half* of the Drop-Lifecycle axis. Where
`drop_panic` is "drop fires but explodes," this is "drop never fires" —
`mem::forget` / `Box::leak` / `Vec::leak` deliberately skip `Drop`. Legitimate for
`'static` upgrades; a silent leak if misused. The witness antigen asks for is the
*documented rationale*.

**What's in the file**:
- `DeliberateLeakNotDocumented` (**suspected**)
- `leak_it` — `mem::forget`s a heap `String`, so its `Drop` never runs (binds)
- `use_it` — an ordinary use that lets the value drop at end of scope (safe sibling — fingerprint-spared in source, still listed)

**What to learn**:
- Why **suspected**, not named: `forget` / `leak` are bare common last-segments
  with no narrowing anchor — `body_calls` matches the last segment, so a domain
  `cache.forget()` / `permissions.leak()` also fires. A positive tell at the loud
  named tier would overclaim.
- The **orthogonality lesson**: the *class* is `provenance = Constructable`
  (`mem::forget` demonstrably skips `Drop`), yet this *instance's* dial sits at
  suspected. "How solid is the class" and "how loud is this instance" are
  independent axes (ADR-039).

**Try this**:
```sh
cargo run --example resource_lifecycle --package antigen
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```
Both `leak_it` and `use_it` appear as `✗ undefended`
`DeliberateLeakNotDocumented` presentations (both `#[presents]`-marked) — the
*fingerprint* difference (`forget` present vs absent) is in the source, per the
section caveat above.

---

## Lesson 27 — `numeric_truncation.rs`: the `size_of`-in-element-count foot-cannon

**File**: [`antigen/examples/numeric_truncation.rs`](../antigen/examples/numeric_truncation.rs)

**Concept introduced**: silent numeric corruption. The count arg of
`ptr::copy_nonoverlapping` is in **elements**, not bytes — so
`copy_nonoverlapping(src, dst, n * size_of::<T>())` over-copies by `sizeof(T)` →
out-of-bounds → UB. clippy has a correctness lint for exactly this.

**What's in the file**:
- `SizeOfInElementCount` (**suspected**)
- `copy_bad` — passes `n * size_of::<u32>()` as the count (binds)
- `copy_good` — passes a plain element count `n`, no `size_of` multiplier (safe sibling — fingerprint-spared in source, still listed)

**What to learn**:
- This is the **worked example of tier-honesty as a self-catch**: the member was
  over-claimed at named, then corrected to **suspected** with the fingerprint
  *unchanged* — the fix was the *tier*, not the shape (ADR-039 §C Amd-1).
- Why demoted *and not dropped*: the co-presence `all_of([copy_nonoverlapping,
  size_of])` fires on idiomatic-correct both-calls code too (a byte-buffer copy, a
  separate-bounds `size_of`) — so it can't be named. But its own fix — `copy(n)`
  with no `size_of` — *is* spared (the `all_of` needs both calls), so it's
  un-correlated (demote), not anti-correlated (drop).
- Graduation to named is **type-aware** (arg-position AND pointee-type — the
  correct `*u8` byte-copy still FPs without the pointee type), a v0.4 resolved-type
  tier, not a syntactic operator-leaf. (Same biosafety toy pattern as Lesson 25.)

**Try this**:
```sh
cargo run --example numeric_truncation --package antigen
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```
Both `copy_bad` and `copy_good` appear as `✗ undefended` `SizeOfInElementCount`
presentations (both `#[presents]`-marked) — the *fingerprint* difference (the
`size_of` multiplier present vs absent) is in the source, per the section caveat
above.

---

## Lesson 28 — `marked_unknown.rs`: recording the danger you can't name yet

**File**: [`antigen/examples/marked_unknown.rs`](../antigen/examples/marked_unknown.rs)

**Concept introduced**: the *felt-but-unnamed danger* — the unease that something
is wrong here, which evaporates the moment you context-switch or an agent
compacts. Three markers let you record it **structurally, at the site, before it's
gone** — without naming the failure-class. They sit *off* the classification axis
(at ⊥, the unnameable) on a magnitude × existence-certainty plane, and surface at
the dial's **non-gating floor**: they never gate (cannot fail CI) and never nag.

**What's in the file** (from `antigen::{aura, dread, red_flag}`):
- `#[aura(trigger = "...")]` on `retry_request` — low magnitude: "something *may*
  be off, check later"
- `#[dread(trigger = "...")]` on `shutdown` — high magnitude, low certainty (the
  *angor animi* corner): "something *is* wrong, look now"
- `#[red_flag(trigger = "...")]` on `authorize` — high existence-certainty: "I'm
  *sure* something is wrong, act now" — auto-escalates on first match
- A commented graffiti-guard demo: `#[dread]` with no `trigger` (and
  `trigger = ""`) are **compile errors**

**What to learn**:
- **`trigger` is REQUIRED** (ADR-041 guard 3): a triggerless or empty marker is a
  compile error, because a marked-unknown with no stated trigger is the
  contentless "this seems off" graffiti the primitive exists to prevent
  (rationale-as-required-field, ADR-005 Amd2).
- **Where they surface today**: `cargo antigen scan --format json` surfaces each
  marker under the top-level **`report.marked_unknowns`** array (fields: `marker`,
  `magnitude`, `existence_certainty`, `trigger`, `file`). Internally they also emit
  as `FindingBody::MarkedUnknown` records into the unified `Finding` population at
  the ADR-041 emit-seam — and *there* a `#[red_flag]` (existence-certainty `Sure`)
  auto-escalates severity — but that escalation lives on the internal Finding, not
  on the scan-report projection (whose `severity` reads `null` today). The
  human-readable report does **not** render marked-unknowns yet (a later audit-dial
  wave). The mark is never lost.

**Try this**:
```sh
cargo run --example marked_unknown --package antigen
cargo run --bin cargo-antigen -- antigen scan --root antigen/examples --format json
```
Find the three entries under `report.marked_unknowns` (aura / dread / red-flag),
each carrying the `trigger` you wrote.

---

## After the beta.2 lessons

| Lesson | Concept |
|---|---|
| 22 — deserialization | trust-boundary deep tier — surface-flag / witness-proof split |
| 23 — time_ordering | silent-in-tests / panic-in-prod clock skew + clean-sibling exclusion |
| 24 — drop_panic | real-`Drop` panic + `impl_of_trait` precision (the v2) |
| 25 — panic_on_index | `get_unchecked` is UB not a panic + biosafety toy pattern |
| 26 — resource_lifecycle | explicit leaks skip `Drop` + provenance-vs-dial orthogonality |
| 27 — numeric_truncation | `size_of`-in-count foot-cannon + the named→suspected self-catch |
| 28 — marked_unknown | the felt-but-unnamed danger — `#[aura]`/`#[dread]`/`#[red_flag]` |

For the catalog (every beta.2 member's tier, fingerprint, and what it catches in
one scannable view), see [`stdlib-families.md`](stdlib-families.md).

---

## See also

- [`tutorial.md`](tutorial.md) — guided walkthrough
- [`concepts.md`](concepts.md) — architectural concepts
- [`macros.md`](macros.md) — macro reference (includes v0.3 prescriptive family)
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL
- [`witness-tiers.md`](witness-tiers.md) — tier semantics
- [`usage-patterns.md`](usage-patterns.md) — pattern recipes
- [`anti-patterns.md`](anti-patterns.md) — what to avoid
- [`troubleshooting.md`](troubleshooting.md) — diagnostic guide
- [`output-formats.md`](output-formats.md) — scan/audit output reference

---

*The examples are real. The patterns are universal. Once you've
worked through the lessons, you've encountered every core
concept antigen ships — from the basic three-move vocabulary through
the full v0.2 family surface: substrate-witness, Oracle lifecycle,
supply-chain defense, convergent evidence, deferred defense, recurrent
emergence, mucosal boundary, VCS information loss, agentic coordination,
category taxonomy, and decisional triage — and the v0.3 prescriptive
work-orchestration board (code IS the board).*
