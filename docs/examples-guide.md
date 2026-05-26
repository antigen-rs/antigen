# Antigen — Examples Guide

> Curated walkthrough of the nine examples in `antigen/examples/`,
> ordered for progressive learning. Each lesson builds on the prior.
>
> Lessons 1–5 cover the core vocabulary (`#[antigen]` / `#[presents]` /
> `#[immune]` / `#[antigen_tolerance]` / `#[descended_from]`) + the
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

## Lesson 1 — `basic.rs`: declare, present, immune

**File**: [`antigen/examples/basic.rs`](../antigen/examples/basic.rs)

**Concept introduced**: the three core moves — declare a failure-class,
mark a vulnerable site, claim immunity.

**What's in the file**:
- `#[antigen]` declaration of `PanickingInDrop`
- A `VulnerableType` whose `Drop` impl could panic (marked
  `#[presents(PanickingInDrop)]`)
- A `SafeType` whose `Drop` impl is verified panic-free (marked
  `#[immune(PanickingInDrop, witness = safe_type_drop_no_panic_test)]`)
- The witness function — a regular function (not `#[test]`) that
  exercises the safe paths

**What to learn**:
- The three-verb structure: `build` an antigen (declare), `give` an
  antigen (presents), `find` defenses (immune)
- Antigen declarations are unit structs
- Witnesses can point at any in-scope identifier
- The `rationale` field on `#[immune]` carries narrative

**Try this**:
```sh
cargo run --example basic --package antigen
cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
```

Look at the scan output. Notice how `basic.rs` declares
`PanickingInDrop` and shows you both a vulnerable site
(`VulnerableType`) and an immune site (`SafeType`). The vulnerable
site appears as an unaddressed presentation.

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
  `#[immune(DropPanicClass, witness = NonPanickingProof::<PhantomVerifiedDropImpl>::verified)]`

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
- An `#[immune(SignedZeroDiscipline, requires = all_of([signers(required = [...]), fresh_within_days(180)]))]` claim
- The `requires` predicate names what the sidecar file must contain for the immunity claim to hold

**What to learn**:
- Substrate-witness leaves: `signers(required = [...])`, `fresh_within_days(N)`, `ratified_doc(path = ...)`, `oracles_complete(files = [...])`, `signed_trailer(...)`
- Combinators: `all_of`, `any_of`, `not`
- The sidecar lives at `.attest/<AntigenName>.json` co-located with the declaration
- Audit tier climbs from `None` → `Reachability` → `Execution` as the sidecar gets scaffolded, then signed by required signers
- This is how to verify disciplines that have NO in-tree witness function (e.g., "I reviewed this against Higham §6.3")

**Try this**:
```sh
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
# See the audit hint progression as you scaffold + sign the sidecar:
cargo run --bin cargo-antigen -- antigen attest scaffold --root antigen/examples SignedZeroDiscipline
cargo run --bin cargo-antigen -- antigen attest sign --root antigen/examples SignedZeroDiscipline --signer "you@example.com"
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

---

## Lesson 7 — `oracle_lifecycle.rs`: Oracle 5-state artifact lifecycle

**File**: [`antigen/examples/oracle_lifecycle.rs`](../antigen/examples/oracle_lifecycle.rs)

**Concept introduced**: Oracle artifacts. When your discipline depends on an *external reference* (a published paper, a ratified ADR, a versioned spec), you want signers to attest against that exact reference — not a free-text URL that goes stale. Oracle records first-class the reference + its stewardship + its lifecycle state.

**What's in the file**:
- Oracle declared via `cargo antigen oracle declare ...` with steward + provenance
- Antigen with `#[immune(..., requires = oracles_complete(files = ["higham-2002-section-6-3"]))]`
- Lifecycle transitions: Draft → Complete (signers attest the Oracle's content matches the reference) → Deprecated (the reference still exists but newer guidance supersedes) → Retired / Revoked

**What to learn**:
- Oracle 5-state machine (Draft / Complete / Deprecated / Retired / Revoked + Reopened)
- `oracles_complete(...)` predicate checks Oracle state at audit time
- Oracle records have signers (who attested), stewards (who maintains the reference link), and provenance
- The audit treats `Complete` as the load-bearing tier; `Deprecated` triggers a "use with caution" hint; `Retired`/`Revoked` fail audit
- This closes the "URLs go stale" problem at the substrate level

**Try this**:
```sh
cargo run --bin cargo-antigen -- antigen oracle list --root antigen/examples
cargo run --bin cargo-antigen -- antigen oracle status --root antigen/examples higham-2002-section-6-3
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

## After the thirteen lessons

By now you've encountered the core vocabulary, four witness tiers, substrate-witness
pipeline, Oracle lifecycle, delta-chained signatures, tolerance tiers, and the v0.2
families (supply-chain defense + convergent-evidence):

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

### Lesson: `#[orient]` — acknowledged orientation with see-also context

**File**: `antigen/examples/deferred_defense_orient.rs`

`#[orient]` is the lightest-weight deferred-defense primitive. All fields optional.
Bare `#[orient]` with no arguments is valid. For ported code, design-flux periods,
or rollback-as-triage sites (ADR-026). No minimum lengths, no required fields.

```sh
cargo run --example deferred_defense_orient --package antigen
```

---

## See also

- [`tutorial.md`](tutorial.md) — guided walkthrough
- [`concepts.md`](concepts.md) — architectural concepts
- [`macros.md`](macros.md) — macro reference
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL
- [`witness-tiers.md`](witness-tiers.md) — tier semantics
- [`usage-patterns.md`](usage-patterns.md) — pattern recipes
- [`anti-patterns.md`](anti-patterns.md) — what to avoid
- [`troubleshooting.md`](troubleshooting.md) — diagnostic guide
- [`output-formats.md`](output-formats.md) — scan/audit output reference

---

*The examples are real. The patterns are universal. Once you've
worked through the nine lessons, you've encountered every core
concept antigen ships in v0.1.0-rc.3.*
