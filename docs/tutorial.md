# Antigen — First 15 Minutes

> **Who this is for**: a Rust developer who has never seen antigen. You know
> Rust, you write production code, you use cargo normally. By the end of this
> tutorial you will have a working antigen declaration, a passing scan, and a
> passing audit in a project you control.

---

## Why antigen exists

You fix a bug. You document why in a commit message. Six months later, a new
team member writes structurally identical code without knowing the lesson
exists. The bug comes back.

Antigen's answer: put the lesson in the type system, next to the code it
protects, in a form that survives developer turnover, AI context cycling, and
time.

One concrete example: tambear's `DeterminismClass` had a method `meet` that
returned `std::cmp::min` of its discriminants. Correct for the discriminant
ordering, wrong for the lattice ordering — the class with the highest
discriminant is the *weakest* element, so lattice-meet should return the
*larger* discriminant (the *weaker* variant). The fix was one line. But later
work added `CommutativityClass` with the same shape and almost made the same
mistake before the structural pattern was recognized.

With antigen, the pattern is declared:

```rust
#[antigen(
    name = "polarity-inverted-class-meet",
    family = "frame-translation",
    fingerprint = r#"item = enum, name = matches("*Class"), has_method("meet", "(self, Self) -> Self")"#,
    summary = "Class enums with strongest-first discriminants must use max for meet, not min.",
)]
pub struct PolarityInvertedClassMeet;
```

Now `cargo antigen scan` finds every enum with this shape and reports it as a
site that presents this failure-class — automatically, without anyone having to
remember the lesson.

---

## Setup

Add antigen to your `Cargo.toml`:

```toml
[dependencies]
antigen = "0.1.0-rc.1"
```

Install the cargo subcommand:

```sh
cargo install cargo-antigen
```

Verify:

```sh
cargo antigen --help
```

```
The "antigen" subcommand of cargo

Usage: cargo antigen <COMMAND>

Commands:
  scan   Scan the workspace for antigen presentations and report unaddressed ones
  audit  Comprehensive immunity coverage report — witness resolution and tier validation
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

> **Note**: `new` and `vaccinate` are in development and hidden until they ship
> in a future release. `scan` and `audit` are what you need for now.

---

## Step 1: Declare a failure-class

Create `src/antigens.rs` in your crate:

```rust
// src/antigens.rs
use antigen::antigen;

/// Drop impls must not panic.
///
/// Panicking inside `Drop` while another panic is already unwinding causes
/// process abort. The fingerprint detects `panic!` and `unreachable!` macro
/// calls in `drop` implementations.
#[antigen(
    name = "panicking-in-drop",
    family = "boundary-violation",
    fingerprint = r#"all_of([item = impl, has_method("drop", "(& mut self)"), any_of([body_contains_macro("panic"), body_contains_macro("unreachable")])])"#,
    summary = "Drop impls must not panic; panic-during-unwind causes process abort.",
    references = ["https://doc.rust-lang.org/std/ops/trait.Drop.html#panics"],
)]
pub struct PanickingInDrop;
```

Declare the module from `src/lib.rs`:

```rust
// src/lib.rs
pub mod antigens;
```

The `#[antigen]` attribute:
- `name` — kebab-case identifier for the failure-class. Used by `#[presents]`
  and `#[immune]` to reference this antigen.
- `family` — the failure-class family from the 8-class taxonomy. Optional but
  useful for grouping.
- `fingerprint` — structural pattern for passive detection. Code matching this
  pattern gets flagged automatically by `cargo antigen scan`, even without an
  explicit `#[presents]` marker.
- `summary` — one-paragraph human description. Shows in scan/audit output.
- `references` — links to bug reports, design docs, or external resources.

For Layer 1 (minimum-viable), only `name` and `fingerprint` are required.

---

## Step 2: Mark a vulnerable site

Now mark code that is vulnerable to this failure-class:

