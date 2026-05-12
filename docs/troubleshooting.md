# Troubleshooting `cargo antigen`

> Reference for errors and warnings you'll encounter running `cargo antigen scan`
> and `cargo antigen audit`. Organized by the output they appear in.

---

## Scan output

### "856 fingerprint matches" (or any large number)

You're scanning a workspace that uses broad demo antigens, or one of your
antigens has a fingerprint that matches many items.

**Root cause**: The antigen workspace ships two demo antigens —
`DemoBrokenWitness` and `NonUnit` — with fingerprints that match every
`fn`, `struct`, `enum`, and `impl` in the scanned source. When you scan
the antigen workspace itself, those fingerprints fire against production
source files (`scan.rs`, `audit.rs`) and all test fixtures, producing
hundreds of matches. This is the fingerprint engine doing its job, not
an error.

**What to check**:
1. Which antigen is generating most of the matches?
   `cargo antigen scan` groups matches by antigen name.
2. Is the antigen a demo/fixture antigen (`DemoBrokenWitness`, `NonUnit`)?
   Those are intentionally over-broad for testing purposes.
3. Is the match count coming from your own antigens? If so, inspect the
   fingerprint for over-broad operators. For example, a bare `item = fn`
   with no `name:` or `attr_present:` constraint will match every function
   in the workspace.

**Narrowing the output**: pass `--root path/to/crate` to limit the scan
to a subtree. Matches in test fixtures and example code are expected.

---

### 39 parse failures (or any parse failure count)

Parse failures surface in two places: the `--format default` summary line
and the per-item error messages. To see the full error text for every
failure, use `--format json`:

```
cargo antigen scan --format json | jq '.report.parse_failures[]'
```

The 39 parse failures in the antigen workspace are all in test fixture
files — they are deliberate error cases that verify the parser rejects
invalid inputs. They are not defects in your workspace unless you see
failures in your own source.

Parse failures fall into five categories:

#### a) "fingerprint failed to re-parse during synthesis: expected `=`"

**32 of 39 in the antigen workspace.** Fixtures that declare antigens with
no fingerprint field (or a deliberately malformed one) fail at scan time
when the synthesizer tries to match against source items. These are
test fixtures; the scan is correctly rejecting them.

If you see this in your own antigen declaration, the fingerprint string is
syntactically invalid. Check for:
- Missing `=` after an operator name (e.g. `item struct` instead of `item = struct`)
- Unknown operator names (see the list below)
- Unbalanced brackets in `all_of([...])` / `any_of([...])`

**Valid operators**: `item`, `name`, `variants`, `has_method`,
`attr_present`, `doc_contains`, `body_contains_macro`, `all_of`,
`any_of`, `not`

#### b) "#[antigen] on enum is not supported in v0.1"

Antigen declarations must be unit structs. Using an enum as the carrier
type is tracked for a future grammar version but is not supported at v0.1.

```rust
// Wrong
#[antigen(name = "my-class", ...)]
pub enum MyFailureClass { ... }

// Right
#[antigen(name = "my-class", ...)]
pub struct MyFailureClass;
```

#### c) "#[descended_from] on X is not a type declaration"

`#[descended_from]` is only meaningful on `struct` and `enum` items.
It was placed on a function or other non-type item.

#### d) "#[antigen_tolerance] requires non-empty rationale"

The `rationale` argument is required and must be non-empty:

```rust
// Wrong — omitted
#[antigen_tolerance(MyAntigen)]

// Wrong — empty string
#[antigen_tolerance(MyAntigen, rationale = "")]

// Right
#[antigen_tolerance(MyAntigen, rationale = "This site constructs the pattern under test.")]
```

#### e) "#[descended_from] forms a cycle"

The lineage graph has a cycle. `#[descended_from]` requires a strict
DAG — no type can be an ancestor of itself through any path. Identify and
remove the back edge.

#### f) "duplicate #[descended_from(X)] declarations"

The same parent was listed more than once on a single descendant. Remove
the duplicate.

---

## Audit output

### "broken: no function named X found in any .rs file under the scan root"

The witness identifier in `#[immune(Antigen, witness = some_fn)]` does not
resolve to any function in the workspace.

```
.\antigen\examples\broken_witness.rs:38  DemoBrokenWitness (witness = `nonexistent_test`)
  tier = None, hint = NoneApplicable
  → broken: no function named `nonexistent_test` found in any .rs file under the scan root
```

**Cause**: the witness function doesn't exist, was renamed, or is in a
file not under the scan root.

**Fix**: either add the function, correct the name in `#[immune]`, or if
there is genuinely no witness yet, replace with
`#[antigen_tolerance(Antigen, rationale = "no witness yet — intentional gap")]`.

---

### "ambiguous: witness name matches N workspace functions"

The witness name in `#[immune]` resolves to multiple functions in the
workspace. The audit cannot pick one.

```
  → ambiguous: witness name matches 2 workspace functions
      - .\antigen\tests\fixtures\atk_a2_005_scope_cross_reactive\tests.rs
      - .\antigen\tests\fixtures\atk_a2_005_scope_cross_reactive\utils.rs
    Fix: rename one of the colliding functions, or qualify the witness path
```

