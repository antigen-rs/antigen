# Antigen — Rust Ecosystem Composition Map

> Catalog of existing Rust ecosystem tools that already handle pieces of the
> failure-class-memory problem. For each tool: what it does, which of antigen's
> 8 failure-classes it covers, its public integration surface, and a CONCRETE
> witness-mechanism integration sketch (i.e., what
> `#[immune(X, witness = <tool>::<thing>)]` actually means and does).
>
> Compose, don't compete. The witness mechanism is the threading point.

---

## How to read this document

Every tool entry has five fields:

1. **What it does** — one-paragraph summary.
2. **Failure-class coverage** — mapped to antigen's 8-class taxonomy
   (1 frame-translation, 2 forgotten-lesson, 3 implicit-coupling,
   4 stale-context, 5 premature-abstraction, 6 incompatible-merger,
   7 boundary-violation, 8 optionality-collapse).
3. **Integration surface** — what antigen needs to call, parse, or check.
4. **Witness-mechanism opportunity** — concrete sketch of what
   `#[immune(X, witness = <tool>::<thing>)]` does at scan-time and run-time.
5. **Gaps** — what the tool does NOT cover that antigen still needs.

The 8 classes are deliberately broad. A tool that covers a class for a
specific failure pattern (e.g., kani covers boundary-violation for
arithmetic-overflow) does not cover the whole class — only the slice it
verifies. The composition matrix at the end records *slices*, not
all-or-nothing coverage.

---

## 1. Clippy — the workhorse linter

**What it does.** Clippy is the Rust ecosystem's de-facto static-analysis
front-end. ~700 lints across categories `correctness` (almost-bugs),
`suspicious` (probably-wrong), `style` (idiom), `complexity` (clarity),
`perf` (slow), `restriction` (cherry-pick: forbid `.unwrap()`,
`mem::forget`, `panic!`, `as_conversions`, etc.), `pedantic` (opinionated),
`nursery` (in-development), `cargo` (manifest-level). Lints are written as
`EarlyLintPass` (AST, no type info) or `LateLintPass` (HIR with full type
information) impls. Lint level controlled via
`#[allow|warn|deny|forbid(clippy::lint_name)]`. Restriction lints are the
key category for failure-class-memory: each is a named
"don't-do-this-here" rule.

**Failure-class coverage.**
- **(7) Boundary-violation** — `clippy::indexing_slicing`, `unreachable`,
  `panic`, `unwrap_used`, `expect_used`, `dbg_macro` all enforce
  trust-boundary checks at structural sites.
- **(2) Forgotten-lesson** — every restriction lint *is* a named
  forgotten-lesson, but globally-applied. Clippy can't say "this lesson
  applies HERE because of structural similarity" — it applies everywhere.
- **(6) Incompatible-merger** — `clippy::float_cmp`,
  `mut_mut`, `cast_lossless` flag known-bad combinations of correct things.
- **(1) Frame-translation** — partial via `clippy::wrong_self_convention`,
  `should_implement_trait` — these flag semantic-name-vs-behavior mismatches.

**Integration surface.**
- Lint names are stable strings: `clippy::unwrap_used`, `clippy::panic`,
  `clippy::indexing_slicing`. These are the public API.
- `cargo clippy --message-format=json` emits structured diagnostics with
  `code.code = "clippy::lint_name"`, span info, and message body. Antigen
  can parse this directly.
- For custom lints: **dylint** (`cargo dylint new`) lets a project ship its
  own out-of-tree `LateLintPass` libraries. Registered via
  `[workspace.metadata.dylint.libraries]` in `Cargo.toml`. This is the
  path for project-specific antigens that need full type information.
- Clippy's lint-config file `clippy.toml` for per-lint thresholds.

**Witness-mechanism opportunity.**

```rust
#[immune(
    PanickingInDrop,
    witness = clippy::lint("clippy::panic", scope = "this_impl"),
    rationale = "Drop impl is lint-clean for clippy::panic; no panic! reachable."
)]
impl Drop for MyType { /* ... */ }
```

What `cargo antigen scan` does:
1. Parses `#[immune(..., witness = clippy::lint("X", ...))]` declarations.
2. Runs `cargo clippy --message-format=json -- -D clippy::X` (or whatever
   level) scoped to the marked item's span.
3. If clippy reports zero hits in that scope, immunity is satisfied.
4. If clippy reports a hit, antigen surfaces it as a *failed witness* —
   the immunity claim is rejected, and `cargo antigen audit` reports a
   broken antibody (more diagnostic than a plain clippy warning, because
   it names the failure-class).

**Strongest variant — dylint custom lint as witness:**

```rust
#[immune(
    FrameTranslation,
    witness = dylint::lint("antigen_polarity_check", target = meet),
    rationale = "Custom dylint pass verifies meet's polarity matches lattice direction."
)]
pub fn meet(a: Class, b: Class) -> Class { /* ... */ }
```

Here the dylint pass IS the antibody. Antigen's role: name the failure-class
(`FrameTranslation`), tie the lint to a specific item, propagate immunity
through `#[descended_from]`. The dylint code does the structural check.

**Gaps.** Clippy is *all-sites or no-sites* — a lint is enabled
crate-wide. Antigen's value-add is *site-specific* immunity claims. Clippy
also doesn't carry "why this lesson exists" memory; antigen's `rationale`
field plus the named antigen IS that memory.

---

## 2. Proptest — strategy-DSL property-based testing

**What it does.** Proptest generates random inputs from a `Strategy`
type, runs the property, and on failure shrinks the counter-example to a
minimal reproducer. Stable seed-based; failing seeds persisted under
`proptest-regressions/` directory so failures replay deterministically.
Configurable via `ProptestConfig` (cases, max_shrink_iters, timeout).
The macro form:

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn property_under_all_inputs(x in any::<i64>(), y in 0i64..100) {
        prop_assert!(my_fn(x, y) >= 0);
    }
}
```

**Failure-class coverage.**
- **(7) Boundary-violation** — exhaustive-ish boundary search; finds
  off-by-one, overflow, edge-value bugs reliably.
- **(6) Incompatible-merger** — composition properties
  (`f(g(x)) == h(x)` style) catch bad combinations.
- **(1) Frame-translation** — round-trip properties
  (`decode(encode(x)) == x`) catch translation-direction bugs.
- **(8) Optionality-collapse** — properties asserting "if condition C, then
  P; else Q" preserve the conditional shape during testing.

**Integration surface.**
- `proptest!` macro generates `#[test]` functions. Discoverable by
  `cargo test`.
- Failing seeds in `proptest-regressions/<crate>/<test-name>.txt`. Antigen
  can read these to know if a witness has *ever* failed.
- The `Strategy` type is the public API for generators; `any::<T>()`,
  `prop::collection::vec`, `prop_oneof!` etc.

**Witness-mechanism opportunity.**

