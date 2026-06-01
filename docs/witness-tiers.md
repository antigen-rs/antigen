# Antigen — Witness Tier Reference

> **v0.2 idiom note**: Examples in this doc use the v0.1 `#[immune(...)]` API.
> For v0.2, use `#[defended_by(X)]` on test functions (code-tier) or
> `#[presents(X, requires=...)]` on the site (substrate-tier). The `#[immune]` form
> still compiles with a deprecation warning — see [`macros.md`](macros.md) for migration.

> The `WitnessTier` gradient that `cargo antigen audit` reports for every
> defended presentation. This document explains what each tier means, when
> each applies, and how the audit reports them honestly per ADR-005
> Amendment 3.

For a tutorial introduction, see [`tutorial.md`](tutorial.md). For audit
output format, see [`output-formats.md`](output-formats.md).

---

## Why tiers (not just pass/fail)

Witness validation is not a binary "verified / not verified." Different
witness types provide different *strengths* of guarantee. A
`#[test]` function passing tells you the code ran without panicking on
the inputs the test exercised. A formal proof tells you the code is
correct on all possible inputs in the proven domain. A clippy lint
reference tells you a pattern-matcher external to antigen has been
configured to catch this case.

These are categorically different. Reporting them all as "verified"
would silently overclaim the weaker witnesses up to the strongest.
ADR-005 Amendment 3 (audit-tier-honesty) requires the audit to report
the *actual* strength of verification performed, never a stronger one.

The four tiers in v0.1.0-rc.1 (per `WitnessTier` enum in
`antigen/src/audit.rs`):

| Tier | Strength | When it applies |
|---|---|---|
| **FormalProof** | Mathematical guarantee on all inputs in the proven domain | Phantom-type witnesses (ADR-013) — turbofish pattern (`Foo::<T>::constructor`) recognized as a sealed type-system proof |
| **Execution** | Empirically verified on tested inputs | Reserved for A4-A5: requires the audit to actually invoke `cargo test` / proptest harness and confirm the witness passes. Not emitted in v0.1 |
| **Reachability** | Witness identifier resolves; audit has not verified runtime behavior | All v0.1 non-FormalProof resolutions: `#[test]` / `#[test]+#[ignore]` / `proptest!` / regular functions / external-tool prefixes (`clippy::`, `kani::`, etc.). The audit hint disambiguates which case |
| **None** | No *passing* evidence — either no witness resolved, or a predicate resolved and failed | Two distinct sub-channels collapse to `None`: (a) **witness-resolution** gap — `Missing`, `NotFound`, or `Ambiguous` witness status (no witness to evaluate); (b) **predicate-evaluation** outcome — a `requires =` substrate-witness predicate was evaluated and *failed* (`DisciplinePredicateFailed`). Tier reports only strength-of-*passing*-evidence, so both non-pass cases share `None`; the `AuditHint` carries which sub-channel and why |

**Why only Reachability for `#[test]` in v0.1**: per ADR-005 Amendment 3
(audit-tier-honesty), the audit reports the work the audit ACTUALLY
PERFORMED. v0.1 walks the workspace and indexes functions; it does NOT
invoke `cargo test`. A `#[test]` function whose run was not invoked sits
at Reachability — its existence is verified, its passing is not.
Promotion to Execution tier requires harness invocation, planned for A4-A5.

**Disambiguating Reachability cases via audit hints**: although all
non-FormalProof resolutions collapse to Reachability tier in v0.1, the
parallel `AuditHint` field distinguishes them. The hint is what the
human-readable diagnostic emits and what consumers should match on:

| Witness shape | Tier | Audit hint |
|---|---|---|
| `#[test]` function (not ignored) | Reachability | `TestAttributePresentNotInvoked` |
| `#[test]` + `#[ignore]` | Reachability | `TestAttributePresentIgnoreSkipped` |
| `proptest!` macro | Reachability | `ProptestPresentNotInvoked` |
| Bare function, no test attribute | Reachability | `FunctionResolves` |
| External-tool prefix (`clippy::`, `kani::`, …) | Reachability | `ExternalToolPrefixRecognized` |
| Phantom-type turbofish (`Foo::<T>::ctor`) | FormalProof | `PhantomTypeShapeRecognized` |