```rust
// src/resource.rs
use antigen::presents;
use crate::antigens::PanickingInDrop;

pub struct ResourceHandle {
    pub id: u32,
}

impl ResourceHandle {
    fn cleanup(&self) -> Result<(), String> {
        Err("simulated failure".to_string())
    }
}

#[presents(PanickingInDrop)]
impl Drop for ResourceHandle {
    fn drop(&mut self) {
        // This panics if cleanup fails — which causes process abort during unwind.
        self.cleanup().expect("cleanup failed");
    }
}
```

`#[presents(PanickingInDrop)]` says: "I know this site is vulnerable to the
`PanickingInDrop` failure-class. The scan should notice this."

---

## Step 3: Run scan

```sh
cargo antigen scan
```

Output:

```
Scanning workspace: .

Scanned 3 files, found 2 antigen-related declarations:
  - 1 antigen declarations
  - 1 explicit #[presents] markers
  - 0 immunity claims

1 unaddressed explicit presentation(s):

  ./src/resource.rs:14  PanickingInDrop on impl
```

> Zero-count lines are suppressed: fingerprint matches, tolerated sites, and
> parse failures only appear when nonzero. A clean project with no fingerprint
> matches produces a shorter summary than one with matches.

The explicit `#[presents]` marker you wrote shows up as an unaddressed explicit
presentation. There are no fingerprint matches — this is expected: the fingerprint
detects `panic!` and `unreachable!` macro invocations, but the `drop` body
here uses `.expect(...)`, which is a method call, not a macro. Method-call
panics aren't visible to v1 fingerprint matching. This is why explicit
`#[presents]` markers matter: they let you annotate sites the fingerprint can't
reach.

The scan is telling you: this site presents the `PanickingInDrop` failure-class
and has no immunity. That's the signal to write a witness.

---

## Step 4: Write a witness and claim immunity

A **witness** is evidence that the failure-class doesn't obtain at this
site — typically a test that verifies the invariant. Add it to your test module:

```rust
// src/resource.rs (continued)

#[cfg(test)]
mod tests {
    use super::*;

    // Verify that Drop on a ResourceHandle does not panic, even when cleanup
    // would fail. The correct approach is to log-and-continue rather than
    // panic!() inside drop.
    #[test]
    fn resource_handle_drop_does_not_panic() {
        // This must not panic. If it does, the test fails (correctly).
        let _ = ResourceHandle { id: 42 };
        // When the binding is dropped, `drop` is called. If drop panics, the
        // test runner catches the panic and marks the test as failed.
    }
}
```

Now declare immunity:

```rust
use antigen::immune;

#[immune(PanickingInDrop, witness = resource_handle_drop_does_not_panic)]
#[presents(PanickingInDrop)]
impl Drop for ResourceHandle {
    fn drop(&mut self) {
        if let Err(e) = self.cleanup() {
            // Log the error instead of panicking — this is the fix.
            eprintln!("ResourceHandle cleanup failed (id={}): {e}", self.id);
        }
    }
}
```

`#[immune(PanickingInDrop, witness = resource_handle_drop_does_not_panic)]`
says: "This site is defended against `PanickingInDrop`. The witness is the test
function `resource_handle_drop_does_not_panic`."

Note that `#[presents]` stays even after adding `#[immune]`. The site is still
a place where the failure-class *could* apply; the immunity claim says it's
defended. Scan distinguishes "presents + immune" from "presents + no immune."

---

## Step 5: Run audit

```sh
cargo antigen audit
```

Output:

```
Auditing workspace: .

Audited 1 immunity claim(s):
  - 1 declared (witness identifier found in workspace — not yet semantically verified)
  - 0 external (delegated to clippy/kani/prusti/etc. — not yet executed by antigen)
  - 0 ambiguous (witness name resolves to multiple workspace functions)
  - 0 broken (witness identifier not found)
  - 0 missing (no witness identifier)

⚠ 1 immunity claim(s) below Execution tier:

  ./src/resource.rs:14  PanickingInDrop (witness = `resource_handle_drop_does_not_panic`)
    tier = Reachability, hint = TestAttributePresentNotInvoked
```