**Fix**: either rename one of the colliding functions so the name is
unique workspace-wide, or (once path-qualified witnesses ship in A3)
qualify the witness with a module path.

This error surfaces the same ambiguity for same-name proptest + free-function
collisions (ATK-W5-007 behavior — the audit correctly refuses to guess).

---

### "tier = None, hint = NoneApplicable" with no further message

This is the `missing` case: `#[immune]` was declared with no `witness`
argument at all, or with an empty string.

```
.\antigen-macros\tests\ui\immune_without_witness.rs:10  DummyAntigen (witness = ``)
  tier = None, hint = NoneApplicable
  → missing: declaration has no witness identifier; a marker without proof
    is not a claim (per ADR-005)
```

**Fix**: add a witness argument, or file an `#[antigen_tolerance]` with an
explicit rationale if no witness exists yet.

---

### "tier = Reachability, hint = TestAttributePresentNotInvoked"

The witness function exists and has `#[test]`, but the audit cannot confirm
the test is actually invoked. This is normal at v0.1 — full Execution-tier
confirmation requires `cargo test` integration, which arrives in A3.

This is a warning, not an error. The witness is real; it just hasn't been
run and reported back to the audit yet.

---

### "tier = Reachability, hint = TestAttributePresentIgnoreSkipped"

The witness function has both `#[test]` and `#[ignore]`. The test exists
but is explicitly excluded from normal runs. The audit surfaces this as a
skipped witness — it's not a broken witness, but the immunity claim depends
on a test you're not running.

**Fix**: either remove `#[ignore]` (if the test is now ready to run), or
document why the test is skipped in `#[antigen_tolerance]` with a rationale.

---

### "26 inherited presentation(s) not re-attested on the descendant"

When a type carries `#[descended_from(Parent)]`, the audit warns that the
ancestor's immunity witness has not been re-attested on the descendant. The
warning looks like:

```
warning: inherited presentation: `Parent` flowed from ["Parent"] to `struct`
via `#[descended_from]`; the witness inherited from the ancestor has not been
re-attested on the descendant.
Add `#[immune(Parent, witness = ...)]` or `#[antigen_tolerance(Parent, rationale = "...")]`
on the descendant.
```

This is state 7 of the 7-state interaction matrix: the descendant inherits
the presentation (is vulnerable) but has no witness that its own code
handles the failure-class.

In the antigen workspace these 26 warnings are in test fixtures that
deliberately exercise this state. In your workspace, each warning indicates
a type that inherits a vulnerability without demonstrating it is addressed.

**To promote to errors in CI**, use `cargo antigen audit --strict`. To
silence a deliberate gap, add `#[antigen_tolerance(Antigen, rationale = "...")]`
on the descendant.

Note: behavioral re-validation (does the ancestor's witness actually apply
to the descendant?) is A4-A5 work; v0.1 audit stops at Reachability tier.

---

### "18 immunity claim(s) below Execution tier"

Every claim at `tier = Reachability` (or lower) appears in this block. At
v0.1, Execution-tier confirmation requires `cargo test` integration. The
audit reports what it can verify statically:

- `Reachability` — the witness function exists and has `#[test]` (or is
  recognized as a phantom-type or external-tool witness)
- `Execution` — reserved for A3+ when `cargo antigen audit` runs `cargo test`
  and verifies the test passed

A `Reachability` result is meaningful: the witness is real and not ignored.
The remaining gap between "function exists with #[test]" and "test actually
ran and passed" is the v0.1 limitation.

---

## Quick diagnostic table

| Symptom | Likely cause | Action |
|---|---|---|
| Very high fingerprint match count | Over-broad fingerprint or demo antigen | Inspect fingerprint operators; narrow with `name:` or `attr_present:` |
| Many parse failures | Fixture files with intentional errors | Check if failures are in `tests/fixtures/`; if in your code, check fingerprint syntax |
| `broken: no function named X` | Witness name is wrong or function deleted | Fix the name or add the function |
| `ambiguous: matches N workspace functions` | Two functions share the same name | Rename one of the colliding functions |
| `tier = None, hint = NoneApplicable` (no message) | Missing witness in `#[immune]` | Add `witness = your_test_fn` or file `#[antigen_tolerance]` |
| `tier = None, hint = NoneApplicable` (broken) | Witness identifier not in workspace | Add or rename the witness function |
| `TestAttributePresentIgnoreSkipped` | Witness is `#[ignore]`d | Remove `#[ignore]` or document with `#[antigen_tolerance]` |
| Inherited presentation warnings (state 7) | `#[descended_from]` type has no witness | Add `#[immune]` or `#[antigen_tolerance]` on the descendant |

---

## References

- [`docs/decisions.md`](decisions.md) — ADR-005 (witness strength tiers),
  ADR-008 (error-span precision), ADR-011 (tolerance discipline)
- [`docs/usage-patterns.md`](usage-patterns.md) — when to use each macro
- [`docs/testing-patterns.md`](testing-patterns.md) — witness types and
  placement conventions
- [`docs/glossary.md`](glossary.md) — antigen, presentation, immunity,
  tolerance, witness, tier
