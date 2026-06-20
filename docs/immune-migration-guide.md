# Migrating off `#[immune]` — the ADR-029 conversion guide

> `#[immune(X, witness = Y)]` and `#[immune(X, requires = ...)]` are
> **deprecated** (ADR-029, *observe-don't-declare*). They still compile — with a
> deprecation warning — but the audit model has moved on. This guide is the
> step-by-step conversion to the two replacement forms.

If your codebase imports antigen and has `#[immune]` declarations, this is the
doc that gets you to the current idiom.

---

## Why `#[immune]` is deprecated

`#[immune]` let a *code site* declare its own immunity: "I am immune to `X`,
here is my witness." ADR-029 (*observe-don't-declare*) rejected that posture.
A site never gets to certify itself — **immunity is observed by the audit, not
declared at the code site.** The new model splits the old single attribute along
the line of *where the evidence actually lives*:

| Old `#[immune]` form | New form | Tier | Goes on |
|---|---|---|---|
| `#[immune(X, witness = a_test)]` | `#[presents(X)]` + `#[defended_by(X)]` | code-tier | the site **and** the test |
| `#[immune(X, requires = <predicate>)]` | `#[presents(X, requires = <predicate>)]` | substrate-tier | the site |

The discriminator (ADR-029 R5): **evidence belongs where it is.** A test
*executes* the defended thing, so the registration lives on the test
(`#[defended_by]`). A substrate predicate (a sign-off, a ratified doc, an
unpinned dep) lives *outside* the code, so it folds onto the site
(`requires =`).

---

## The deprecation warning you'll see

A live `#[immune]` site emits this at `cargo build`:

```text
warning: use of deprecated unit struct
`__ANTIGEN_IMMUNE_DEPRECATED_<Antigen>_<n>::AntigenImmuneDeprecated`:
use #[defended_by] on tests (code-tier) or #[presents(requires=...)] for
substrate evidence — ADR-029
  --> src/your_file.rs:NN:1
   |
NN | / #[immune(
   | | ...
   = note: `#[warn(deprecated)]` on by default
   = note: this warning originates in the attribute macro `immune`
```

The warning fires **at the `#[immune]` author's call site** — not at callers of
the annotated item. (The macro routes the deprecation through a scoped, named
`const` block so that callers of an `#[immune]`-annotated function never see a
spurious warning. If you have *two* `#[immune]` stacked on one item, migrating
also clears a known const-collision — see
[stacked markers just work](#bonus-stacked-markers-just-work) below.)

The migration target produces **zero warnings**.

---

## Case 1 — `witness =` (code-tier)

This is the most common form: a site marked immune, with a `#[test]` as its
witness.

### Before

```rust
use antigen::immune;
use crate::antigens::UlpDistanceRolledByHand;

#[immune(
    UlpDistanceRolledByHand,
    witness = ulp_wrapper_delegates_to_canonical_test,
    rationale = "This wrapper is a thin pass-through to the canonical \
                 implementation. The witness verifies the across-zero behavior."
)]
fn ulp_distance(a: f64, b: f64) -> u64 {
    canonical::ulp_distance_f64(a, b)
}

#[test]
fn ulp_wrapper_delegates_to_canonical_test() {
    assert_eq!(ulp_distance(1.0, 1.0), 0);
    // ... the across-zero killer assertion ...
}
```

### After

Split the one attribute into two markers, each on its natural home:

```rust
use antigen::{defended_by, presents};
use crate::antigens::UlpDistanceRolledByHand;

// The SITE gets #[presents] — it has the structural shape of the failure-class.
#[presents(UlpDistanceRolledByHand)]
fn ulp_distance(a: f64, b: f64) -> u64 {
    canonical::ulp_distance_f64(a, b)
}

// The TEST gets #[defended_by] — it registers what it defends.
#[test]
#[defended_by(UlpDistanceRolledByHand)]
fn ulp_wrapper_delegates_to_canonical_test() {
    assert_eq!(ulp_distance(1.0, 1.0), 0);
    // ... the across-zero killer assertion ...
}
```

### The three mechanical steps

1. **Move the antigen type to a `#[presents(X)]` on the original site.** Drop the
   `witness =` and `rationale =` — `#[presents]` takes only the positional
   antigen type.
2. **Add `#[defended_by(X)]` to the witness test.** This is the function the old
   `witness =` pointed at. `#[defended_by]` takes *exactly one* positional
   argument (the antigen type); it carries no `witness =`/`requires =`/`rationale =`.
3. **Drop the `rationale`.** It has no slot in the new model. If the rationale
   carries information worth keeping, move it to a doc comment on the test or the
   site — it documents *why*, which is exactly what a comment is for.

### What the audit reports after

Running `cargo antigen audit` on the migrated code produces a **clean defended
verdict**:

```text
Immune-state verdicts (ADR-029 — observed, not declared):
  1 defended, 0 undefended, 0 substrate-gap (across 1 presents-site(s))
  ✓ ...:NN  UlpDistanceRolledByHand — defended at Reachability by ...:MM
```

> **Tier honesty.** The verdict reads **"defended at Reachability"**, not
> Execution. Reachability means the audit confirmed the witness *exists and is
> wired to the site* — it does not *run* the test; the audit does not invoke
> `cargo test`. "Defended" here is an honest statement about the *circuit being
> wired*, not a claim that the test passed. That is the correct, non-overclaiming
> reading.

---

## Case 2 — `requires =` (substrate-tier)

When the old `#[immune]` carried `requires = <predicate>` instead of
`witness =`, the migration is a one-attribute rename: `#[immune]` →
`#[presents]`. The predicate moves across unchanged.

### Before

```rust
#[immune(
    crate::antigens::VacuousCompletionFalseGreen,
    requires = signers(required = ["alice"], against = "any")
)]
pub fn completion_state_with(&self, /* ... */) -> CompletionState { /* ... */ }
```

### After

```rust
#[antigen::presents(
    crate::antigens::VacuousCompletionFalseGreen,
    requires = signers(required = ["alice"], against = "any")
)]
pub fn completion_state_with(&self, /* ... */) -> CompletionState { /* ... */ }
```

The `requires =` predicate grammar is identical on both attributes — the macro
emits the same `antigen:requires:v1:<json>` doc marker that `cargo antigen scan`
reads from either. So predicates like
`all_of([signers(...), ratified_doc(...), fresh_within_days(...)])` migrate
verbatim. (For a full worked substrate-witness predicate, see
[`composition.md`](composition.md) and the `substrate_witness` example under
`antigen/examples/`.)

> Note one syntax difference between the two cases above: `#[presents]` rejects
> `witness = ...` outright (with a migration message pointing at `#[defended_by]`),
> and `#[defended_by]` rejects `requires = ...`. Each new primitive carries only
> the evidence kind that belongs on it.

---

## Bonus: stacked markers just work

Migrating buys you something concrete: **multiple antigen markers stack cleanly
on one item.** Two `#[presents]` on the same function compile fine — so a method
that defends against two failure-classes just lists both:

```rust
#[antigen::presents(crate::antigens::VacuousCompletionFalseGreen, requires = signers(...))]
#[antigen::presents(crate::antigens::ParallelStateComputationDiverges, requires = signers(...))]
pub fn signature_state_with(&self, /* ... */) -> CampsiteState { /* ... */ }
```

This matters most if you have **two `#[immune]` attributes stacked on one item**
today — that's a configuration that has hit a known antigen const-collision bug.
**Migrating to `#[presents]` resolves it**, because `#[presents]` emits
`#[doc = ...]` markers and doc-markers stack legally. The ADR-029 migration *is*
the fix; no separate workaround needed. (This is a *reason to migrate*, never a
hazard of migrating.)

> **Background (skip unless you're curious how the collision worked).** The
> `#[immune]` deprecation machinery emits a `const` item per attribute. An early
> version made that const *anonymous* (`const _: () = {…}`), so two stacked
> `#[immune]` on a method in an `impl` block produced two `const _` items in
> associated-const position — which Rust rejects ("`const` items in this context
> need a name" / duplicate definitions). The macro was later hardened to emit a
> *named*, per-emission const, so even stacked `#[immune]` now compiles — but
> migrate regardless: `#[immune]` is deprecated, and `#[presents]` is the form
> built to stack.

---

## Quick reference

```text
#[immune(X, witness = some_test)]      →  #[presents(X)]            on the site
                                          #[defended_by(X)]         on some_test

#[immune(X, requires = <predicate>)]   →  #[presents(X, requires = <predicate>)]

rationale = "..."                      →  drop it (move to a doc comment if useful)
```

**Imports** change from `use antigen::immune;` to
`use antigen::{presents, defended_by};` (drop `immune`; add whichever of
`presents`/`defended_by` you now use).

---

## Verifying your migration

After converting a site, confirm two things:

1. **It compiles with no deprecation warning.**
   ```sh
   cargo build
   ```
   A clean build (no `use of deprecated unit struct ...` line) means the
   `#[immune]` is gone.

2. **It audits clean.**
   ```sh
   cargo antigen audit
   ```
   A code-tier migration should show the site as `defended` (at Reachability);
   a substrate-tier migration shows `defended` if its `requires =` predicate is
   satisfied by a signed `.attest/` sidecar, or `substrate-gap` if the sidecar
   is missing (intent recorded, evidence not yet present — a warning, not a
   failure).

If `cargo antigen audit` reports the site as `undefended`, the circuit isn't
wired — most often the `#[defended_by]` antigen type doesn't match the
`#[presents]` antigen type, or the witness test wasn't marked. Check that the
**same antigen type** appears on both the site and its witness.

---

## See also

- [`macros.md`](macros.md) — full reference for `#[presents]`, `#[defended_by]`,
  and the `requires =` predicate grammar
- [`composition.md`](composition.md) — how the new forms compose with tests,
  proptest, lints, and formal verification
- [`decisions.md`](decisions.md) — observe-don't-declare, and substrate-witness
  predicates
- `antigen/examples/basic.rs` — a complete compileable `#[presents]` +
  `#[defended_by]` pair
- `antigen/examples/substrate_witness.rs` — a complete `#[presents(requires=...)]`
  substrate-tier example

---

*`#[immune]` declared. `#[presents]` + `#[defended_by]` observe. The migration
moves the certifying voice from the code site to the audit — where it belongs.*
