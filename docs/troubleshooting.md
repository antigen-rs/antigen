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

## Discipline-witness audit hints

When `cargo antigen audit` evaluates a substrate-witness predicate (`requires = ...`
on `#[immune]` or `#[antigen_tolerance]`), it reads the `.attest/` sidecar and
evaluates the predicate against on-disk evidence. Every result carries a
`hint` field that names exactly what the audit found. Here is every hint, what
caused it, and how to fix it.

### Immunity-claim hints

#### `discipline-predicate-passed-substrate-current`

**What it means**: everything is correct. The sidecar was read, the predicate
passed, and every signer's fingerprint matches the current code. This is the
highest tier available from on-disk substrate evidence.

**No action needed.**

---

#### `discipline-predicate-passed-via-delta-chain`

**What it means**: the predicate passed, but at least one signer's basis is
`DeltaFrom` rather than `Fresh`. The attestation is a carry-forward from a prior
signature, not a full re-review. Tier is still `Execution` — the carry-forward
is valid — but the audit surfaces it so you know.

**When to act**: if the cumulative drift since the last Fresh attestation is
significant enough that the carry-forward rationale no longer covers the
current code, ask the signer to do a Fresh re-attestation.

---

#### `discipline-substrate-delta-chain-near-cap`

**What it means**: the predicate passed, but a signer's delta chain depth is at
`cap - 1`. The *next* delta will be refused; the signer must do a Fresh
re-attestation before they can continue carry-forwarding.

**Fix**: the signer runs `cargo antigen attest sign` (not `attest delta`) against
the current code.

---

#### `discipline-substrate-stale`

**What it means**: the predicate uses `against = "current"` (the default), and
at least one signer's `signed_against_fingerprint` does not match the current
code fingerprint. The signer attested a previous version; the code has changed
since.

**What to check**: what changed? If it's a meaningful change to the algorithm or
discipline being attested, the signer needs to re-review. If it's a cosmetic
change (variable rename, comment), the signer can use `attest delta` with a
rationale explaining why the change preserves the discipline.

**Fix**: either `cargo antigen attest sign` (fresh review) or `cargo antigen
attest delta --from <old-fingerprint> --rationale "..."` (carry-forward for
minor changes).

---

#### `discipline-sidecar-missing`

**What it means**: `#[immune]` declared `requires = <predicate>`, but no sidecar
exists at the expected location: `<source_file_stem>.attest/<AntigenName>.json`.

**Fix**:

```sh
cargo antigen attest scaffold \
    --file src/numerics.rs \
    --antigen SignedZeroDiscipline \
    --item sinh
```

Then sign the scaffolded sidecar.

---

#### `discipline-sidecar-schema-invalid`

**What it means**: a sidecar file exists but didn't parse as a valid
`Ratification` JSON schema — missing required fields, wrong type, or JSON
syntax error.

**Fix**: run `cargo antigen audit --format json` to see the parse error.
Common causes:
- Missing `"schema_version"` field (must be `"v1"`)
- Missing `"kind"` field (must be `"immunity"` or `"tolerance"`)
- Malformed fingerprint value (not a string)
- Editing the JSON by hand and introducing a typo

The fastest fix is usually `cargo antigen attest scaffold` to rebuild the
sidecar from scratch, then re-sign.

---

#### `discipline-predicate-failed`

**What it means**: the sidecar was read and is schema-valid, but the predicate
evaluated to false. The per-leaf details will appear in the full audit output
(`--format json` or the `detail` field).

**What to check**: which leaf failed?
- `signers(required = ["alice"])` failed → alice is not in the sidecar's
  signers list, or alice's entry is stale and `against = "current"` is set
- `ratified_doc(path = "docs/discipline.md")` failed → the file doesn't
  exist, or the version is below `min_version`, or the anchor isn't present
- `fresh_within_days(90)` failed → the most recent current-fingerprint signer
  is more than 90 days old

**Fix**: address the specific failed leaf. Add the missing signer, update the
discipline document, or arrange a re-attestation.

---

### Tolerance-claim hints

#### `tolerance-vibes-grade`

**What it means**: `#[antigen_tolerance(X)]` was declared without the
`sidecar = true` opt-in. The tolerance is "vibes-grade" — an inline rationale
with no on-disk attestation. The audit reports `WitnessTier::None` and
`EvidenceKind::None` for this site.

This is the default state for all tolerance claims. It's not an error, but
it means the tolerance is unstructured — no one has formally attested it.

