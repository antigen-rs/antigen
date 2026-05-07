# Tambear adoption log

> Living document tracking every antigen-related thing tambear does as it
> adopts, uses, refines, or removes antigen functionality. Each entry
> documents: why it was tried, what was done, whether it worked, what got
> removed if it didn't help, and what problem each thing solved.
>
> This log serves both projects:
> - **Tambear**: a record of integration decisions for context when later
>   sessions revisit them.
> - **Antigen team**: real-world adoption feedback to inform the project's
>   priorities.
>
> Entries are append-only by convention; the most recent appears at the bottom
> of each section. Markdown headings name what changed; body explains why,
> what, and how it played out.

---

## Conventions

Each entry has the shape:

```markdown
### [YYYY-MM-DD] Short descriptive title

**Why**: motivation for trying this — what problem were we trying to solve?

**What**: concrete change. File paths, code snippets, command invocations.

**Result**: did it work? What happened?

**Verdict**: keeping it / refining it / removed it because <reason> / not yet
sure (revisit by date).

**Lessons**: what the antigen team should know from this experience.
```

Keep entries brief but specific. The point is to make experience replayable
and for the antigen team to extract patterns across many projects' adoption
logs eventually.

---

## Entries

### [2026-05-07] First integration: path dep + antigens.rs module

**Why**: Validate that antigen-the-tool actually works against tambear's real
codebase, not just toy examples. Tekgy + Claude wanted to be the first user
during the pre-team scaffolding so we could:
- Surface adoption friction before the antigen JBD team launches
- Generate real performance data (how slow is scan on a real workspace?)
- Provide feedback on output ergonomics
- Demonstrate the inheritance-from-tambear.md "Phase 2 reciprocity" arc isn't
  hypothetical

**What**:

1. Added `antigen = { path = "../../../antigen/antigen" }` to
   `R:/tambear/crates/tambear/Cargo.toml`.
2. Created `R:/tambear/crates/tambear/src/antigens.rs` with two antigen
   declarations: `PolarityInvertedClassMeet` (the originator failure-class
   from GAP-BIT-EXACT-1) and `PanickingInDrop` (a stdlib-seed-catalog
   instance).
3. Registered the module via `pub mod antigens;` in `lib.rs`.
4. Ran `cargo antigen scan --root R:/tambear/crates` and `cargo antigen audit`.

**Result**:

- Tambear `cargo check` clean in 1.7s incremental.
- `cargo antigen scan` finds both declarations in 0.3s (217 files scanned).
- `cargo antigen audit` reports 0 immunity claims (because no `#[immune]`
  declarations exist yet — the antigens are declared but not yet protecting
  any code).
- No tambear class enum currently has a `meet` method, so the
  polarity-inverted-class-meet fingerprint correctly doesn't fire on any
  tambear code — confirming the antigen is for prospective protection.

**Verdict**: keeping it. The integration is minimal, reversible, and proves
the inheritance-arc is operational.

**Lessons**:

1. **Performance is not a barrier**: 217-file scans in 0.3s is well within CI
   budgets. The 11s first-run is dominated by `cargo-antigen` release-mode
   compile; subsequent scans are fast.
2. **5-step integration was straightforward**: Cargo.toml dep + module file +
   `pub mod` + `cargo check` + scan. No surprises.
3. **The "no setup" scan was already useful**: even before adding ANY
   declarations, running scan against tambear gave us "0 declarations across
   216 files in 0.3s" — establishing baseline performance and confirming
   the tool walks real-world code without errors.
4. **Output ergonomics asymmetry**: scan's brief summary works for clean
   workspaces; audit's diagnostic-rich output is more useful when something
   needs attention. The antigen team should consider making scan more
   diagnostic-rich for the "all clean" case.
5. **Honest no-bug finding is correct behavior**: tambear's class enums
   don't have meet methods, so the polarity-inverted antigen doesn't fire.
   This is the discipline working — the tooling doesn't generate false
   positives just because a class enum exists.

---

## Future entries

The following are placeholder slots for upcoming work. As the team takes
each on, the slots get replaced with real entry text.

### [2026-05-07 (later)] Third antigen + first real cleanup driven by antigen discipline