---

## FormalProof tier

**Strength**: mathematical guarantee covering all inputs in the proven
domain.

**Recognized witnesses** (v0.1.0-rc.1):

- **Phantom-type witnesses** (ADR-013) — the type system itself proves
  immunity. Recognition shape: a path with turbofish syntax
  (`Foo::<TypeParam>::constructor`). The audit's
  `detect_phantom_type_witness` matches this shape and classifies as
  `WitnessKind::PhantomType { proof_type, type_params, constructor }`.

The witness IS the type structure; no runtime test needed because the
proof lives in the compiler's type-checking pass. Construction of the
proof token can only happen via the sealed constructor, so if the code
compiles, the proof holds.

**Example** (full version in `antigen/examples/phantom_witness.rs`):

```rust
use antigen::immune;
use std::marker::PhantomData;

pub struct NonPanickingProof<T> {
    _marker: PhantomData<T>,
    _seal: (),  // private field — only the sealed constructor can produce one
}

impl<T> NonPanickingProof<T> {
    pub const fn verified() -> Self {
        Self { _marker: PhantomData, _seal: () }
    }
}

pub struct PhantomVerifiedDropImpl;

#[immune(
    DropPanicClass,
    witness = NonPanickingProof::<PhantomVerifiedDropImpl>::verified,
    rationale = "Phantom-type token constructible only via the sealed `verified` constructor."
)]
impl Drop for PhantomVerifiedDropImpl { /* ... */ }
```

The turbofish form (`::<>`) is what triggers phantom-type recognition.
Audit reports FormalProof tier with the `PhantomTypeShapeRecognized`
hint when the shape recognizes. (Recognition is shape-only; behavioral
verification that the constructor IS sealed is the developer's
responsibility — the audit recognizes the shape but cannot prove the
constructor's soundness, so the hint name explicitly names the
recognition-but-not-validation surface.)

**Future tier extensions** (not v0.1.0-rc.1; see [`roadmap.md`](roadmap.md)):

- `kani::proof_fn` / `prusti::specification` / `verus::proof` /
  `creusot::specification` witnesses where the consuming workspace
  actually executes the verifier and the proof passes. v0.1.0-rc.1
  reports these as Reachability tier with the
  `ExternalToolPrefixRecognized` hint (see below); A4-A5 will ship the
  harness invocation that can promote them to FormalProof tier when the
  verifier confirms.

---

## Execution tier (reserved for A4-A5)

**Strength**: empirically verified by the audit actually invoking the
witness harness and confirming a passing run.

**Status in v0.1.0-rc.1**: NOT EMITTED. The Execution tier exists in the
`WitnessTier` enum as a forward-compatibility slot, but the v0.1 audit
does not invoke `cargo test`, the proptest harness, or any external
verifier. All test-function witnesses sit at `Reachability` tier with
their corresponding `TestAttributePresentNotInvoked` /
`ProptestPresentNotInvoked` / `TestAttributePresentIgnoreSkipped` hint.

**When Execution tier ships (A4-A5 per [`roadmap.md`](roadmap.md))**:

- `#[test]` function whose `cargo test` invocation passed → promoted from
  Reachability to Execution
- `proptest!` function whose harness invocation completed without
  property violation → promoted from Reachability to Execution
- External-tool prefixes (`kani::`, `prusti::`, `verus::`, `creusot::`,
  `flux::`) whose tool invocation produced a passing proof → promoted
  from Reachability to FormalProof (the external tool produces the
  guarantee; antigen recognizes the invocation result, not the proof
  structure)

**Example** (current v0.1 behavior — reports Reachability, not Execution):

