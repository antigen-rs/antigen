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

### [pending] Phase 1-8 deconstruction of `PolarityInvertedClassMeet` and `PanickingInDrop`

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