```rust
proptest! {
    #[test]
    fn meet_polarity_under_all_pairs(a in any::<Class>(), b in any::<Class>()) {
        let r = meet(a, b);
        prop_assert!(r <= a && r <= b, "meet must be a lower bound");
        prop_assert_eq!(meet(a, b), meet(b, a), "meet must commute");
    }
}

#[immune(
    FrameTranslation,
    witness = proptest::property(meet_polarity_under_all_pairs, cases = 10000),
    rationale = "Lower-bound + commutativity proven by 10k random pairs; \
                 polarity inversion would fail the lower-bound check."
)]
pub fn meet(a: Class, b: Class) -> Class { /* ... */ }
```

What `cargo antigen scan` does:
1. Resolves `meet_polarity_under_all_pairs` to a discovered `#[test]` function.
2. Verifies the function uses `proptest!` (structural check on the macro
   expansion or attribute) and the configured case count >= the antigen's
   declared minimum.
3. Optionally: runs the witness via `cargo test
   meet_polarity_under_all_pairs` and reports pass/fail.
4. Checks `proptest-regressions/` for any saved failures and surfaces them
   as broken-witness alerts.

**Strongest variant — antigen-stdlib ships proptest *strategies* for known
failure-classes.** A generator that produces "hard cases for
frame-translation" (locally-consistent inputs that disagree at
boundaries). Users plug it in:

```rust
proptest! {
    #[test]
    fn meet_witness(
        (a, b) in antigen_stdlib::frame_translation::adversarial_pair::<Class>()
    ) {
        prop_assert!(meet(a, b) <= a);
    }
}
```

Antigen-stdlib OWNS the adversarial strategies; user owns the
property-assertion.

**Gaps.** Proptest is randomized — coverage is statistical, not
exhaustive. A property held by 10000 cases may fail at case 10001.
Antigen complements: the *named* failure-class persists even when the
proptest runs are bounded.

---

## 3. QuickCheck — older property testing

**What it does.** Older Rust port of the Haskell QuickCheck. Less
ergonomic than proptest, weaker shrinking, no persisted regressions, but
still in widespread use. Trait-based generators (`Arbitrary` trait) rather
than proptest's strategy-DSL. Macro:
`#[quickcheck] fn prop(x: i64) -> bool { ... }`.

**Failure-class coverage.** Same shape as proptest but weaker shrinking
means weaker minimal-reproducer guarantees. Coverage of (7), (6), (1), (8)
is the same in principle.

**Integration surface.**
- `#[quickcheck]` attribute marks test functions.
- `Arbitrary` trait is the generator surface.
- No persistent failure storage — re-runs may pass/fail differently.

**Witness-mechanism opportunity.**

```rust
#[quickcheck]
fn meet_lower_bound(a: Class, b: Class) -> bool {
    let r = meet(a, b);
    r <= a && r <= b
}

#[immune(
    FrameTranslation,
    witness = quickcheck::property(meet_lower_bound, tests = 1000),
)]
pub fn meet(a: Class, b: Class) -> Class { /* ... */ }
```

Antigen scan: resolve `meet_lower_bound`, verify `#[quickcheck]`
attribute presence, confirm `Arbitrary` impls exist for parameter types.

**Recommendation.** Treat quickcheck as *equivalent witness shape* to
proptest but document proptest as the preferred mechanism. Don't gate
adoption on which library the user prefers.

**Gaps.** No persisted regressions means antigen can't detect "this
witness has ever failed in CI history" without separate infra. Proptest
is strictly more capable as an immunity witness.

---

## 4. cargo-mutants — mutation testing

**What it does.** cargo-mutants generates *mutants* of the code under
test (replaces `+` with `-`, replaces a function body with `Default::default()`,
swaps `&&` with `||`, returns the wrong value, etc.) and runs the test
suite against each mutant. Outcomes per mutant:
- **caught** — at least one test failed (good)
- **missed** — all tests passed (the mutation is invisible to your tests)
- **timeout** — test took too long
- **unviable** — mutant doesn't compile

A "missed" mutant means the test suite lacks a property that would
distinguish correct from broken code. Annotation surface:
`#[mutants::skip]` (and `#[cfg_attr(test, mutants::skip)]`) — placed on
functions to opt-out. Configured via `.cargo/mutants.toml`.

**Failure-class coverage.**
- **(2) Forgotten-lesson** — mutation testing is *the* answer to
  "are your tests still actually checking what they claim?". A test that
  passed for years but doesn't catch any nontrivial mutation is a
  forgotten-lesson signal.
- **(7) Boundary-violation** — boundary mutants (off-by-one, comparison
  flip) directly probe boundary checks.
- **(8) Optionality-collapse** — mutants that delete branches probe
  whether the conditional structure was load-bearing.

**Integration surface.**
- CLI: `cargo mutants` produces `mutants.out/` with `caught.txt`,
  `missed.txt`, `timeout.txt`, `unviable.txt`. Each line is a mutant
  description with file:line and the mutation applied.
- Annotation: `#[mutants::skip]` (literal string match in attributes —
  search for the substring, not full attribute parse).
- Config: `mutants.toml` for path/regex skip rules.

**Witness-mechanism opportunity.**

```rust
#[immune(
    ForgottenLesson,  // family: signal-loss-through-test-rot
    witness = mutants::no_missed_mutants(target = meet, threshold = 0),
    rationale = "Tests catch every mutation of meet; no test rot has occurred."
)]
pub fn meet(a: Class, b: Class) -> Class { /* ... */ }
```

What `cargo antigen scan` does:
1. Reads `mutants.out/` from the most recent run.
2. Filters mutant entries whose file:line span is inside `meet`.
3. If zero mutants are in `missed.txt` or `caught.txt` count >= total,
   immunity satisfied.
4. If any are missed, antigen reports the *specific* missed mutants as
   evidence of broken antibody.

**Strongest variant — antigen-aware mutants.** Antigen ships its own
mutant generator catalog keyed to failure-classes:
- `FrameTranslation` mutants: swap polarity (replace `<` with `>`, swap
  argument order in commutative-looking calls).
- `BoundaryViolation` mutants: remove `.checked_*` calls, swap `<` with
  `<=`.
- `IncompatibleMerger` mutants: substitute one constructor for another
  in a sibling enum variant.

Then `#[immune(X)]` requires that the antigen-class-specific mutant
catalog for X be 100% caught.

**Gaps.** Mutation testing is slow (build × N mutants). For CI, antigen
needs incremental mode: only re-mutate items whose source has changed
since last `mutants.out/`. cargo-mutants supports `--in-place` and
`--baseline` partially; antigen tooling would need to wrap this.

---

## 5. cargo-careful — sanitizer-enabled test runs

**What it does.** Wrapper that runs `cargo test` with the standard library
rebuilt with extra debug assertions, and integrates with sanitizers
(AddressSanitizer, MemorySanitizer, ThreadSanitizer, LeakSanitizer) on
nightly. Catches UB that miri can't (real concurrency races, real memory
errors at native speed).

**Failure-class coverage.**
- **(7) Boundary-violation** — buffer overruns, OOB indexing.
- **(3) Implicit-coupling** — TSan finds data races (unstated
  cross-thread coupling).
- **(6) Incompatible-merger** — LSan finds leaks where ownership composed
  wrongly.

