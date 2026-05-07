# Case study: how antigen would have caught the DeterminismClass → CommutativityClass reincidence

> The proof artifact. Aspirational pseudocode showing how antigen-the-tool would have
> structurally caught the failure described in [`docs/origin.md`](../origin.md). Closes
> the loop the origin story opens. The actual implementation will follow these shapes
> (with refinements) once the antigen JBD team's Sweeps A1-A4 ship.
>
> **Status**: Pseudocode demonstration. The macros and cargo subcommands described
> here do not yet exist; the syntactic shape conforms to ADR-009 (adoption gradient)
> and ADR-010 (fingerprint grammar v1).

## The situation, recapped

In April 2026, tambear's `DeterminismClass` enum was redesigned after the
GAP-BIT-EXACT-1 bug — a polarity inversion where `meet = std::cmp::min` returned the
*strongest* class instead of the weakest, because the discriminant ordering was
strongest-first while the lattice ordering was reverse-strictness.

The fix was applied. The lesson learned was:

> When a class enum represents "strength of claim" and discriminants are ordered
> with strongest first, the lattice meet operation is `max`, not `min`.

That lesson lived in dev memory, the GAP-BIT-EXACT-1 issue, and the docstring of
`DeterminismClass::meet()`. Months later, DEC-030 v2 introduced `CommutativityClass`
— structurally identical shape, independently arrived at — with `meet =
std::cmp::min`. The same illness, untreated, until math-researcher and pathmaker
caught it during pre-implementation substrate verification.

The illness had been healed once. The healing didn't propagate.

## How antigen-the-tool would have prevented this

The fix to GAP-BIT-EXACT-1 generates a lasting structural artifact: an antigen
declaration that captures what the failure looked like, what the fix looked like,
and what to look for in similar new types.

### Step 1: Declare the antigen at fix time

When DeterminismClass is corrected to use `meet = max`, the team adds:

```rust
// In tambear/src/antigens/lib.rs (a new tambear module for antigen declarations)

use antigen::{antigen, presents, immune};

/// Polarity-inverted class meet: the same failure shape as GAP-BIT-EXACT-1.
///
/// When a Rust enum represents "strength of claim" and its discriminants are
/// ordered with the strongest variant first (smallest discriminant = strongest),
/// the lattice meet operation must use `max` (in discriminant ordering), not `min`.
/// This is because lattice ordering is reverse-strictness while discriminant
/// ordering is forward-strictness.
///
/// Antigen-stdlib candidate; see [`antigen-stdlib::ecosystem::frame_translation`].
#[antigen(
    name = "polarity-inverted-class-meet",
    family = "frame-translation",
    summary = "Class enum + reverse-discriminant-ordering + meet operation; \
               polarity must be max (discriminant ordering), not min.",
    fingerprint = "
        item: enum,
        name: matches('*Class'),
        variants: 3..=8,
        has_method('meet', '(Self, Self) -> Self'),
        any_of([
            attr_present('repr(u8)'),
            doc_contains('strength'),
            doc_contains('lattice')
        ])
    ",
    references = [
        "GAP-BIT-EXACT-1",
        "DEC-030 §1.1",
        "https://github.com/tambear-rs/tambear/issues/...",
    ],
    adr = "DEC-030",
)]
pub struct PolarityInvertedClassMeet;

// The original DeterminismClass declares immunity:

#[immune(
    PolarityInvertedClassMeet,
    witness = determinism_class_meet_polarity_test,
    rationale = "DeterminismClass uses max-in-discriminant-ordering for meet, \
                 verified by the proptest below."
)]
pub enum DeterminismClass {
    BitExact,
    MathematicallyEquivalent,
    ArchConditional,
    ChoiceContingent,
}

impl DeterminismClass {
    pub fn meet(self, other: Self) -> Self {
        // max in discriminant ordering = weakest in lattice ordering = correct.
        if (self as u8) > (other as u8) { self } else { other }
    }
}

#[cfg(test)]
proptest! {
    #[test]
    fn determinism_class_meet_polarity_test(
        a in any::<DeterminismClass>(),
        b in any::<DeterminismClass>(),
    ) {
        // The witness: meet returns the lattice-weaker (discriminant-larger) class.
        let result = a.meet(b);
        prop_assert!(result as u8 >= a as u8);
        prop_assert!(result as u8 >= b as u8);
        prop_assert_eq!(a.meet(b), b.meet(a)); // commutative
    }
}
```

