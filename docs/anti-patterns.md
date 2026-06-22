# Antigen — Anti-Patterns Gallery

> Common mistakes when adopting antigen, with the structural reason
> each is a mistake and the correct shape instead. Complements
> [`troubleshooting.md`](troubleshooting.md) (which covers
> errors-during-use); this catalog covers
> patterns-that-look-fine-but-aren't.

For positive patterns, see [`usage-patterns.md`](usage-patterns.md).
For witness-tier semantics that some of these anti-patterns violate,
see [`witness-tiers.md`](witness-tiers.md).

---

## Anti-pattern 1 — Theatrical witness

**What it looks like**:

```rust
#[presents(MyAntigen)]
fn vulnerable() { /* ... */ }

#[defended_by(MyAntigen)]
#[test]
fn todo_test() {
    // TODO: write actual test
    assert!(true);
}
```

**Why it's an anti-pattern**: the audit reports `Reachability` tier
(the witness test exists), but the witness doesn't actually verify
anything. You've registered a defense backed by a test that passes
trivially. The structural memory says "this is defended"; reality says
"nothing defends it."

**Catch**: scan + audit will report the defense at `Reachability`
(not `Execution` or `FormalProof`) — honest tier reporting. But the
audit can't know your test is theatrical; only honest authoring catches
this.

**Correct shape**: either write a real test that exercises the
failure-class, OR use `#[antigen_tolerance]` with a genuine
rationale:

```rust
#[antigen_tolerance(
    MyAntigen,
    rationale = "Pending real witness; tracking in issue #5678; current behavior \
                 acceptable for now."
)]
fn known_acceptable_for_now() { /* ... */ }
```

The honest move is one of: write the witness, tolerate the gap, or
remove the defense entirely.

---

## Anti-pattern 2 — Empty rationale on tolerance

**What it looks like**:

```rust
#[antigen_tolerance(MyAntigen, rationale = "OK")]
fn tolerated() { /* ... */ }
```

**Why it's an anti-pattern**: per ADR-011, `rationale` is required at
parse time, and an empty-string-equivalent rationale defeats the
purpose. "OK" / "TODO" / "fixme" / "fingerprint false positive"
without explanation are all this pattern.

**Catch**: ADR-011 requires non-empty rationale; "OK" technically
passes the non-empty check but fails the spirit. The structural
discipline (rationale-as-required-field, postures §6) demands an
*actual* justification.

**Correct shape**:

```rust
#[antigen_tolerance(
    MyAntigen,
    rationale = "Test fixture deliberately constructs the vulnerable pattern to verify the \
                 fingerprint catches it. Vulnerability is the point.",
    see = ["pr:internal/repo#234"],
)]
fn test_fingerprint_detects_vulnerable_case() { /* ... */ }
```

Genuine rationale answers: **why is the failure-class structurally
present here without being a real defect?** If you can't construct
that answer, the failure-class is probably a real defect — not a
tolerable one.

---

## Anti-pattern 3 — Designed-not-recognized antigen

**What it looks like**:

```rust
// "I think this might be a problem someday"
#[antigen(
    name = "potential-deadlock-pattern",
    fingerprint = r#"item = fn, body_contains_macro("Mutex")"#,
    summary = "Functions using Mutex might deadlock if called recursively.",
)]
pub struct PotentialDeadlockPattern;
```

**Why it's an anti-pattern**: per ADR-006 (recognition-not-design),
new antigens should *recognize existing structure in substrate*, not
*extend the design speculatively*. This declaration has no
substrate-grounded instances; it's a hypothetical worry.

**The cost**: speculative antigens generate noise (every function
using Mutex now flags as fingerprint match) without naming a real
failure-class. Adopters lose trust in the scan output; the discipline
weakens.

**Catch**: the project's ADR-006 threshold says three independent
substrate-grounded instances clear the recognition bar. If you can't
point to three actual bugs that fit this shape, the antigen is
premature.