```rust
use antigen::immune;
use crate::antigens::PanickingInDrop;

#[immune(
    PanickingInDrop,
    witness = safe_drop_no_panic_test,
    rationale = "SafeType::drop verified panic-free by direct test."
)]
impl Drop for SafeType { /* ... */ }

#[test]
fn safe_drop_no_panic_test() {
    drop(SafeType { data: None });
    drop(SafeType { data: Some(String::from("x")) });
}
```

In v0.1: the audit reports `tier = Reachability`, `hint =
TestAttributePresentNotInvoked` for this immunity claim — accurately
reflecting that the function exists with a `#[test]` attribute but its
run was not invoked by the audit. CI is responsible for actually running
the test; the audit reports the work IT performed, not what CI might do.

---

## Reachability tier

**Strength**: the function exists in scope; audit hasn't verified it was
executed or that it actually defends.

**When it applies**:

- Bare-identifier witness resolves to a workspace function that isn't
  recognized as `#[test]`, `#[proptest]`, or a known external-tool prefix
- The function exists; the audit can find it; but the audit doesn't
  know what kind of verification it provides

**Example**:

```rust
#[immune(MyAntigen, witness = my_helper_function)]
fn defended() { /* ... */ }

fn my_helper_function() { /* not a test, just a function */ }
```

The audit reports Reachability tier with audit-hint `FunctionResolves`.
This is **not an error** — it's honest reporting that the witness
exists but its semantic meaning is unverified.

**To upgrade to Execution tier**: convert the helper to a `#[test]`
function (or write a new test) and point the witness at it.

---

## External-tool witnesses (Reachability tier + ExternalToolPrefixRecognized hint)

**Note**: there is no separate `ExternalUnvalidated` tier in v0.1.0-rc.1.
External-tool witnesses resolve to `WitnessStatus::External { tool_hint }`,
which the audit maps to `WitnessTier::Reachability` with the
`ExternalToolPrefixRecognized` audit hint. The discipline is the same as
the legacy "ExternalUnvalidated" framing (external delegation is weaker
than execution); only the tier name differs.

**When it applies**:

- Witness names a known external tool: `clippy::`, `kani::`, `prusti::`,
  `verus::`, `creusot::`, `flux::`
- The consuming workspace can't (or doesn't) execute the external tool
  as part of `cargo antigen audit`

**Example**:

```rust
#[immune(MyAntigen, witness = clippy::no_panic_in_drop)]
impl Drop for SafeType { /* clippy lint covers this case */ }
```

