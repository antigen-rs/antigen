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

**Introduced in**: `api-shape.md`.

### B-cell memory

**Biological referent**: the persistence of antigen-specific B-cells (and their plasma
cell descendants producing antibodies) long after an infection clears, providing rapid
response to reinfection.

**Rust ecosystem analog**: `#[antigen]` declarations themselves, plus the immunity
markers across the codebase. The MEMORY persists past the specific bug that motivated
the antigen declaration. New code in the structural family inherits the immunity via
`#[descended_from]`.

**Note**: the *crisis case* this addresses is "corrected designs don't carry the failure
that motivated them" — the originating insight from tambear adversarial's reflection.

**Introduced in**: `design-intent.md`.

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
fingerprint grammar must be precise enough to minimize false positives; user can mark
specific sites with `#[antigen_tolerance(reason = "...")]` for documented exceptions.

**Introduced in**: `design-intent.md` (what could kill it).

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

**Introduced in**: `api-shape.md`.

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