**Integration surface.**
- CLI: `cargo +nightly careful test`.
- Output: standard cargo-test output plus sanitizer reports (text format
  per sanitizer, file:line + stack).
- No annotation surface; runs against the whole test suite.

**Witness-mechanism opportunity.**

```rust
#[immune(
    DataRaceBoundaryViolation,
    witness = careful::tsan_clean(test = parallel_meet_test),
    rationale = "ThreadSanitizer reports zero races on the parallel meet test."
)]
pub fn parallel_meet(...) { /* ... */ }
```

What scan does:
1. Runs `cargo +nightly careful test parallel_meet_test` with TSan
   enabled (or reads cached output from CI).
2. Parses sanitizer output for race reports referencing the marked item.
3. Zero hits = immunity satisfied.

**Gaps.** Careful runs are slow and require nightly. Mostly relevant to
unsafe/concurrent code; not a general-purpose witness for safe Rust.

---

## 6. Kani — model checking via CBMC

**What it does.** AWS-developed bounded model checker. `#[kani::proof]`
attributes mark verification harnesses; kani uses `kani::any()` for
non-deterministic input, `kani::assume(cond)` to constrain it, and
`assert!()` for properties. Compiles to GOTO via CBMC. Catches:
arithmetic overflow, panics, OOB indexing, assertion failures,
unreachable-but-reachable code, certain unsafe-code preconditions.
Bounded — `#[kani::unwind(N)]` controls loop unrolling.

Other attributes:
- `#[kani::should_panic]` — proves the harness DOES panic (negative
  assertion).
- `#[kani::stub(target, replacement)]` — replaces a function for the
  harness.
- `#[kani::solver(...)]` — picks the SAT solver.

**Failure-class coverage.**
- **(7) Boundary-violation** — kani's strongest suit: arithmetic
  overflow, panics, indexing.
- **(6) Incompatible-merger** — proves composition invariants hold
  symbolically.
- **(1) Frame-translation** — for finite-domain frame translations
  (e.g., `Class` enums with bounded variants), kani can exhaustively
  prove polarity invariants.

**Integration surface.**
- `#[kani::proof]` attribute identifies harnesses.
- CLI: `cargo kani` runs all harnesses; `cargo kani --harness <name>`
  runs one. Output: VERIFICATION SUCCESSFUL or list of failed properties
  with counterexample traces.
- JSON output via `--format json` for tooling integration.

**Witness-mechanism opportunity.**

```rust
#[kani::proof]
#[kani::unwind(8)]
fn meet_polarity_proof() {
    let a: Class = kani::any();
    let b: Class = kani::any();
    let r = meet(a, b);
    assert!(r.le(&a) && r.le(&b));
    assert!(meet(a, b) == meet(b, a));
}

#[immune(
    FrameTranslation,
    witness = kani::proof(meet_polarity_proof),
    rationale = "Symbolic proof: for ALL Class pairs, meet returns a lower bound \
                 and is commutative. Stronger than proptest — exhaustive."
)]
pub fn meet(a: Class, b: Class) -> Class { /* ... */ }
```

What scan does:
1. Resolves `meet_polarity_proof` to an item with `#[kani::proof]`.
2. Verifies the harness exists and the named function is reachable.
3. Optionally: runs `cargo kani --harness meet_polarity_proof --format
   json` and parses VERIFICATION SUCCESSFUL.
4. If `cargo kani` reports a counterexample, antigen surfaces it as a
   *broken proof* witness with the full trace.

**Witness strength ladder.** Antigen treats kani witnesses as STRONGER
than proptest witnesses for the same property because kani is exhaustive
(within unwind bounds) where proptest is statistical. The audit report
distinguishes:
- *bounded-exhaustive proof* (kani, unwind <= bound)
- *statistical witness* (proptest, N cases)
- *unit-test witness* (single case)

**Gaps.** Kani is bounded — loops > unwind limit are not proven, only
checked-up-to-bound. Pure functional code with bounded recursion is
ideal; long-running stateful code is hard.

---

## 7. Prusti — Viper-based deductive verification

**What it does.** Specifies Rust code with `#[requires(...)]` (preconditions),
`#[ensures(...)]` (postconditions), `#[pure]` (logical-mode functions),
`#[trusted]` (assume without proof), `#[invariant(...)]` (loop
invariants), `#[predicate]` (named logical formulas). Translates to
Viper, dispatches to Z3. Stronger than kani for inductive properties; can
verify arbitrary unbounded loops if the invariant is right.

**Failure-class coverage.**
- **(7) Boundary-violation** — preconditions encode trust boundaries
  formally; the verifier proves no caller violates them.
- **(1) Frame-translation** — postconditions encode semantic invariants;
  verified to hold for all valid inputs.
- **(8) Optionality-collapse** — pre/post pairs preserve conditional
  shape across call boundaries.
- **(3) Implicit-coupling** — explicit pre/post make all coupling
  visible to the verifier.

**Integration surface.**
- `#[requires(expr)]`, `#[ensures(expr)]`, `#[pure]`, `#[trusted]`,
  `#[invariant(expr)]`, `#[predicate]` — attribute API.
- CLI: `cargo prusti`. Output: per-function VERIFIED or counterexample.
- Specifications live IN the source as macro-style attributes; visible to
  any reader of the source.

**Witness-mechanism opportunity.**

```rust
#[ensures(result <= a && result <= b)]
#[ensures(forall(|c: Class| c <= a && c <= b ==> c <= result))]
pub fn meet(a: Class, b: Class) -> Class { /* ... */ }

#[immune(
    FrameTranslation,
    witness = prusti::contract(meet),
    rationale = "Prusti-verified post-condition encodes lattice meet axiomatically."
)]
pub fn meet(a: Class, b: Class) -> Class { /* same as above */ }
```