Audit reports `tier = Reachability, hint = ExternalToolPrefixRecognized`
(JSON: `"reachability"` / `"external-tool-prefix-recognized"`). The
witness IS recognized; antigen delegates the actual verification to
clippy (compose-don't-compete per ADR-002), but clippy isn't executed
inside antigen's audit pipeline in v0.1.0-rc.1.

**Future upgrade** (per [`roadmap.md`](roadmap.md) Sweep A4): harness
invocation will execute the external tool and upgrade resolved witnesses
to Execution or FormalProof tier as appropriate. The
`ExternalToolInvoked` hint exists in the enum as a forward-compatibility
slot but is not emitted in v0.1.

**Cross-crate witnesses**: when a witness like `dep_crate::test_fn`
points to a function in a dependency that the consuming workspace can't
execute, the witness reports Reachability tier with the
`ExternalToolPrefixRecognized` hint by the same discipline (ADR-005
Amendment 3, A3.5 substrate).

---

## None tier

**Strength**: no *passing* evidence. The `WitnessTier::None` variant
(serialized `"none"` in JSON). Tier reports strength-of-*passing*-evidence
only, so every non-pass shares `None` regardless of *why* it didn't pass —
the `AuditHint` is the channel that carries the why.

`None` collapses two structurally-distinct sub-channels. **(1) Witness-resolution
gaps** — there was no evidence to evaluate. **(2) Predicate-evaluation failures**
— a `requires =` substrate-witness predicate *was* evaluated and *failed*. These
are opposites (absence-of-evidence vs evidence-of-absence) and call for opposite
fixes, so reading the `AuditHint` is mandatory to tell them apart.

**When it applies — (1) witness-resolution gaps**:

- `#[immune]` without a `witness =` field → `WitnessStatus::Missing` →
  audit hint `NoneApplicable`
- `witness = fn_name` where `fn_name` doesn't exist in the workspace →
  `WitnessStatus::NotFound { reason }` → audit hint `NoneApplicable` (or
  `FabricatedPathPrefix` when the path's module prefix doesn't exist —
  ATK-A2-011)
- `witness = fn_name` where multiple functions named `fn_name` exist →
  `WitnessStatus::Ambiguous { candidates }` → audit hint `AmbiguousResolution`

**When it applies — (2) predicate-evaluation failures**:

- `requires = <predicate>` where the substrate-witness predicate was
  evaluated against the `.attest/` sidecar and did not pass → audit hint
  `DisciplinePredicateFailed` (or `TolerancePredicateFailed` for a
  tolerance sidecar). Unlike the resolution gaps above, the evidence
  *exists and was checked* — it just didn't satisfy the predicate. Per-leaf
  detail on *which* leaf failed and *why* is the diagnostic the audit hint
  alone cannot yet carry (a known DX gap; see the per-leaf-diagnostics work).

**Example (witness not found)**:

```rust
#[immune(MyAntigen, witness = nonexistent_test)]
impl Drop for SafeType { /* ... */ }
```

Audit reports `tier = None, hint = NoneApplicable` and surfaces the
diagnostic:

```
⚠ MyAntigen — witness `nonexistent_test` not found in workspace
  Site: src/safe_type.rs:42
```

**Resolution paths** (audit suggests):

a) Add a `#[test]` function exercising the witness path (Execution tier upgrade)
b) Point the witness at a runnable test (`#[test]` without `#[ignore]`)
c) Rename colliding functions or qualify ambiguous witness paths
d) Add the witness function to the workspace if it's missing
e) Tolerate the gap with `#[antigen_tolerance(...)]` if intentional

---

## Audit hints

Each audit entry carries an `AuditHint` that explains the specific case
behind its tier. The hint is what consumers should match on for routing
logic — tier alone collapses several distinct shapes into the same value
(particularly at `Reachability`).

`AuditHint` is serialized **kebab-case** in JSON (per `#[serde(rename_all
= "kebab-case")]` in `antigen/src/audit.rs`); the Rust variant names are
PascalCase. The table below lists both forms.

| JSON hint | Rust variant | Resulting tier | Meaning |
|---|---|---|---|
| `phantom-type-shape-recognized` | `PhantomTypeShapeRecognized` | FormalProof | Turbofish phantom-type witness shape matched (ADR-013); constructor sealing not validated |
| `phantom-type-construction-validated` | `PhantomTypeConstructionValidated` | FormalProof | Phantom-type construction validated (future; not emitted in v0.1) |
| `test-attribute-present-not-invoked` | `TestAttributePresentNotInvoked` | Reachability | Function has `#[test]`; audit did not invoke `cargo test` |
| `test-attribute-present-ignore-skipped` | `TestAttributePresentIgnoreSkipped` | Reachability | Function has `#[test]` AND `#[ignore]`; `cargo test` would skip it by default |
| `proptest-present-not-invoked` | `ProptestPresentNotInvoked` | Reachability | `proptest!` macro invocation found; harness not invoked |
| `function-resolves` | `FunctionResolves` | Reachability | Workspace function exists with no testing attribute; behavior not verified |
| `external-tool-prefix-recognized` | `ExternalToolPrefixRecognized` | Reachability | External-tool prefix recognized (`clippy::`, `kani::`, …); tool not invoked |
| `external-tool-invoked` | `ExternalToolInvoked` | (future) | External tool actually invoked (A4+; not emitted in v0.1) |
| `ambiguous-resolution` | `AmbiguousResolution` | None | Witness name matches more than one workspace function (ATK-A2-005) |
| `fabricated-path-prefix` | `FabricatedPathPrefix` | None | Witness path's module prefix doesn't exist; last segment found but in an unrelated location (ATK-A2-011) |
| `none-applicable` | `NoneApplicable` | None | Catch-all for Missing / NotFound when no more-specific hint applies |
| `inherited-presentation-not-re-attested` | `InheritedPresentationNotReAttested` | (state-7 diagnostic, separate channel) | Inherited Presentation lacks re-attestation on the descendant site; state 7 of the 7-state interaction matrix (ADR-018). Surfaces via `audit.inherited_unaddressed[]` rather than as a per-immunity audit entry |

