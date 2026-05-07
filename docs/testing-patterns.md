# Antigen — Testing Patterns

> When to write tests, what kind of test, and how testing integrates with the
> antigen project's discipline. The goal is *legible* coverage — tests that prove
> what they claim to prove and that fail loudly when they should.

---

## Test categories and when to use each

### Unit tests (in-source `#[cfg(test)] mod tests`)

**For**: small pure functions, parsers, helpers. Anything where the boundary is
function-level and the assertions are about input/output relationships.

**Examples in antigen**:
- `split_top_level_commas` in `antigen/src/scan.rs` — pure string splitter
- `parse_kv` — pure key=value parser
- `is_kebab_case` in `antigen-macros/src/parse.rs` — pure boolean predicate

**Conventions**:
- Place inside the module being tested via `#[cfg(test)] mod tests { ... }`
- One test function per behavior, not per function-being-tested
- Function names describe the property: `split_commas_respects_brackets`,
  `parse_kv_strips_quotes`, `is_kebab_case_rejects_uppercase`

### Integration tests (`tests/` directory)

**For**: end-to-end behavior across multiple modules. Anything where the boundary
is at the public API surface or the compiled-binary level.

**Examples** (planned, expanding as the project grows):
- `tests/scan_finds_declared_antigens.rs` — calls `antigen::scan::scan_workspace`
  on a fixture directory and verifies expected declarations appear
- `tests/cargo_antigen_scan_integration.rs` — invokes `cargo run --bin
  cargo-antigen` as a subprocess against a fixture project, verifies output
  format
- `tests/macro_validation.rs` — TryBuild-based test for proc-macro error messages
  (verifies `#[antigen]` rejects bad arguments with helpful errors)

**Conventions**:
- One file per integration scenario
- File name describes the scenario: `<surface>_<expected_behavior>.rs`
- Fixtures under `tests/fixtures/` — small Rust projects with known antigen
  declarations
- Use `tempfile` or in-tree fixtures depending on what's appropriate

### Property tests (`proptest!`)

**For**: structural properties that should hold across all inputs. Especially
useful for parsers, fingerprint matchers, and graph operations.

**Examples** (planned):
- Fingerprint grammar: any well-formed fingerprint string round-trips through
  parse → emit → parse with structural equality
- Scan invariants: any well-formed Rust file produces a `ScanReport` with
  `total_declarations() == antigens.len() + presentations.len() + immunities.len()`
- Witness validation: any function name that resolves to a real `#[test]`
  passes the witness presence check

**Conventions**:
- `proptest!` blocks live alongside unit tests when the property is local
- For larger property tests, dedicate a `tests/<property>_proptest.rs` file
- Failing seeds are committed to `proptest-regressions/` and treated as
  permanent regression tests

### Doctests (in `///` doc comments)

**For**: API documentation that's also a test. Especially valuable when the
documentation IS the contract — readers expect the example to work.

**Antigen-specific note**: macro doc-tests are typically `ignore`d because they
reference user-defined types that don't exist in the macro crate. This is the
standard pattern for `serde`, `serde_derive`, and similar projects.

**When to use ignore vs run**:
- `ignore` → the example demonstrates user-side usage; running it would require
  a non-existent crate
- (no marker) → the example is self-contained and should compile + run as part
  of `cargo test --doc`

### TryBuild tests (compile-fail and pass-test)

**For**: proc-macro error messages and compile-time invariants. The critical
tool for ensuring that misuse of `#[antigen]` (e.g., missing `name`, wrong
attribute target) produces helpful diagnostics.

**Examples** (planned for sweep A2):
- `tests/ui/antigen_missing_name.rs` — a file with `#[antigen(fingerprint = "x")]`
  that should fail compilation; expected error message stored in
  `tests/ui/antigen_missing_name.stderr`
- Similar for: missing fingerprint, non-kebab-case name, applied to non-unit
  struct, witness missing on `#[immune]`

**Conventions**:
- Use the `trybuild` crate
- One test file per misuse pattern
- `.stderr` files committed for diff-against-expected matching

---

## The failing-as-passing pattern

A test where the expected outcome is **failure** with a specific structure. The
test passes if-and-only-if the failure mode is exhibited as expected.

**When to use**:
- Documenting a known limitation that should NEVER be fixed silently
- Proving a misconception was MEANINGFULLY wrong (per `case-study-determinism-class.md`'s
  Framing-N pattern)
- Encoding adversarial expectations: "this antigen MUST detect this site, otherwise
  the antigen's fingerprint is broken"

**Example pattern**:

```rust
#[test]
fn polarity_inverted_class_meet_must_detect_min_polarity() {
    // This test PASSES when the antigen correctly identifies meet=min as the
    // failure pattern. It FAILS as a regression if a future change makes the
    // antigen think meet=min is acceptable.

    let report = scan_workspace_string(SAMPLE_WITH_MEET_MIN_BUG);
    let unaddressed = report.unaddressed_presentations();

    assert!(
        !unaddressed.is_empty(),
        "polarity-inverted-class-meet antigen MUST detect a meet=min impl. \
         If this assertion fails, the antigen's fingerprint has lost coverage \
         of its core failure pattern. See case-study-determinism-class.md."
    );
}
```

