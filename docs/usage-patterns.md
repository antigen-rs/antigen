# Antigen — Usage Patterns

> **v0.2 idiom note**: Patterns below reference the v0.1 `#[immune]` API in some examples.
> For v0.2, use `#[defended_by(X)]` on test functions (code-tier) or
> `#[presents(X, requires=...)]` on the site (substrate-tier). See [`macros.md`](macros.md).

> Concrete recipes for applying antigen's vocabulary to real-world failure
> classes. Each pattern answers: what kind of failure is this, where does it
> live in code, and which vocabulary primitives express it correctly.
>
> Patterns here have been through the encounter-tier (at least one concrete
> real-world instance) and retire-to-documentation disposition: existing
> vocabulary handles them without extension; the insight is about *how* to
> apply the vocabulary, not about adding to it.

---

## Declaring a new antigen vs presenting an existing one

**Is this a new failure-class, or a new site in a known failure-class?**

When you recognize a structural vulnerability in your code, the first question
is whether this is a *new* failure-class (needs an `#[antigen]` declaration)
or a *new site* of a *known* failure-class (just needs `#[presents]` or
will be caught by the existing fingerprint).

### Use `#[presents]` (or expect the fingerprint to fire) when:

- The failure-class already has an `#[antigen]` declaration, and your site
  exhibits the same structural pattern
- The code fits the existing fingerprint — `cargo antigen scan` already finds it
- You're applying a known pattern from the antigen-stdlib or a dependency

```rust
// PanickingInDrop already exists. Your new Drop impl panics.
// Just add #[presents] — or wait for scan to catch it.
#[presents(PanickingInDrop)]
impl Drop for MyResourceHandle {
    fn drop(&mut self) {
        self.cleanup(); // which might panic
    }
}
```

### Declare a new `#[antigen]` when:

- You've encountered a failure-class with no existing structural name
- The failure-class has a repeatable structural pattern (not just "this specific
  code is wrong")
- You have at least one concrete real-world instance of the failure to ground
  the fingerprint

The threshold is ADR-006 recognition-not-design: you're *recognizing* a pattern
you've observed, not *designing* a hypothetical category. If you can point to
a concrete instance and describe its structural shape, declare the antigen.

### What "structural pattern" means in practice

A structural fingerprint uses the antigen operators (`item`, `has_method`,
`attr_present`, etc.) to describe the code shape — not its runtime behavior or
logical correctness. A good fingerprint matches when the *structure* is present,
regardless of whether the code happens to be correct in this specific instance.

If you can't describe the failure in structural terms (`item = impl`, `impl Drop
for`, `fn drop`, `body_contains_macro("panic")`), it may be a logic bug rather
than a failure-class. Logic bugs are tests' job; antigen covers structural
patterns.

---

## Antigens at composition boundaries

**When two independent implementations must agree**

### What this pattern is

Some failure-classes don't live at a single code site — they live at the
*boundary* between two implementations that are supposed to agree on some
property. An optimized kernel path and a reference path must produce the same
result. An incremental implementation and a full recompute must converge.
Two codepaths computing the same mathematical function through different
algebraic routes must match within tolerance.

These are **composition-boundary failure-classes**: the failure is not that
either implementation is wrong in isolation, but that their relationship
(the contract between them) can break silently.

### Canonical instance

In numerical math libraries: an `ExpKernelState` carries partial
intermediate values (`expm1_r`, shift `k`) and reconstructs the final
result via `(1 + expm1_r) << k`. A standalone `exp.rs` computes the same
quantity through a different code path. Both paths are correct individually;
they can diverge in edge cases (different rounding at intermediate steps,
different handling of subnormal inputs) in ways that neither path's own
unit tests detect.

The failure-class is: **cross-implementation consistency can silently break**.

### How to declare the antigen

Declare the antigen on the type or module that participates in the
composition — typically the kernel or internal implementation type:

```rust
#[antigen(
    name = "KernelReconstructionDivergence",
    summary = "Kernel state reconstruction and standalone implementation diverge \
               on edge-case inputs; neither path's unit tests catch the disagreement",
    fingerprint = r#"
        all_of([
            item = struct,
            name: matches("*KernelState*")
        ])
    "#,
    references = ["internal:exp-reconstruction-analysis"],
)]
pub struct ExpKernelState { ... }
```

> **v1 grammar note**: the v1 fingerprint grammar matches at item level —
> `item`, `name`, `variants`, `has_method`, `attr_present`, `doc_contains`,
> `body_contains_macro`. There is no `has_field_named` operator yet
> (body-level structure matching is W6b / ADR-015 work). The `name: matches`
> operator is the v1 proxy for naming-convention-based structural patterns.
> Antigen declarations for composition-boundary antigens typically rely on
> `#[presents]` at the consistency test site rather than passive fingerprint
> detection — the fingerprint here is a belt-and-suspenders passive scanner.