The complete hint set lives in `antigen/src/audit.rs::AuditHint`; this
table reflects v0.1.0-rc.1 hint enumeration.

---

## Reading audit output

Human-readable audit output groups by tier. The per-tier sub-counts
(`formal-proof`, `execution`) are emitted only when their count is
greater than zero; the conventional summary lines (`declared`,
`external`, `ambiguous`, `broken`, `missing`) are always emitted.

```
Auditing workspace: .

Audited 12 immunity claim(s):
  - 6 formal-proof (phantom-type or formal-verification tool — compile-time evidence)
  - 3 declared (witness identifier found in workspace — not yet semantically verified)
  - 1 external (delegated to clippy/kani/prusti/etc. — not yet executed by antigen)
  - 0 ambiguous (witness name resolves to multiple workspace functions)
  - 2 broken (witness identifier not found)
  - 0 missing (no witness identifier)

✓ 6 immunity claim(s) at Execution tier or higher:
  path/to/file.rs:LINE  AntigenType (witness = `witness_expression`)
    tier = FormalProof, hint = PhantomTypeShapeRecognized
  ...

⚠ 6 immunity claim(s) below Execution tier:
  ...
```

The confirmed-claims section (added in A3.5 per ATK-A3-019) makes
FormalProof and Execution tier achievements visible in human output.
Adding a phantom-type witness now produces visible positive signal,
not silent classification.

---

## Tier-honesty discipline

Per ADR-005 Amendment 3, the audit:

- **Reports actual tier, not maximal tier**. If `proptest` is the
  witness, v0.1 audit reports Reachability with hint
  `ProptestPresentNotInvoked` — not Execution, because the audit did
  not invoke the proptest harness.
- **Surfaces below-Execution claims explicitly**. All claims at
  `Reachability` or `None` tier produce ⚠ warnings in human-readable
  output.
- **Never silently upgrades**. The witness shape determines the tier;
  the audit cannot promote a Reachability witness to Execution without
  evidence (which v0.1 cannot produce).
- **Reports external-tool delegation honestly**. A witness with an
  external-tool prefix (`clippy::`, `kani::`, …) reports Reachability
  tier with the `ExternalToolPrefixRecognized` hint — the prefix is
  recognized, the tool is not invoked.

This is sub-clause F (ADR-005) applied to the audit reporting surface:
every trust claim requires validation at the strength it's claimed,
and downgrades are explicit when validation can't reach the claimed
strength.

---

## Substrate-witness tier (ADR-019)

ADR-019 (discipline-witnesses) introduces a fourth `EvidenceKind`:
`SubstrateState` — evidence derived from on-disk substrate (JSON
sidecars, git trailers, oracle files) rather than from code-level
witnesses. Substrate witnesses use the same `WitnessTier` enum but
have a different ceiling, different hint vocabulary, and different
CLI interaction surface (`cargo antigen attest / check`).

### EvidenceKind and tier ceilings

| `EvidenceKind` | What produces it | Max `WitnessTier` |
|---|---|---|
| `TypeSystemProof` | Phantom-type witnesses (ADR-013) | `FormalProof` |
| `Behavioral` | `#[test]`, proptest, external tools | `Execution` (at A4-A5; `Reachability` in v0.1) |
| `SubstrateState` | Substrate-witness predicates (ADR-019) | `Execution` — ceiling by design |
| `None` | No witness / witness fails | `None` |