That's the structural memory. The antigen names the failure-class. The fingerprint
describes what kinds of code are vulnerable. The witness proves immunity. The
references point to ratified DECs and the original gap. Nothing about this declaration
will drift over time; if the witness ever fails, CI catches it.

### Step 2: Months later, DEC-030 v2 introduces CommutativityClass

Without antigen, the polarity inversion ships. With antigen, here's what happens.

A pathmaker writes:

```rust
// In crates/tambear/src/lattice/commutativity.rs (a new file)

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum CommutativityClass {
    Strict = 0,
    RoundingEquivalent = 1,
    ArchConditional = 2,
    ChoiceContingent = 3,
}

impl CommutativityClass {
    /// Meet: returns the strongest class that holds along both axes simultaneously.
    pub fn meet(self, other: Self) -> Self {
        // !! polarity-inverted: returns strongest, should return weakest
        if (self as u8) < (other as u8) { self } else { other }
    }
}
```

The pathmaker runs `cargo check`. It passes (it's syntactically valid Rust).
They run `cargo antigen scan`:

```
$ cargo antigen scan

Scanning workspace... matching against 47 active antigens.

⚠ Unaddressed antigen presentation:
  antigen: polarity-inverted-class-meet
  family:  frame-translation
  site:    crates/tambear/src/lattice/commutativity.rs:5
  matched: enum CommutativityClass with strongest-first discriminants
           and a meet method of signature (Self, Self) -> Self

  Summary: Class enum + reverse-discriminant-ordering + meet operation;
           polarity must be max (discriminant ordering), not min.

  References:
    - GAP-BIT-EXACT-1
    - DEC-030 §1.1
    - https://github.com/tambear-rs/tambear/issues/...

  This antigen is in the project's antigen library and your code matches
  its fingerprint. Either:

    a) Add #[immune(PolarityInvertedClassMeet, witness = ...)] with a witness
       function/proptest that verifies meet returns the lattice-weaker class.
       Suggested witness pattern:

           proptest! {
             #[test]
             fn commutativity_class_meet_polarity_test(...) {
                 let result = a.meet(b);
                 prop_assert!(result as u8 >= a as u8);
                 ...
             }
           }

    b) Mark the site explicitly as out-of-scope:

           #[antigen_tolerance(
             PolarityInvertedClassMeet,
             reason = "..."
           )]

       (Use sparingly; the antigen exists for a reason.)

Scan complete: 1 unaddressed presentation, 0 broken witnesses, 47 antigens active.
```

The pathmaker reads the output. They look at the suggested witness pattern. They
write it. They run it. **The test fails.**

```
running 1 test
test commutativity_class_meet_polarity_test ... FAILED

failures:
   commutativity_class_meet_polarity_test:
       prop_assertion failed at proptest test_runner ... iteration 1
       a = Strict (discriminant 0)
       b = ArchConditional (discriminant 2)
       result = Strict (discriminant 0)
       assertion: result as u8 >= a as u8
                  0 >= 0 ✓
       assertion: result as u8 >= b as u8
                  0 >= 2 ✗  FAILED

       The meet() returned the lattice-stronger class instead of the
       lattice-weaker. This is the polarity-inverted-class-meet antigen
       firing in your test.
```

The pathmaker realizes the meet polarity is wrong. They look at the antigen's
references. They find DEC-030 §1.1 and the GAP-BIT-EXACT-1 fix. They flip `<` to `>`:

```rust
impl CommutativityClass {
    pub fn meet(self, other: Self) -> Self {
        // max in discriminant ordering = weakest in lattice ordering = correct.
        if (self as u8) > (other as u8) { self } else { other }
    }
}
```

They re-run the proptest. It passes. They add the immunity declaration:

```rust
#[immune(
    PolarityInvertedClassMeet,
    witness = commutativity_class_meet_polarity_test,
    rationale = "CommutativityClass uses max-in-discriminant-ordering for meet, \
                 same pattern as DeterminismClass. See DEC-030 §1.1 and \
                 GAP-BIT-EXACT-1."
)]
pub enum CommutativityClass { ... }
```

They run `cargo antigen scan` again:

```
$ cargo antigen scan

Scanning workspace... matching against 47 active antigens.

✓ All antigen presentations have witnesses.
✓ All witnesses pass.

Scan complete: 0 unaddressed presentations, 0 broken witnesses, 47 antigens active.
```

The illness is healed. The healing propagates. The structural memory carried forward.

### Step 3: The vaccination operation (if multiple class enums had been introduced)

If, instead of one new class enum, the DEC-030 v2 work had introduced **three** new
class enums (CommutativityClass, RoundingClass, MultiAxisClass) — without manual
review, all three could have shipped with the same polarity inversion.

After the first one is found and fixed manually, the team can run:

```
$ cargo antigen vaccinate \
    --antigen PolarityInvertedClassMeet \
    --pattern 'enum *Class with meet method'

Searching for matching sites...

Found 3 matching sites in the workspace:
  1. crates/tambear/src/lattice/commutativity.rs:5
       enum CommutativityClass — IMMUNE (witness: commutativity_class_meet_polarity_test)
  2. crates/tambear/src/lattice/rounding.rs:8
       enum RoundingClass — UNADDRESSED PRESENTATION
  3. crates/tambear/src/lattice/multi_axis.rs:12
       enum MultiAxisClass — UNADDRESSED PRESENTATION

For each unaddressed site, antigen will:
  - Add #[presents(PolarityInvertedClassMeet)]
  - Generate a stub witness file at tests/antigen_witnesses/<site>_polarity.rs
  - Add #[immune(PolarityInvertedClassMeet, witness = ...)] referring to the stub

The stub witness uses the same proptest pattern as the IMMUNE site (1).
You will need to fill in any class-specific details and run the witness manually.

Apply vaccination to sites 2 and 3? [Y/n] Y

Applied. Generated witness stubs:
  - tests/antigen_witnesses/rounding_class_polarity.rs
  - tests/antigen_witnesses/multi_axis_class_polarity.rs

Next: cargo test --test antigen_witnesses
```

The team runs the witnesses. RoundingClass passes (their meet was already correct).
MultiAxisClass fails (the same polarity inversion). The team fixes MultiAxisClass.
All three classes are now immune. The vaccination has spread immunity across the
structural family in one bulk operation.

This is the pattern that doesn't exist in the Rust ecosystem today, and it's the
load-bearing primitive for projects with many structurally-similar types.

### Step 4: Future class enums automatically inherit immunity

A year later, a different team member adds yet another class enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum FutureClass {
    Best = 0,
    Better = 1,
    Good = 2,
    Acceptable = 3,
}

impl FutureClass {
    pub fn meet(self, other: Self) -> Self {
        if (self as u8) < (other as u8) { self } else { other }   // wrong polarity
    }
}
```

`cargo antigen scan` runs in CI. It matches the polarity-inverted-class-meet antigen
against FutureClass. It flags the unaddressed presentation. The PR fails CI with the
same diagnostic the pathmaker saw earlier. The new team member reads the antigen's
documentation, learns the lesson without ever needing to know the GAP-BIT-EXACT-1
history, and fixes their code before merging.

The illness is never even attempted. The structural memory has done its job.

### Step 5: The antigen propagates to other Rust projects

Tambear publishes its `tambear-antigens` crate to crates.io. Other Rust projects
that build class-style enums import it:

```toml
# Their Cargo.toml
[dependencies]
antigen = "0.1"
tambear-antigens = "0.1"
```

```rust
// Their code
use tambear_antigens::PolarityInvertedClassMeet;

#[derive(...)]
#[repr(u8)]
pub enum OtherProjectClass {
    Strong, Medium, Weak,
}

// cargo antigen scan flags this exactly the same way it flagged CommutativityClass.
```

Tambear's specific lesson — derived from a real bug in a real project — is now
available to every Rust codebase that wants to inoculate against the same failure
shape. The healing has propagated beyond tambear into the broader ecosystem.

This is the compounding the antigen project bets on. Each project's failure-class
memory becomes shareable. The "lesson lost when senior engineers leave" failure mode
dissolves at the ecosystem level.

## What this case study demonstrates

The pseudocode walks through the **complete user journey** of a single antigen:

1. **Declaration at fix-time**: the antigen captures the failure-class structurally
   (lines 25-50 of the first code block above)
2. **Detection on new code**: `cargo antigen scan` flags the matching site (the
   ⚠ output above)
3. **Guided remediation**: the scan output suggests the witness pattern
4. **Validation**: the witness fails on incorrect code, surfacing the bug
5. **Immunity declaration**: once fixed, the immunity is recorded structurally
6. **Vaccination**: when the same pattern exists across multiple sites, bulk
   inoculation is a single command
7. **Future inheritance**: new code automatically inherits the immunity discipline
8. **Cross-project propagation**: the lesson becomes available to other Rust projects
   via published antigen crates

Every step is **structural**. The lesson lives in code, not commit messages. The
healing propagates because the antigen is an artifact, not a memory.

## What's still pseudocode

The macros (`#[antigen]`, `#[presents]`, `#[immune]`), the cargo subcommands
(`cargo antigen scan`, `cargo antigen vaccinate`), and the fingerprint grammar
shown here are pseudocode for a tool that doesn't exist yet. The antigen JBD team
will refine the syntax, the diagnostic output, the fingerprint grammar, and the
implementation details during sweeps A1-A5.

But the **shape is right**. This case study is a contract between the design intent
and what the tool must accomplish. When the tool ships and antigen-the-tool is
imported into tambear, this case study will be re-written with the actual syntax.
At that point, this document will be the **first integration test** of antigen
into a real Rust project.

The illness will be cured before it appears.

That's the project.

---

## Update — 2026-05-07: real integration data

The scaffolding session shipped working `#[antigen]`, `#[presents]`, `#[immune]`
macros, a working `cargo antigen scan`, and a working `cargo antigen audit`
with witness validation. Before the JBD team launch, the team-lead ran the
tool against tambear's actual codebase to gather real-world adoption data.

### Phase 1: Bare scan, no setup, no antigen-side work

```sh
cd R:/antigen
cargo run --release --bin cargo-antigen -- antigen scan --root R:/tambear/crates
```

Result:

```
Scanned 216 files, found 0 antigen-related declarations:
  - 0 antigen declarations
  - 0 presentations
  - 0 immunity claims

✓ No unaddressed presentations.

real    0m0.341s
```

**216 Rust source files (across 7 workspace crates, ~50,000+ lines including
generated proc-macro outputs and large recipe modules) scanned in 341ms.**

Per ADR-008 (named-observer terminal stratum) and Risk A3 (slow scan kills
adoption), this is the kind of performance that lets antigen scan land in CI
without becoming a bottleneck.

The first run included a release-mode compile of `cargo-antigen` (10.8s); the
0.341s is the warm-cache scan time. CI will see ~0.5s overhead per scan once
the binary is cached.

### Phase 2: Step-by-step naive integration

Five steps to first-antigen-declared-and-discovered:

1. **Add path dependency** to `R:/tambear/crates/tambear/Cargo.toml`:
   ```toml
   antigen = { path = "../../../antigen/antigen" }
   ```

2. **Create antigens module** at `R:/tambear/crates/tambear/src/antigens.rs`
   with `PolarityInvertedClassMeet` and `PanickingInDrop` declarations
   (using the syntax shown earlier in this case study).

3. **Register the module** by adding `pub mod antigens;` to
   `R:/tambear/crates/tambear/src/lib.rs`.

4. **Compile**: `cargo check` — succeeds in 1.7s incremental.

5. **Re-scan**:
   ```
   Scanned 217 files, found 2 antigen-related declarations:
     - 2 antigen declarations
     - 0 presentations
     - 0 immunity claims
   ```

The integration works end-to-end. Tambear is now the first project to use
antigen.

### Phase 3: What we learned

**Performance**: 217-file scan in <0.5 seconds. Acceptable for CI.

**Adoption friction**: 5 steps, ~10 minutes for first antigen integration
(including reading docs and figuring out the macro syntax). Subsequent
declarations land in seconds. The 60-second-per-additional-antigen target
(ADR-008) is met.

**Honest finding about tambear's actual code**: tambear's main-branch class
enums (`DeterminismClass`, `FiniteClass`, `NyquistClass`, `PdeClass`) do NOT
currently have `meet` methods. The original GAP-BIT-EXACT-1 polarity-inversion
was already corrected during DEC-007 work, and the team's substrate verification
discipline caught the near-miss in `CommutativityClass` before it shipped.

So:

- Antigen DOESN'T flag any tambear class enum as polarity-inverted-class-meet
  vulnerable in the current codebase. **This is the correct behavior.**
- Antigen's value for tambear is **prospective**: any future class enum
  introduced with a meet method inherits the structural memory of why
  polarity matters, without anyone needing to remember GAP-BIT-EXACT-1.

This is exactly what structural failure-class memory is supposed to do. The
tooling didn't find a bug because there isn't one. It will find the next one
if it appears.

**Honest finding about output ergonomics**: scan's brief summary works for the
"workspace is clean" case but doesn't show the full diagnostic context that
makes audit's output more useful. Sweep A2 should consider making `cargo
antigen scan` more diagnostic-rich for this case.

### Phase 4: What the team inherits

The integration with tambear is committed and ready for the team to extend:

- `R:/tambear/crates/tambear/Cargo.toml` has the path dependency
- `R:/tambear/crates/tambear/src/antigens.rs` has 2 seed declarations + TODO
  markers for follow-up antigens
- The scan demonstrably works against real-world Rust code

The team can:

1. Phase 1-8 the antigens themselves (currently declared without formal review)
2. Add `#[presents]` markers across tambear's actual code where applicable
3. Author witnesses and add `#[immune]` declarations
4. Extend the antigen catalog as new failure patterns surface in tambear's
   sweeps

This is what the antigen project being "the tool tambear depends on" looks
like in practice. Phase 2 of the inheritance arc (per
[`inheritance-from-tambear.md`](inheritance-from-tambear.md)) is now real, not
aspirational.

For ongoing experience reports, the [`tambear-adoption-log.md`](tambear-adoption-log.md)
captures every antigen-related thing tambear does going forward — what worked,
what didn't, what got removed, what problem each addition solved.

---

## References

- [`docs/origin.md`](../origin.md) — the post-mortem narrative motivating the project
- [`docs/decisions.md` ADR-001](../decisions.md#adr-001--failure-class-memory-is-structural-not-documentary) — failure-class memory is structural
- [`docs/decisions.md` ADR-009](../decisions.md#adr-009--adoption-gradient-antigen-meets-consumers-at-any-discipline-level) — adoption gradient (the syntax shown is the enriched layer)
- [`docs/decisions.md` ADR-010](../decisions.md#adr-010--fingerprint-grammar-v1-syn-based-ast-visitor-pattern) — fingerprint grammar v1
- [`docs/expedition/api-shape.md`](api-shape.md) — sketch of the macro and cargo subcommand surface
- [`docs/expedition/inheritance-from-tambear.md`](inheritance-from-tambear.md) — phase 2 (tambear adopts antigen as code-level DEC extension)
- [`docs/expedition/tambear-adoption-log.md`](tambear-adoption-log.md) — ongoing tambear-uses-antigen experience log