The audit found your immunity claim and verified the witness function exists in
the workspace. The tier is `Reachability` — the witness function is present and
has a `#[test]` attribute, but antigen can't yet verify it was actually invoked
and passed (that's `Execution` tier, which requires running the test suite).

To reach `Execution` tier: run `cargo test` and pass. The v1 audit reports
`Reachability` as the automated check; `Execution` will be promoted in a future
version when test-run integration lands.

**No broken witnesses, no missing witnesses, no ambiguous witnesses.** That's a
clean audit.

---

## What you built

In five steps you:

1. Declared a failure-class (`PanickingInDrop`) with a structural fingerprint
2. Marked a vulnerable site with `#[presents]`
3. Ran `cargo antigen scan` and saw the presentation surfaced
4. Wrote a test witness and declared `#[immune]`
5. Ran `cargo antigen audit` and got a clean result

The failure-class declaration now lives in your source tree. When a new team
member adds a `Drop` impl later, the fingerprint will detect it automatically
and the scan will surface it — no manual action required from anyone who knows
the lesson.

---

## Next steps

**More failure-classes to declare**: the 8-class failure taxonomy
(`docs/expedition/design-intent.md`) gives you the vocabulary. The seed catalog
(`docs/expedition/stdlib-seed-antigens.md`) has 10 concrete antigens with
fingerprints ready to adapt.

**Composition boundaries**: if your code has two implementations that must agree
(an optimized path and a reference path, an incremental and a full recompute),
see [`docs/usage-patterns.md`](usage-patterns.md) — the "Antigens at
composition boundaries" pattern shows how to mark the consistency test as the
vulnerable site.

**When to use `#[antigen_tolerance]`**: if the scan surfaces a site you own but
it isn't actually vulnerable (a test that deliberately constructs the failure
pattern, a translation layer that *is* the frame boundary), see the
`#[antigen_tolerance]` pattern in [`docs/usage-patterns.md`](usage-patterns.md).

**Where declarations live**: see
[`docs/where-to-look-for-antigens.md`](where-to-look-for-antigens.md) for
conventions on organizing antigen declarations in larger projects.

**Fingerprint syntax**: see [`docs/fingerprint-grammar.md`](fingerprint-grammar.md)
for the full operator reference with worked examples.

**Macro reference**: see [`docs/macros.md`](macros.md) for the complete
attribute syntax, all fields, and discipline notes for every macro.

**Witness tiers**: see [`docs/witness-tiers.md`](witness-tiers.md) for what
each audit tier (`Reachability`, `Execution`, `FormalProof`) means and how to
reach a higher tier.

**Scan/audit output**: see [`docs/output-formats.md`](output-formats.md) for
a field-by-field reference of human-readable and JSON output, including the
full JSON schema.

**Inheritance**: `#[descended_from]` propagates `#[presents]` and `#[immune]`
through derived types, copy-paste relationships, and structural similarity. A
type that inherits from a vulnerable type is itself flagged. See the
`#[descended_from]` examples in `antigen/examples/`.

---

## Troubleshooting

**`cargo antigen` not found**: make sure `~/.cargo/bin` is in your `PATH` and
`cargo install cargo-antigen` completed successfully. Run `cargo antigen --help`
to verify the binary is reachable.

**Fingerprint matches on sites you don't own**: fingerprints match structurally
across your entire workspace, including test fixtures and examples. Use
`#[antigen_tolerance]` to silence intentional matches. See
[`docs/usage-patterns.md`](usage-patterns.md).

**Witness reported as broken**: the witness name must match a function that
exists in a `.rs` file under the scan root. Check spelling, module path, and
that the function isn't behind a `#[cfg]` gate that the scan can't see. Run
`cargo antigen audit --format json` for details.

**Witness reported as ambiguous**: two functions in the workspace have the same
name. Qualify the witness path: `witness = my_module::resource_handle_drop_does_not_panic`.

**`has_method` fingerprint not matching**: check the receiver form. User-natural
Rust syntax (`"(&mut self)"`, `"(&self)"`) works — the engine canonicalizes both
sides via `proc_macro2` at parse time. If the pattern still doesn't match, the
likely cause is a token-class distinction: `"(Self, Self)"` (type name) is not the
same as `"(self, Self)"` (receiver keyword). See the receiver-rendering reference
table in [`docs/fingerprint-grammar.md`](fingerprint-grammar.md).