**Why SubstrateState cannot reach FormalProof**: a JSON sidecar is
empirical on-disk state — it's the strongest assertion a human
reviewer can produce about a discipline decision, but it does not
constitute a mathematical guarantee covering all possible inputs.
The ceiling at `Execution` is structurally exact: the sidecar plus a
passing predicate means the discipline decision was empirically made
and recorded, which is Execution-strength evidence.

### Signature tiers

Substrate-witness signatures carry a `SignatureStrength` that records
identity-binding fidelity of each signer. Three tiers in v0.1:

| `SignatureStrength` | Serde value | Identity binding | When to use |
|---|---|---|---|
| `TextStamp` | `"text_stamp"` | Name + timestamp only; no external validation | LLM agents, reviewers without git config |
| `GitTrust` | `"git_trust"` | `git config user.name + user.email` at sign time; fingerprint-pinned | Default for git-configured humans |
| `CryptoSigned` | `"crypto_signed"` | Cryptographic binding (DSSE-PAE + Sigstore transparency log) | v0.4+ activation path; schema-reserved in v0.1 |

The `signature_allow` field on the `signers` predicate leaf declares
which strengths the project accepts for this antigen. Default (empty
list) accepts all three. The `signature_prefer` field, when set,
generates an informational hint when signers use a lower-than-preferred
strength but does not fail the predicate.

### Predicate grammar (sealed leaf set)

Substrate-witness predicates are composed from five sealed leaf
primitives using three combinators:

**Combinators**: `all_of([...])`, `any_of([...])`, `not(...)`

**Leaf primitives**:

| Leaf | Key assertion |
|---|---|
| `ratified_doc(path, anchor?)` | Named doc file exists with YAML frontmatter `status: ratified`; optional anchor section present |
| `signers(required, roles?, against?, signature_allow?, signature_prefer?)` | Sidecar `signers[]` contains all required names with optional role/currency/strength constraints |
| `signed_trailer(key, role?, count?)` | Git log on commits touching this item has matching trailer entries |
| `oracles_complete(files)` | Listed oracle files exist with `status: complete` |
| `fresh_within_days(days)` | Newest current-fingerprint signature is within `days` of today |

### Substrate-witness audit hints

Substrate-witness evaluation uses `SubstrateAuditHint` (also aliased as
`AuditHint` within `antigen-attestation`) — a parallel vocabulary to the
code-witness `AuditHint` in `antigen/src/audit.rs`. Both can fire on the
same audit result.

`SubstrateAuditHint` is serialized **kebab-case** (per
`#[serde(rename_all = "kebab-case")]`). The 13 variants, grouped by
claim kind:

**Immunity-claim substrate hints** (7 variants):

| JSON hint | Rust variant | Tier | Meaning |
|---|---|---|---|
| `discipline-sidecar-missing` | `DisciplineSidecarMissing` | None | No `.attest/` directory or no sidecar for this antigen |
| `discipline-sidecar-schema-invalid` | `DisciplineSidecarSchemaInvalid` | None | Sidecar exists but did not parse as valid `Ratification` schema |
| `discipline-predicate-failed` | `DisciplinePredicateFailed` | None | Sidecar parsed but substrate-witness predicate failed |
| `discipline-substrate-stale` | `DisciplineSubstrateStale` | Reachability | Predicate passes but ≥1 signature is stale relative to the current fingerprint |
| `discipline-substrate-delta-chain-near-cap` | `DisciplineSubstrateDeltaChainNearCap` | Execution | Predicate passes, all current, but a signer's chain depth is near the cap — next delta will be refused |
| `discipline-predicate-passed-via-delta-chain` | `DisciplinePredicatePassedViaDeltaChain` | Execution | Predicate passes, all current; ≥1 signer's basis is `DeltaFrom` (within caps) — informational carry-forward note |
| `discipline-predicate-passed-substrate-current` | `DisciplinePredicatePassedSubstrateCurrent` | Execution | Predicate passes, all current, all signers' bases are `Fresh` — the strongest substrate-witness state in v0.1 |

**Tolerance-claim substrate hints** (4 variants):