(In practice, the `#[immune]` and the contract sit on the same item.
Antigen reads the contract from the function's `#[ensures]` clauses.)

What scan does:
1. Confirms `#[ensures]` clauses exist on the marked item.
2. Confirms a recent `cargo prusti` run reported VERIFIED for that item
   (cached in `target/prusti/` or re-run on demand).
3. Surfaces failed contracts as broken-witness.

**Gaps.** Prusti is heavy — install requires Viper/JVM toolchain. Slow.
Verification can fail spuriously on idioms the verifier doesn't model
(complex closures, certain async patterns). Not a casual-adoption tool.

---

## 8. Creusot — WhyML-based deductive verification

**What it does.** Translates Rust to Coma (Why3 IR) and dispatches via
Why3 to Z3/Alt-Ergo/CVC4/etc. Annotations:
`#[requires(...)]`, `#[ensures(...)]`, `#[invariant(...)]`,
`#[variant(...)]` (termination measure), `#[trusted]`, `#[predicate]`,
`#[logic]` (mathematical functions in spec mode), `#[ghost]` (proof-only
state). Stronger logic surface than prusti for some patterns
(quantifiers, ghost state).

**Failure-class coverage.** Same as Prusti — pre/post/invariant cover
(7), (1), (8), (3). `#[variant]` adds a coverage angle:
**termination-as-failure-class** (a function that should terminate but
might not).

**Integration surface.**
- `#[requires]`, `#[ensures]`, `#[invariant]`, `#[variant]`,
  `#[ghost]`, `#[predicate]`, `#[logic]`, `#[trusted]`.
- CLI: `cargo creusot`. Output: per-VC PROVED or unproved.
- Why3 IDE for interactive proof of stuck VCs.

**Witness-mechanism opportunity.**

```rust
#[creusot::ensures(result@ <= a@ && result@ <= b@)]
pub fn meet(a: Class, b: Class) -> Class { /* ... */ }

#[immune(
    FrameTranslation,
    witness = creusot::contract(meet),
)]
pub fn meet(a: Class, b: Class) -> Class { /* ... */ }
```

Same shape as prusti integration. Antigen treats creusot and prusti as
*alternative formal-verification backends* for the same witness slot.

**Gaps.** Same as prusti — heavy install, slow, partial coverage of Rust
idioms. Active development; some idioms unsupported.

---

## 9. Verus — Rust-native verification

**What it does.** Uses Z3 directly with custom Rust-syntax extensions for
`spec`, `proof`, and `exec` modes. `proof fn` — proof-only functions.
`spec fn` — pure logical functions. `exec fn` — runtime code (default).
`requires`, `ensures`, `decreases` (termination), `invariant` (loop). All
syntax inside `verus! { ... }` macro blocks. Tight Rust integration; uses
Rust's type system for memory/aliasing reasoning.

**Failure-class coverage.** Same family as prusti/creusot — but verus's
strength is *low-level systems code*: pointer reasoning, raw memory,
custom allocators. Covers (7), (3), (1) for unsafe code more
naturally than prusti/creusot.

**Integration surface.**
- `verus! { ... }` macro wraps annotated code.
- `requires`, `ensures`, `decreases`, `invariant`, `proof fn`, `spec fn`,
  `assert(cond) by { /* proof */ }`.
- CLI: `verus <file>` (not yet a clean cargo subcommand at time of
  writing; check current state).

**Witness-mechanism opportunity.**

```rust
verus! {
    fn meet(a: Class, b: Class) -> (r: Class)
        ensures r.le(a), r.le(b),
                forall|c: Class| c.le(a) && c.le(b) ==> c.le(r),
    {
        // implementation
    }
}

#[immune(
    FrameTranslation,
    witness = verus::contract(meet),
)]
```

Antigen scans for the verus! macro presence and the `ensures` clause on
the marked item.

**Gaps.** Verus syntax is its own dialect inside the macro; less
familiar to Rust developers. Tooling less polished than kani/prusti at
time of writing. Strongest for systems code; overkill for ordinary
business logic.

---

## 10. Miri — interpreter for UB detection

**What it does.** Rust's MIR-level interpreter. Runs tests under
emulation, detects: out-of-bounds memory access, use-after-free,
uninitialized data use, alignment violations, invalid type values,
data races, leaks, intrinsic precondition violations, and aliasing
violations (Stacked Borrows / Tree Borrows). Slow (~50× native), but
catches things sanitizers miss.

**Failure-class coverage.**
- **(7) Boundary-violation** — OOB, alignment, bad types.
- **(3) Implicit-coupling** — data races, aliasing violations.
- **(6) Incompatible-merger** — leaks (compositional ownership errors).
- **(2) Forgotten-lesson** — silent UB in unsafe code that "worked" for
  years; miri surfaces as immediate error.

**Integration surface.**
- CLI: `cargo +nightly miri test`, `cargo +nightly miri run`.
- `MIRIFLAGS` env var configures (e.g., `-Zmiri-tree-borrows`).
- No annotation surface; affects whole test suite.

**Witness-mechanism opportunity.**

```rust
#[immune(
    UnsafeBoundaryViolation,
    witness = miri::clean(tests = ["unsafe_module::*"]),
    rationale = "All unsafe-module tests pass under miri with default flags."
)]
mod unsafe_module { /* ... */ }
```

What scan does:
1. Reads cached `cargo miri test` output (or runs it).
2. Filters test results to those covering the marked scope.
3. Zero miri errors = immunity satisfied.

**Gaps.** Miri only checks code that runs under tests. Doesn't help if
the unsafe code's failure mode is never exercised. Antigen-stdlib could
ship adversarial-input strategies for known unsafe failure modes
(buffer-edge, zero-sized types, integer-overflow-in-pointer-math) and
gate immunity on miri-clean execution of those.

---

## 11. rustc deprecation system

**What it does.** Built-in `#[deprecated(since = "...", note = "...")]`
attribute. Calls to deprecated items emit `deprecated` warning at use
sites, propagating across crate boundaries. The warning is structurally
attached to the item, not to specific call sites — but each call site
gets the warning during compilation.

**Failure-class coverage.**
- **(2) Forgotten-lesson** — deprecation IS named failure-class memory:
  "this thing is wrong, don't use it." But the *reason* is in a free-text
  `note` — not structured.
- **(4) Stale-context** — a deprecation note declares "this is stale,
  use X instead."

**Integration surface.**
- `#[deprecated(since = "ver", note = "free-text")]`.
- Compiler emits `deprecated` lint at use sites; suppressible with
  `#[allow(deprecated)]`.
- The warning carries the `note` text but no structured machine-readable
  reason or replacement reference.

**Witness-mechanism opportunity.** Antigen extends deprecation with
*named* failure-classes:

```rust
#[deprecated(since = "0.5", note = "Use meet_v2 — meet had FrameTranslation bug")]
#[antigen_deprecated(reason = FrameTranslation, replacement = meet_v2)]
pub fn meet(a: Class, b: Class) -> Class { /* ... */ }
```

`cargo antigen scan` reads `#[antigen_deprecated]` and produces a
report: "callers of `meet` are exposed to FrameTranslation; replacement
`meet_v2` declares `#[immune(FrameTranslation)]`. Migration is a
mechanical rename."

This turns deprecation from "free-text warning" into "structured
failure-class transition with verifiable replacement."

**Gaps.** Standard deprecation has no machine-readable reason or
replacement pointer. Antigen has to add its own attribute alongside.

---

## 12. RustSec / cargo-audit — supply-chain advisory database

**What it does.** RustSec maintains an advisory database of known
vulnerabilities in published crates, identified by RUSTSEC-YYYY-NNNN IDs
(and cross-referenced to CVE-YYYY-NNNNN where applicable). Each advisory
has a YAML record with affected versions, patched versions, severity,
description, and references. `cargo audit` checks `Cargo.lock` against
the DB.

**Failure-class coverage.**
- **(2) Forgotten-lesson** — advisories ARE failure-class memory at the
  ecosystem level. Each RUSTSEC ID is a named, persistent record of
  "this thing went wrong, here's the fix."
- **(4) Stale-context** — using an old version of a vulnerable crate is
  the canonical stale-context bug.

**Integration surface.**
- Advisory DB at github.com/rustsec/advisory-db, structured YAML.
- `cargo audit` CLI; JSON output for CI.
- Each advisory has a stable RUSTSEC ID — perfect cross-reference target
  for antigen declarations.

**Witness-mechanism opportunity.** Antigen makes the link between an
advisory and a structural pattern:

```rust
#[antigen(
    name = "TimeOverflow_RUSTSEC_2020_0071",
    family = BoundaryViolation,
    fingerprint = "chrono::DateTime arithmetic with untrusted input",
    references = ["https://rustsec.org/advisories/RUSTSEC-2020-0071"],
)]
pub struct TimeOverflowAntigen;

#[presents(TimeOverflowAntigen)]
pub fn parse_user_timestamp(s: &str) -> DateTime<Utc> { /* ... */ }
```

Now `cargo antigen scan` knows: this function is vulnerable to the
specific failure pattern recorded in RUSTSEC-2020-0071. Auditors can
trace from antigen marker to advisory and back.

**Antigen-stdlib opportunity.** Auto-generate antigens from the RustSec
DB: for every advisory, an antigen declaration with the RUSTSEC ID as
the name and the affected crate's API as the structural fingerprint.

**Gaps.** RustSec covers *known* vulnerabilities in *published* crates.
Antigen extends this to in-house code and to failure-classes that
haven't yet manifested as CVEs.

---

## 13. cargo-deny — supply-chain policy

**What it does.** Configurable checks across four axes:
- **advisories** — like cargo-audit, against RustSec DB.
- **licenses** — allow/deny lists; SPDX-aware.
- **bans** — forbid specific crates or duplicate versions.
- **sources** — restrict to trusted registries / git URLs.

Configured via `deny.toml`. Enforced in CI.

**Failure-class coverage.**
- **(2) Forgotten-lesson** — banning a crate that "we know causes X" IS
  named failure-class memory at the dependency level.
- **(3) Implicit-coupling** — ban duplicate versions to prevent
  cross-version coupling bugs.

**Integration surface.**
- `deny.toml` config file. Public, parseable.
- CLI: `cargo deny check`. JSON output for CI.

**Witness-mechanism opportunity.**

```toml
# deny.toml
[bans]
deny = [
    { name = "old-crypto-lib", reason = "antigen:WeakRandomness" },
]
```

Antigen reads `deny.toml`, recognizes `reason = "antigen:X"` markers,
and treats the ban as a witness for `WeakRandomness` immunity at the
*manifest* level. `cargo antigen scan` then knows: this whole crate is
immune to `WeakRandomness` because the dependency that would have
introduced it is banned.

**Gaps.** cargo-deny is a manifest-level tool; it can't reason about
patterns inside source code. Pairs with antigen's site-level analysis,
not a substitute.

---

## 14. cargo-bisect-rustc — regression hunting

**What it does.** Binary-searches Rust toolchain versions to identify
which compiler change introduced a regression. Output: a specific
nightly version range and (with `--regress=ice`) the offending commit.

**Failure-class coverage.**
- **(4) Stale-context** — when a regression appears, bisect locates the
  context shift. Doesn't prevent stale-context, but diagnoses it.

**Integration surface.**
- CLI: `cargo bisect-rustc`. Output: a versioned cause.
- Not annotation-based; more of a diagnostic tool.

**Witness-mechanism opportunity.** Limited. cargo-bisect-rustc is
diagnostic, not preventive. Antigen could record "this antigen
manifested at toolchain rust X.Y; immunity validated against Z.W" — a
*temporal* witness — but this is fringe.

**Gaps.** Doesn't prevent or witness anything; just diagnoses.

---

## 15. cargo-fuzz — coverage-guided fuzzing

**What it does.** Wraps libFuzzer (and via plugins: AFL++, honggfuzz).
`fuzz_target!(|input: &[u8]| { ... })` macro defines fuzz targets.
Coverage-guided: mutates inputs to maximize new code-path discovery.
Persistent corpus on disk. Crashes saved as reproducer files.

**Failure-class coverage.**
- **(7) Boundary-violation** — fuzzer is *the* tool for finding panics,
  OOB, integer overflow, parser failures on adversarial input.
- **(8) Optionality-collapse** — branch-coverage-driven fuzzing
  exercises the conditional structure thoroughly.

**Integration surface.**
- `fuzz_target!` macro inside `fuzz/fuzz_targets/<name>.rs`.
- CLI: `cargo fuzz run <name>`.
- Corpus and crash files in `fuzz/corpus/` and `fuzz/artifacts/`.

**Witness-mechanism opportunity.**

```rust
// fuzz/fuzz_targets/parse_user_input.rs
fuzz_target!(|data: &[u8]| {
    let _ = my_crate::parse_user_input(data);  // must not panic
});
```

```rust
#[immune(
    BoundaryViolation,
    witness = fuzz::target("parse_user_input", min_runs = 10_000_000, no_crashes = true),
    rationale = "10M+ fuzz inputs, no panics or crashes."
)]
pub fn parse_user_input(data: &[u8]) -> Result<Parsed, ParseError> { /* ... */ }
```

What scan does:
1. Locates `fuzz/fuzz_targets/parse_user_input.rs`.
2. Reads `fuzz/artifacts/parse_user_input/` — if any crash files exist,
   immunity is broken.
3. Optionally: confirms recent fuzz run hit minimum runs (from CI logs
   or local cache).

**Gaps.** Fuzzing is statistical, like proptest, but operates on raw
byte slices rather than typed inputs. Best for parsers, deserializers,
and other byte-eaters. Less natural for typed business logic (proptest
better there).

---

## 16. cargo-tarpaulin / cargo-llvm-cov — coverage

**What it does.** Line/branch coverage of test runs. tarpaulin uses
ptrace; llvm-cov uses LLVM source-based coverage instrumentation.
Output: HTML reports, lcov, JSON. Identifies code reached vs not reached
by the test suite.

**Failure-class coverage.**
- **(2) Forgotten-lesson** (negative) — coverage tells you which code
  has *no* test exercise, where forgotten-lesson is most likely.

**Integration surface.**
- CLI: `cargo tarpaulin`, `cargo llvm-cov`. JSON/lcov output.
- No annotation surface; whole-program.

**Witness-mechanism opportunity.** Coverage is a *prerequisite* for
other witnesses to be meaningful, not itself an antibody. Antigen can
assert: "this immunity claim requires the witness function to actually
execute the marked item — confirmed by coverage." That is, the
witness-validation step in `cargo antigen scan` includes:

1. Run the witness (proptest / kani / etc.).
2. Run with coverage instrumentation.
3. Verify the marked item is hit.
4. If not hit, the witness is non-binding (it didn't exercise the code).

This catches the autoimmunity failure: a `#[immune]` declaration whose
witness function never actually executes the immune item.

**Gaps.** Coverage doesn't witness behavioral properties — only
exercise. Always a prerequisite, never a sole witness.

---

## 17. rustdoc tests (doctests)

**What it does.** Code examples in `///` comments are extracted and
compiled+run as tests. Attributes: `ignore`, `no_run`, `should_panic`,
`compile_fail`, `edition2015|2018|2021|2024`, `standalone_crate`,
`ignore-<target>`. Discovered by `cargo test --doc`.

**Failure-class coverage.**
- **(2) Forgotten-lesson** — `compile_fail` doctests are *the canonical
  documented failure-class*: "this code, which a user might write, MUST
  NOT compile." Type-level invariants documented in usage examples.
- **(1) Frame-translation** — usage-example doctests demonstrate the
  intended frame; if the example breaks, the frame has shifted.

**Integration surface.**
- Triple-backtick blocks in doc comments.
- Per-block attribute strings in the language tag (e.g., `compile_fail`).
- `cargo test --doc` runs them.

**Witness-mechanism opportunity.**

```rust
/// Compile-time-checked: cannot construct a Class out of a non-Class enum value.
///
/// ```compile_fail
/// let _: my_crate::Class = my_crate::Class::from_raw(42);  // private constructor
/// ```
///
/// ```
/// let c = my_crate::Class::Strict;
/// assert!(c <= my_crate::Class::Strict);
/// ```
#[immune(
    BoundaryViolation,
    witness = doctest::compile_fail(crate_root, "my_crate::Class::from_raw"),
    rationale = "Doctest verifies private constructor cannot be called from outside."
)]
pub enum Class { /* ... */ }
```

What scan does:
1. Parses doctests on the marked item.
2. Identifies the `compile_fail` block matching the witness pattern.
3. Confirms `cargo test --doc` includes it and reports it as expected-fail.

**Gaps.** Doctests are slow (each may compile a fresh crate). Best for
compile-time invariants and usage-shape checks; not a behavioral
witness.

---

## 18. Phantom-type / typestate witnesses (no external tool)

**What it does.** Pure-Rust pattern: encode invariants in the type
system. `Sealed<T>` with private constructor; `PhantomData<State>` for
state-machine encoding; `Send`/`Sync` auto-trait reasoning;
sealed traits via private supertraits. The compiler IS the verifier.

**Failure-class coverage.**
- **(7) Boundary-violation** — typestate makes invalid sequences
  uncallable.
- **(1) Frame-translation** — distinct types for distinct frames; mixing
  is a type error.
- **(5) Premature-abstraction** — explicit type-level distinctions
  surface where an abstraction was over-eager.
- **(8) Optionality-collapse** — distinct types for the conditional
  branches preserve the structural shape.

**Integration surface.**
- `PhantomData<T>`, sealed traits, private constructors, marker types.
- No external tool; the compiler IS the witness validator.

**Witness-mechanism opportunity.**

```rust
pub struct PolarityProof<T> { _marker: PhantomData<T> }