### Where to put `#[presents]`

**Mark the consistency test**, not the implementations.

The failure lives in the *relationship* between two code sites, not in
either site individually. In executable terms, the consistency test IS the
composition boundary — it's where the two paths meet and where divergence
becomes observable. The test function is the right proxy for the edge.

```rust
// tests/kernel_consistency.rs

#[presents(KernelReconstructionDivergence)]
#[proptest]
fn kernel_vs_standalone_agrees(input: f64) {
    let kernel_result = ExpKernelState::compute(input);
    let standalone_result = exp_standalone(input);
    prop_assert!((kernel_result - standalone_result).abs() < TOLERANCE,
        "kernel path and standalone path diverged at input={}", input);
}
```

**Why not mark both implementations?** The failure-class is a property of
their *relationship*, not of either implementation's code location. Marking
`ExpKernelState` and `exp_standalone` separately would suggest each is
independently vulnerable — but neither is. The shared antigen on the
consistency test correctly captures: this test is the site where the
failure-class matters and where immunity is demonstrated.

### How to claim immunity

Immunity is on the same site — the consistency test — once the test
actually verifies the invariant:

```rust
#[immune(KernelReconstructionDivergence,
    witness = proptest::kernel_vs_standalone_agrees)]
#[presents(KernelReconstructionDivergence)]  // still presents; immunity covers it
#[proptest]
fn kernel_vs_standalone_agrees(input: f64) { ... }
```

Alternatively, separate the declaration from the proof:

```rust
// The vulnerable boundary:
#[presents(KernelReconstructionDivergence)]
fn run_kernel_consistency_suite() { ... }

// The witness that defends it:
#[immune(KernelReconstructionDivergence,
    witness = proptest::kernel_vs_standalone_proptest)]
#[proptest]
fn kernel_vs_standalone_proptest(input: f64) { ... }
```

### Witness tier profile

Composition-boundary antigens with proptest witnesses currently report
**Reachability** tier with hint `ProptestPresentNotInvoked` (v0.1 audit
does not invoke the proptest harness; Execution tier ships at A4-A5
when harness invocation lands). The proptest shape is still stronger
than a single-input `#[test]` in practice because property-based
testing, when invoked by CI, explores the input space and is more
likely to find edge cases where divergence occurs.

For mathematical implementations where the domain is bounded and the divergence
profile is predictable, consider:

- `proptest` with a custom `Strategy` that focuses on edge cases (subnormal
  inputs, inputs near range boundaries, inputs where both paths take different
  code branches)
- Oracle-based witnesses: one path is the "reference" whose output is trusted;
  the other is the "optimized path" whose output must match

The tier does NOT reach **FormalProof** via proptest — proptest is empirical
coverage, not a mathematical guarantee. If `name@version` is
mission-critical, a phantom-type witness or formal verification adapter
(kani/creusot) is appropriate; see ADR-013 and the formal-verification
witness pattern.