**Why**: ULP-CANON-1 spot check (item 2 from the post-DEC-029 roadmap)
discovered that despite the original 8 sign-magnitude re-implementation sites
being cleaned up in 2026-04-25, **two NEW sites had emerged in 2026-05-06
expedition work** (commits `f53bc1c` chains-E/F/G and `8c0555f`
sweep_29c_kappa_oracle). Each was a 7-line hand-rolled Dawson 2012 ULP
distance function, complete with the signed-zero sign-flip mechanics.

The pre-existing pattern detector at `tambear-substrate/src/pattern.rs` only
catches the inline single-expression form (`($A.to_bits() as i64).wrapping_sub(...)
.abs()`); it missed the multi-statement function form because those don't
match the same AST shape.

This is exactly the failure-class adversarial gardened about: "corrected
designs don't carry the failure that motivated them." Even with:
- The canonical `oracle_compare::ulp_distance_f64` existing
- The 8 prior sites cleaned up
- A pattern detector specifically for this pattern
- The lesson documented in roadmap-after-dec029.md

...two new sites STILL re-rolled the function from scratch. Different agents,
different contexts, different sweeps.

**What**:

1. Added a third antigen `UlpDistanceRolledByHand` to
   `crates/tambear/src/antigens.rs`:
   - Family: `forgotten-lesson` (the meta-pattern of lessons re-emerging)
   - Fingerprint: documented (multi-statement function form NOT covered by
     pattern.rs)
   - References: ULP-CANON-1 (original cleanup), `f53bc1c`, `8c0555f`,
     `tambear-substrate/src/pattern.rs`, Dawson 2012
2. Replaced the hand-rolled function in
   `crates/tambear/tests/dd_subnormal_sweep_oracle.rs` with a thin delegation
   to the canonical: `tambear::primitives::oracle_compare::ulp_distance_f64`
3. Same replacement in `crates/tambear/tests/sweep_29c_kappa_oracle.rs`
4. Verified both files' tests still pass (7 tests + 4 tests) — the canonical
   produces identical results for these test inputs

**Result**:

- `cargo test --workspace --lib` still passes 1397 tests
- `cargo antigen scan` against tambear finds 3 declarations (was 2)
- The 2 hand-rolled functions are gone; both files use the canonical
- ULP-CANON-1 is now actually complete (8 original sites + 2 newly-discovered
  sites all cleaned up)
- The `UlpDistanceRolledByHand` antigen now structurally documents the failure
  pattern, including the meta-finding that the pattern.rs detector has known
  blind spots (multi-statement function form)

**Verdict**: keeping it. This is the discipline working. The cleanup itself
wasn't dramatic (each replacement was 7 lines → 1 line), but the fact that
the failure-class re-emerged across sweeps is itself the antigen's
load-bearing case.

**Lessons**:

1. **The ULP-CANON-1 "complete" status from 2026-04-25 was actually
   incomplete-as-of-2026-05-07** because new sweeps reintroduced the pattern.
   This validates the antigen-stdlib argument that **failure-class memory
   needs to live in code, not in commit messages or roadmap items**. A
   commit-message-tracked cleanup gets reintroduced; a structural antigen
   doesn't (because `cargo antigen scan --hunt` would have flagged the new
   sites at PR time).
2. **Pattern detectors with blind spots create false confidence.** The
   `tambear-substrate/src/pattern.rs` pattern set covers inline forms but
   misses multi-statement function forms. The team thought ULP-CANON-1 was
   covered structurally; it was only partially covered. This is a Phase 3
   future-extensions.md item: fingerprint hunting needs to handle structural
   variants of the same pattern, not just specific expressions.
3. **The antigen-driven cleanup added net structural value beyond just the
   line replacement**: the antigen declaration now serves as documentation
   for any future contributor, with explicit references to all known sites
   AND the meta-finding about pattern.rs's blind spots. This is more durable
   than a commit message.
4. **Antigen team feedback**: the structural-fingerprint grammar needs to
   handle "function whose body matches pattern X" not just "expression
   matching pattern X." This is ADR-010's v1 grammar limitation; future
   versions should compose patterns over function bodies. (TODO(team) marker
   added to relevant scan code.)

[antigen team note: pattern-over-function-body is a real feature gap;
consider adding to ADR-010 amendment proposal for v2 grammar.]

### [2026-05-07 (later still)] PanickingInDrop and PolarityInvertedClassMeet: prospective protection only

**Why**: After the UlpDistanceRolledByHand cleanup, completed the audit pass
for the OTHER two declared antigens — looking at where in tambear they would
apply today.