impl PolarityProof<FrameTranslation> {
    /// Construction proves polarity correctness — only callable when
    /// the lattice direction matches the meet implementation.
    pub fn established_by_construction() -> Self { Self { _marker: PhantomData } }
}

#[immune(
    FrameTranslation,
    witness = phantom::proof(PolarityProof::<FrameTranslation>::established_by_construction),
)]
pub fn meet(a: Class, b: Class) -> Class { /* ... */ }
```

What scan does:
1. Resolves the witness path to a function/method.
2. Confirms it returns a `PolarityProof<FrameTranslation>` (or similar
   antigen-named witness type).
3. Confirms the type's constructor is private (sealed) — the only way to
   construct it is the named function.
4. Type-checking guarantees no other path produces the witness.

This is the strongest possible witness: compile-time-impossible-to-violate.
No runtime check needed.

**Gaps.** Designing phantom-type witnesses is hard. Many failure-classes
don't admit type-level encoding. Best for lattice-shaped or
state-machine-shaped invariants. Antigen-stdlib should ship a starter
catalog of phantom-type witness templates.

---

## 19. cargo-semver-checks — API compatibility

**What it does.** Compares two versions of a crate's public API and
flags semver violations (removed item, changed signature, narrowed
trait bound, etc.). Catches accidental breaking changes.

**Failure-class coverage.**
- **(3) Implicit-coupling** — silent API breaks that downstream consumers
  rely on.
- **(4) Stale-context** — public-API drift versus declared semver tier.

**Integration surface.**
- CLI: `cargo semver-checks check-release`. JSON output.
- Compares git refs or published versions.

**Witness-mechanism opportunity.**

```rust
#[antigen(
    name = "SemverDrift",
    family = ImplicitCoupling,
    fingerprint = "public API change without major version bump",
)]
pub struct SemverDriftAntigen;

