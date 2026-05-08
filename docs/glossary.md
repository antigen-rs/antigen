# Antigen — Glossary

> Anchors every term currently in flight to: biological referent + Rust ecosystem
> analog + introducing doc. Vocabulary drift is a known failure mode (DEC-022 catches
> it in tambear); this glossary is the antibody.

---

## Core terms

### antigen

**Biological referent**: a molecule (often on a pathogen's surface) that the immune
system recognizes as non-self and responds to.

**Rust ecosystem analog**: a named, structurally-fingerprinted **failure-class**. e.g.,
`FrameTranslation`, `BoundaryViolation`, `OptionalityCollapse`. Declared via
`#[antigen(name = "...", fingerprint = "...")]`.

**Introduced in**: `design-intent.md`, `api-shape.md`.

### antibody

**Biological referent**: a protein produced by B-cells that specifically binds and
neutralizes one antigen.

**Rust ecosystem analog**: an **immunity witness** — a test, proptest, phantom-type
proof, or delegated lint that proves immunity to a specific antigen. Required parameter
of `#[immune(antigen, witness = ...)]`.

**Note**: "antibody" is used colloquially in design docs but the ratified API uses
"witness" because antibodies in biology are *response*, while Rust witnesses are
*proof-of-immunity-claim*. The biology rhymes; the Rust term is more precise.

**Introduced in**: `design-intent.md` (metaphor), `api-shape.md` (witness).

### vaccination

**Biological referent**: deliberate exposure to a weakened antigen so that B-cells
develop memory before encountering the live pathogen.

**Rust ecosystem analog**: applying a known immunity pattern across a structural family
of types. The cargo subcommand is `cargo antigen vaccinate <antigen> <pattern>`.
Operates on a refinement-lattice of types (e.g., "every enum named `*Class`").

**Introduced in**: `api-shape.md`.

### immunity

**Biological referent**: the state of being protected against a specific pathogen due to
prior exposure or active defense.

**Rust ecosystem analog**: a `#[immune(antigen, witness = ...)]` declaration on a
function/type/method, with a witness that is checked by tooling. Immunity is *claimed*
by the declaration AND *verified* by the witness; the marker without the witness is not
a claim.

**Introduced in**: `design-intent.md`, `api-shape.md`.

### fragility / vulnerability

**Biological referent**: susceptibility to a specific pathogen; the absence of immunity.

**Rust ecosystem analog**: code marked `#[presents(antigen)]` — explicitly declares
vulnerability to a known failure-class without claiming immunity. `cargo antigen scan`
flags every presentation that lacks a corresponding immunity declaration.

**Note**: "fragility" was used in early design discussions; the ratified macro is
`#[presents]` (paralleling MHC presentation in biology).

**Introduced in**: `api-shape.md`.

### presentation

**Biological referent**: MHC Class I/II protein complex displaying antigen fragments on a
cell's surface so immune patrol can detect them.

**Rust ecosystem analog**: the `#[presents(antigen)]` decorator on Rust code. The code
*shows* what failure-class it's vulnerable to. Without presentation, the immune system
(cargo-antigen scan) cannot find the vulnerability.

**Introduced in**: `api-shape.md`.

---

## Inheritance terms

### descended_from

**Biological referent**: B-cell lineage (clonal expansion). When a B-cell encounters its
target antigen, it divides; daughter cells inherit the parent's antibody specificity but
may mutate slightly.

**Rust ecosystem analog**: the `#[descended_from(other_function)]` decorator. Propagates
`#[presents]` and `#[immune]` markers from the source function to the descended function.
If the descendant's witness no longer applies (signature divergence, behavioral change),
cargo-antigen flags it for re-justification.

**Closest existing academic analog**: Eiffel's design-by-contract with inheritance
(Meyer 1992, 1997) — pre/post-conditions inherited through subclassing with covariance
/ contravariance rules. Antigen's `#[descended_from]` is the Rust-ecosystem analog of
inherited contracts at the failure-class level rather than the predicate level. See
`docs/expedition/academic-context.md` §2.

**Introduced in**: `api-shape.md`.

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

**Rust ecosystem analog (currency layer)**: the *recency of verification* on
`#[immune(X, witness = Y)]` claims. The witness was attested against a particular
version of the protected item; if the item changes, the verification is stale —
the analog of declining circulating antibody titer. `cargo antigen audit`
re-running witnesses is the recall-response / booster analog. The currency layer
is in flight as a Sweep A1 finding (scout-routed; task #12 Phase 1-8).

**Note**: the *crisis case* this addresses is "corrected designs don't carry the
failure that motivated them" — the originating insight from tambear adversarial's
reflection. Pattern-memory persists; verification-currency requires periodic
re-attestation.

**Introduced in**: `design-intent.md`. Stratified-memory refinement: Sweep A1
closure (2026-05-07).

### lineage

**Biological referent**: B-cell or T-cell lineage from a single progenitor through
multiple clonal expansions.

**Rust ecosystem analog**: a chain of `#[descended_from]` declarations connecting an
original antigen-bearing function to its derived/refined/copied descendants. Cargo-antigen
walks the lineage to determine inherited markers.

**Introduced in**: `api-shape.md`.

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

**Introduced in**: `api-shape.md`.

### Pathogen Recognition Receptor (PRR)

**Biological referent**: receptors on innate-immunity cells that recognize broad classes
of pathogens (TLRs, NLRs, RLRs).

**Rust ecosystem analog**: the structural-pattern matcher in `cargo antigen scan`. Scans
code for patterns matching declared antigen fingerprints. Innate immunity = always-on
structural checks (compile-time, type-system phantom types). Adaptive immunity =
failure-pattern-specific tests.

**Introduced in**: `design-intent.md`.

### T-cell receptor

**Biological referent**: highly specific receptor on T-cells that recognizes one antigen
displayed by MHC.

**Rust ecosystem analog**: a named-failure-class fingerprint that recognizes ONE specific
structural pattern. More precise than PRRs (innate); less broad. Each `#[antigen(...)]`
declaration creates a kind of T-cell-receptor analog in the cargo tooling.

**Introduced in**: `design-intent.md`.

---

## Response terms

### cytokine

**Biological referent**: signaling molecules released during immune response that recruit
additional immune cells and modulate inflammation.

**Rust ecosystem analog**: when an antigen presentation fires, build-time signals
propagate through the call graph. Specifically, `#[propagates_presentations]` (opt-in)
causes callers of presenting functions to be marked as derived-presentations. Avoids
indiscriminate cytokine storm by being opt-in rather than default.

**Introduced in**: `api-shape.md` (composition rules section).

### inflammation

**Biological referent**: localized immune response that escalates if the pathogen
persists; can become chronic if dysregulated.

**Rust ecosystem analog**: not directly modeled in v1; potential future feature where
sustained antigen presentation in a code area triggers escalating warnings or required
review. Reserved for future versions.

**Introduced in**: `design-intent.md` (biological mapping).

### autoimmunity

**Biological referent**: failure mode where the immune system attacks healthy self-tissue
because tolerance mechanisms broke down.

**Rust ecosystem analog**: cargo-antigen scan over-flagging legitimate code as fragile.
Tolerance check: distinguishing "this code structurally matches an antigen fingerprint
but is in fact correct" from "this code is genuinely vulnerable." Initial mitigation: the
fingerprint grammar must be precise enough to minimize false positives; users can mark
specific sites with `#[antigen_tolerance(X, rationale = "...")]` for documented
exceptions (per ADR-011).

**Introduced in**: `design-intent.md` (what could kill it). Tolerance carrier ratified
in ADR-011 (Sweep A1, 2026-05-07/08).

---

## Composition terms

### witness

**Biological referent**: not a direct biological term; appears in the metaphor as the
"proof" that an antibody actually neutralizes its target antigen (e.g., binding affinity
measurements).

**Rust ecosystem analog**: the proof-of-immunity-claim required by `#[immune(antigen,
witness = ...)]`. Acceptable witness shapes: test function, proptest block, phantom-type
construction, formal-verification harness reference, custom-lint reference. The witness
is checked by tooling; immunity without witness is not a claim.

**Academic lineage**: the witness-as-proof concept descends from Necula's
proof-carrying code (Necula, "Proof-Carrying Code," POPL 1997) — code accompanied
by a checkable proof of a stated property. Antigen's witness pluralism extends this
by accepting heterogeneous proof shapes (test, proptest, formal verification, lint,
phantom type) under one vocabulary. See `docs/expedition/academic-context.md` §10
and §11.

**Introduced in**: `api-shape.md`.

### witness-validity tiers

**Definition** (per ADR-001 Amendment 1, Change 4): the `witness` parameter of
`#[immune]` accepts proofs at four progressively-stronger tiers:

- **Reachability tier** — the witness identifier resolves to a function/test that
  exists. Floor; v0.0.x audit currently lives here.
- **Execution tier** — the witness runs without panic and asserts a non-trivial
  property. Sweep A2-A3 lift.
- **Behavioral-alignment tier** — the witness exercises behavior that matches the
  antigen's structural fingerprint. Sweep A4-A5 work; ADR-005 open question.
- **Formal-proof tier** — the witness is a verified compile-time proof (phantom-type
  construction, kani/prusti/verus/creusot proof annotation). Sweep A4+ via ADR-002
  witness delegation.

ADR-005 sub-clause F applies to whichever tier is current. Audit's `--format=json`
output includes a `witness_tier` field for CI gates.

**Introduced in**: ADR-001 Amendment 1 (2026-05-08).

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

**Introduced in**: ADR-013 (2026-05-08). Pre-existing api-shape.md sketch. Academic
lineage: refinement-type proof carriers (Liquid Haskell, Flux); seal-trait
private-constructor patterns.

### family / failure-class family

**Biological referent**: pathogens grouped by structural similarity (e.g., influenza
strains, SARS-CoV variants).

**Rust ecosystem analog**: the `family` parameter on `#[antigen(...)]`. Groups related
failure-classes for shared structural fingerprints and shared vaccination patterns. The
8 first-principles classes form parent families: `frame-translation`, `forgotten-lesson`,
`implicit-coupling`, `stale-context`, `premature-abstraction`, `incompatible-merger`,
`boundary-violation`, `optionality-collapse`.

**Introduced in**: `design-intent.md`, `api-shape.md`.

### composition (of antigens)

**Biological referent**: not a direct biological term; in immunology, response to
multi-antigen pathogens involves coordinated B-cell and T-cell responses.

**Rust ecosystem analog**: combining antigen markers via Rust's existing composition
mechanisms (trait impls, generics, derive macros). Antigen propagation rules specify
how `#[presents]` and `#[immune]` flow through composition. See `api-shape.md`
"Composition rules" section.

**Introduced in**: `api-shape.md`.

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

**Introduced in**: `design-intent.md`, `revolutionary-and-not.md`.

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
discharge the immunity duty (`#[immune]` or `#[antigen_tolerance]` at the call site).

**Deferred to Sweep A3-A4 implementation**; v0.1.0 ships without it but the carrier
is ratified.

**Introduced in**: ADR-014 (2026-05-08).

---

## Architectural patterns

### tiered substrate / carrier-strength hierarchy

**Definition** (per ADR-001 Amendment 1, Change 1): the project-wide architectural
pattern where every primitive sits on a strength-of-evidence gradient rather than
being binary. Memory carriers sit on a drift-resistance hierarchy:

```
  compile-time-checked   (type system, phantom-types, kani/prusti proofs)
          ↑
  scan-time-checked      (#[antigen], #[immune], #[presents], #[descended_from])
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

**Three-window convergence** (per Sweep A1 closure, ADR-003 empirical defense):
biology (vertebrate immunology), past-self gardening (March-April 2026
naming-checkability frame), academic lineage (Hoare 1969 → Eiffel 1992 → Koka →
Liquid Haskell → Flux). When three independent traditions converge on the same
primitive, the underlying architecture is real, not metaphor-dependent.

**Introduced in**: ADR-001 Amendment 1 (2026-05-08), naturalist closure narrative.

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
(`#[presents]`, `#[immune]`, `#[descended_from]`, `#[antigen_tolerance]`,
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
- `#[immune(... witness)]` — proof of immunity (ADR-001/002/005)
- `#[antigen_tolerance(... rationale)]` — justification for waiver (ADR-011)
- `#[antigen_generates(... rationale)]` — justification for generated
  presentation (ADR-014)

Sub-clause F (ADR-005) applied at the API level — every trust-extending primitive
carries its own justification. The discipline propagates from existing ADRs to
new ADRs without explicit coordination, which is how a load-bearing principle
should behave.

**Introduced in**: naturalist closure narrative finding 3 (Sweep A1 closure,
2026-05-08). May be ratified as a small ADR-005 amendment in A2.

---

## Tooling terms

### cargo antigen

**Definition**: the cargo subcommand binary, published as the `cargo-antigen` crate.
Provides `scan`, `new`, `vaccinate`, `audit` subcommands.

**Introduced in**: `api-shape.md`.

### antigen library / antigen registry

**Definition**: the (eventual) collection of named antigens distributed via crates.io.
`antigen-stdlib` is the first; project-specific antigens (e.g., `tambear-antigens`) extend
it. No central registry — community-driven via crate publication.

**Introduced in**: `revolutionary-and-not.md`.

---

## Disciplines inherited from tambear

These terms come from tambear's DECs and team-briefing disciplines. They apply to antigen
because the antigen team inherits these disciplines from the JBD methodology.

### sub-clause F (trust boundary)

**Origin**: tambear DEC-022.

**In antigen**: every antigen declaration's witness MUST be validated by tooling before
the immunity claim is trusted. The trust boundary lives at `cargo antigen scan` time
(checking the witness exists and is valid) and at compile time (for phantom-type
witnesses).

### substrate over memory

**Origin**: tambear standing constraint.

**In antigen**: cargo-antigen tooling reads the codebase as ground truth. Documentation
about antigens is informational; the source-of-truth is the `#[antigen]` / `#[presents]`
/ `#[immune]` declarations themselves.

### narrow-then-lift

**Origin**: tambear DEC-022 sub-clause discipline.

**In antigen**: antigen fingerprints should narrow to what the structural pattern can
*actually* match; if a fingerprint is overly broad, narrow it before adding to the
library. Avoid speculative claims; lift narrowed fingerprints into more general patterns
only when evidence supports it.

### proptest-locks-the-narrow-truth

**Origin**: tambear documentation-accuracy discipline.

**In antigen**: every antigen declaration's documentation must reflect what the
fingerprint actually matches and what the witness actually proves. Proptests on the
witness ensure the docstring stays accurate.

### recognition-not-design

**Origin**: tambear convergence-patterns work; named in DEC-032 placeholder.

**In antigen**: antigen *recognizes* failure-classes that already exist in code. It does
not *design* failure-classes from scratch. Each new antigen is a recognition, not an
invention. The 8-class first-principles taxonomy is recognition of existing structure;
antigen-stdlib is recognition of existing common patterns.

### conditional-lean-collapse

**Origin**: tambear V4 / coordination disciplines.

**In antigen**: when routing antigen declarations through composition, preserve the
conditional structure. e.g., if a function is "fragile to X under condition C, immune to
X under condition !C," do not collapse to "fragile to X" or "immune to X." Express the
conditional via separate antigen instances or refined fingerprints.

---

## Glossary maintenance

This glossary is itself a tambear-style discipline artifact. As the antigen project
matures, terms WILL drift in meaning. The discipline:

1. Every PR that introduces new vocabulary or refines existing terms updates this glossary.
2. Every section header in design docs adds a glossary cross-reference (e.g., "see
   glossary: vaccination, lineage").
3. Vocabulary drift is treated as a sub-clause E violation (coordinate-explicitness
   failure) and triggers a glossary review.

Maintained by: the antigen team. Last updated: 2026-05-07 (initial scaffold).