### What the audit surfaces

With the pattern above, `cargo antigen audit` reports:

- The consistency test site as a **Presentation** (vulnerable to divergence)
- The proptest witness at **Reachability** tier with hint
  `ProptestPresentNotInvoked` (v0.1 audit recognizes the proptest shape
  but does not invoke the harness; CI is responsible for running the
  proptest). A4-A5 promotes this to Execution tier when the harness
  invocation lands.

Without `#[immune]`, the audit reports the site as an unaddressed
presentation: "this composition boundary has no verified consistency
witness." That's the signal to write the consistency test.

### What to watch for

- **Tests that always pass because the boundary is never actually exercised**:
  if the proptest `Strategy` generates inputs only in the "easy" region
  where both paths agree, the witness is theatrical (ATK-A3-011 class). Use
  domain knowledge to construct a strategy that hits the edge cases.
- **Marking the implementations instead of the consistency test**: this
  produces two unaddressed presentations (one per implementation) that never
  relate to each other. The audit can't see the connection. Mark the boundary
  (the consistency test), not the endpoints.
- **Using a unit test instead of a proptest**: a single-input consistency
  check proves agreement at one point. Proptest gives broader coverage.
  For mathematical correctness, proptest or a formal tool is the right
  witness tier.

---

## Prospective antigens — declaring before there are vulnerable sites

**You know the failure class exists; no site has it yet**

### What this pattern is

A prospective antigen is declared before the codebase has any `#[presents]`
sites or fingerprint matches. You're codifying institutional memory of a
failure-class that exists in the ecosystem, even though your project hasn't
yet developed the structural pattern that triggers it.

Example: a team that has seen "panicking in Drop leads to double-panic" in
production adds a `PanickingInDrop` antigen at project initialization, before
writing any `Drop` impls. When the first `Drop` impl appears, the scan catches
it immediately.

### When to use it

Use a prospective antigen when:

- You've seen the failure-class in another codebase or in production, and
  recognize the structural pattern that enables it
- The project's architecture or roadmap makes the vulnerable pattern likely
  to appear (e.g., async runtime work → `Drop` implementations)
- You want the structural memory committed *before* the vulnerable code exists,
  so the first developer to write the risky pattern gets immediate feedback

### The silent-failure risk

Prospective antigens hide fingerprint bugs longer. A fingerprint that produces
zero matches is indistinguishable from a prospective antigen that correctly
has no matches *and* from a prospective antigen whose fingerprint is silently
wrong.

Mitigation: verify fingerprint examples against the engine while writing the
antigen. Run `cargo antigen scan` against a minimal test file that contains the
structural pattern the fingerprint should match, and confirm it fires. Don't
wait for production code to validate the fingerprint.

```rust
// Before committing PanickingInDrop, verify the fingerprint fires
// against a test impl in a scratch file:
//
// impl Drop for TestDrop {
//     fn drop(&mut self) {
//         panic!("deliberate");
//     }
// }
//
// Then run: cargo antigen scan --root /path/to/scratch
// and confirm the match appears.
```

### What the scan output looks like

When a prospective antigen has no fingerprint matches and no `#[presents]`
sites:

```
antigen scan
  Antigens declared:  1
  Sites presenting:   0
  Sites immune:       0

  0 fingerprint matches (no matching items found).
  0 presentations unaddressed (all immune or tolerated).
```

This is expected. The antigen is ready; no vulnerable code exists yet.

### Relationship to tolerance

Prospective antigens and `#[antigen_tolerance]` solve different problems:

- **Prospective antigen**: declares a failure-class before vulnerable sites
  exist; the expected audit result is "zero presentations"
- **`#[antigen_tolerance]`**: explicitly exempts a specific site that
  fingerprint-matches but is not actually vulnerable; the site exists

Don't use tolerance to suppress fingerprint matches from a prospective antigen
that hasn't been validated. Validate the fingerprint first.