#[immune(
    SemverDrift,
    witness = semver_checks::no_breakage(against = "v1.0.0"),
)]
pub mod stable_api { /* ... */ }
```

Scan runs `cargo semver-checks` against the named baseline; reports
breakage as broken witness.

**Gaps.** API-level only; doesn't catch *behavioral* breakage with
unchanged signatures.

---

## 20. cargo-nextest — test runner with structure

**What it does.** Faster, more configurable test runner. Per-test
isolation (own process), structured output, retries, filtering by
runtime profile. JSON output via `--message-format`.

**Failure-class coverage.** No direct coverage; it's an *infrastructure*
piece. Relevant because antigen's witness-running step benefits from
nextest's per-test isolation and structured reporting.

**Integration surface.**
- CLI: `cargo nextest run`. JSON output for tooling.
- Structured filtering: `cargo nextest run -E 'test(antigen_witness::)'`.

**Witness-mechanism opportunity.** Antigen's CI integration uses
nextest's filter expressions to run *only* witness functions:
```bash
cargo nextest run -E 'test(witness::*) + test(*_proptest)'
```
And antigen ingests the JSON output to validate every named witness
ran and passed.

**Gaps.** Pure infrastructure. Doesn't witness anything itself.

---

## 21. Flux — liquid types for Rust (compile-time refinement witnesses)

**What it does.** Flux (Lehmann, Geller, Vazou, Jhala; PLDI 2023) extends
Rust's type checker with *liquid types* — refinement predicates that are
discharged by an SMT solver (Z3) at `cargo check` time, not at a separate
verifier invocation. Specifications attach to function signatures and type
definitions via `#[flux::sig(...)]`:

```rust
#[flux::sig(fn(x: i32{v: v > 0}, y: i32{v: v > 0}) -> i32{v: v > 0})]
fn add_positive(x: i32, y: i32) -> i32 { x + y }
```

If the refinement predicate is violated, the build fails at `cargo check`
with a type error. No separate `cargo flux` step required in the happy path;
the annotation IS the proof obligation, discharged automatically.

**Failure-class coverage.**
- **(7) Boundary-violation** — precondition refinements (`{v: v >= 0}`,
  `{v: v < len}`) encode trust boundaries as types; violation is a type
  error, not a runtime panic.
- **(1) Frame-translation** — postcondition refinements encode semantic
  invariants; any implementation that violates the invariant fails at
  `cargo check`.
- **(8) Optionality-collapse** — conditional refinements preserve
  structural conditional shape through call boundaries.
- **(6) Incompatible-merger** — composition properties expressible as
  SMT-dischargeable predicates can encode compatibility invariants.

**Integration surface.**
- `#[flux::sig(...)]` on functions and methods — the primary annotation.
- `#[flux::refined_by(...)]` on type definitions — refinement predicates
  for struct/enum fields.