| JSON hint | Rust variant | Tier | Meaning |
|---|---|---|---|
| `tolerance-vibes-grade` | `ToleranceVibesGrade` | — | `#[antigen_tolerance]` declared without `sidecar = true`; ADR-011 vibes-grade gap |
| `tolerance-sidecar-missing` | `ToleranceSidecarMissing` | None | `sidecar = true` but no sidecar found |
| `tolerance-predicate-failed` | `TolerancePredicateFailed` | None | Tolerance sidecar exists, predicate failed |
| `tolerance-predicate-passed-substrate-current` | `TolerancePredicatePassedSubstrateCurrent` | Execution | Tolerance sidecar exists, predicate passes, all signers current and Fresh |

**Kind-mismatch hints** (2 variants):

| JSON hint | Rust variant | Tier | Meaning |
|---|---|---|---|
| `discipline-sidecar-kind-mismatch-expected-immunity-got-tolerance` | `DisciplineSidecarKindMismatchExpectedImmunityGotTolerance` | None | `#[immune]` site but sidecar `kind = Tolerance` — site was converted from `#[antigen_tolerance]` without regenerating the sidecar |
| `tolerance-sidecar-kind-mismatch-expected-tolerance-got-immunity` | `ToleranceSidecarKindMismatchExpectedToleranceGotImmunity` | None | `#[antigen_tolerance]` site but sidecar `kind = Immunity` |

**Compound contradiction** (1 variant):

| JSON hint | Rust variant | Tier | Meaning |
|---|---|---|---|
| `discipline-immunity-tolerance-contradiction` | `DisciplineImmunityToleranceContradiction` | None | Site declares both `#[immune]` and `#[antigen_tolerance]` for the same antigen — logically incoherent |

### Quick-disambiguation: substrate-witness state to tier

| Situation | Tier | `SubstrateAuditHint` |
|---|---|---|
| No sidecar or unparseable sidecar | None | `discipline-sidecar-missing` / `discipline-sidecar-schema-invalid` |
| Sidecar parses, predicate fails | None | `discipline-predicate-failed` |
| Predicate passes, stale signature present | Reachability | `discipline-substrate-stale` |
| Predicate passes, all Fresh | Execution | `discipline-predicate-passed-substrate-current` |
| Predicate passes, has DeltaFrom basis | Execution | `discipline-predicate-passed-via-delta-chain` |
| Predicate passes, chain depth near cap | Execution | `discipline-substrate-delta-chain-near-cap` |

### Examples

**Basic** — a single TextStamp signer, all-strengths allowed:

```json
// .attest/SignedZeroDiscipline.json
{
  "kind": "Immunity",
  "antigen": "SignedZeroDiscipline",
  "predicate": {
    "name": "signers",
    "required": ["alice"]
  },
  "signers": [
    {
      "name": "alice",
      "role": "math-reviewer",
      "strength": "text_stamp",
      "signed_at": "2026-05-19T12:00:00Z",
      "signed_against_fingerprint": "abc123",
      "basis": { "type": "Fresh", "reasoning": "Verified sinh/cosh signed-zero behavior in PR #47." }
    }
  ]
}
```

Resulting audit hint: `discipline-predicate-passed-substrate-current`
at `Execution` tier (all signers Fresh, predicate passes).

**Mid** — two signers required, git-trust-only project policy:

```json
// .attest/SafeDropDiscipline.json
{
  "kind": "Immunity",
  "antigen": "SafeDropDiscipline",
  "predicate": {
    "name": "all_of",
    "children": [
      {
        "name": "signers",
        "required": ["alice", "bob"],
        "signature_allow": ["git_trust", "crypto_signed"]
      },
      { "name": "fresh_within_days", "days": 90 }
    ]
  },
  "signers": [ /* ... */ ]
}
```

If alice signed with `text_stamp` but `signature_allow` only permits
`git_trust`/`crypto_signed`, the predicate fails →
`discipline-predicate-failed` at `None` tier.

**Advanced** — delta-chain anti-laundering on a carry-forward:

