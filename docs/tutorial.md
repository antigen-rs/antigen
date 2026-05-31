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
antigen = "=0.1.0-rc.3"
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
  scan      Scan the workspace for antigen presentations and report unaddressed ones
  audit     Comprehensive immunity coverage report — witness resolution and tier validation
  attest    Manage `.attest/<Antigen>.json` substrate-witness sidecars (ADR-019)
  tolerate  Manage tolerance-ratification sidecars (ADR-019 §tolerance tier)
  oracle    Manage Oracle artifact-class records (ADR-021 §D3)
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

> **Note**: `scan` and `audit` are what you need for this tutorial. `attest`, `tolerate`, and `oracle` cover advanced workflows (substrate-witness sidecars, tolerance ratification, Oracle artifact lifecycle) — see [`witness-tiers.md`](witness-tiers.md) and the substrate-witness section later in this tutorial. `new` and `vaccinate` are in development and hidden until they ship in a future release.

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
  and `#[defended_by]` to reference this antigen.
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
and has no defense wired up yet. That's the signal to write a witness.

---

## Step 4: Write a witness and wire up defense

> **Two tiers of defense.** Before writing the witness, know the choice you're
> making. The quick test is **can a test execute the thing you're defending?**
> If *yes* — the failure-class is about behavior — use `#[defended_by]` (this step).
> If *no* — the failure-class is about substrate state a test can't verify (a stale
> doc, an unpinned dependency, an un-reviewed discipline) — use `requires =` on the
> presents-site instead, covered in the
> [Discipline-witnesses](#discipline-witnesses-when-the-witness-lives-outside-the-code)
> section below. `PanickingInDrop` is the first kind, so we use `#[defended_by]` here.

A **code-tier witness** is a test that verifies the invariant. Write the test and
annotate it `#[defended_by(PanickingInDrop)]`:

```rust
use antigen::defended_by;

#[cfg(test)]
mod tests {
    use super::*;

    // Verify that Drop on a ResourceHandle does not panic, even when cleanup
    // would fail. The correct approach is to log-and-continue rather than
    // panic!() inside drop.
    #[test]
    #[defended_by(PanickingInDrop)]
    fn resource_handle_drop_does_not_panic() {
        // This must not panic. If it does, the test fails (correctly).
        let _ = ResourceHandle { id: 42 };
        // When the binding is dropped, `drop` is called. If drop panics, the
        // test runner catches the panic and marks the test as failed.
    }
}
```

`#[defended_by(PanickingInDrop)]` registers this test as a code-tier witness for
the `PanickingInDrop` failure-class. When `cargo antigen audit` runs, it observes
this registration and cross-references it against every `#[presents(PanickingInDrop)]`
site — reporting `defended at Reachability` when the test function is present.

The `#[presents(PanickingInDrop)]` marker on the Drop impl stays as-is. The site
is still a place where the failure-class *could* apply; the defense evidence lives
on the test, not on the site. Audit observes the relationship between the two and
reports the verdict.

---

## Step 5: Run audit

```sh
cargo antigen audit
```

Output:

```
Auditing workspace: .

Immune-state verdicts (ADR-029 — observed, not declared):
  1 defended, 0 undefended, 0 substrate-gap (across 1 presents-site(s))
  ✓ src/resource.rs:14  PanickingInDrop — defended at Reachability
```

The audit observed your `#[defended_by(PanickingInDrop)]` registration and the
`#[presents(PanickingInDrop)]` site, cross-referenced them, and reported `defended at
Reachability`. The tier is `Reachability` — the witness function is present and
has a `#[test]` attribute, but antigen can't yet confirm it was actually invoked
and passed (that's `Execution` tier, which requires running the test suite).

To reach `Execution` tier: run `cargo test` and pass. The v0.2 audit reports
`Reachability` as the automated check; `Execution` will be confirmed in a future
version when test-run integration lands.

**The site is defended. No undefended presentations, no substrate-gaps.** That's a
clean audit.

---

## What you built

In five steps you:

1. Declared a failure-class (`PanickingInDrop`) with a structural fingerprint
2. Marked a vulnerable site with `#[presents]`
3. Ran `cargo antigen scan` and saw the presentation surfaced
4. Wrote a test witness and registered it with `#[defended_by]`
5. Ran `cargo antigen audit` and saw `defended at Reachability`

The failure-class declaration now lives in your source tree. When a new team
member adds a `Drop` impl later, the fingerprint will detect it automatically
and the scan will surface it — no manual action required from anyone who knows
the lesson.

---

## Discipline-witnesses: when the witness lives outside the code

The tutorial above works when you can write a test that proves the failure-class
doesn't apply — the test IS the evidence. But some failure-classes can't be
verified by a test. Their proof requires human expert judgment, not program
execution.

Consider the `SignedZeroDiscipline` failure-class from tambear — the project
that gave antigen its motivation. The discipline is:

> `sinh(x)` must return `−0.0` when `x` is `−0.0`, not `+0.0`. The IEEE 754
> standard mandates it; most approximation paths silently return `+0.0`.

A test *can* check this — but the test only says "the current implementation
gets this edge case right." It doesn't say "someone who understood the IEEE 754
signed-zero semantics actually reviewed this approximation and confirmed it is
correct." The discipline is mathematical; it requires mathematical expertise
to verify; a passing test alone doesn't encode who provided that expertise.

That's what discipline-witnesses are for.

### Step 1: declare the antigen

```rust
// src/antigens.rs
use antigen::antigen;

/// sinh(x) must preserve signed-zero: sinh(−0.0) = −0.0, not +0.0.
///
/// IEEE 754 §6.3 mandates that signed-zero is preserved under all odd
/// functions. Most ULP-distance approximation paths silently produce +0.0
/// from −0.0 arguments because intermediate subtraction introduces the
/// sign-flip. Any implementation of sinh (or any odd function) that uses
/// subtraction-based ULP approximation must be reviewed by someone who
/// understands this.
#[antigen(
    name = "signed-zero-discipline",
    family = "frame-translation",
    fingerprint = r#"item = fn, name = matches("sinh|cosh|tanh|asinh|acosh|atanh")"#,
    summary = "Transcendental functions must preserve signed-zero per IEEE 754 §6.3.",
)]
pub struct SignedZeroDiscipline;
```

### Step 2: mark the vulnerable site

```rust
// src/numerics.rs
use antigen::presents;
use crate::antigens::SignedZeroDiscipline;

#[presents(SignedZeroDiscipline)]
pub fn sinh(x: f64) -> f64 {
    // ULP-distance approximation — must be reviewed for signed-zero correctness.
    let e = x.exp();
    (e - 1.0 / e) / 2.0
}
```

### Step 3: understand why a test isn't enough

You can write a test:

```rust
#[test]
fn sinh_preserves_signed_zero() {
    assert_eq!(sinh(-0.0_f64).to_bits(), (-0.0_f64).to_bits());
}
```

This test passes. And it should be there. But `#[defended_by(SignedZeroDiscipline)]`
on the test says "a test proves the mathematical discipline is satisfied." That's
overclaiming for a discipline like this. The test
shows the current code produces the right answer for this input. It doesn't show
that a domain expert reviewed the algorithm and confirmed it will *always*
produce the right answer for this input.

The distinction matters when the code changes. A future optimization might
change the approximation path. The test might still pass (because the edge case
is preserved) but now there's no record that someone with floating-point expertise
reviewed the new path.

### Step 4: scaffold a discipline-witness sidecar

```sh
cargo antigen attest scaffold \
    --antigen SignedZeroDiscipline \
    --source-file src/numerics.rs \
    --item-path sinh \
    --fingerprint <item-fingerprint>
```

> `attest scaffold` takes `--antigen`, `--source-file`, `--item-path`, and an
> optional `--fingerprint`. The fingerprint is the item's current structural
> digest. **You usually don't need to pass it**: if you omit `--fingerprint`,
> scaffold scans the source file's crate and auto-fills the matching item's
> digest for you. To obtain a digest explicitly — for the `sign` step, for
> scripting, or to hand-edit a sidecar — use the dedicated verb:
>
> ```sh
> cargo antigen fingerprint --antigen SignedZeroDiscipline --item-path sinh
> # or, JSON for scripting:  cargo antigen fingerprint ... --format json
> ```
>
> (`cargo antigen scan --format json` also emits a `structural_fingerprint` on
> each immunity entry.) With a real digest in place — auto-filled or explicit —
> the `against = "current"` / `fresh_within_days` predicates evaluate against it
> at audit time.

This creates `src/.attest/SignedZeroDiscipline.json` (the sidecar lives in a
`.attest/` directory next to the source file, named for the antigen):

```json
{
    "schema_version": "v1",
    "kind": "immunity",
    "antigen": { "name": "signed-zero-discipline" },
    "source_file": "src/numerics.rs",
    "items": [
        {
            "item_path": "sinh",
            "current_fingerprint": "<computed from current code>",
            "signers": [],
            "oracles": [],
            "extensions": {}
        }
    ]
}
```

The `current_fingerprint` is a hash of the current code at `sinh`. The sidecar
records who reviewed this fingerprint and when. When the code changes, the
fingerprint changes, and any signer who signed against the old fingerprint is
now stale — the audit will surface this and ask for re-attestation.

### Step 5: the math reviewer signs

The domain expert who has reviewed the implementation for signed-zero
correctness runs:

```sh
cargo antigen attest sign \
    --sidecar src/.attest/SignedZeroDiscipline.json \
    --item-path sinh \
    --signer alice \
    --role math-researcher \
    --fingerprint <same-fingerprint-you-scaffolded-with>
```

> `--signer` and `--fingerprint` are both required flags. `--signer alice` is the
> name the `signers(required = ["alice"])` predicate matches against — it matches
> the signer NAME, which is why the predicate below uses `["alice"]`, not
> `["math-researcher"]` (the role is a separate, optional tag). `--fingerprint`
> must match the sidecar's `current_fingerprint` (the value from scaffold), so
> `against = "current"` can confirm the signature is against the present code.

This adds their entry to the sidecar:

```json
"signers": [
    {
        "name": "alice",
        "role": "math-researcher",
        "date": "2026-05-20",
        "signed_against_fingerprint": "<current fingerprint>",
        "basis": { "Fresh": { "reasoning": "Reviewed IEEE 754 §6.3 compliance. The ULP approximation (e^x - e^-x)/2 preserves sign at x=−0 because e^−0 = e^0 = 1, so the subtraction produces −0 from (1 - 1) when the argument is −0." } },
        "strength": "git-trust"
    }
]
```

### Step 6: attach a substrate-witness predicate to the site

```rust
// src/numerics.rs
use antigen::presents;

#[presents(SignedZeroDiscipline, requires = signers(required = ["alice"], against = "current"))]
pub fn sinh(x: f64) -> f64 {
    let e = x.exp();
    (e - 1.0 / e) / 2.0
}
```

The `requires = signers(...)` predicate says: "this site's defense requires the
sidecar to record alice as a current signer." The audit evaluates this predicate
against the `.attest/SignedZeroDiscipline.json` sidecar at audit time and reports
`defended` (predicate passes) or `substrate-gap` (predicate fails — sidecar missing
or not signed).

### Step 7: run audit

```sh
cargo antigen audit
```

Output (abridged):

```
./src/numerics.rs:5  SignedZeroDiscipline
  tier = Execution, hint = discipline-predicate-passed-substrate-current
  evidence_kind = SubstrateState
  signature_strength = git-trust
```

`Execution` tier from `SubstrateState` evidence means: the sidecar was read from
disk, the predicate evaluated, alice's signature was verified as current against
the present fingerprint, and the result is the highest tier achievable from
on-disk substrate evidence. The audit is saying "someone with the right credentials
attested to this discipline claim, and the code hasn't changed since they attested."

### The difference

With a code-witness (`witness = sinh_preserves_signed_zero`):

> "A test function named `sinh_preserves_signed_zero` exists and has `#[test]`."

With a discipline-witness (`requires = signers(required = ["alice"], against = "current")`):

> "Alice — a human with a declared role — reviewed this specific version of the
> code and signed that it satisfies the signed-zero discipline. If the code changes,
> Alice's signature goes stale and the audit will surface it."

Both can coexist on the same site. The test verifies behavior; the signer verifies
discipline. A future code change that breaks the test will surface through test
failure. A change that preserves the test but compromises the mathematical discipline
will surface through attestation staleness. Both signals matter.

---

**v0.2 stdlib families**: antigen ships 7 stdlib families beyond the core vocabulary.
Each demonstrates a distinct failure mode with worked examples:

| Family | What it covers | Example |
|--------|---------------|---------|
| Deferred-Defense (ADR-023) | `#[anergy]`, `#[immunosuppress]`, `#[poxparty]`, `#[orient]` — loudness-as-discipline primitives for intentional non-immunity | `deferred_defense_*.rs` |
| Recurrent-Emergence (ADR-022) | Failure-classes that return after being solved via `#[descended_from]` propagation | `recurrent_emergence.rs` |
| Mucosal-Boundary (ADR-027) | Boundary defense + delegate centralization via `#[mucosal]` / `#[mucosal_delegate]` | `mucosal_boundary.rs` |
| Supply-Chain (ADR-025) | Exact-pin enforcement, content-hash attestation, proc-macro sandboxing | `supply_chain_*.rs` |
| VCS-Information-Loss (ADR-026) | Git history as immune substrate — rollback-without-triage, force-push amnesia, refactor-losing-WHY | `vcs_info_loss.rs` |
| Agentic-Coordination (ADR-028) | Session/agent boundary `SubstrateAlignment` failures (wake without delta injection, cross-crate delegate gaps) | `agentic_coordination.rs` |
| Convergent-Evidence (ADR-024) | Multi-modality independence discipline, iterated witnesses | `convergent_*.rs` |

**Antigen category**: every antigen carries a `category` field — `SubstrateAlignment`
(representation diverges from actual state; use `requires =`) or `FunctionalCorrectness`
(verb produces wrong output; use `witness =`). See `antigen_category.rs` and
`docs/examples-guide.md` for the full taxonomy. The quick test: *can a test exercise
the thing you're defending?* If yes → `FunctionalCorrectness`. If no → `SubstrateAlignment`.

**Triage-commit vs orient**: `#[triage_commit]` is the decisional rollback primitive
(ADR-026) — carries triage classification, target SHA, author identity, rationale,
and time-bound. `#[orient]` is the deferral primitive (ADR-023) — carries a path-out
and a horizon. Different speech-acts. See `triage_commit.rs` for the full 5-color
scale and contrast.

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

**Inheritance**: `#[descended_from]` propagates `#[presents]` markers
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