- `#[flux::trusted]` — mark a function as trusted without specifying its
  spec (for FFI boundaries or intentional spec holes).
- Build integration: Flux plugs into `rustc` via a custom driver; `cargo
  check` invokes it transparently when Flux is installed.

**Witness-mechanism opportunity.**

Flux is categorically distinct from kani, prusti, creusot, and verus:
it is the only Rust verification tool that discharges at `cargo check`
rather than at a separate verifier invocation. This means a Flux-backed
immunity claim is the only **compile-time-discharged refinement witness**
in antigen's inventory (phantom-types are compile-time but are type-
construction witnesses rather than refinement-predicate witnesses).

Integration pattern:

```rust
#[flux::sig(fn(a: Class, b: Class) -> Class{v: v <= a && v <= b})]
pub fn meet(a: Class, b: Class) -> Class { /* ... */ }

#[immune(
    FrameTranslation,
    witness = flux::sig(meet),
    rationale = "Flux type signature encodes meet's lower-bound invariant; \
                 discharged at cargo check, not at runtime."
)]
pub fn meet(a: Class, b: Class) -> Class { /* same as above */ }
```

What `cargo antigen scan` does:
1. Detects `#[flux::sig]` on the marked item (attribute presence check;
   shallow, no Flux installation required for detection).
2. Confirms the crate builds cleanly (implies Flux discharged the spec
   successfully — if the spec is violated, the build would have failed
   before antigen runs).
3. Status: `declared (compile-time refinement — verified by cargo check
   if build succeeded)`.

This is the *lightest possible audit path* for a formal-verification
witness: the proof obligation is always discharged if the code compiles.
No separate tool invocation, no cached output parsing, no CI step beyond
the normal build.

**Gaps.** Flux requires nightly Rust (at time of writing) and a separate
Flux compiler installation. The refinement predicate language is
value-level (integer arithmetic, ordering, set membership) — it cannot
express behavioral properties beyond arithmetic/structural constraints.
For complex behavioral properties (liveness, termination, pointer
aliasing), kani/prusti/verus are more appropriate.

---

## Composition matrix

Failure-class coverage by tool. `D` = direct/strong coverage,
`P` = partial coverage (slice of the class), `–` = not applicable.

| Failure class                  | clippy | proptest | quickcheck | mutants | careful | kani | prusti | creusot | verus | flux | miri | deprec. | rustsec | deny | fuzz | cov | doctest | phantom |
|--------------------------------|:------:|:--------:|:----------:|:-------:|:-------:|:----:|:------:|:-------:|:-----:|:----:|:----:|:-------:|:-------:|:----:|:----:|:---:|:-------:|:-------:|
| 1 Frame-translation            |   P    |    P     |     P      |    P    |    –    |  P   |   D    |    D    |   D   |  D   |  –   |    P    |    –    |  –   |  –   |  –  |    P    |    D    |
| 2 Forgotten-lesson             |   P    |    P     |     P      |    D    |    P    |  P   |   P    |    P    |   P   |  P   |  P   |    D    |    D    |  D   |  P   |  P  |    D    |    P    |
| 3 Implicit-coupling            |   P    |    P     |     P      |    P    |    D    |  P   |   D    |    D    |   D   |  P   |  D   |    –    |    –    |  P   |  –   |  –  |    –    |    P    |
| 4 Stale-context                |   –    |    –     |     –      |    –    |    –    |  –   |   –    |    –    |   –   |  –   |  –   |    P    |    D    |  P   |  –   |  –  |    –    |    –    |
| 5 Premature-abstraction        |   –    |    –     |     –      |    P    |    –    |  –   |   –    |    –    |   –   |  –   |  –   |    –    |    –    |  –   |  –   |  –  |    –    |    P    |
| 6 Incompatible-merger          |   P    |    D     |     P      |    P    |    P    |  D   |   D    |    D    |   D   |  P   |  P   |    –    |    –    |  P   |  –   |  –  |    –    |    P    |
| 7 Boundary-violation           |   D    |    D     |     D      |    D    |    D    |  D   |   D    |    D    |   D   |  D   |  D   |    –    |    P    |  –   |  D   |  –  |    P    |    D    |
| 8 Optionality-collapse         |   –    |    P     |     P      |    P    |    –    |  P   |   D    |    D    |   D   |  D   |  –   |    –    |    –    |  –   |  P   |  –  |    –    |    D    |

Reading the matrix:

- **Boundary-violation (7)** is heavily covered — most tools attack it.
  Antigen's value here is *naming* and *site-specificity*: which bound,
  why it matters, what witness suffices.
- **Stale-context (4)** is barely covered. Only deprecation, semver
  checks, and dependency tools touch it. **Antigen has a primitive
  responsibility here.**
- **Premature-abstraction (5)** has almost no ecosystem coverage.
  **Antigen-native.**
- **Forgotten-lesson (2)** has many partial coverers (each tool encodes
  a kind of memory) but no tool *names* the lesson structurally.
  **Antigen IS the naming layer.**
- **Frame-translation (1)** is best covered by formal verification
  (prusti/creusot/verus) and phantom types — heavyweight. Antigen needs
  a lightweight default for the common case.

---

## Gaps requiring antigen-native primitives

These are the failure-classes / capabilities the ecosystem does NOT cover
adequately. Antigen has to provide them itself:

### Gap 1. Named-failure-class memory across crate boundaries

No existing tool carries a stable, named failure-class identifier that
propagates from declaration to consumer. Deprecation has free-text
`note`. RustSec has IDs but is supply-chain-only. Clippy lints are
global, not site-specific.

**Antigen primitive:** the `#[antigen(name = "...", family = ...)]`
declaration with a stable, cross-crate identifier. The antigen library
catalog is the missing layer.

### Gap 2. Site-specific vulnerability marking

`#[presents(X)]` on a specific function is something no existing tool
provides. Lints apply globally. Tests are independent. Verification
harnesses cover one item at a time but don't *broadcast* to other items
in the same family.

**Antigen primitive:** `#[presents(X)]` and `cargo antigen scan` walking
the call graph and structural family.

### Gap 3. Structural inheritance of vulnerability and immunity

`#[descended_from(other_function)]` — propagation through copy-paste,
derivation, structural similarity. No existing tool does this. Clippy
sees no relationship between two unrelated functions that share a
shape. Test coverage doesn't propagate.

**Antigen primitive:** the descended-from edge in the antigen graph,
walked at scan time, with immunity-propagation rules and re-justification
flags when the witness no longer applies cleanly.

### Gap 4. Failure-class taxonomy at first-principles granularity

The 8-class taxonomy is antigen's contribution. No existing tool reasons
in terms of frame-translation, optionality-collapse, premature-abstraction
as named families. Each has to be defined, fingerprinted, and populated
with concrete instances by the antigen-stdlib crate.

**Antigen primitive:** the taxonomy itself, in `antigen-stdlib`, with
concrete antigens populating each family.

### Gap 5. Stale-context primitive