**When to act**: if your CI policy requires structured tolerance attestation,
enable `sidecar = true` and follow the tolerance attestation workflow (see
`cargo antigen tolerate scaffold`). If vibes-grade tolerance is acceptable in
your project, no action is needed.

---

#### `tolerance-sidecar-missing`

**What it means**: `#[antigen_tolerance(X, sidecar = true)]` opted into
structured attestation, but no sidecar exists at the expected location.

**Fix**:

```sh
cargo antigen tolerate scaffold \
    --file src/numerics.rs \
    --antigen SignedZeroDiscipline \
    --item legacy_sinh
```

---

#### `tolerance-predicate-failed`

**What it means**: the tolerance sidecar exists but the predicate failed. Same
class as `discipline-predicate-failed` — consult the per-leaf details.

A failing tolerance predicate means the tolerance isn't actually attested. Fix
the specific predicate failure or re-sign the sidecar.

---

#### `tolerance-predicate-passed-substrate-current`

**What it means**: structured tolerance is fully attested. Predicate passed, all
signers are current Fresh. Tier is `Execution`.

**No action needed.** The tolerance is formally attested and current.

---

### Kind-mismatch hints

These appear when the sidecar's `"kind"` field doesn't match what the macro
declaration expects.

#### `discipline-sidecar-kind-mismatch-expected-immunity-got-tolerance`

**What it means**: `#[immune(X, requires = ...)]` found a sidecar with
`"kind": "tolerance"`. The site switched from `#[antigen_tolerance]` to
`#[immune]` but the sidecar wasn't regenerated.

**Fix**: delete the old sidecar and scaffold a new one with `attest scaffold`.
The kind is set at scaffold time; it can't be changed by editing the JSON
(the sidecar would pass schema validation but produce this hint).

---

#### `tolerance-sidecar-kind-mismatch-expected-tolerance-got-immunity`

**What it means**: `#[antigen_tolerance(X, sidecar = true, requires = ...)]`
found a sidecar with `"kind": "immunity"`. The symmetric case: site switched
from `#[immune]` to `#[antigen_tolerance]` but the sidecar wasn't regenerated.

**Fix**: delete the old sidecar and scaffold a new one with `tolerate scaffold`.

---

### Compound-contradiction hint

#### `discipline-immunity-tolerance-contradiction`

**What it means**: both `#[immune(X)]` and `#[antigen_tolerance(X, sidecar =
true)]` are declared on the same site for the same antigen. A site cannot be
simultaneously immune (compliant) and tolerating (non-compliant). The audit
reports `WitnessTier::None` and overrides both individual tier reports.

**Fix**: decide which declaration is correct and remove the other. If the site
is genuinely immune, remove the tolerance. If it's genuinely tolerating (known
gap, accepted), remove the immunity claim.

---

### Discipline-witness hint quick reference

| Hint | Tier | Meaning | Fix |
|---|---|---|---|
| `discipline-predicate-passed-substrate-current` | Execution | All current, all Fresh — clean | None |
| `discipline-predicate-passed-via-delta-chain` | Execution | Carry-forward active — informational | Fresh re-sign if drift is significant |
| `discipline-substrate-delta-chain-near-cap` | Execution | Next delta will be refused | Fresh re-sign now |
| `discipline-substrate-stale` | Reachability | Code changed since last sign | `attest sign` or `attest delta` |
| `discipline-sidecar-missing` | None | No sidecar file | `attest scaffold`, then sign |
| `discipline-sidecar-schema-invalid` | None | Sidecar JSON malformed | Fix JSON or `attest scaffold` fresh |
| `discipline-predicate-failed` | None | Predicate evaluated false | Fix the failing leaf |
| `tolerance-vibes-grade` | None | Tolerance unattested (no sidecar opt-in) | Add `sidecar = true` and `tolerate scaffold` if needed |
| `tolerance-sidecar-missing` | None | Opted into sidecar, file missing | `tolerate scaffold`, then sign |
| `tolerance-predicate-failed` | None | Tolerance predicate false | Fix the failing leaf |
| `tolerance-predicate-passed-substrate-current` | Execution | Tolerance fully attested | None |
| `discipline-sidecar-kind-mismatch-expected-immunity-got-tolerance` | None | Sidecar kind wrong | `attest scaffold` fresh |
| `tolerance-sidecar-kind-mismatch-expected-tolerance-got-immunity` | None | Sidecar kind wrong | `tolerate scaffold` fresh |
| `discipline-immunity-tolerance-contradiction` | None | Both `#[immune]` and `#[antigen_tolerance]` on same site | Remove one declaration |

---

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