The assertion `!unaddressed.is_empty()` is the failing-as-passing form: the
test passes when *something is wrong with the input* (the antigen detects a
problem), and fails when nothing is wrong (the antigen lost its detection).

**Where this lives**:
- antigen-stdlib's per-antigen test files include failing-as-passing tests for
  each antigen's core detection patterns
- Adversarial-driven test fixtures (antigen team's adversarial role authors them)

---

## Test fixture conventions

Test fixtures (small Rust projects used as scan/audit targets) live under:

```
tests/fixtures/
├── basic_antigen_declared/      ← single antigen, single presents, single immune
│   ├── Cargo.toml
│   └── src/lib.rs
├── presents_without_immune/     ← unaddressed presentation (scan should flag)
│   ├── Cargo.toml
│   └── src/lib.rs
├── descended_inheritance/       ← #[descended_from] propagation test (sweep A4)
└── cross_crate_antigen/         ← consuming antigens from a published crate
```

**Fixture conventions**:
- Each fixture is a complete, compilable Rust project
- Has its own `Cargo.toml` so `cargo check` works on it standalone
- Source files are minimal — only the antigen patterns being tested
- README.md in each fixture explains the test's purpose

**Fixture invocation**:
- Integration tests in `tests/` use `Path::new("tests/fixtures/<name>")` to
  point at fixtures
- The `scan_workspace` function is given the fixture root
- Expected outputs are compared against fixture-specific expected results

---

## Coverage philosophy

The project does NOT aim for 100% line coverage as a metric. We aim for:

1. **Every public API function has at least one test** — proving the contract
2. **Every error path has a test** — proving the error handling does what the docs claim
3. **Every property the docs assert has a proptest** — per the
   `proptest-locks-the-narrow-truth` discipline (inherited from tambear DEC-022)
4. **Every named failure mode has a failing-as-passing test** — proving the
   detection works AND doesn't silently regress

Coverage tools (`cargo-tarpaulin`, `cargo-llvm-cov`) are useful diagnostics but
not gates. A test that exercises code without asserting anything meaningful is
worse than no test (false confidence).

---

## CI integration

Per `.github/workflows/ci.yml`:

| Check | When | Strictness |
|-------|------|-----------|
| `cargo check` | Every push, every PR | Hard fail |
| `cargo test --workspace` | Every push (3 OS × 2 toolchains) | Hard fail |
| `cargo fmt --check` | Every push | Hard fail |
| `cargo clippy -- -D warnings` | Every push | Hard fail (with pedantic) |
| `cargo doc --no-deps` (RUSTDOCFLAGS=-D warnings) | Every push | Hard fail |
| MSRV check (`cargo check` on Rust 1.75) | Every push | Hard fail |

Tests must pass on stable AND beta. MSRV (`1.75`) is checked separately to catch
unintentional dependence on newer features.

---

## When NOT to write a test

- For **truly trivial code** (one-line getters, struct constructors that just
  store fields) — the cost-benefit is wrong; the test adds noise without value
- For **third-party functionality** — don't test that `walkdir` walks
  directories or that `serde_json` serializes JSON
- For **non-deterministic behavior** that resists reasonable assertion (UI
  events, time-dependent output) — handle these via mocking or skip them

The default is: **write the test that proves the behavior the docs claim**. If
the docs don't claim a behavior, either the test is unnecessary or the docs
need to be written first (per `proptest-locks-the-narrow-truth`).

---

## Witness-validation testing (Sweep A2-A3)

Specific to antigen: the `cargo antigen audit` command needs to validate that
witnesses resolve to real test/proptest functions. Testing this requires:

1. **Fixture projects** with various witness types (test, proptest, kani, clippy
   reference)
2. **Integration tests** that invoke `cargo antigen audit` against fixtures and
   verify the validation output
3. **Failing-as-passing tests** for the witness-not-found case ("when the
   witness identifier doesn't resolve, audit MUST report a broken witness")

The audit's witness-validation discipline mirrors the antigen project's broader
philosophy: a marker without proof is not a claim. The audit tests prove the
audit enforces that.

---

## Test discipline as substrate

Tests are themselves substrate. They survive sessions, encode invariants, and
form the structural memory of "what the project must do correctly." This
mirrors the broader antigen pattern: the **tests are the failing-as-passing
witnesses for the project's own behavior**.

When the team adds a new ADR or amends an existing one, they should ask:
- What property does this ADR commit us to?
- What test (or proptest) locks that property?
- Does the existing test suite cover it, or do we need a new test?

Per `proptest-locks-the-narrow-truth`, **the docs cannot claim what the tests
don't verify**. Adding documentation without backing tests is a failure mode
the team explicitly catches.

---

## References

- [`docs/decisions.md`](decisions.md) — ratified ADRs (ADR-001 through ADR-010)
- [`docs/process.md`](process.md) — the formal ADR lifecycle
- [`docs/expedition/inheritance-from-tambear.md`](expedition/inheritance-from-tambear.md) — the proptest-locks-the-narrow-truth discipline as inherited
- [`docs/expedition/case-study-determinism-class.md`](expedition/case-study-determinism-class.md) — the failing-as-passing pattern shown in context
- [`.github/workflows/ci.yml`](.github/workflows/ci.yml) — current CI gates