```json
{
  "name": "alice",
  "strength": "git_trust",
  "signed_against_fingerprint": "def456",
  "basis": {
    "type": "DeltaFrom",
    "parent_fingerprint": "abc123",
    "rationale": "Re-verified: refactor preserved all signed-zero behavior; no semantic delta.",
    "chain_depth": 1,
    "cumulative_fingerprint": "deadbeef"
  }
}
```

Hint: `discipline-predicate-passed-via-delta-chain` at `Execution` tier.
If `chain_depth` reaches the configured cap, the next delta is refused
and `discipline-substrate-delta-chain-near-cap` fires while still passing.

### What SubstrateState does NOT do

- **Does not replace tests**. A passing `signers` predicate means a
  human attested they reviewed the discipline decision — it does not
  verify that the code actually upholds the discipline at runtime.
  Use code-level witnesses for behavioral verification; use substrate
  witnesses for discipline-decision attestation.
- **Does not produce FormalProof tier**. On-disk empirical state
  cannot constitute a mathematical guarantee regardless of how many
  signers or how strong their signatures.
- **Does not validate signature authenticity in v0.1**. `GitTrust`
  records the git-config identity at sign time; it does not verify
  that the git config itself was accurate. `CryptoSigned` (v0.4+)
  provides cryptographic binding. `TextStamp` is self-declared.
- **Does not cache**. The evaluator reads the sidecar fresh from disk
  on every audit invocation (no-cache discipline per ADR-019 §M2).
  The sidecar is the substrate; the substrate is the source of truth.

---

## Choosing a witness type

Practical guidance for v0.1.0-rc.1 (tier values reflect what the audit
actually emits; future tier-promotion paths noted in parentheses):

| If you have… | Use this witness | v0.1 tier (hint) |
|---|---|---|
| A `#[test]` function | `witness = test_fn_name` | Reachability (`TestAttributePresentNotInvoked`) — promotes to Execution at A4-A5 |
| A `proptest!` covering broad input space | `witness = proptest_fn_name` | Reachability (`ProptestPresentNotInvoked`) — promotes to Execution at A4-A5 |
| A phantom-type proof token | `witness = NonPanickingProof::<MyType>::verified` | FormalProof (`PhantomTypeShapeRecognized`) |
| A clippy lint rule | `witness = clippy::lint_name` | Reachability (`ExternalToolPrefixRecognized`) — promotes to Execution at A4-A5 |
| A formal proof in kani/prusti/verus/creusot/flux | `witness = kani::proof_fn` (etc.) | Reachability (`ExternalToolPrefixRecognized`) — promotes to FormalProof at A4-A5 |
| Just a helper function (not a test) | `witness = helper_fn` | Reachability (`FunctionResolves`) |
| A discipline decision sidecar (ADR-019) | `#[immune(..., requires = signers(["alice"]))]` | Execution if predicate passes and all Fresh (`DisciplinePredicatePassedSubstrateCurrent`) |
| Nothing yet — placeholder | `#[antigen_tolerance(...)]` with rationale | (tolerated; not an immunity claim) |

The discipline is honest: a theatrical test that always passes still
reports its tier honestly. The audit reports the shape it recognized,
not whether the witness actually verifies the failure class — semantic
verification (does the witness mean what it should?) is behavioral-tier
work for A4-A5. The [`troubleshooting.md`](troubleshooting.md) document
covers the "witness passes but doesn't mean what it should" failure-class
family (ATK-A2-003/004/005/011/012).

---

## See also

- ADR-002 (compose, don't compete — witness pluralism)
- ADR-005 (sub-clause F at every trust boundary)
- ADR-005 Amendment 3 (audit-tier-honesty)
- ADR-007 (anti-YAGNI: all witness families committed)
- ADR-013 (phantom-type witness recognition)
- ADR-019 (discipline-witnesses: substrate-witness predicate family)
- [`macros.md`](macros.md) — `#[immune]` macro reference
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL
- [`output-formats.md`](output-formats.md) — full audit output reference
- [`troubleshooting.md`](troubleshooting.md) — diagnostic guide