**What**: 
- Grepped tambear for `impl Drop` blocks across all 7 workspace crates
- Confirmed earlier finding that tambear's class enums (DeterminismClass,
  FiniteClass, NyquistClass, PdeClass) don't have meet methods

**Result**:

- **Zero `impl Drop` blocks** anywhere in tambear's codebase. Tambear is
  pure-functional math; types aren't holding RAII resources, file handles,
  or external state that needs Drop cleanup. So PanickingInDrop has no current
  surface to fire on.
- Same finding for PolarityInvertedClassMeet (already documented earlier):
  no class enums currently have meet methods.

**Verdict**: keeping both antigens as declared, even though they don't fire
today. They serve **prospective protection**: when someone later introduces a
type that holds a Mutex/File/external handle (Drop impl) or a class enum
with strength-class lattice (meet method), they'll inherit the structural
memory of these failure-classes without having to remember the original bugs
or read tambear's history.

**Lessons**:

1. **Antigens that don't fire today are still doing work**: they make
   future-additions inherit the discipline. The 2 declared-but-not-firing
   antigens cost nothing at runtime, take ~50 lines of declaration each, and
   immunize all future contributors against the patterns. This is exactly
   what the inheritance-from-tambear.md "Future reciprocity" section
   anticipated: tambear becomes a teaching surface for failure-class memory
   that propagates beyond the current codebase.
2. **The full picture of A's findings**:
   - PolarityInvertedClassMeet: prospective (4 class enums exist; no meet
     methods today)
   - PanickingInDrop: prospective (0 impl Drop blocks today)
   - UlpDistanceRolledByHand: ACTIVE (2 sites found and cleaned up; future
     re-rolls would be flagged)
3. **The real-world adoption story is a mix**: 1 active antigen + 2
   prospective antigens. This is honest. Many production projects will look
   like this — most antigens prevent future bugs; a few catch current ones.
   The mix is fine; the substrate is doing its job whether the antigen fires
   today or in 2 years.

[antigen team note: when designing v0.1 docs, be explicit that "antigen
doesn't fire" is GOOD news for the consumer's codebase, not a sign the
antigen is useless. The educational/prospective value is real.]

### [pending] Phase 1-8 deconstruction of `PolarityInvertedClassMeet`, `PanickingInDrop`, and `UlpDistanceRolledByHand`

(Aristotle thread, after JBD team launch. The antigens were declared without
formal Phase 1-8 review during pre-team scaffolding; same status as the
foundational ADRs.)

### [pending] First `#[presents]` markers in tambear

(Adding presentation markers on actual tambear code that exhibits known
failure-classes. Initially probably to demonstrate the workflow rather than
catch real bugs; later the markers become substrate for refactoring decisions.)

### [pending] First `#[immune]` declaration with witness

(Once a real witness pattern emerges — likely a proptest verifying a class
enum's invariant — declaring immunity. This is when audit becomes
operationally valuable.)

### [pending] Adding antigens for tambear-specific failure-classes

(Patterns that recur in tambear's sweeps may not be in `antigen-stdlib` yet;
they get declared in `crates/tambear/src/antigens.rs` first, then promoted to
`tambear-antigens` (a future crate) or contributed to `antigen-stdlib`.)

### [pending] CI integration with `--strict` flag

(Adding `cargo antigen scan --strict` to tambear's CI pipeline. Initially this
will be advisory-only; later, broken witnesses or unaddressed presentations
fail the build.)

### [pending] `#[descended_from]` propagation across tambear's recipe family

(Tambear has hundreds of recipes that share structural shapes. Once
descended_from propagation lands in antigen, tambear's recipes can declare
inheritance from canonical recipes, and immunity can propagate automatically.)

---

## Updates from antigen-team to this log

When the antigen JBD team takes actions on tambear's behalf (e.g., adapting
the scan output based on tambear's feedback, fixing bugs surfaced by tambear's
adoption), they should annotate entries here with cross-references to the
corresponding antigen-side commits or ADRs.

Example annotation format:

```
[antigen team note 2026-XX-XX]: addressed Lesson 4 (output ergonomics
asymmetry) in commit abc1234 by enriching scan output for the "all clean"
case. See ADR-NNN for the design rationale.
```

This builds the bidirectional feedback loop that makes the
"antigen-graduates-from-tambear" relationship work in practice.