---

## When to use `#[antigen_tolerance]`

**The fingerprint matched, but this site isn't actually vulnerable**

### What this pattern is

The fingerprint engine finds code matching an antigen's structural pattern —
but the match is by design, not by vulnerability. A test that deliberately
constructs the failure pattern to verify detection. A `Drop` impl that *must*
call a function that *could* panic because the error has nowhere else to go.
A type that matches a `frame-translation` fingerprint because it IS the
translation layer.

`#[antigen_tolerance]` is how you tell antigen: "I see the match; it's
intentional; here's why."

### The decision tree

When the scan surfaces an unaddressed presentation on a site you own:

```
Is this site genuinely vulnerable?
├── Yes → add #[presents(X)] on the site, write a test + add #[defended_by(X)] on it (code-tier),
│         or add #[presents(X, requires=...)] for substrate evidence
└── No → why not?
    ├── The site matches by design (it's the translation layer, the test
    │   fixture, the intentional construction) → #[antigen_tolerance]
    ├── The failure-class doesn't apply to this site structurally, but the
    │   fingerprint matched → investigate: is the fingerprint over-broad?
    │   File an issue; use #[antigen_tolerance] with rationale explaining
    │   the false positive while the fingerprint is refined.
    └── You haven't gotten to it yet → leave the presentation unaddressed;
        the audit warning is correct; this is the signal to come back
```

**Do not use `#[antigen_tolerance]` to silence warnings you haven't thought
about.** The required `rationale` field is the guard: if you can't write a
sentence explaining why this site is safe, you haven't thought about it.

### Good rationale vs. weak rationale

The `rationale` field is required by the parser. What it should contain:

**Good**: explains the structural reason the failure-class doesn't apply here.

```rust
#[antigen_tolerance(
    PolarityInvertedClassMeet,
    rationale = "This is the test fixture that deliberately constructs the \
                 inverted-polarity case to verify the fingerprint catches it. \
                 The 'vulnerability' is the point of the test."
)]
fn test_fingerprint_detects_inverted_meet() { ... }
```

```rust
#[antigen_tolerance(
    PanickingInDrop,
    rationale = "This Drop impl calls log::error! which cannot panic in \
                 practice (the logging infrastructure is initialized before \
                 any Drop runs). The fingerprint matches syntactically; the \
                 structural risk doesn't obtain here because the call site \
                 is panic-free by construction."
)]
impl Drop for ResourceHandle { ... }
```

**Weak** (rejected by reviewers, though not by the parser):

```rust
#[antigen_tolerance(
    PanickingInDrop,
    rationale = "This is fine"  // explains nothing
)]
```

```rust
#[antigen_tolerance(
    PanickingInDrop,
    rationale = "TODO: investigate later"  // this is not tolerance; this is deferral
)]
```

The rationale should be legible to a future team member reading cold. "This is
fine" doesn't survive team turnover. The structural reason does.

### When to use `until`

`#[antigen_tolerance]` accepts an optional `until` field for time-bounded
tolerances:

```rust
#[antigen_tolerance(
    ImplicitCouplingViaFeatureFlag,
    rationale = "tokio feature is used transitively; refactoring to an \
                 executor-agnostic design is tracked in issue #234. Accepted \
                 until we complete the sansio migration.",
    until = "2026-09-01"
)]
pub struct MyService { ... }
```

Use `until` when:
- The tolerance is a known technical debt with a resolution plan
- A refactoring is in progress and the warning would fire throughout the
  transition period
- A version upgrade will remove the failure-class from scope

**Do not use `until` to create an expiry you have no intention of enforcing.**
When `until` expires, `cargo antigen audit --strict` fails. The field creates a
real deadline; treat it as one.

Without `until`, the tolerance is permanent — which is correct for sites that
match by design and will never be vulnerable.

### Tolerance vs. not marking at all