**Correct shape**: wait until you've encountered the failure-class
*at least once* in real code (yours or a dependency's). Then declare
it with `references` pointing to the actual instance:

```rust
#[antigen(
    name = "recursive-lock-on-shared-mutex",
    fingerprint = r#"item = fn, ..."#,  // refined fingerprint based on actual case
    summary = "Recursive call paths to shared-Mutex functions deadlock; \
               first observed in PR #1234.",
    references = ["pr:internal/repo#1234", "internal:incident-2024-08-15"],
)]
pub struct RecursiveLockOnSharedMutex;
```

---

## Anti-pattern 4 — Over-broad fingerprint

**What it looks like**:

```rust
#[antigen(
    name = "any-impl-block-with-panic",
    fingerprint = r#"
        item = impl,
        body_contains_macro("panic")
    "#,
    // ...
)]
pub struct AnyImplBlockWithPanic;
```

**Why it's an anti-pattern**: this matches *every* impl block in your
codebase containing `panic!`, regardless of context. Test scaffolding
that intentionally constructs panic cases, debug-only assertions in
unsafe blocks, fixture code that exists to verify panic-handling
elsewhere — all matched.

**The cost**: high false-positive rate; signal drowns in noise;
adopters either ignore the scan output or pollute their codebase with
`#[antigen_tolerance]` to silence each match.

**Catch**: per ADR-010 Amendment 4 (filter/proof split), fingerprints
filter (recall-tuned candidate filters); witnesses prove. **False
positives are expected** at the fingerprint layer. But there's a
difference between *some* false positives and *predominantly* false
positives.

**Correct shape**: narrow the fingerprint to the *structurally
specific* shape of the failure-class:

```rust
#[antigen(
    name = "panicking-in-drop",
    fingerprint = r#"
        item = impl,
        has_method("drop", "(& mut self)"),
        body_contains_macro("panic")
    "#,
    // ...
)]
pub struct PanickingInDrop;
```

The added constraint (`has_method("drop", ...)`) narrows from "any
impl with panic" to "Drop impls with panic" — the actual
failure-class shape. False-positive rate drops dramatically; signal
becomes useful.

---

## Anti-pattern 5 — Tier overclaim

**What it looks like**:

```rust
// "Clippy lint catches the pattern, so this is FormalProof tier."
#[presents(MyAntigen, requires = clippy::no_unsafe_operations)]
fn defended() { /* ... */ }
```

**Why it's an anti-pattern**: clippy is a *lint*. It catches patterns
heuristically; it doesn't formally prove anything. The audit reports
`Reachability` tier with `ExternalToolPrefixRecognized` hint —
honestly recognizing that the clippy delegation is *delegation*, not
formal verification. Treating the lint as a proof tells one story; the
audit's tier-honest reporting tells another.

**Catch**: the audit reports the *actual* verification strength. A
comment or commit message claiming "FormalProof tier" is contradicted
by the audit's `Reachability` tier report.

**Correct shape**: describe the witness as what it is — a heuristic
lint at `Reachability` tier:

```rust
// Clippy lint catches the pattern at warn level; configured in clippy.toml.
// Lint is heuristic, not a proof — tier is Reachability per witness-tiers.md.
#[presents(MyAntigen, requires = clippy::no_unsafe_operations)]
fn defended() { /* ... */ }
```

When you need FormalProof, use a phantom-type proof witness
(`#[presents(MyAntigen, proof = ...)]`) backed by a verifier.

---

## Anti-pattern 6 — Witness inventing

**What it looks like**:

```rust
#[presents(MyAntigen, requires = surely_this_sidecar_exists)]
fn defended() { /* ... */ }
```

(where `surely_this_sidecar_exists` doesn't actually resolve to any
sidecar evidence in the workspace)

**Why it's an anti-pattern**: the witness predicate doesn't resolve.
The audit reports `None` tier with `WitnessNotFound` hint. The
structural memory registers a defense; reality is the defense is
broken.

This often happens during:
- LLM agents generating code without verifying witness predicates exist
- Renames that broke the witness reference
- Fingerprint-engine cargo-cult: copy-pasting a defense from another
  project

**Catch**: `cargo antigen audit` surfaces broken witnesses
immediately. The diagnostic message: `witness identifier not found in
any .rs file under the scan root`.

**Correct shape**: verify the witness resolves before authoring the
defense. If you're an LLM agent generating this, grep the codebase for
the test or sidecar before naming it. If you're refactoring and renamed
a test, update the `#[defended_by]` site to match.

The honest fallback: if no witness exists yet, use
`#[antigen_tolerance]` with a rationale explaining the gap.

---

## Anti-pattern 7 — Same-named-antigens collision (cross-crate)

**What it looks like**:

```rust
// crate-a/src/antigens.rs
#[antigen(name = "panicking-in-drop", ...)]
pub struct PanickingInDrop;

// crate-b/src/antigens.rs (different crate, different definition)
#[antigen(name = "panicking-in-drop", ...)]  // different fingerprint, same name
pub struct PanickingInDrop;
```

**Why it's an anti-pattern**: cross-crate identity uses
`canonical_path` at `name@version` granularity (per ADR-017), so
same-named antigens from different crates are distinguishable. But
*conceptually*, having the same name for structurally different
failure-classes invites confusion.

**The cost**: adopters tracking defenses across both crates may
conflate them. Cross-references in `references` fields become
ambiguous. LLM agents reading both may misalign understanding.

**Correct shape**: use distinctive naming that includes context, or
use unique prefixes per crate:

```rust
// crate-a: rust-stdlib-style names
#[antigen(name = "drop-impl-panics-during-unwind", ...)]
pub struct DropImplPanicsDuringUnwind;

// crate-b: domain-specific names
#[antigen(name = "geometry-class-meet-polarity-inverted", ...)]
pub struct GeometryClassMeetPolarityInverted;
```

Or use the `family` field to distinguish:

```rust
#[antigen(
    name = "panicking-in-drop",
    family = "geometry-numerical-stability",  // disambiguates from generic
    // ...
)]
pub struct PanickingInDrop;
```

The bundled stdlib catalog reserves ecosystem-wide failure-class names;
per-project names should be distinguishable from the catalog's names.

---

## Anti-pattern 8 — Inheritance without re-attestation

**What it looks like**:

```rust
#[antigen(name = "memory-unsafety-class", ...)]
pub struct MemoryUnsafetyClass;

#[antigen(name = "use-after-free-class", ...)]
#[descended_from(MemoryUnsafetyClass)]
pub struct UseAfterFreeClass;

// later, on a vulnerable site:
#[presents(UseAfterFreeClass)]
fn vulnerable() { /* ... */ }

// no #[defended_by(UseAfterFreeClass)] here, but the parent was defended somewhere
```

**Why it's an anti-pattern**: per ADR-005 sub-clause F, inheritance
does NOT transitively register a defense. Descendants must *re-attest*.
The parent's defense doesn't automatically apply to the child site.

The audit reports `inherited-presentation-not-re-attested` for the
inherited presentation that lacks its own defense.

**Catch**: `cargo antigen audit` surfaces inherited-but-not-re-attested
sites explicitly.

**Correct shape**: each descendant site that presents the failure-
class needs its own defense (or tolerance):

```rust
#[presents(UseAfterFreeClass)]
fn vulnerable() { /* ... */ }

/// Specific test for the use-after-free shape at this site.
#[defended_by(UseAfterFreeClass)]
#[test]
fn use_after_free_specific_test() { /* ... */ }
```

The witness might be the same test used for the parent, or a more
specific one. The point: the structural memory must be
*re-attested*, not assumed-via-inheritance.

---

## Anti-pattern 9 — Fingerprint following design-doc, not engine

**What it looks like**:

```rust
#[antigen(
    name = "has-some-method",
    fingerprint = r#"item = impl, has_method("drop", "(&mut self)")"#,  // bug
    // ...
)]
pub struct HasSomeMethod;
```

**Why it's an anti-pattern**: pre-ADR-010 Amendment 5, the fingerprint
engine compared user-pattern-strings against proc_macro2-rendered
signatures. proc_macro2 renders `&mut self` as `& mut self` (space
after `&`). The user's pattern `"(&mut self)"` silently matched zero
`impl Drop` blocks.

This is the **phantom-tier-from-design-doc-not-ratified-code**
pattern: writing what the language *looks like* vs what the engine
*actually produces*.

The engine pre-tokenizes user pattern strings through proc_macro2, so both
forms work. But the structural class of mistake — writing fingerprints against
assumed engine behavior rather than verified behavior — remains a real
anti-pattern.

**Catch**: ADR-010 Amendment 5's engine canonicalization handles the
spacing case. For other tokenization-asymmetry cases (e.g.,
`self` vs `Self` — receiver keyword vs type alias — proc_macro2
preserves the distinction; antigen does NOT silently bridge them),
the docs warn explicitly.

**Correct shape**: when authoring fingerprints, verify against the
actual matcher behavior. For `has_method`, the documented receiver
form is `(self, ...)` for by-value, `(& self, ...)` for by-reference,
`(& mut self, ...)` for by-mutable-reference. See
[`fingerprint-grammar.md`](fingerprint-grammar.md#has_method) for
the receiver-rendering reference table.

---

## Anti-pattern 10 — Treating antigen as documentation system

**What it looks like**:

```rust
#[antigen(
    name = "this-function-is-important",
    fingerprint = r#"name = matches("critical_function")"#,
    summary = "This function is critical; please be careful when modifying.",
)]
pub struct ThisFunctionIsImportant;
```

**Why it's an anti-pattern**: antigen is not a documentation system.
This declaration uses the antigen vocabulary to mark importance, not
to name a failure-class. There's no structural failure-class shape;
there's no witness verification; there's no defensive pattern.

**The cost**: antigen's substrate becomes diluted. The scan output
mixes real failure-classes with importance-markers. Adopters trying to
understand the scan can't distinguish "this is dangerous" from "this
is important to its author."

**Catch**: there's no automated catch for this; it's a discipline
question. ADR-006 (recognition-not-design) is the structural defense
— antigens recognize *failure-classes*, not *concerns*.

**Correct shape**: importance markers belong in:
- Documentation (docstrings, README, code-architecture docs)
- ADR / decision records
- Comments

Antigen captures *what fails this class of code structurally*. If you
can't answer "what specifically fails here?" the antigen is probably
not the right tool. Use docs.

---

## Anti-pattern 11 — Premature antigen-stdlib

**What it looks like**:

Adding domain-specific antigens to an ecosystem-wide stdlib because
"someone else might find these useful."

**Why it's an anti-pattern**: per ADR-006 and the recognition-grounded
governance model, stdlib antigens require multiple in-the-wild instances
across distinct codebases. Adding speculative or domain-specific antigens
to the stdlib pollutes the ecosystem-wide failure-class memory with patterns
most adopters don't need.

**Catch**: an antigen-stdlib contribution should carry substrate-grounded
evidence of cross-codebase relevance before it earns a place in the shared
catalog.

**Correct shape**: keep domain-specific antigens in your own
`src/antigens.rs`. When you observe the same failure-class
independently across three or more codebases, propose to stdlib.

---

## Anti-pattern 12 — Macro-emitted marker assumed visible to source-walking scan

**What it looks like**:

Designing a feature where a proc-macro expands an attribute into a structured comment or doc-attribute (e.g., emitting JSON metadata as a doc-comment), then expecting a source-walking scanner (one that uses `syn::parse_file` to read WRITTEN source) to discover that emitted marker.

**Why it's an anti-pattern**: proc-macro expansion happens at compile time; source-walking tools read the source file before expansion runs. The emitted marker exists in the expanded TokenStream but not in the file on disk. The scanner walks past it, finds nothing, reports "no antigen-related declarations here."

**The cost**: silent feature breakage. The feature compiles (macro expands fine), the tests pass (if they invoke the macro), but the user-facing tooling reports the feature isn't present. This is precisely the substrate-alignment failure-class antigen is designed to catch in OTHER codebases.

**Real instance**: a substrate-tier witness form parsed correctly and emitted a JSON marker at macro-expansion time. But `cargo antigen scan` walks written source via `syn::parse_file` and never saw the post-expansion doc marker, so every substrate-witness defense reported `tier = None, hint = NoneApplicable`. The fix routed both the macro and the scanner through a shared parser, so both halves read from the same representation.

**Correct shape**: when a feature bridges proc-macro expansion and source-walking tools, route BOTH through the same parser. Don't rely on macro-emission to a side-channel that the scanner can't read. The shared parser is the single source of truth.

---

## Anti-pattern 13 — Unanchored gitignore pattern silently hiding source

**What it looks like**:

A `.gitignore` entry without a leading `/` that's intended to match a specific directory at the repo root, but which (per gitignore semantics) actually matches at ANY depth. Example: `cache/` (no leading slash) excludes `<repo>/cache/`, `<repo>/subproject/cache/`, and `<repo>/anything/anywhere/cache/`.

**Why it's an anti-pattern**: git's view of disk diverges from disk silently. Build tools see all files (they read disk); git sees a subset (it filters through gitignore). The discrepancy survives every CI run until someone clones fresh and the missing files become user-visible.

**The cost**: catastrophic at rare moments. A fresh agent waking up to a cloned repo finds an empty directory where the build expected files. Recursive proofs, witness machinery, source modules — anything in the silently-excluded directory simply isn't there in the cloned tree, even though it was there on the original machine.

**Real instance**: a `.gitignore` had `cache/` (no anchor) intended to exclude only a top-level `<repo>/cache/` build directory. It silently matched a nested `<repo>/subcrate/src/cache/` source module and would have erased that module — imports, sidecars, and all — from any fresh clone. The build kept succeeding on the original machine; a substrate-alignment audit caught the gap before anyone cloned fresh.

**Correct shape**: gitignore patterns intended to apply only at repo root need an explicit `/` prefix. Audit `.gitignore` periodically against expected substrate. Treat gitignore as substrate-alignment-critical — what git knows differs from what disk has, and that gap is invisible from inside the implementation lane.

**Antigen primitive**: `UnanchoredGitignorePattern` is a candidate stdlib antigen in the supply-chain / substrate-alignment families.

---

## Anti-pattern 14 — Cross-project conflation in adopter-facing docs

**What it looks like**:

Your tool has sibling tools that USE it (e.g., a downstream coordination CLI that composes with your library via subprocess). You list those sibling tools in your tool's "Shipped" block, roadmap, or status section — as if their shipping is part of your tool's manifest.

**Why it's an anti-pattern**: each project's adopter-facing surface should be about that project ONLY. Adopters scanning your docs and seeing a sibling tool's name might think "is this a macro? a feature? do I need to install something?" Cross-project conflation costs comprehension; readers spend cycles untangling what's part of YOUR tool vs what's a separate dependency.

**The cost**: cognitive load on first-encounter readers; documentation drift as the sibling tool's release cadence diverges from yours; ambiguity about scope and boundaries.

**Real instance**: a roadmap and README briefly listed a downstream coordination tool's release alongside antigen's own macros and CLI surface. That tool composes with antigen via subprocess (ADR-002) — its own repo, its own release history. Listing it as if it were an antigen deliverable confused the architecture, and a docs pass corrected it back to antigen-only scope.

**Correct shape**: each project's adopter-facing docs talk about that project only. Sibling tools, if mentioned at all, go in clearly-marked ecosystem sections — never in shipping manifests. Cross-references to sibling tools belong in their own dedicated doc (e.g., an "ecosystem.md") if substrate justifies; otherwise readers discover sibling tools through their own search rather than through your docs.

---

## Meta anti-pattern — Discipline drift

**What it looks like**:

Adopting antigen, declaring antigens, but never running scan/audit;
or running them but ignoring the output; or having defenses
that haven't been verified in months.

**Why it's an anti-pattern**: structural memory only operates if the
substrate is current. If antigens decay (witnesses break and nobody
notices; fingerprints get out of sync with code; tolerances accumulate
without review), the structural memory becomes false structural
memory.

**The cost**: adopters who started antigen with good intent end up
with a codebase that *says* it's defended (defenses on record) but whose
defenses are stale.

**Catch**: this is at the discipline-tier, not the tooling-tier.
Antigen's tooling will tell you what's broken at audit time; the
discipline is to *actually run audit* on a cadence and *actually
address* the findings.

**Correct shape**: integrate scan + audit into CI. Make audit-clean a
gate before merge. Treat tolerance claims as having expiry dates
(`until = "..."` field) and revisit. The structural-tier discipline
includes *running* the tools structurally, not just *declaring*
substrate.

---

## See also

- [`troubleshooting.md`](troubleshooting.md) — diagnostic guide for
  what scan/audit output means when it surfaces unexpected results
- [`usage-patterns.md`](usage-patterns.md) — positive patterns that
  these anti-patterns invert
- [`witness-tiers.md`](witness-tiers.md) — tier semantics referenced
  in several anti-patterns
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL
  reference; helps avoid anti-pattern 4 and 9

---

*Anti-patterns are how the discipline operates honestly. Each pattern
is something an adopter might do that looks fine and produces real
harm. Naming them structurally is the same discipline antigen itself
operationalizes — failure-class memory at the meta-tier.*