There is no Rust ecosystem tool for "this fact must be re-verified at
this boundary." cargo-semver-checks is the closest, but only for public
API. Internal stale-context (developer's mental model of caller
guarantees) is unaddressed.

**Antigen primitive:** freshness markers (`#[fresh_as_of(commit)]`,
`#[requires_reverification_at_boundary(...)]`). To be designed by the
JBD team — this is novel territory.

### Gap 6. Premature-abstraction primitive

No tool detects "this abstraction was made against limited evidence;
the new use case doesn't fit." Phantom types help if used carefully,
but there's no first-class mechanism for "track when this abstraction
was justified, against what evidence, and require re-justification at
expansion."

**Antigen primitive:** `#[abstraction_evidence(...)]` markers that record
the supporting cases at design time, and `cargo antigen scan` flagging
new uses that don't structurally match the recorded evidence. To be
designed by the JBD team.

### Gap 7. Witness-validation infrastructure

No existing tool runs heterogeneous witnesses (test + proptest + kani +
phantom-type + clippy lint) and aggregates results into a single
immunity report. cargo-nextest is closest for test orchestration but
only handles tests. Antigen needs a witness dispatcher that knows how
to invoke each tool and ingest its output.

**Antigen primitive:** the `cargo antigen scan` witness dispatcher, with
adapters for each integrated tool.

### Gap 8. Cross-tool composition vocabulary

Each tool has its own annotation namespace (`#[test]`, `#[kani::proof]`,
`#[mutants::skip]`, `#[deprecated]`...). There is no shared vocabulary
that says "this kani proof and that proptest both witness immunity
to FrameTranslation." Antigen IS this vocabulary.

**Antigen primitive:** the `witness = <expr>` field syntax plus the
adapter set that maps `clippy::lint(...)`, `kani::proof(...)`,
`proptest::property(...)`, etc., to a uniform "did the witness pass?"
result.

---

## Integration priority recommendations

Order matters for adoption — partner with tools whose users already feel
the failure-class-memory pain, where the integration is mechanically
shallow, and where the partnership produces obvious shared value.

### Tier 1 — Ship in initial release (Phases 2–3)

1. **proptest** — by far the most natural witness substrate. Existing
   users already write properties; antigen names what they're properties
   *of*. Adapter is shallow: parse `proptest!` block presence, run via
   `cargo test`, ingest pass/fail. **Lowest cost, highest immediate
   value.**

2. **`#[test]` (built-in)** — every Rust crate has unit tests. The
   simplest possible witness: name a `#[test]` function. Adapter: trivial
   (cargo test runs it, antigen confirms it exists and passed). **Zero
   adoption barrier.**

3. **clippy (built-in lints, not custom)** — cite clippy's restriction
   lints as witnesses. Adapter: parse `cargo clippy --message-format=
   json`, scope to span. **High value because everyone runs clippy
   already.**

4. **rustdoc compile_fail tests** — as type-system witness for
   constructor-restriction antigens. Adapter: parse doc comments, ingest
   `cargo test --doc` results. **Cheap to integrate; valuable for
   API-shape antigens.**

### Tier 2 — Phase 4 (community-library era)

5. **kani** — the strongest practical witness for finite-domain
   antigens (enum lattices, bounded state machines). Adapter: parse
   `cargo kani --format json`. Worth the install friction for
   high-stakes antigens. **Bring in once antigen has uptake; flagship
   "we have formal-verification witnesses" feature.**

6. **cargo-mutants** — an unusual but powerful witness shape: "tests
   catch all mutations of this function" is exactly the
   forgotten-lesson antibody. Adapter: parse `mutants.out/`. **Pairs
   well with proptest; together they cover a lot of (2) and (7).**

7. **dylint custom lints** — for project-specific antigens that need
   full type information. Adapter: invoke `cargo dylint`, parse JSON
   diagnostics. **Power-user feature; ships with antigen-stdlib's
   custom lints first.**

### Tier 3 — Phase 5+ (broader composition)

8. **miri + cargo-careful** — for unsafe-code antigens. Pairs with
   antigen-stdlib's unsafe-pattern catalog. **Valuable but narrow
   audience.**

9. **cargo-fuzz** — for parser/deserializer antigens. Adapter checks
   `fuzz/artifacts/` for crashes and CI logs for run counts. **High
   value for the security-conscious user; modest install cost.**

10. **Flux (liquid types)** — uniquely lightweight among formal-verification
    witnesses: the spec is discharged at `cargo check`, so antigen's adapter
    needs only to confirm `#[flux::sig]` presence and that the build
    succeeded. No separate tool invocation to parse. The *only* compile-time-
    discharged refinement-predicate witness in the inventory. **Bring in
    alongside or before prusti/verus; the adapter is the simplest of all
    formal-verification witnesses.**

11. **prusti / creusot / verus** — heavy formal verification. Adapter
    pattern is uniform (each emits per-item verified/unverified). Bring
    in *one* (probably verus, given Rust-native syntax) as the formal
    witness backend. **Niche but high-prestige; signals that antigen
    interoperates with the verification frontier.**

12. **RustSec advisory cross-references** — auto-generate antigens
    from advisory DB entries. Adapter: pull RustSec YAML, emit antigen
    declarations. **Connects antigen to the established
    vulnerability-naming ecosystem; broadens adoption story.**

### Tier 4 — Out of scope for v1

- **cargo-bisect-rustc** — diagnostic, not a witness. Skip.
- **cargo-nextest** — infrastructure; wrap if convenient but not a
  witness target.
- **cargo-tarpaulin / cargo-llvm-cov** — meta-witness (the witness
  actually exercised the item). Add as a *prerequisite check*, not as a
  primary witness slot. Phase 5.
- **cargo-semver-checks** — interesting future composition for the
  ImplicitCoupling family. Phase 5+.
- **cargo-deny / cargo-audit** — manifest-level; orthogonal to the
  per-item witness mechanism. Document as "compose at the project
  level; antigen reads `deny.toml` for project-wide claims." Phase 5+.

---

## Summary — the threading

Antigen's positioning rests on three threading observations:

1. **Every existing tool encodes a kind of immunity for a slice of the
   failure space.** None of them name what they're collectively trying
   to do.

2. **The ecosystem already has rich annotation surfaces:**
   `#[test]`, `#[kani::proof]`, `proptest!`, `#[deprecated]`,
   `#[mutants::skip]`, `#[requires]`/`#[ensures]`, `#[flux::sig]`,
   doctest tags. Antigen doesn't add a new test runner or a new verifier
   — it adds a *vocabulary* that points at these existing surfaces.

3. **The witness mechanism IS the composition.** `witness = <X>` is the
   one sentence that does the threading. Every adapter implements the
   same contract: given a witness expression, did the underlying tool
   confirm immunity? Adapters are shallow because they're parsing
   already-public outputs, not duplicating tool logic.

The four-line elevator pitch:

> Antigen names failure-classes. Existing tools verify them. Antigen
> threads the verification through a single annotation
> (`#[immune(X, witness = ...)]`) so the failure-class memory survives
> in the substrate even when the practitioner does not.

Compose, don't compete. The witness mechanism is the threading point.