If a site has no `#[presents]` and no `#[antigen_tolerance]`, the scan reports
it as an unaddressed presentation (state 2: passively detected). The audit
warning is active and correct.

`#[antigen_tolerance]` is different from simply not marking the site because:
- It is **explicit**: a future reader knows someone thought about this
- It is **grounded**: the rationale explains why it's not a vulnerability
- It is **auditable**: `cargo antigen audit` lists all tolerances; reviewers
  can verify the rationale still holds
- It is **time-bounded if warranted**: `until` creates a real re-review trigger

The alternative — deleting the `#[antigen]` declaration entirely or setting
the fingerprint to exclude these sites — destroys the structural memory of why
the fingerprint exists. Tolerance is preferable to exclusion because it
preserves the signal while recording the exception.

### Inherited tolerance

`#[antigen_tolerance]` covers inherited presentations (state 4 absorbs
state 6/7). If a child antigen inherits a presentation from a parent via
`#[descended_from]` and the child's site is a legitimate match-by-design,
mark the child's site with tolerance. The same rationale-required discipline
applies; the same audit-visibility applies.

### What the audit shows

With `#[antigen_tolerance]`:

```
cargo antigen audit

✓ 12 presentations addressed (immune)
  3 presentations tolerated (antigen_tolerance)
    - PolarityInvertedClassMeet on src/tests.rs:45
      rationale: "Test fixture that constructs the failure pattern"
      until: (no expiry)
    - PanickingInDrop on src/resource.rs:112
      rationale: "log::error! is panic-free by construction here"
      until: (no expiry)
    - ImplicitCouplingViaFeatureFlag on src/lib.rs:8
      rationale: "Tracked in issue #234; sansio migration in progress"
      until: 2026-09-01
  0 presentations expired (tolerance past until date)
  0 presentations unaddressed (no immune or tolerance)
```

The tolerated-count is visible and distinct from addressed. Reviewers who
want to audit tolerance quality run `cargo antigen audit --list-tolerances`
and read each rationale. The vocabulary makes the decision visible without
hiding it.

---

## Patterns for substrate-witness predicates

Discipline-witnesses (`requires = <predicate>` on `#[immune]`) express that a
human — not a test — validated something the code can't verify for itself.
These patterns guide which predicate leaf to reach for in common situations.

### `witness = fn` vs `requires = all_of([...])`

| Mechanism | What it asserts | Audit tier | When to use |
|---|---|---|---|
| `witness = test_fn` | The test function exists and has `#[test]` | `Reachability` | Behavioral verification — the code does the right thing |
| `requires = signers(...)` | A named human reviewed this version of the code | `Execution` (when current + Fresh) | Human expertise required — math correctness, protocol compliance, security review |
| `requires = all_of([signers(...), ratified_doc(...)])` | Multiple forms of evidence all satisfied | `Execution` | Belt-and-suspenders: behavioral test + human review + governing document |

**Reach for `witness = fn`** when the discipline can be verified computationally —
a property test, a unit test, a comparison against a reference implementation.
Tests are fast, cheap, and run in CI. They're the right answer when "running the
code proves the invariant."

**Reach for `requires = ...`** when the discipline cannot be verified by running
code — when the claim is "a person who understands the domain reviewed this and
confirmed it is correct." Math correctness, floating-point discipline, cryptographic
protocol compliance, domain-specific invariants — these require a human expert, not
a test runner. `requires =` binds the attestation to the code fingerprint; if the
code changes, the attestation goes stale and the audit surfaces it.

Both can coexist on the same site:

```rust
#[immune(SignedZeroDiscipline,
    witness = sinh_preserves_signed_zero,   // test: does it produce the right answer?
    requires = signers(required = ["alice"], against = "current")  // human: did a domain expert review the algorithm?
)]
```

### `signers` vs `ratified_doc` vs `oracles_complete`

