# Antigen — Glossary

> Anchors every term in antigen's vocabulary to: biological referent + Rust ecosystem
> analog. Vocabulary drift is a known failure mode; this glossary is the antibody.

---

## Core terms

### antigen

**Biological referent**: a molecule (often on a pathogen's surface) that the immune
system recognizes as non-self and responds to.

**Rust ecosystem analog**: a named, structurally-fingerprinted **failure-class**. e.g.,
`FrameTranslation`, `BoundaryViolation`, `OptionalityCollapse`. Declared via
`#[antigen(name = "...", fingerprint = "...")]`.

### antibody

**Biological referent**: a protein produced by B-cells that specifically binds and
neutralizes one antigen.

**Rust ecosystem analog**: an **immunity witness** — a test (annotated `#[defended_by(X)]`),
proptest, phantom-type proof, or substrate predicate (`#[presents(X, requires=...)]`) that
provides evidence for a specific antigen. In the v0.2 ADR-029 model, immunity is **observed
by audit** from this evidence, not claimed at the vulnerable site.

*(Removed (ADR-029): `#[immune(antigen, witness = ...)]` was the v0.1 form that claimed
immunity at the site. Use `#[defended_by]` on the test function and `#[presents]` on the
site instead.)*

**Note**: "antibody" is used colloquially in design docs but the ratified API uses
"witness" because antibodies in biology are *response*, while Rust witnesses are
*evidence observed by audit*. The biology rhymes; the Rust term is more precise.

### vaccination

**Biological referent**: deliberate exposure to a weakened antigen so that B-cells
develop memory before encountering the live pathogen.

**Rust ecosystem analog**: applying a known immunity pattern across a structural family
of types — operating on a refinement-lattice of types (e.g., "every enum named
`*Class`"). This is a conceptual operation in the metaphor; there is no shipped
`vaccinate` subcommand. In practice, a `family` grouping on `#[antigen]` plus
`#[descended_from]` inheritance carries a pattern across a structural family.

### immunity

**Biological referent**: the state of being protected against a specific pathogen due to
prior exposure or active defense.

**Rust ecosystem analog** (v0.2): a per-presents-site **verdict** observed by `cargo antigen audit`
when the site has sufficient defense evidence. Evidence can be a `#[defended_by(X)]` test
function (code-tier), `#[presents(X, requires = ...)]` substrate predicate, or `proof =` phantom
construction. Immunity is **observed by audit** from the evidence, never declared at the
vulnerable site (ADR-029 observe-not-declare).

*(Removed v0.1 form (ADR-029): `#[immune(antigen, witness = ...)]` directly claimed immunity
at the site. The audit now observes defense evidence and reports `defended` / `undefended` /
`substrate-gap` — the site never stamps itself immune.)*

### immunity claim *(anti-pattern — a reserved term, not a synonym for "defense")*

**Biological referent**: a false declaration of safety — like a pathogen's molecular mimicry
signalling "non-threat" to suppress the immune response that would otherwise catch it. The claim
itself is the danger: it turns off the scrutiny.

**Rust ecosystem analog**: the **rejected** pattern (ADR-029) of a site *asserting* it is immune
instead of presenting evidence for the audit to *observe*. **Antigen never claims immunity** — it
surfaces a fingerprint match to inspect, and observes defense evidence to report a per-site verdict;
it never says "trust me, this is safe." Stated sharply: **an immunity claim is itself a pathogen** —
asserting safety suppresses the inspection that catches failure, so the all-clear becomes the silent
failure. The deprecated `#[immune]` form *was* an immunity claim (the site stamped itself);
observe-don't-declare replaced it with evidence the audit reads.

> This term is **reserved** for naming that anti-pattern. A **defense** is *evidence* (a
> `#[defended_by]` witness); an **immunity claim** is the *assertion without evidence*. Don't use
> "immunity claim" as a neutral synonym for "defense" — that dilutes the one word that names the
> thing antigen exists to refuse.

### fragility / vulnerability

**Biological referent**: susceptibility to a specific pathogen; the absence of immunity.

**Rust ecosystem analog**: code marked `#[presents(antigen)]` — explicitly declares
vulnerability to a known failure-class without claiming immunity. `cargo antigen scan`
flags every presentation that lacks a corresponding defense.

**Note**: "fragility" was used in early design discussions; the ratified macro is
`#[presents]` (paralleling MHC presentation in biology).

### presentation

**Biological referent**: MHC Class I/II protein complex displaying antigen fragments on a
cell's surface so immune patrol can detect them.

**Rust ecosystem analog**: the `#[presents(antigen)]` decorator on Rust code. The code
*shows* what failure-class it's vulnerable to. Without presentation, the immune system
(cargo-antigen scan) cannot find the vulnerability.

---

## Inheritance terms

### descended_from

**Biological referent**: B-cell lineage (clonal expansion). When a B-cell encounters its
target antigen, it divides; daughter cells inherit the parent's antibody specificity but
may mutate slightly.

**Rust ecosystem analog**: the `#[descended_from(other_function)]` decorator. Propagates
`#[presents]` markers from the source function to the descended function, inheriting
the structural vulnerability declaration. Defense evidence (`#[defended_by]` / `requires=`)
must be re-attested at the descendant if inherited witnesses no longer apply.

**Closest existing academic analog**: Eiffel's design-by-contract with inheritance
(Meyer 1992, 1997) — pre/post-conditions inherited through subclassing with covariance
/ contravariance rules. Antigen's `#[descended_from]` is the Rust-ecosystem analog of
inherited contracts at the failure-class level rather than the predicate level.

### B-cell memory

**Biological referent**: the persistence of antigen-specific B-cells long after an
infection clears. Critically, B-cell memory is **stratified**: memory cells persist
for decades (10-15 years for hepatitis B; 24+ years for rabies), but *circulating
antibody titer* decays on a much shorter timescale (half-life ~30 days). On
re-exposure, memory cells trigger a recall response producing high antibody titers
within 3-4 days.

**Rust ecosystem analog (pattern-memory layer)**: `#[antigen]` declarations
themselves — they don't decay. The pattern is permanent across project lifetime;
new code in the structural family inherits via `#[descended_from]`. This is the
B-cell-memory layer.

**Rust ecosystem analog (currency layer)**: the *recency of defense evidence* — whether
`#[defended_by]` witnesses or `requires=` substrate predicates still reflect the current
state of the protected item. If the protected code changes, the evidence may be stale
(analog of declining circulating antibody titer). `cargo antigen audit` observing substrate-gap
or stale-sidecar hints is the recall-response / booster analog.

**Note**: the *crisis case* this addresses is "corrected designs don't carry the
failure that motivated them" — antigen's originating insight (see `origin.md`).
Pattern-memory persists; verification-currency requires periodic re-attestation.

### lineage

**Biological referent**: B-cell or T-cell lineage from a single progenitor through
multiple clonal expansions.

**Rust ecosystem analog**: a chain of `#[descended_from]` declarations connecting an
original antigen-bearing function to its derived/refined/copied descendants. Cargo-antigen
walks the lineage to determine inherited markers.

---

## Recognition terms

### structural fingerprint

**Biological referent**: the molecular shape that a pathogen-recognition receptor matches
against (e.g., bacterial cell-wall lipopolysaccharide patterns, viral RNA double-stranded
shape).

**Rust ecosystem analog**: the `fingerprint` parameter on `#[antigen(...)]`. A structural
pattern (initially free-text; eventually a structured grammar) that cargo-antigen scan
matches against new code to find sites that should be flagged for the antigen even
without explicit `#[presents]`. The "innate immunity" surface.

### Pathogen Recognition Receptor (PRR)

**Biological referent**: receptors on innate-immunity cells that recognize broad classes
of pathogens (TLRs, NLRs, RLRs).

**Rust ecosystem analog**: the structural-pattern matcher in `cargo antigen scan`. Scans
code for patterns matching declared antigen fingerprints. Innate immunity = always-on
structural checks (compile-time, type-system phantom types). Adaptive immunity =
failure-pattern-specific tests.

### T-cell receptor

**Biological referent**: highly specific receptor on T-cells that recognizes one antigen
displayed by MHC.

**Rust ecosystem analog**: a named-failure-class fingerprint that recognizes ONE specific
structural pattern. More precise than PRRs (innate); less broad. Each `#[antigen(...)]`
declaration creates a kind of T-cell-receptor analog in the cargo tooling.

### original antigenic sin

**Biological referent**: the immune system's bias toward recall of a previously-encountered
antigen when it meets a *variant*. The memory receptor, imprinted on the first-seen strain,
preferentially fires on the original epitope and under-binds the new one — so a known memory
is present but its recognition is fitted too tightly to the first instance to serve the
variant. (Also: antigenic imprinting. The seasonal-flu-vaccine reformulation problem is the
canonical case.)

**Rust ecosystem analog**: a recognition surface fitted to the *shape of its first observed
instance* rather than to the *extension of its class* — so a later instance the surface
should match escapes it. A **cross-family cognate**: it grounds failure-classes in two
families.

- **VCS-info-loss family** — [`RefactorWithoutPreservationOfWhy`](../antigen/src/stdlib/vcs_info_loss.rs):
  the known-but-uncontextualized memory of the corrected code biases future developers toward
  re-deriving the original bug (the "first antigen" is the original failure-context; the clean
  refactor is the variant the imprinted memory under-serves).
- **Recognition-scope (meta) family** — the fingerprint-scope meta-antigen
  [`AntigenFingerprintDivergesFromClassExtension`](../antigen/src/stdlib/dogfood.rs): a declared
  antigen's fingerprint, fitted to instance #1, diverges from the class its summary claims (under-
  *or* over-coverage; divergence is symmetric).

**Paired remedy**: **affinity maturation** (somatic hypermutation) — the receptor is broadened
and refined to bind the variant; survival-selection ratifies the matured lineage. Already
glossary-grounded as the biological referent of [ratification](#ratification). The division of
labor is biology-faithful: **recall lives in the germline-encoded broad receptor (the
fingerprint), precision in the affinity-matured receptor (the structural witness)** — broaden
the fingerprint to the class's extension, and let the witness carry precision.

**Introduced in**: inline in `vcs_info_loss.rs` + `docs/decisions.md` (per-primitive cognates);
recognized as a cross-family cognate and lifted here during the `antigen-dx-dogfood` arc.

---

## Response terms

### cytokine

**Biological referent**: signaling molecules released during immune response that recruit
additional immune cells and modulate inflammation.

**Rust ecosystem analog**: when an antigen presentation fires, build-time signals
propagate through the call graph. Specifically, `#[propagates_presentations]` (opt-in)
causes callers of presenting functions to be marked as derived-presentations. Avoids
indiscriminate cytokine storm by being opt-in rather than default.

### inflammation

**Biological referent**: localized immune response that escalates if the pathogen
persists; can become chronic if dysregulated.

**Rust ecosystem analog**: not directly modeled in v1. The escalation analog —
sustained antigen presentation in a code area driving escalating warnings or required
review — is a recorded routing-organ direction; see [`roadmap.md`](roadmap.md).

### autoimmunity

**Biological referent**: failure mode where the immune system attacks healthy self-tissue
because tolerance mechanisms broke down.

**Rust ecosystem analog**: cargo-antigen scan over-flagging legitimate code as fragile.
Tolerance check: distinguishing "this code structurally matches an antigen fingerprint
but is in fact correct" from "this code is genuinely vulnerable." Initial mitigation: the
fingerprint grammar must be precise enough to minimize false positives; users can mark
specific sites with `#[antigen_tolerance(X, rationale = "...")]` for documented
exceptions (per ADR-011).

**Introduced in**: ADR-011 (tolerance carrier).

---

## Composition terms

### witness

**Biological referent**: not a direct biological term; appears in the metaphor as the
"proof" that an antibody actually neutralizes its target antigen (e.g., binding affinity
measurements).

**Rust ecosystem analog**: the **evidence of defense** for a presented failure-class.
A witness is registered as a `#[defended_by(Antigen)]` test/proptest function (code-tier),
a `#[presents(Antigen, requires = <predicate>)]` substrate predicate (substrate-tier), or a
`#[presents(Antigen, proof = <expr>)]` phantom-type / formal-proof construction. The witness
is checked by `cargo antigen audit`; a presents-site without any witness is reported
`undefended`. Immunity is **observed by audit** from the witness, never claimed at the site
(ADR-029).

**Academic lineage**: the witness-as-proof concept descends from Necula's
proof-carrying code (Necula, "Proof-Carrying Code," POPL 1997) — code accompanied
by a checkable proof of a stated property. Antigen's witness pluralism extends this
by accepting heterogeneous proof shapes (test, proptest, formal verification, lint,
phantom type) under one vocabulary.

**Vocabulary disambiguation — "witness" in Rust ecosystem usage**: the word "witness"
appears in multiple Rust patterns with structurally-different meanings. Antigen uses
"witness" specifically for **proof-of-immunity-claim**, not for either:

- **Type-witness for const-fn dispatch** (e.g., the `typewit` crate's `TypeEq<L, R>`)
  — uses `witness` to mean "compile-time proof that two types are equal so const-fn
  branches can specialize." Different pattern, different use case. Antigen's witness
  is about *immunity to a failure-class*; typewit's witness is about *type-level
  equality for polymorphism*.
- **Proof-of-knowledge witness** in zero-knowledge cryptography — uses "witness"
  for the secret value the prover demonstrates knowledge of. Out-of-domain for antigen.

When ADR-013/W7 talk about "phantom-type witnesses," they mean the immunity-encoding
pattern (see `phantom witness / phantom-type witness` below), not typewit-style
type-witnesses. The vocabulary collision is real and worth flagging in user-facing
docs to prevent ecosystem-wide drift.

**The code-tier vs substrate-tier choice** (the two evidence channels):
- Use **`#[defended_by(Antigen)]`** on a test/proptest function when the defense evidence is
  a Rust function, test, or formal-verification entry-point *in the source tree*. The audit
  resolves the name to a live symbol. This is the code-tier witness path.
- Use **`#[presents(Antigen, requires = <predicate>)]`** when the defense evidence lives in
  substrate *outside* the source tree — a ratified doc, a team sign-off sidecar,
  oracle-completion markers. This is the substrate-tier witness path (see `substrate-witness`
  below). The audit detects which evidence is present and routes accordingly.

  Quick heuristic: **can a test execute the thing you're defending?** Yes → `#[defended_by]`;
  no (it's about substrate state a test can't verify) → `requires =`. These are co-equal
  siblings, not basic vs advanced. Category-mapping (ADR-028): `FunctionalCorrectness` →
  `#[defended_by]`; `SubstrateAlignment` → `requires =`.

### witness-validity tiers

**Definition** (per ADR-001 Amendment 1, Change 4): a defense witness
(`#[defended_by]` test, `requires=` predicate, or `proof=` construction) is graded at
four progressively-stronger tiers:

- **Reachability tier** — the witness identifier resolves to a function/test that
  exists. Floor; the current audit lives here.
- **Execution tier** — the witness runs without panic and asserts a non-trivial
  property.
- **Behavioral-alignment tier** — the witness exercises behavior that matches the
  antigen's structural fingerprint.
- **Formal-proof tier** — the witness is a verified compile-time proof (phantom-type
  construction, kani/prusti/verus/creusot proof annotation).

ADR-005 sub-clause F applies to whichever tier is current. Audit's `--format=json`
output includes a `witness_tier` field for CI gates.

**Introduced in**: ADR-001 Amendment 1.

### phantom witness / phantom-type witness

**Biological referent**: high-affinity antibody that confirms binding via constructive
recognition rather than catalytic action — the existence of the antibody-antigen complex
*is* the proof.

**Rust ecosystem analog**: a witness expressed as a typed path with type parameters
(e.g., `PolarityProof::<FrameTranslation>::established_by_construction`). The
constructor's compile-time success encodes the proof: if the code compiles, the
proof holds. Audit recognizes this shape via `WitnessKind::PhantomType { proof_type,
type_params, constructor }` and reports at the formal-proof tier of witness validity.

v0.1 ships **recognize-and-warn**: audit recognizes the phantom-type *shape* but
cannot verify whether the construction encodes meaningful preconditions (a trivial
`pub fn () -> Self { Self(PhantomData) }` shape-matches but proves nothing).
Construction-validation is deferred to a future ADR.

**Real-world prior art**: the
[`witnessed`](https://crates.io/crates/witnessed) crate ships exactly this pattern
as `Witnessed<T, W>`, where `W` is a phantom type encoding a verification proof and
`T` is the value being witnessed. The crate uses `PhantomData<fn() -> W>` to keep
the wrapper Send/Sync-transparent. Any function-signature requiring `Witnessed<T,
ValidatedShape>` refuses unverified values at compile-time. This is the canonical
real-world example of the phantom-type-witness pattern, and a first-class
detection target alongside hand-rolled `PhantomData<T>` constructions.

**Vocabulary disambiguation — distinct from typewit-style witnesses**: this entry
covers phantom-type witnesses *for invariant-encoding* (the `Witnessed<T, W>` /
`PolarityProof::<FrameTranslation>` pattern). It is **not** the same as the
[`typewit`](https://crates.io/crates/typewit) crate's `TypeEq<L, R>`, which uses
"witness" to mean type-level equality proof for const-fn dispatch. Same word,
different patterns. See the `witness` entry above for the broader disambiguation.

**Introduced in**: ADR-013. Academic lineage: refinement-type proof carriers
(Liquid Haskell, Flux); seal-trait private-constructor patterns.

### family / failure-class family

**Biological referent**: pathogens grouped by structural similarity (e.g., influenza
strains, SARS-CoV variants).

**Rust ecosystem analog**: the `family` parameter on `#[antigen(...)]`. Groups related
failure-classes into inheritance-clusters for shared structural fingerprints and shared
vaccination patterns. Per the ratified ADR (`decisions.md` "Optional fields"), `family`
maps to **one of the 8 first-principles classes OR a project-specific family** — it is an
open-vocabulary grouping label, not a sealed enum. The 8 first-principles classes
(`frame-translation`, `forgotten-lesson`, `implicit-coupling`, `stale-context`,
`premature-abstraction`, `incompatible-merger`, `boundary-violation`,
`optionality-collapse`) are available as family names, and the stdlib also defines
project-specific families (`vcs-information-loss`, `mucosal-boundary`, `recurrent-emergence`,
`dogfood`, …) that group antigens by domain rather than by first-principles class.

The 8 first-principles classes are a **set-level classification axis** the stdlib *as a
whole* commits to spanning (ADR-007 coverage commitment) — not a per-antigen constraint on
what `family` may hold. This mapping (which antigen instances which first-principles class)
is currently a stdlib-level commitment with **no per-antigen carrier field**: it is not
held in `family=`, and as of this writing it is not tracked in a dedicated per-antigen
field or doc-comment convention either. A project-specific family and a first-principles
class are different lenses on the same antigen: the family is its inheritance-cluster (what
`family=` carries); the first-principles class is the abstract failure-shape it instances
(a set-level coverage axis, not yet individually carried).

### composition (of antigens)

**Biological referent**: not a direct biological term; in immunology, response to
multi-antigen pathogens involves coordinated B-cell and T-cell responses.

**Rust ecosystem analog**: combining antigen markers via Rust's existing composition
mechanisms (trait impls, generics, derive macros). Antigen propagation rules specify
how `#[presents]` and `#[defended_by]` flow through composition. See
[`composition.md`](composition.md).

---

## Adoption terms

### stdlib antigens

**Biological referent**: not a direct biological term; analogous to the *standard
vaccinations* given to a population (DPT, MMR, etc.) — the basic immunity everyone
should have.

**Rust ecosystem analog**: `antigen-stdlib`, a future companion crate providing 20-50+
ready-made antigens for common Rust failure-classes (use-after-move-conceptually-equivalent,
panicking-in-Drop, lock-order-inversion, async-in-sync-context, etc.). Adoption flywheel:
users get value without writing antigens themselves.

---

## Carriers (macros and attributes)

### #[antigen_tolerance]

**Definition** (per ADR-011): the macro that declares a site as a legitimate
fingerprint match — an opt-out from immunity-or-flagging when the site genuinely
exhibits the failure-class pattern but is correct in context (test fixtures
demonstrating the pattern, examples deliberately constructing it, code-generation
sites where the context makes the pattern fine).

```rust
#[antigen_tolerance(
    PolarityInvertedClassMeet,
    rationale = "Test fixture deliberately constructs failure pattern to verify witness.",
    until = "v1.0",  // optional
    see = ["..."],   // optional open-vocabulary references
)]
```

**Required fields**: antigen type (positional), `rationale` (non-empty string).
**Optional fields**: `until` (non-empty if present), `see` (string array).
**Item-level only in v1**; module-level deferred to future ADR.
**Tolerance dominates** over `#[presents]` on the same site (the marker becomes dead code; audit warns).

**Biological referent**: peripheral tolerance via T-regulatory cells / anergy —
the immune system's mechanism for not attacking self despite recognition signals.

**Introduced in**: ADR-011 (2026-05-08). Substrate already named the path in
`cargo-antigen/src/main.rs:185` before ratification.

### #[antigen_generates]

**Definition** (per ADR-014): the macro that proc-macro and macro_rules authors
apply to declare that invocations of their macro emit code presenting a named
antigen. Closes the macro-expansion structural-blindness gap (sibling to ADR-012's
function-body blindness fix).

```rust
#[antigen_generates(
    PanickingInDrop,
    rationale = "This derive emits a Drop impl that may panic if the inner type's destructor panics.",
)]
#[proc_macro_derive(SomeDerive)]
pub fn some_derive(input: TokenStream) -> TokenStream { ... }
```

`cargo antigen scan`'s synthesis pass recognizes `#[antigen_generates]` annotations
and emits synthetic presentations at invocation sites, requiring consumers to
discharge the defense duty (`#[defended_by]` or `#[antigen_tolerance]` at the call site).

**Introduced in**: ADR-014.

### #[anergy]

**Definition** (per ADR-023): declares a site as in a deferred-but-muted posture
toward a known failure-class. `until` is REQUIRED — anergy without a time-bound
degrades to silent tolerance. `reason` minimum 20 characters.

**Biological referent**: T-cell or B-cell anergy — the cell encounters its antigen
but fails to respond due to lack of co-stimulation. Alive but unresponsive;
reversible when co-stimulation arrives.

**Distinguishes from `#[antigen_tolerance]`**: tolerance is "this site is correct
despite the fingerprint match." Anergy is "I know I'm not immune; here's why I'm
deferring and what will re-engage the response."

**Introduced in**: ADR-023 (2026-05-22). Shipped in v0.2.0-alpha.1.

### #[immunosuppress]

**Definition** (per ADR-023): declares a site as under surgical silencing — a
specific check family is deliberately muted for a bounded duration. Duration cap
enforced at **parse time** (compile error if `until - since > cap`). `rationale`
minimum 20 characters.

**Biological referent**: pharmacological or pathological immunosuppression —
the immune system deliberately reduced to prevent rejection or autoimmune damage.
Time-bounded; expected to be revisited.

**Distinguishes from `#[anergy]`**: immunosuppress is active, systemic, deliberate
intervention. Anergy is receptor-level unresponsiveness. Both are time-bounded;
immunosuppress has a machine-enforced duration cap.

**Introduced in**: ADR-023 (2026-05-22). Shipped in v0.2.0-alpha.1.

### #[poxparty]

**Definition** (per ADR-023): declares a controlled-exposure exercise — chaos
test, fault injection, or red-team exercise targeting a known failure-class.
Structurally isolated via `antigen-poxparty` Cargo feature (NOT in default
feature set). `exercise_type` minimum 20 characters. `until` required.

**Biological referent**: pox parties — intentional exposure to a pathogen to
build immunity in a controlled setting. Deliberate, bounded, expected to produce
a trained response.

**Structural isolation**: primary via `#[cfg(feature = "antigen-poxparty")]`
gate (items inside inactive blocks never reach macro expansion). Secondary via
`CARGO_FEATURE_ANTIGEN_POXPARTY` env var check (best-effort).

**Introduced in**: ADR-023 (2026-05-22). Shipped in v0.2.0-alpha.1.

### #[orient]

**Definition** (per ADR-023): declares an orientation period — explicit
acknowledgment that the site lacks immunity during an orientation phase. The
lightest-weight deferred-defense primitive. All fields optional; bare `#[orient]`
with no arguments is valid.

**Biological referent**: orientation period — immune system present but not yet
trained to recognize a specific threat. Building up response repertoire.

**ADR-026 use case**: rollback-as-triage sites use `#[orient]`-shape to declare
that a rollback is a triage action, not a defense.

**Introduced in**: ADR-023 (2026-05-22). Shipped in v0.2.0-alpha.1.

### ContentHashMismatch

**Definition** (per ADR-025): the NON-NEGOTIABLE stdlib antigen for the
content-replacement-at-fixed-version attack class. `Cargo.lock` pins VERSION but not
CONTENT-HASH — a compromised registry can serve different content at the same pinned
version. Requires proactive first-attestation via `cargo antigen verify content-hash record`.

**Biological referent**: antigenic-identity verification. The immune system identifies
pathogens by molecular shape (antigen epitopes); if the shape changes while the label
stays the same, identity verification catches the substitution. The Cargo version field
is the "label"; the content hash is the "molecular shape."

**Attack pattern**: chalk/debug/eslint-config (2025) — publisher replaces tarball at
a fixed version without changing the version string. Cargo.lock checksum is recorded at
first-resolve time and is NOT re-verified if version is unchanged.

**Named limitations**:
- First-attestation gap: the antigen is dormant until `cargo antigen verify content-hash record` creates the sidecar. Absence of sidecar emits `content-hash-no-attestation` (not a silent pass).
- v0.2 hash-source is Cargo.lock checksum; crates.io tarball SHA-256 verification is v0.3+.

**Introduced in**: ADR-025 (2026-05-22). Shipped in v0.2.0-alpha.2.

### WitnessClass

**Definition** (per ADR-024): a public enum enumerating the categorical type of
witness backing a defense claim. Used as the element type for `#[diagnostic]`'s
`modalities` argument.

**Six variants**: `StaticAnalysis` (clippy, custom lints), `PropertyTest` (proptest, quickcheck),
`FormalVerification` (Kani, Prusti, Verus), `ManualReview` (PR review, ADR substrate),
`RuntimeFuzz` (cargo-fuzz, AFL), `SubstrateWitness` (`.attest/` sidecar predicates).

**Independence discipline (ADR-024 C1)**: `#[diagnostic]`'s `min_independent` counts
distinct `WitnessClass` CATEGORIES, not raw witness count. Two proptest property tests
are both `PropertyTest` class — they count as one independent class, not two. The compile-
time enforcement prevents vacuously-unsatisfiable claims (min_independent > distinct categories).

**Introduced in**: ADR-024 (2026-05-22). Shipped in v0.2.0-alpha.2 as `antigen::WitnessClass`.

### SeedKind

**Definition** (per ADR-024): a public enum discriminating the RNG-seed policy for
`#[clonal]`-style iterated witness evaluation.

**Four variants**: `Random` (thread_rng), `EntropyFromCi` (CI environment variables),
`TimestampSeeded` (millisecond-precision wall clock), `Fixed(u64)` — **REJECTED for `#[clonal]`
at compile time** (a fixed seed means iterations are NOT independent;
"independent iterations with fixed seed" is a contradiction).

**Biological referent**: B-cell clonal expansion — each daughter cell is an independent
activation of the same template. Fixed seed defeats the independence claim; every
iteration replays the same RNG path.

**Introduced in**: ADR-024 (2026-05-22). Shipped in v0.2.0-alpha.2 as `antigen::SeedKind`.

### SupplyChainDefenseFamily

**Definition** (per ADR-025): the eleven-antigen stdlib family targeting the 2026+
dependency-boundary threat landscape. Groups antigens that share the Distributed-Boundary
Innate-Immunity biological cognate.

**Biology cognate**: Distributed-Boundary Innate-Immunity — a multi-cell-type integrated
system. The family covers the dependency trust boundary in its entirety: version pinning, content
verification, maintainer attestation, build-script containment.

**Members**: `ContentHashMismatch`, `UnsandboxedProcMacro`, `UnpinnedDependency`,
`UnpinnedTransitiveDependency` (NARROW), `UnattestedDependencyInclusion`,
`DependencyUpgradeWithoutDiffReview`, `AutoDependencyChainWithoutPinning`,
`MaintainerChangeWithoutReattestation`, `SuddenDependencyExpansion`,
`UnsandboxedBuildScript`, `PostInstallScriptInDependency`.

**Introduced in**: ADR-025 (2026-05-22). Shipped in v0.2.0-alpha.2.

### ConvergentEvidenceFamily

**Definition** (per ADR-024): the seven-macro first family of the temporal-arc cohort
(backward-looking evidence aggregation). Groups macros for structuring how independent
witnesses converge on a defense claim over time.

**Biology cognate (dual-axis)**:
- Immunology-proper (`#[clonal]`, `#[igg]`, `#[crossreactive]`, `#[polyclonal]`,
  `#[monoclonal]`, `#[adcc]`): mapped to B-cell and antibody biology.
- Clinical-medicine (`#[diagnostic]`): mapped to differential diagnosis workflows.
  Per ADR-024's dual-axis honesty: not all convergent primitives are
  immunology-grounded; `#[diagnostic]` is clinical-medicine.

**Sibling families** (ratified in ADR-024 but shipped in later alphas): Recurrent
Emergence Family (present-looking), Prescriptive Work-Orchestration Family (forward-looking).

**Introduced in**: ADR-024 (2026-05-22). Shipped in v0.2.0-alpha.2.

---

## Architectural patterns

### tiered substrate / carrier-strength hierarchy

**Definition** (per ADR-001 Amendment 1, Change 1): the project-wide architectural
pattern where every primitive sits on a strength-of-evidence gradient rather than
being binary. Memory carriers sit on a drift-resistance hierarchy:

```
  compile-time-checked   (type system, phantom-types, kani/prusti proofs)
          ↑
  scan-time-checked      (#[antigen], #[presents], #[defended_by], #[descended_from])
          ↑
  test-suite-checked     (proptest, regression tests, witness functions)
          ↑
  review-discipline      (PR review, mentorship, ADR cross-references)
          ↑
  documentation          (rustdoc, README, design docs, CHANGELOG)
          ↑
  commit-message         (commit log, issue tracker, post-mortems)
          ↑
  human/agent memory     (mentorship, conversation, in-context working memory)
```

Antigen's role: push failure-class memory upward in this hierarchy whenever the
class admits structural recognition.

**Convergent across the project**: witness-validity tiers (ADR-001 Amendment 1
Change 4), filter-vs-proof tiers (ADR-010 amendment 4 deferred), recognition tiers
(ADR-006), guarantee tiers (ADR-007). When proposing a new primitive, the right
question is "what's its tier in the hierarchy?" before "is it correct?"

**Three-window convergence** (ADR-003 empirical defense): biology (vertebrate
immunology), the naming-checkability frame, and academic lineage (Hoare 1969 →
Eiffel 1992 → Koka → Liquid Haskell → Flux). When three independent traditions
converge on the same primitive, the underlying architecture is real, not
metaphor-dependent.

**Introduced in**: ADR-001 Amendment 1.

### passive surface / fingerprint scan

**Definition** (per ADR-001 Amendment 1, Change 2): the *recognition-not-yet-marked*
half of antigen's design. `cargo antigen scan` walks the codebase and recognizes
unmarked code that structurally matches a declared antigen's `fingerprint`.
Catches vulnerable code that the original author did not mark — including code
authored before the antigen was declared.

The biological analog is **innate immunity** — broad pattern recognition (PRRs)
that fires against pathogen-associated molecular patterns without requiring prior
adaptive memory.

**5 interaction states** with the active surface (per Change 2):
1. **Marked + matched** — `#[presents(X)]` is on a site that also matches X's
   fingerprint (intentional + recognized; audit reports as doubly-marked)
2. **Passively detected** — no marker, but fingerprint matches (scan reports
   needs-immunity-or-tolerance)
3. **Inconsistent** — `#[presents(X)]` is on a site that does NOT match X's
   fingerprint (audit warns; either marker is wrong or fingerprint is wrong)
4. **Tolerated** — `#[antigen_tolerance(X)]` is on a site that matches X's
   fingerprint (legitimate match acknowledged)
5. **Stale tolerance** — `#[antigen_tolerance(X)]` is on a site that no longer
   matches (tolerance is dead weight; audit warns it can be removed — the
   descended_from-style stale-reference pattern applied to tolerances)

**Introduced in**: ADR-001 Amendment 1 (2026-05-08).

### active surface / explicit marker

**Definition** (per ADR-001 Amendment 1, Change 2): the *intent-carrying* half of
antigen's design. The developer explicitly marks code with attribute macros
(`#[presents]`, `#[defended_by]`, `#[descended_from]`, `#[antigen_tolerance]`,
`#[antigen_generates]`). Active markers are unambiguous, document intent, and
survive refactoring as long as the marked items survive.

The biological analog is **adaptive immunity** — antigen-specific antibody
production after the immune system has built memory of a specific pathogen.

The two surfaces are dual-load-bearing: active markers carry intent; passive
fingerprints carry recognition. Adoption at Layer 1 (per ADR-009) depends on
the passive surface — consumers benefit from antigen-stdlib's fingerprints
without authoring their own markers.

**Introduced in**: ADR-001 Amendment 1 (2026-05-08).

### rationale-as-required-field

**Definition** (transverse principle, observed across ADR-005, ADR-009, ADR-011,
ADR-014, and potentially ADR-001 Amendment 1's tolerance state): every primitive
that extends trust requires a justification field. The pattern:

- `#[antigen(... summary)]` — human description (ADR-009 Layer 2)
- `#[presents(... requires=)]` / `#[defended_by]` — evidence of defense (ADR-029)
- `#[antigen_tolerance(... rationale)]` — justification for waiver (ADR-011)
- `#[antigen_generates(... rationale)]` — justification for generated
  presentation (ADR-014)

Sub-clause F (ADR-005) applied at the API level — every trust-extending primitive
carries its own justification. The discipline propagates from existing ADRs to
new ADRs without explicit coordination, which is how a load-bearing principle
should behave.

**Introduced in**: ADR-005 (sub-clause F at the API level).

---

## Tooling terms

### cargo antigen

**Definition**: the cargo subcommand binary, published as the `cargo-antigen` crate.
Provides `scan`, `audit`, `attest`, `tolerate`, `oracle`, `verify`, `vcs`,
`mucosal-map`, and `fingerprint` subcommands. See [`cli-reference.md`](cli-reference.md).

### antigen library / antigen registry

**Definition**: the (eventual) collection of named antigens distributed via crates.io.
`antigen-stdlib` is the first; project-specific antigens (e.g., a `my-project-antigens` crate) extend
it. No central registry — community-driven via crate publication.

---

## Disciplines inherited from the origin project

These terms come from the origin project's design decisions and team-briefing disciplines.
They apply to antigen because the antigen team inherits these disciplines from its
multi-agent development methodology.

### sub-clause F (trust boundary)

**Origin**: the origin project's trust-boundary design decision.

**In antigen**: every antigen declaration's witness MUST be validated by tooling before
the defense is trusted. The trust boundary lives at `cargo antigen scan` time
(checking the witness exists and is valid) and at compile time (for phantom-type
witnesses).

### substrate over memory

**Origin**: a standing constraint of the origin project.

**In antigen**: cargo-antigen tooling reads the codebase as ground truth. Documentation
about antigens is informational; the source-of-truth is the `#[antigen]` / `#[presents]`
/ `#[defended_by]` declarations themselves.

- **substrate-currency** (temporal sub-pattern): substrate-as-of-author-time ≠
  substrate-as-of-consumer-time. A claim or finding verified against the substrate at
  authorship time may be stale by the time it is consumed — files change, findings get
  corrected, commits land between the moment of writing and the moment of reading.
  The discipline: verify against the substrate at consumption time, not just at authorship
  time. It manifests across three operational layers: tracker-tier context drift,
  reporter-tier unverified findings, and claim-propagation-tier routing without
  re-verification.

### narrow-then-lift

**Origin**: a sub-clause discipline of the origin project's design decisions.

**In antigen**: antigen fingerprints should narrow to what the structural pattern can
*actually* match; if a fingerprint is overly broad, narrow it before adding to the
library. Avoid speculative claims; lift narrowed fingerprints into more general patterns
only when evidence supports it.

### proptest-locks-the-narrow-truth

**Origin**: the origin project's documentation-accuracy discipline.

**In antigen**: every antigen declaration's documentation must reflect what the
fingerprint actually matches and what the witness actually proves. Proptests on the
witness ensure the docstring stays accurate.

### recognition-not-design

**Origin**: the origin project's convergence-patterns work.

**In antigen**: antigen *recognizes* failure-classes that already exist in code. It does
not *design* failure-classes from scratch. Each new antigen is a recognition, not an
invention. The 8-class first-principles taxonomy is recognition of existing structure;
antigen-stdlib is recognition of existing common patterns.

### conditional-lean-collapse

**Origin**: the origin project's coordination disciplines.

**In antigen**: when routing antigen declarations through composition, preserve the
conditional structure. e.g., if a function is "fragile to X under condition C, immune to
X under condition !C," do not collapse to "fragile to X" or "immune to X." Express the
conditional via separate antigen instances or refined fingerprints.

### grammar-vs-vocabulary cut

**Origin**: ADR-015 (systems review of ADR-010).

**In antigen**: the distinction between the *grammar* of fingerprint expressions
(node-kind × field-path × constraint-op + Boolean composition — the load-bearing
structural commitment) and the *vocabulary* of named operators that project the
grammar (`item:`, `name: matches(...)`, `body_contains_macro(...)`, etc. — the
projection surface). ADR-010 ratified the vocabulary explicitly; the grammar was
implicit. ADR-015 surfaces the grammar as the structural commitment; the vocabulary
grows with stdlib content.

### evaluator trait

**Origin**: ADR-015 §S3.

**In antigen**: a per-fingerprint trait abstracting the engine that evaluates a
fingerprint against a Rust source AST. Private in v0.1; goes public when a second
backend ratifies (likely ADR-016's temporal evaluator). Two implementations expected
in `antigen-fingerprint`: `SynEvaluator` (item-level operators, native syn) and a
deferred body-pattern backend (Path 1 library / Path 2 subprocess / Path 3 defer
per ADR-015's deferred sibling decision).

### delegation-boundary discipline

**Origin**: ADR-015 + ADR-013 + ADR-016 convergence; postures.md V0+1 candidate.

**In antigen**: the architectural posture that the boundary between antigen-grammar
and external substrate is the design surface; the implementation across that boundary
is delegation, not reinvention. Each engine is a delegation-boundary; the evaluator
trait abstracts. Operationalises ADR-002 (compose-don't-compete) at engine-axis +
witness-axis + temporal-axis.

### accept-and-note discipline

**Origin**: ADR-016 (substrate-honesty refinement).

**In antigen**: when a macro arg-parser receives a field whose audit-side check is
not yet implemented (e.g., `verified_at` before its audit-side check ships),
the parser MUST accept the field with an explicit known-limitation note rather than
silently accept (a sub-clause F violation) or reject (a forward-compat block).
The pattern matches ADR-001 Amendment 1 Change 4's witness-tier-deferral discipline.

### audit tier-honesty

**Origin**: ADR-005 Amendment 3.

**In antigen**: the discipline that audit's status output (`is_well_formed()`,
`WitnessStatus`, `witness_tier` field in JSON) must report the tier its verification
work *actually* supports — never a stronger tier. Where verification at the claimed
tier has not occurred, audit MUST either (a) report at the lower tier its work
actually supports, OR (b) emit a tier-honesty audit hint. Sub-clause F applied at
the audit reporting surface.

### verified_at / evidence / stale_after

**Origin**: ADR-016 (temporal recognition surface).

**In antigen**: optional fields on `#[antigen]`, `#[presents]`, and `#[defended_by]`
declarations carrying temporal claims:

- `verified_at = "<commit-hash>"` — the commit at which this declaration was last
  verified. `cargo antigen audit` walks `git log` to determine
  whether HEAD is reachable and how far past.
- `evidence = ["<URL-or-commit-or-RFC>"]` — supporting evidence for the
  declaration's claim. Audit can verify accessibility (opt-in via
  `--check-evidence`).
- `stale_after = <interval>` — declaration becomes "stale" past this interval.
  Syntax: `commits(N)`, `days(N)`, `version("X.Y.Z")`.

The macro-parser accepts these per the accept-and-note discipline; the audit
implements the corresponding checks.

### analysis-level × temporality grid

**Origin**: ADR-016 grid framing; ratified jointly across ADR-015 + ADR-016.

**In antigen**: the two-axis recognition substrate. Analysis-level axis: syn / HIR /
MIR / runtime (ADR-015 picks engines along this axis). Temporality axis: snapshot /
longitudinal (ADR-016 picks the temporal level). v0.1 populates two cells:
(syn, snapshot) and (runtime, longitudinal). Future ADRs populate other cells as
substrate evidence accumulates.

### filter / proof split (fingerprints filter; witnesses prove)

**Origin**: ratified at ADR-010 Amendment 3 Clause D; promoted to top-level
architectural principle at ADR-010 Amendment 4.

**In antigen**: the operative semantic posture across antigen's recognition surface.
Fingerprints are recall-tuned candidate filters; precision lives in the witness
layer (composed via ADR-002). False positives from the filter are EXPECTED and not
failure states; ADR-011 `#[antigen_tolerance]` is the structural relief valve for
matches-by-design. The split is what makes cheap syntax-level fingerprint operators
sufficient for v1: precision pushes to witness, not filter.

### rationale-as-required-field

**Origin**: ADR-001 Amendment 1 Change 7 (originating observation); ADR-005
Amendment 2 (transverse sub-clause F discipline).

**In antigen**: the principle that every primitive that extends trust requires an
explicit justification field (named `rationale`, `summary`, `references`, `witness`,
or an ADR-specific equivalent) by default. Five+ manifestations across the carrier
set. New trust-extending primitives without justification fields require active
argument; primitives with justification fields are the unmarked default.

### depth-shift discipline

**Origin**: posture #7 (`postures.md`); canonical example at ADR-005 Amendment 3's
motivating Finding.

**In antigen**: the load-bearing structural commitment lives one tier deeper than
the visible decision. Before drafting, deconstructing, or rejecting any proposal,
ask "what is the X−1 commitment that determines whether X works?" The discipline
is *operationally self-producing* — each application of substrate-honesty creates
the conditions for the next tier of substrate-honesty to surface. Operational
signature: **no fixed point**. The instances are the same pattern across
operational layers, not analogy at different scales.

The discipline's structural sibling is biology-as-search-heuristic: both are
instances of *recursion-into-substrate one tier below the visible question* at
different operational layers (failure-mode discovery vs design-decision
identification). The biological cognate is immune-system scale-invariance —
the same recognition shapes operate at every scale, by structural identity not
analogy.

### structural-tier vs maintenance-tier

**Origin**: `structural-memory.md` whitepaper; implicit throughout earlier substrate.

**In antigen**: the distinction between two kinds of correctness-carrier in
software. **Maintenance-tier** carriers (documentation, tests, comments, wiki
pages) require ongoing human effort to stay current; they drift silently when
maintenance lags. **Structural-tier** carriers (type signatures, trait impls,
borrow-checker constraints, antigen declarations) have their currency enforced
by the same machinery that compiles or scans the code; they cannot be silently
wrong because the system itself fails to compile / scan / pass checks when the
substrate diverges from reality. Antigen extends the structural tier to *memory
of failure-classes*, which had previously only had maintenance-tier carriers
(commit messages, post-mortems, ADRs, tribal knowledge).

### co-native (by design)

**Origin**: project posture from inception; explicit in
`for-llm-collaborators.md`, `structural-memory.md`, `concepts.md`.

**In antigen**: the property that both human developers and AI agents can read
the same vocabulary natively, without translation. Declarations in code are
readable as Rust syntax by humans and as parseable structured data by LLMs; the
biological metaphor is universal lived experience for humans + unambiguous
semantic cognate for LLMs; audit reports come in both human-readable and JSON
forms; the vocabulary's structure is the same for both audiences. Co-native is
load-bearing for hybrid teams because failure-class memory carried by tools that
require translation between cognition types fails in either direction.

### multi-component architecture

**Origin**: `concepts.md`; first-principles multi-component framing.

**In antigen**: the framing that antigen is not a single tool but a vocabulary
that lets teams compose multiple kinds of structural immunity. Seven components
currently named (enumeration is open per recognition-not-design): (1) dev-in-the-
loop immunity, (2) passive scan/lint/tool, (3) test-integration, (4) knowledge-
ecosystem (references), (5) cross-version/lineage, (6) cross-crate/ecosystem,
(7) real-time/CI feedback. Components compose orthogonally; a team can adopt
at the floor (component 2 alone) and grow as discipline matures. The components
fabric is biology-tier (C1-C3, C5-C7) and engineered-boundary tier (C4, the
knowledge-ecosystem references which extend past biology's domain).

### encoded readiness (as memory type)

**Origin**: `structural-memory.md` whitepaper, section 4; biological cognate.

**In antigen**: a type of memory in which the *recognition machinery itself is
the memory*, contrasted with **storage-and-retrieval memory** in which records
are stored and fetched when needed. The biological immune system's B-cells and
T-cells don't retrieve records of past pathogens; they ARE records — the cell's
recognition machinery encodes the memory. Antigen operates on encoded-readiness
memory: `#[antigen]` declarations don't store records of past bugs for retrieval,
they encode the recognition machinery for the failure-class. The structural
property differs from storage-and-retrieval: storage memory can become stale
silently as the reality it describes drifts; encoded memory is structurally
entangled with the system that enforces it, so divergence is visible as
recognition mismatch rather than silent drift.

---

## Marked-unknown terms (ADR-041)

The three markers for the *felt-but-unnamed* danger — a developer's honest "something
is wrong here, I can't name the failure-class yet." They sit on a two-axis plane
(**magnitude** × **existence-certainty**) that is off the classification dial: a marked
unknown is not a named antigen and never gates a build. The scan re-reads them; the
learning core (below) clusters them into draft fingerprints.

### marked unknown

**Biological referent**: a *sense of alarm* before identification — the immune system
mounting a response to a not-yet-characterized threat (the felt-danger that precedes a
named pathogen).

**Rust ecosystem analog**: a site annotated `#[aura]`, `#[dread]`, or `#[red_flag]`
declaring a felt-but-unnamed worry, with a **required** `trigger` string saying what
feels wrong. Surfaced by `cargo antigen scan` as a `MarkedUnknown`; never a named
failure-class, never a build gate. The canonical noun for the whole family is *marked
unknown*.

**Introduced in**: ADR-041. Carriers in
[`antigen-macros/src/lib.rs`](../antigen-macros/src/lib.rs) (`aura` / `dread` /
`red_flag`); structured output in [`antigen/src/finding.rs`](../antigen/src/finding.rs)
(`Magnitude`, `ExistenceCertainty`).

### magnitude

**Biological referent**: the *intensity* of the alarm — how loud the felt-danger is.

**Rust ecosystem analog**: one axis of the marked-unknown plane, a three-level ordinal
`smell → aura → dread` (the `Magnitude` enum, `finding.rs`). Higher magnitude is a
stronger, more-localized worry. Orthogonal to existence-certainty (the other axis) — a
mark fixes a *corner* of the plane.

### existence-certainty

**Biological referent**: how *sure* the immune system is that there is a real threat at
all — distinct from how alarming it would be if real.

**Rust ecosystem analog**: the second axis of the marked-unknown plane, `unsure` / `sure`
(the `ExistenceCertainty` enum, `finding.rs`). `#[dread]` is high-magnitude but `unsure`
(scared, not certain); `#[red_flag]` is `sure` (certain something is wrong). The two
markers differ on this axis, not on magnitude.

### #[aura]

**Definition**: the **light** marked unknown — low magnitude, `unsure`. "Something *may*
be off here, I can't name it, check later." A mild substrate-smell; surfaces at the
dial's non-gating floor, never gates, never nags. The `trigger` argument is **required**.

**Biological referent**: a faint prodrome — a vague sense that precedes (and may never
become) a recognized illness.

**Introduced in**: ADR-041. Carrier: `antigen::aura`.

### #[dread]

**Definition**: high magnitude, **low** existence-certainty (`unsure`) — the *angor
animi* corner: "something *is* wrong here, I can't name it, look now." Scared, not sure.
Surfaces at the non-gating floor; the `trigger` is **required** (a triggerless `#[dread]`
is a compile error). The marker the learning core's `propose` clusters by default.

**Biological referent**: *angor animi* — the clinical "sense of impending doom" a patient
reports before a diagnosable event; a strong felt-alarm without a name.

**Introduced in**: ADR-041. Carrier: `antigen::dread`.

### #[red_flag]

**Definition**: **high** existence-certainty (`sure`), unnameable — "I'm *sure*
something is wrong here, I can't name it, act now." The one marked unknown that records
at the highest internal severity on first match, because its defining axis is
certainty-that-something-is-wrong. The `trigger` is **required**.

**Biological referent**: the clinician's "red flag" — a sign that, even unexplained,
mandates immediate action.

**Introduced in**: ADR-041. Carrier: `antigen::red_flag`.

---

## Learning-core terms (ADR-044/045/047/048)

The **learning core** (`antigen::learn`) is the affinity-maturation arm: it takes a
*cluster* of marked unknowns that share a structural shape and drafts a candidate
fingerprint for the failure-class they might share — gated by a self-tolerance check so
the draft never flags known-good code. Its CLI surface is `cargo antigen propose`.

> **The load-bearing line.** The learning core **drafts and routes**, it does not
> **name**. A draft is a *ratifiable suggestion*, never an auto-asserted `#[presents]`
> and never a named class. On antigen's own marks, `propose` **routes the draft to a
> human ratifier** rather than promoting it: the machine does the syntactic half; a
> human (or an incident) ratifies the semantic half. See
> [observe-don't-declare](#observe-dont-declare).

### anti-unify

**Biological referent**: somatic-hypermutation generalization — broadening a receptor so
it binds a *family* of related epitopes rather than only the first one seen.

**Rust ecosystem analog**: the generator step
(`antigen::learn::propose::anti_unify`). Given a cluster of structurally-similar marked
items, it produces a draft `Fingerprint` by keeping the features shared by **every**
member (the skeleton conjuncts: item-kind, trait-impl identity, body-calls common to all)
and wrapping the features present in **some but not all** members in an `any_of([...])`
disjunction — the *discriminating* signal. It generalizes **to disjunction**, never the
naive "drop the differing leaves" collapse (which over-generalizes to "any `Drop` impl"
and would flag clean code). The output is a **hypothesis**, not a promotable fingerprint.

**Introduced in**: ADR-045. Defined in
[`antigen/src/learn/propose.rs`](../antigen/src/learn/propose.rs).

### clean corpus

**Biological referent**: *self* — the body's own healthy tissue the immune system must
not attack. Negative selection in the thymus deletes lymphocytes that bind self.

**Rust ecosystem analog**: the set of items the **operator** supplies and labels as
known-good, against which a draft is checked (`cargo antigen propose --clean-root`). The
gate promotes a draft only if it **spares every clean-corpus item**. Load-bearing:
antigen **never auto-labels** unmarked code as clean — the operator supplies and labels
the corpus, and the gate is only as strong as that corpus (a corpus-bounded check, not a
total guarantee). The adjectival form is *clean-corpus* (as in "a clean-corpus item"); the
canonical noun is *clean corpus*.

**Introduced in**: ADR-044/047.

### GATE-G (the self-tolerance gate)

**Biological referent**: the **thymus** — the organ that performs negative selection,
deleting any immature lymphocyte whose receptor binds self before it can cause
autoimmunity.

**Rust ecosystem analog**: the self-tolerance gate (`antigen::learn::self_tolerance`,
GATE-G in ADR-047) that every draft must pass before it can be promoted. It decides three
checks: **(A)-binary** (the draft carries a discriminating signal, not just bare
structure), **near-miss non-vacuity** (the clean corpus actually exercised the draft — see
*near-miss*), and **spare-clean** (the draft binds no clean-corpus item). A draft that
fails spare-clean is **autoimmune** and refused; a draft that is safe but has no near-miss
**routes to a human**.

**Honest scope**: GATE-G is safe for the v0.5 generator-fed path — `anti_unify` emits a
flat top-level `all_of`, which the gate's primitives read correctly. It is *not*
unconditionally safe for any hand-constructed draft shape; a draft built with a
`Not(structural-anchor)` combinator is a known gap, closed before any future re-mint
surface ships. Do not read GATE-G as safe for arbitrary drafts.

**Introduced in**: ADR-045 (the gate), ADR-047 (the three-check hardening). Defined in
[`antigen/src/learn/self_tolerance.rs`](../antigen/src/learn/self_tolerance.rs).

### near-miss

**Biological referent**: an affinity-maturation control — a self-antigen *almost* bound by
a maturing receptor proves the receptor is discriminating in a real neighborhood, not
binding at random.

**Rust ecosystem analog**: a clean-corpus item that matches **all-but-one** of a draft's
top-level conjuncts and is spared by failing exactly that one (`is_near_miss`,
`self_tolerance.rs`). A near-miss is the proof that the gate made a *real in-family
discrimination* — it spared an item it plausibly could have flagged, not one it was never
near. A draft needs **≥2 conjuncts** to have a valid near-miss; with no near-miss in the
corpus the gate cannot certify the draft generalizes, so it **routes to a human**
(`NotCorpusWitnessable`) rather than promote.

**Introduced in**: ADR-047 §Mechanics.

### route-to-human

**Biological referent**: handing an unresolvable recognition decision up to a
higher-order check rather than acting on incomplete evidence.

**Rust ecosystem analog**: the first-class outcome where a draft is **safe** (it spares
the corpus and carries a discriminating signal) but the corpus holds **no near-miss**, so
the gate cannot certify the draft generalizes — it hands the candidate to a human ratifier
instead of promoting it (`ToleranceVerdict::NotCorpusWitnessable`; the `route-to-human`
JSON outcome). This is the gate being honest, not a failure: `cargo antigen propose` exits
`0` and prints the drafted candidate for inspection. It is the outcome on antigen's own
marks: antigen anti-unifies a draft from its own `#[dread]` marks and routes it to human
ratification — it does not name a class for itself.

**Introduced in**: ADR-047/051.

### PromotedDraft

**Biological referent**: a matured, survival-selected receptor lineage — the antibody that
passed negative selection and is cleared for use.

**Rust ecosystem analog**: a capability-token type (`PromotedDraft`, `self_tolerance.rs`)
whose mere existence is **structural proof** that all three GATE-G checks held —
(A)-binary, near-miss non-vacuity, and spare-clean. It carries a `tier` (the gate-assigned
score). There is **no public constructor, no `From<Fingerprint>`, no `Default`, no
`Deserialize`** — it cannot be forged from hand-written JSON; the only way to obtain one is
to pass the gate. Even a `PromotedDraft` is a *ratifiable suggestion at a calibrated tier*,
never an auto-asserted or auto-named class.

**Introduced in**: ADR-048 (the capability-token), ADR-049 (the score).

### propose (verb + CLI)

**Biological referent**: the affinity-maturation reaction proposing a matured receptor
lineage for survival selection.

**Rust ecosystem analog**: two surfaces of the same operation. The **library** function
`antigen::learn::propose::propose(cluster, clean_corpus)` runs anti-unify → GATE-G and
returns `Result<PromotedDraft, ProposeOutcome>` — every non-promotion reason is legible,
never a bare `None`. The **CLI** verb `cargo antigen propose` re-acquires the marked
cluster under `--cluster-root`, collects the operator-supplied corpus under `--clean-root`,
routes them through `propose()`, and **renders** the outcome as a ratifiable suggestion. A
`propose` run leaves the source tree **byte-unchanged** — it observes, it does not write
markers. See [`cli-reference.md`](cli-reference.md#propose) and
[`examples-guide.md`](examples-guide.md).

**Introduced in**: ADR-045 (library); the CLI.

### observe-don't-declare

**Biological referent**: the immune system surfacing a recognition for higher-order
confirmation rather than committing to a response on its own — the syntactic/semantic line
a single cell must not cross alone.

**Rust ecosystem analog**: the discipline (ADR-044) that the learning core may draft the
**syntactic** half of a failure-class (the fingerprint a machine can bind by construction)
but must never assert the **semantic** half (that this names a real failure-class). The
machine *observes* and *suggests*; the human or an incident *declares* (ratifies) the draft
into a named class. This is why `propose` renders a suggestion and never writes a
`#[presents]`, and why `"promoted": false` is always emitted in the JSON.

**Introduced in**: ADR-044.

> **Vocabulary note — "ratify" / "ratification".** In the learning-core sense above, a
> *human ratifies a draft into a named failure-class* (the semantic half of
> observe-don't-declare). This is distinct from the ADR-019 [`ratification`](#ratification)
> noun (the `.attest/<Antigen>.json` sidecar struct recording that a discipline was
> reviewed). They rhyme — both are a human blessing — but name different things: the
> learning-core sense is the **act** of naming a draft; the ADR-019 sense is the on-disk
> **record**. Use "ratify a draft" for the former and "a Ratification sidecar" for the
> latter.

---

## Maturing-organism terms (the v0.6 efferent organs)

The learning-core terms above cover how a class is *born* (the afferent arc: mark →
anti-unify → gate). These cover how a born class *lives* over its lifetime — matures,
is sensed for drift, is classified, and is curated. Each is a typed, tested
`antigen::learn` library API; the `cargo antigen` verb that drives the full loop
end-to-end is the v0.7 frontier.

### life-record

**Biological referent**: an immune cell's accumulated history — every exposure,
every selection event — kept as the cell's own past, never overwritten.

**Rust ecosystem analog**: a learned class's append-only autobiography
(`antigen::learn::life_record`): a typed event stream (`Born`, `Matured`,
`Scored`, `Drifted`, `Ratified`, `Retired`, …) that every sensor reads. Current state
is **derived, never stored** — `is_retired()` is a fold over the stream (did a
`Retired` event ever happen?), not a flag kept in sync. The record stores *events*
(what happened), not *claims* (what is true now), which is why it cannot drift out of
sync with itself — the same property that makes a `.git` history trustworthy. A forget
is a pushed `Retired` tombstone, never an erasure.

### Affinity (the 2-vector)

**Biological referent**: binding affinity — how tightly an antibody binds its target.
A maturing B-cell climbs toward higher affinity through somatic hypermutation, selected
against self.

**Rust ecosystem analog**: a learned class's **height**, recorded as a 2-vector
`Affinity { recall, precision }` — deliberately **not a scalar**. The two axes trade
off: **recall** (BIND-TIGHT — the fraction of the defect cluster the fingerprint
matches) and **precision** (SPARE-CLEAN — the fraction of clean code it correctly
spares). `Affinity` is `PartialOrd` with **no `Ord`**: two affinities where one wins on
recall and the other on precision are genuinely incomparable (`partial_cmp` returns
`None`), and that incomparability is the point — a single number would silently pick a
point on the trade-off and hide the choice it makes. The "maturation ceiling" is not a
threshold someone set; it is the Pareto frontier a draft can no longer improve off of.

> **It is not a probability.** The 2-vector is the honest placeholder for a calibrated
> confidence antigen does not yet compute — score calibration is the v0.7 frontier. Read
> it as a trade-off surface, never as "how likely this is a real failure."

### drift-detection (ADWIN)

**Biological referent**: sensing that the pathogen population has shifted — that a
once-effective defense is losing its grip — and knowing when you have watched long
enough to be sure.

**Rust ecosystem analog**: a batch-pure change-point detector
(`antigen::learn::adwin`, after Bifet-Gavaldà 2007) that watches a class's affinity
trajectory for a downward change. Its verdict is three-valued — `Drift`, `NoDrift`, and
**`UnderPowered`** — and the third is the spine, not a corner case. Below a
statistical-power threshold a change is *mathematically undetectable*; the detector then
returns `UnderPowered` ("I cannot yet see drift for this class, and here is exactly when
I will be able to") rather than a false `NoDrift`. At today's scale (classes have matured
only a handful of times) `UnderPowered` is the **default verdict**, and a detector that
fires zero and says so is the correct, honest organ — the same organ that fires once
trajectories lengthen, with no code change.

> **Silence has two distinct causes — "no drift" and "can't see" — and they are distinct
> verdicts.** Collapsing them into one `bool` is exactly the silent-miscalibration antigen
> exists to catch.

### CURATE

**Biological referent**: the contraction phase — after an immune response, the body
prunes its expanded memory pool, keeping the lineages still worth the cost and letting
the rest decay. The pruning is itself gated, so a memory still defending the body is
never deleted by mistake.

**Rust ecosystem analog**: the **efferent (act) organ** (`antigen::learn::curate`) — the
one station on antigen's sense → classify → act arc that *acts* on a learned class rather
than sensing or classifying it. It maps a class verdict to one action on the
**reversible-first ladder**:

| Rung | Action | From verdict | Reversible? |
|---|---|---|---|
| 1 | **Keep** | WellDefended | yes — null action |
| 2 | **Hold** | Dormant | yes — discards nothing |
| 3 | **RouteToHuman** | RouteToHuman | yes — escalates the undecidable |
| 4 | **ReArm** | Evaded | yes — records a drift, broadens coverage |
| 5 | **Forget** | Obsolete | **no — the only irreversible rung** |

The ladder is ordered reversible → irreversible deliberately: `Forget` is the last rung
and is reachable from a single verdict (`Obsolete`) alone, type-enforced by
`is_auto_forgettable`. A future edit that tried to forget any other verdict would have to
delete the gate to do it. CURATE's load-bearing property is not what it forgets — it is
what it is structurally incapable of forgetting.

### conservatism-JOIN

**Biological referent**: a tolerance checkpoint on memory deletion — if any one of the
signals that would license pruning a memory cell is unreadable, the cell is spared.

**Rust ecosystem analog**: the safety floor that fuses the sensor channels before CURATE
acts (`antigen::learn::discriminator::fuse_channels`). **If either channel is blind — the
drift detector returns `UnderPowered`, or the silent shape-sensor returns `Indeterminate`,
or a drift signal arrives non-finite — the fused verdict is `RouteToHuman`, regardless of
what the other channel says.** A blind channel cannot endorse an irreversible forget.
Because every class is drift-blind by default at today's scale, this is why the system
cannot auto-forget anything today: the honest behavior is "the loud sensor sees nothing
yet; route the undecidable to a human," never a fabricated signal.

---

## Glossary maintenance

This glossary is itself a discipline artifact in the origin project's tradition. As the antigen project
matures, terms WILL drift in meaning. The discipline:

1. Every PR that introduces new vocabulary or refines existing terms updates this glossary.
2. Every section header in design docs adds a glossary cross-reference (e.g., "see
   glossary: vaccination, lineage").
3. Vocabulary drift is treated as a sub-clause E violation (coordinate-explicitness
   failure) and triggers a glossary review.

Maintained by: the antigen team. Last updated: 2026-05-22 (v0.2 supply-chain +
convergent-evidence campaign: added ContentHashMismatch, WitnessClass, SeedKind,
SupplyChainDefenseFamily, ConvergentEvidenceFamily carrier entries).

---

## Substrate-witness terms (ADR-019 + ADR-020)

### substrate-witness

**Origin**: ADR-019 substrate-witness predicate family.

**Biological referent**: the immune system checks substrate other than the target cell
itself — B-cell memory (germinal-center history), antibody secretion, oracle-completion
markers, signed git trailers. The recognition reads the surrounding substrate, not just
the immediate target.

**Rust ecosystem analog**: a `#[presents(Antigen, requires = <predicate>)]` expression that
evaluates substrate other than the code being audited — ratified docs, team sign-off
records, oracle-completion markers.

**In antigen**: a witness predicate that reads from `.attest/` JSON sidecars rather than
from the Rust source AST. Extends the witness vocabulary (ADR-001, ADR-002) to discipline
failure-classes whose immunity evidence lives outside the code.

**The code-tier vs substrate-tier choice**: the substrate-tier witness
(`#[presents(Antigen, requires = ...)]`) is one of two evidence channels; the other is the
code-tier witness (`#[defended_by]`). Quick heuristic: **can a test execute the thing you're
defending?** If yes, reach for `#[defended_by]` (code-tier — a test, proptest, lint, or
formal-proof runs it). If no — the failure-class is about substrate state that a test can't
verify (a sign-off record, a ratified doc, an unpinned dependency, an un-reviewed discipline)
— reach for `requires =` (substrate-tier — `cargo antigen audit` evaluates the predicate
against the `.attest/` sidecar). These are co-equal siblings, not basic vs advanced; the choice
is driven by what kind of failure-class you're defending against. See [`### witness`](#witness)
(§Composition terms) for the full contrast-pair with category-mapping
(FunctionalCorrectness → `#[defended_by]`; SubstrateAlignment → `requires=`).

### ratification

**Origin**: ADR-019 §M3 schema.

**Biological referent**: germinal-center B-cells undergo somatic hypermutation and
affinity-maturation — the resulting antibody lineage is "ratified" as effective by
survival selection.

**Rust ecosystem analog**: the JSON sidecar at `.attest/<Antigen>.json`. Serde-derived;
single source of truth for audit/CLI/editor validation.

**In antigen**: the structured on-disk record that a named discipline was reviewed. A
`Ratification` struct carries `schema_version`, `kind` (Immunity or Tolerance),
`antigen` identifier, and a list of `ItemRatification` entries (one per presented item).

### attestation

**Origin**: ADR-019 §M4 CLI; ADR-020 cross-cutting attestation primitive.

**Biological referent**: the act of an immune cell recognizing and recording an encounter.

**Rust ecosystem analog**: `cargo antigen attest sign` — adds a `Signer` entry to a
sidecar, recording that a named reviewer verified the item against a stated fingerprint.

**In antigen (ADR-019)**: the act of signing a sidecar at a specific fingerprint with
stated identity-binding strength (`TextStamp | GitTrust | CryptoSigned`).

**In antigen (ADR-020)**: the `attested = (who, allowed_types, why, scope)` macro
parameter that declares review intent at code-authoring time, independent of any sidecar.
Layer 1 compatible (no sidecar required at compile time).

### signer-basis

**Origin**: ADR-019 §M3 schema.

**Biological referent**: affinity-maturation lineage — was this antibody produced by a
fresh encounter (germinal-center reaction) or by carry-forward from a prior memory cell
(anamnestic response)?

**Rust ecosystem analog**: the `SignerBasis` enum in `antigen-attestation::schema`.
`Fresh { reasoning }` = fresh review of the current state. `DeltaFrom { prior_fingerprint,
cumulative_root_fingerprint, chain_depth, rationale }` = carry-forward from a prior
attestation with explicit anti-laundering safeguards.

### delta-chain

**Origin**: ADR-019 §M4 anti-laundering safeguards.

**Biological referent**: the chain of anamnestic (recall) responses between germinal-center
reactions. Too many recall responses without a fresh encounter risks immune memory drift.

**Rust ecosystem analog**: a sequence of `SignerBasis::DeltaFrom` entries for a single
signer at a single item. The chain-depth cap (default 3) prevents laundering: gradual
code drift via many small deltas that each look innocuous.

**Anti-laundering safeguards**: chain-depth cap (enforced at write time and audit time);
`cumulative_root_fingerprint` tracking (schema field); non-empty rationale required.

### tolerance-ratification

**Origin**: ADR-019 §Decision; closes ADR-011 open question.

**Biological referent**: immune tolerance — the deliberate decision to NOT attack a
self-antigen or harmless foreign antigen. Documented in the immune system's regulatory T-cell
records.

**Rust ecosystem analog**: `cargo antigen tolerate` CLI family; `RatificationKind::Tolerance`
sidecar. The same `Ratification` schema as immunity, with `kind = tolerance` discriminator.

**In antigen**: replaces the "vibes-grade" `#[antigen_tolerance(X, rationale = "...")]`
inline annotation with a structured sidecar carrying `who`, `date`, and explicit rationale.
`tolerance-vibes-grade` audit hint surfaces sites that haven't opted in.

### evidence-kind

**Origin**: ADR-019 §Decision third axis; ADR-005 Amendment 3 extension.

**Biological referent**: the three arms of the immune system — innate/germline-encoded
(TypeSystemProof), trained (Behavioral), adaptive/substrate (SubstrateState) — are parallel
evidence kinds, not a ranked scale.

**Rust ecosystem analog**: `EvidenceKind` enum: `None | TypeSystemProof | Behavioral |
SubstrateState`. Third orthogonal axis on `ImmunityAudit` alongside `WitnessTier` and
`AuditHint`.

**Key property**: EvidenceKind is a parallel axis (NOT ordered scale). CI gates specify
exact-kind requirements, not threshold comparisons. Per-kind ceilings: TypeSystemProof →
FormalProof; Behavioral → Execution; SubstrateState → Execution.

### signature-strength

**Origin**: ADR-019 §Decision (the notary-arc grounding).

**Biological referent**: notary arc — pre-institutional peer testimony (TextStamp), civic
notary with place-bounded accountability (GitTrust), notary public with universal license
and cryptographic sealing (CryptoSigned).

**Rust ecosystem analog**: `SignatureStrength` enum: `TextStamp | GitTrust | CryptoSigned`.
Ordinal (retains `PartialOrd + Ord` for weakest-link reporting). `TextStamp` = name +
timestamp, no external verification. `GitTrust` = identity bound to `git config user.name
+ email`. `CryptoSigned` = DSSE-PAE-encoded with Sigstore identity (v0.4+).

### discipline-vs-machinery unification

**Origin**: ADR-019 §Decision asymmetry rule.

**In antigen**: substrate-witnesses and cross-crate witnesses share discipline-level
unification (tier-honesty; SubstrateState evidence kind; Execution ceiling) but NOT
machinery (separate parsers, separate recognition pipelines). Enforced via in-code comment
blocks and a unification-guardrail precision test.

### closed-set tool bright-line

**Origin**: ADR-019 §Decision §4.

**In antigen**: the 4-point rule for leaf primitives that invoke external binaries:
(1) binary named in leaf source, (2) has own release process/package-managed, (3) does
NOT execute user-supplied code, (4) invocation args fixed except for declared substrate
parameters. Replaces vague "ecosystem tools" with a testable criterion at leaf-design review.

### cross-cutting attestation

**Origin**: ADR-020 cross-cutting attestation primitive.

**Biological referent**: a notary witnesses any document regardless of domain — a civic
notary doesn't specialize in property transfers vs wills; they witness any document
presented to them. Grounded by B6 notary arc.

**In antigen**: `attested = (who, allowed_types, why, scope)` as a macro parameter on ANY
antigen macro (`#[antigen]`, `#[presents]`, `#[antigen_tolerance]`). Cross-cutting = applies
across all macro types without domain specificity. Elevates the REVIEW layer from implicit
to explicit. Layer 1 compatible: no sidecar required at compile time.

### coverage-frontier (the *spatial* frontier — the SHIPPED referent)

**Origin**: `audit/coverage.rs` (the coverage / reachability audit); ADR-005 Amendment 3
(audit reports its own tier honestly); `decisions.md` "scan-coverage ignorance frontier".

**Biological referent**: ignorance — the boundary of what the immune system has *encountered*.
Not yet anergy (a tolerated-but-known antigen) and not immunity; simply the edge of what has
been seen at all. Distinct from a *judgement* about what was seen.

**Rust ecosystem analog**: the **spatial** boundary of what the scan REACHED — the set of
workspace members / sites `cargo metadata` reported but that the scan did not inspect
(`UnreachedSite` / `UnreachedCause`). It answers *"what did we not look at?"*. The
coverage-frontier is empty iff every reachable site was scanned; a non-empty frontier is a
tier-honest *"we did not reach here"*, NOT an assertion that those sites are clean.

**Disambiguation (ADR-044 Amendment 1)**: this is the ONLY meaning of the bare word
**"frontier"** in antigen. The *epistemic* concept once called "frontier-honesty" is renamed
**claim-scope honesty** (next entry) to keep the Vocabulary-Lock invariant (one term, one
referent). Do not write "frontier" for the don't-over-claim discipline.

### claim-scope honesty (the *epistemic* discipline — renamed from "frontier-honesty")

**Origin**: ADR-044 (cross-cutting v0.4 spec constraint), renamed per ADR-044 Amendment 1.
Descriptive gloss: **"proven-not-potential honesty."** Formally grounded in Cousot & Cousot
1977 (soundness-without-completeness) + Rice 1953 (the trade is forced).

**Rust ecosystem analog**: the labeling duty that every verdict-emitting or candidate-generating
component report **what it ACTUALLY proved — scoped to its real reach, with its own soundness
boundary — never the potential-maximum its category could provide.** It is the *epistemic* dual
of the confidence dial (ADR-039): the dial claim-scopes DETECTION; this claim-scopes GENERATION
and INVOCATION. Carried per-ADR as a three-clause **claim-scope statement** (what it proves /
what it does NOT prove / who ratifies the undecidable half).

**Why NOT "soundness-honesty"** (a rejected candidate): antigen is *calibrated-under-claiming*,
NOT formally sound in the Cousot sense (ADR-044 qualified-nuance #1). Naming it "soundness-X"
would re-import the very over-claim the discipline disclaims — itself a claim-scope violation.

**Why NOT "frontier"**: "frontier" is reserved for the spatial coverage-frontier (previous
entry). The two rhyme (both about boundaries/reach) but are orthogonal: coverage-frontier =
*what we did not inspect*; claim-scope honesty = *do not over-claim what we DID inspect*.