| Goal | Use |
|---|---|
| Record that named reviewers signed off on this code | `signers(required = ["alice", "bob"])` |
| Assert a discipline document exists and is versioned | `ratified_doc(path = "docs/discipline.md", min_version = "1.0")` |
| Assert external evidence was reviewed and attested | `oracles_complete([oracle-id])` |

**Use `signers` when** the evidence is "this specific person reviewed this specific
version of the code." The signature is tied to the code fingerprint — if the code
changes, the signature goes stale and the audit says so. This is the right leaf
for code review as a discipline.

**Use `ratified_doc` when** the evidence is "this discipline is governed by a
document that the team maintains." The document needs to exist, be versioned (so
you can floor on `min_version`), and optionally contain an anchor (a section that
must be present). Use this when the discipline is stable and lives in prose rather
than code.

**Use `oracles_complete` when** the evidence is "someone attested they reviewed
an external reference — a paper, a standard, a specification." Oracles are oracle
artifacts with their own lifecycle (`Draft`/`Complete`/`Deprecated`/`Retired`).
The oracle system is heavier than a simple `ratified_doc`; use it when the
evidence source itself needs lifecycle management.

### `against = "current"` vs `against = "any"`

```
signers(required = ["alice"], against = "current")  // default
signers(required = ["alice"], against = "any")
```

**`against = "current"` (default)**: alice's signature must be against the
*current* code fingerprint. If the code changes after alice signs, alice's
signature is stale and the audit surfaces `discipline-substrate-stale`. This
is the right policy for most cases — you want re-attestation when the code
changes.

**`against = "any"`**: alice need only have signed *at some point* in the
history of this item, regardless of fingerprint. Use this for "ever-reviewed"
gates where the accumulated history of review matters and staleness is expected
(e.g., a long-running research project where domain experts review periodically
but not on every code change).

The `against` field is a lease policy, not a security boundary. Both variants
can be attested falsely by anyone with git write access; the difference is what
the audit considers current.

### Delta-attestation vs Fresh: when has the code changed enough?

When code changes are minor — a variable rename, a comment update, a refactor
that preserves the mathematical structure — asking a domain expert to re-sign
from scratch is friction without value. Delta-attestation covers this:

```sh
cargo antigen attest delta \
    --file src/numerics.rs \
    --antigen SignedZeroDiscipline \
    --item sinh \
    --from <prior-fingerprint> \
    --rationale "Renamed internal variable; algebraic structure unchanged"
```

The `attest delta` command records that the signer reviewed the *diff* between
the prior fingerprint and the current one, found it preserved the discipline, and
carried their attestation forward. The audit tracks chain depth (default cap: 3)
— after 3 carry-forwards, a Fresh re-attestation is required regardless of how
small each individual change was.

**Use delta-attestation when**: the diff is a refactor that a domain expert can
confirm preserves the invariant with a quick review.

**Use Fresh attestation when**: the algorithm changed, the mathematical approach
changed, or the diff requires the same depth of analysis as the original review.

The non-empty `rationale` is required — the audit rejects empty rationales at
parse time. The rationale is the domain expert saying in their own words why the
carry-forward is valid.

### Oracles vs `ratified_doc`: which has lifecycle?

```
ratified_doc(path = "docs/ieee754-compliance.md")
oracles_complete(["ieee-754-sincos-section-6-3"])
```

`ratified_doc` says "this file exists and has `min_version >= X` in its
frontmatter." The file's lifecycle is managed by your team through normal git
commits. Simple and low-overhead.

`oracles_complete` says "this oracle artifact — which has its own declared
stewards, lifecycle state, and transition history — is in Complete state and
someone attested they reviewed it." The oracle system is appropriate when:

- The evidence source is shared across multiple antigens (e.g., "the IEEE 754
  standard" as a single oracle referenced by many `signers` predicates)
- You need to track who is responsible for maintaining the reference (stewards)
- The reference could become Deprecated or Retired (e.g., an internal spec that
  gets superseded)
- You need version-pinning that records exactly which version was reviewed

For most single-antigen documentation, `ratified_doc` is simpler. Reach for
`oracles_complete` when the evidence source has its own organizational lifecycle.

### Signature-tier choice

The `signature_allow` field on a `signers` leaf is a categorical allow-list —
not an ordinal threshold. A signer with `TextStamp` strength against
`allow = [CryptoSigned]` fails not because they are "weaker" but because they
are the wrong type.

| Tier | How recorded | Identity binding | When to require |
|---|---|---|---|
| `TextStamp` | Name + date, no cryptographic binding | Minimal — anyone can write any name | Soft attestation, internal teams with high trust |
| `GitTrust` | Git commit authorship (git config `user.name/email`) | Bound to committer identity | Standard discipline-witness; default recommendation |
| `CryptoSigned` | DSSE envelope with cryptographic signature | Strongest available in v0.1 (v0.4+ full activation) | Cross-org attestation, safety-critical claims |

**Recommendation**: use `GitTrust` minimum for any meaningful discipline claim.
`signature_allow = [GitTrust, CryptoSigned]` is the pattern that binds the signer's
identity to their git commit history while allowing stronger signatures when
available.

`TextStamp` alone is documentation-quality intent — it records that someone said
they reviewed something, but it doesn't bind the claim to a verifiable identity.
Use it only for low-stakes claims or internal workflows where git identity is
impractical.

### Combining leaves: `all_of` and `any_of`

```rust
#[immune(SignedZeroDiscipline, requires = all_of([
    signers(required = ["alice"], against = "current"),
    ratified_doc(path = "docs/ieee754-discipline.md", min_version = "1.0"),
    fresh_within_days(180),
]))]
```

**`all_of`**: all child predicates must pass. Use for "must have both a signer
AND a governing document AND be reasonably fresh." This is co-stimulation —
all signals required, missing one → anergy (predicate fails).

**`any_of`**: at least one child must pass. Use for "either alice OR bob has
reviewed" or "either a test OR a signer attestation." This is redundant pathway
coverage — one strong signal is sufficient.

**`not`**: the child must NOT pass. Rarely needed in practice; useful for
"this site must NOT have been signed by the legacy reviewer whose credentials
we no longer trust."

Zero-leaf `all_of([])` and `any_of([])` are rejected at parse time — they are
semantic no-ops that vacuously pass. If you want "no predicate," don't add
`requires =` at all.

---

- [`docs/tutorial.md`](tutorial.md) — end-to-end walkthrough of first antigen
  declaration + scan + audit, starting from zero
- [`docs/macros.md`](macros.md) — full reference for all five attribute macros
  with syntax, examples, and field-by-field documentation
- [`docs/witness-tiers.md`](witness-tiers.md) — WitnessTier gradient reference
  with per-tier semantics and expected audit output
- [`docs/output-formats.md`](output-formats.md) — scan/audit output reference:
  human-readable and JSON, field-by-field
- [`docs/fingerprint-grammar.md`](fingerprint-grammar.md) — full operator
  reference for the fingerprint DSL with worked examples
- [`docs/decisions.md`](decisions.md) — ADR-002 (compose-not-compete),
  ADR-005 (sub-clause F + witness tier honesty), ADR-011 (`#[antigen_tolerance]`
  mechanics and rationale-as-required-field), ADR-013 (phantom-type
  witness recognition + WitnessTier gradient)
- [`docs/testing-patterns.md`](testing-patterns.md) — property test
  conventions, failing-as-passing pattern
- [`docs/glossary.md`](glossary.md) — antigen, presentation, immunity,
  witness, WitnessTier, tolerance
- [`docs/where-to-look-for-antigens.md`](where-to-look-for-antigens.md) —
  project layout conventions for antigen declarations and related files
