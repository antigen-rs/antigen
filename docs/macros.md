# Antigen — Macro Reference

> Comprehensive reference for antigen's attribute macros. For tutorials,
> see [`tutorial.md`](tutorial.md); for placement conventions, see
> [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md); for
> patterns, see [`usage-patterns.md`](usage-patterns.md).

Core macros are imported from the `antigen` crate:

```rust
use antigen::{antigen, presents, defended_by, descended_from, antigen_tolerance};
```

Prescriptive / work-orchestration macros:

```rust
use antigen::{panel, ddx, rx, refer, biopsy, culture, quarantine, triage};
```

---

## Quick reference — core macros

| Macro | Applies to | Purpose |
|---|---|---|
| `#[antigen]` | unit struct | Declare a named failure-class with a structural fingerprint |
| `#[presents]` | any item | Mark code as a site that exhibits a known failure-class |
| `#[defended_by]` | test / proptest fn | Register a test function as a **code-tier witness** for a failure-class (ADR-029) |
| `#[presents(requires=)]` | any item | Attach a **substrate-tier witness** predicate (sidecar evidence) to a presents-site |
| `#[presents(proof=)]` | any item | Attach a **phantom-type / formal-proof** witness to a presents-site |
| `#[immune]` | — | *(**Removed** — ADR-029; use `#[defended_by]` or `#[presents(requires=)]`. See [migration guide](immune-migration-guide.md))* |
| `#[descended_from]` | unit struct (antigen) | Declare inheritance from a parent failure-class |
| `#[antigen_tolerance]` | any item | Explicitly tolerate a fingerprint match (with required rationale) |

## Quick reference — marked-unknown markers (the felt-but-unnamed plane, ADR-041)

The single most perishable knowledge in software is the unease you can't yet
name. These three markers let you record it *structurally* — `trigger` is
**required** on all three (a triggerless marker is graffiti and is a compile
error). They sit on a 2-D plane: **magnitude** × **existence-certainty**.

| Macro | Plane corner | Purpose |
|---|---|---|
| `#[aura(trigger = "...")]` | low magnitude | "something *may* be off here, can't name it, check later" — a mild substrate-smell; never gates, never nags |
| `#[dread(trigger = "...")]` | high magnitude, low certainty | the *angor animi* corner: "something *is* wrong here, I can't name it, look now" |
| `#[red_flag(trigger = "...")]` | high existence-certainty | "I'm *sure* something is wrong, can't name it, act now" — records at the highest internal severity (see note); never gates, never alerts |

> **What "highest severity" means here — and what it does *not* do.** All three
> markers are passive records; none gates, fails your build, or alerts. `cargo
> antigen scan --format json` surfaces each under the top-level
> `report.marked_unknowns` array (fields: `marker`, `magnitude`,
> `existence_certainty`, `trigger`, `file`, `line`, `shape_digest`,
> `structural_digest`). Internally, each marker also
> emits a `Finding` carrying a `severity` field: `#[red_flag]` (existence-certainty
> `Sure`) and `#[dread]` both compute `High`, `#[aura]` computes `Medium`
> (`MarkedUnknown::to_finding`). That severity is a **reserved routing hint** (the
> consuming routing organ is chartered, not built — see
> [`roadmap.md`](roadmap.md)) — `#[red_flag]`'s "escalation" is
> *that it records at `High`*, **not** that anything fires: it never gates CI, never
> changes an exit code, never alerts. And the severity is **internal to the
> `Finding` only** — the user-facing `report.marked_unknowns` projection carries no
> `severity` field today. The human-readable scan report does not render
> marked-unknowns; today a marker is a structural record you *query* (via
> `--format json`), not a console line and not an action.

See [`docs/concepts.md`](concepts.md) and ADR-041 for the plane; a worked
example is in [`antigen/examples/marked_unknown.rs`](../antigen/examples/marked_unknown.rs).

> **These marks are the input to the Learning-Core loop.** A *cluster* of
> marked-unknown sites that share a body-shape is what antigen's `propose()`
> anti-unifies into a draft fingerprint (cluster → propose → test →
> promote/prune, governed by a self-tolerance gate). Important scope note:
> **the Learning-Core is a library API (`antigen::learn`), not a CLI
> command — there is no `cargo antigen propose`.** A draft it produces is a
> *hypothesis to ratify*, never an auto-asserted `#[presents]`. See
> [`roadmap.md`](roadmap.md) and the `antigen::learn` module rustdoc.

## Quick reference — prescriptive / work-orchestration macros

| Macro | Shape | Purpose |
|---|---|---|
| `#[panel(needs, ...)]` | S1 Role-workflow | Ordered review checklist (who fills + who reviews) |
| `#[rx(treatment, ...)]` | S1 Role-workflow | Treatment prescription — what must be done before the site ships |
| `#[refer(to, ...)]` | S1 Role-workflow | Referral to an external owner; anchored at the code site |
| `#[biopsy(location, request_text, ...)]` | S1 Role-workflow | Deep investigation request at a sub-site |
| `#[ddx(symptom, rule_out, ...)]` | S2 Elimination | Differential — competing hypotheses to rule out one by one |
| `#[triage(priority_order, ...)]` | S3 Ordering | Re-validatable priority order over code-site references |
| `#[culture(test_kind, ...)]` | S4 Frame-only | Time-boxed observation window |
| `#[quarantine(scope, reason, ...)]` | S4 Frame-only | Isolated region under a deliberate hold |

`cargo antigen audit` renders each work-need's verdict: `Pending` / `Fulfilled` / `Overdue` / `OutOfFrame`.

All macros are **identity transforms** — they validate attribute syntax
at compile time and emit the original item unchanged. The semantic work
(scanning, matching, witness validation) lives in `cargo antigen scan`
and `cargo antigen audit`, which parse source independently.

---

## `#[antigen(...)]`

Declare a named failure-class with a structural fingerprint.

### Required attributes

| Field | Type | Purpose |
|---|---|---|
| `name` | string (kebab-case) | Identifier for the failure-class |
| `fingerprint` | string (DSL) | Structural pattern; see [`fingerprint-grammar.md`](fingerprint-grammar.md) |

### Optional attributes

| Field | Type | Purpose |
|---|---|---|
| `family` | string | Parent class (typically one of the 8 first-principles failure classes from `docs/decisions.md` ADR-010) |
| `summary` | string | Human-readable description; surfaces in audit output |
| `references` | array of strings | Open-vocabulary cross-references (URLs, CVE IDs, ADR IDs, RFC IDs, issue IDs, post-mortem links) |
| `category` | path | `AntigenCategory::SubstrateAlignment` or `AntigenCategory::FunctionalCorrectness` (ADR-028) |
| `provenance` | path | *How we know this class exists* (ADR-039 §C): `Provenance::Encountered` / `Constructable` / `Heuristic` / `Imagined`. Omitted ⇒ `Imagined` |
| `presentation` | path | `Presentation::Passive` (tooling/scan-side, the default) or `Presentation::Active` (user-facing). Omitted ⇒ `Passive` |

> **Path fields are read as token paths — do not `use`-import them.** `category`,
> `provenance`, and `presentation` are parsed as path expressions, so a
> `use antigen::Provenance;` (etc.) trips `unused_imports` under `-D warnings`.
> Write `provenance = Provenance::Constructable` directly without an import.
>
> **`provenance` is the honest-labeling on-ramp** (ADR-039): admission is
> permissive (name / see / imagine a class → it's admissible), so the *label* is
> what stays truthful. Omitting `provenance` defaults to `Imagined` — the weakest
> claim — because an unlabeled antigen honestly *is* the weakest claim. Provenance
> is the class's evidence basis, distinct from the dial-derived confidence tier
> (suspected/named) — the audit-time calibration the confidence-dial wave will
> report; provenance sets the floor that tier may graduate from.

### Applies to

Unit structs only. The struct carries no data — it's a vocabulary token.

### Example

```rust
use antigen::antigen;

/// Drop impls must not panic; panic-during-unwind causes process abort.
#[antigen(
    name = "panicking-in-drop",
    family = "boundary-violation",
    fingerprint = r#"
        item = impl,
        has_method("drop", "(& mut self)"),
        any_of([
            body_contains_macro("panic"),
            body_contains_macro("unreachable"),
        ])
    "#,
    summary = "Drop impls must not panic; panic-during-unwind causes process abort.",
    references = [
        "https://doc.rust-lang.org/std/ops/trait.Drop.html#panics",
        "CVE-EXAMPLE-2024-001",
    ],
)]
pub struct PanickingInDrop;
```

### Layer 1 minimum viable

```rust
#[antigen(
    name = "my-failure-class",
    fingerprint = r#"item = fn, name = matches("dangerous_*")"#,
)]
pub struct MyFailureClass;
```

Only `name` and `fingerprint` are required. Per ADR-009 adoption gradient,
projects can start at Layer 1 and add `family`, `summary`, `references`
as discipline matures.

### Behavior

`cargo antigen scan` collects every `#[antigen]` declaration in the
workspace and treats each as a recognized failure-class. The
`fingerprint` field drives passive detection — any item in the codebase
matching the structural pattern is reported as a `[fingerprint match]`,
even without an explicit `#[presents]` marker.

### Discipline

- **kebab-case names**: enforced at parse time; `name = "PanickingInDrop"` is rejected
- **Non-empty fingerprint**: enforced at parse time
- **Reference shapes**: open-vocabulary; the tool accepts any string. Conventions documented in `docs/usage-patterns.md`.

### See also

- ADR-001 (carrier set + identity transform)
- ADR-009 (Layer 1 → Layer 2 → Layer 3 adoption gradient)
- ADR-010 (fingerprint grammar v1)
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL reference

---

## `#[presents(antigen_type)]`

Mark code as exhibiting an antigen's structural pattern (vulnerability
declaration).

### Required arguments

- One positional argument: the antigen type (unit struct declared with `#[antigen]`)

### Applies to

Any Rust item: `fn`, `impl`, `struct`, `enum`, `trait`, `mod`, `type`, `const`, `static`, `use`, etc.

### Example

```rust
use antigen::presents;
use crate::antigens::PanickingInDrop;

#[presents(PanickingInDrop)]
impl Drop for VulnerableType {
    fn drop(&mut self) {
        let _val = self.data.as_ref().unwrap();  // could panic
    }
}
```

### Behavior

`cargo antigen scan` collects every `#[presents]` site and reports it.
`cargo antigen audit` then observes whether the site is *defended* (via
`#[defended_by]` on a test or `requires=` predicate) or *undefended*.
The audit output groups presentations by antigen type with per-site verdicts.

### Discipline

- **The antigen type must resolve**: the scan checks that `PanickingInDrop`
  is a known antigen type. Unresolved references surface as errors.
- **Co-location encouraged**: put `#[presents]` at the actual vulnerable
  site (the impl, the function, the struct). For composition-boundary
  failure-classes, the consistency test is the site — see
  [`usage-patterns.md`](usage-patterns.md#antigens-at-composition-boundaries).

### See also

- ADR-001 (carrier set)
- [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) — placement conventions
- [`usage-patterns.md`](usage-patterns.md) — pattern recipes

---

## `#[defended_by(antigen_type)]`

Register a test or proptest function as a **code-tier witness** for a failure-class (ADR-029).

This is the primary idiom for code-tier defense. The macro is placed on the test function
(or proptest function), not on the vulnerable site. Audit cross-references it against all
`#[presents(AntigenType)]` sites and reports `defended at Reachability` when the test is
reachable, `defended at Execution` when it is confirmed executed.

### Required arguments

- First positional: the antigen type path (e.g. `PanickingInDrop` or `crate::antigens::Foo`)

### Applies to

Functions annotated with `#[test]`, `proptest!`, or any runnable test harness.

### Example

```rust
use antigen::defended_by;

#[test]
#[defended_by(PanickingInDrop)]
fn resource_handle_drop_does_not_panic() {
    let _ = ResourceHandle { id: 42 };
    // If drop panics, the test fails — correctly catching PanickingInDrop
}
```

The `#[presents(PanickingInDrop)]` marker stays on the Drop impl site; this test is the
evidence that the class is defended there.

### Key distinction from `#[immune]` (deprecated)

`#[defended_by]` registers evidence *at the witness site* — audit observes the defense.
The deprecated `#[immune]` was placed at the *vulnerable site* and *claimed* immunity.
The observe-not-declare inversion (ADR-029) means the site never asserts its own defense;
audit cross-references the evidence and reports the verdict.

### See also

- ADR-029 (Immunity Is Observed, Not Declared)
- `#[presents]` — marks the vulnerable site
- `#[presents(requires=)]` — substrate-tier witness for evidence outside the code

---

## `#[immune(...)]` — REMOVED (ADR-029)

> **The `#[immune]` proc-macro has been removed.** It was the v0.1 immunity-claim
> API (deprecated in v0.2 per ADR-029, *immunity is observed, not declared*) and
> is no longer exported by `antigen` / `antigen-macros`. Code that still imports
> or applies `#[immune]` will not compile against this version.

Migrate each `#[immune]` site to the observe-don't-declare idiom:

- **Code-tier defense** (`witness = a_test`): put `#[presents(AntigenType)]` on
  the site and `#[defended_by(AntigenType)]` on the test that defends it.
- **Substrate-tier defense** (`requires = <predicate>`): fold it onto the site as
  `#[presents(AntigenType, requires = <predicate>)]`.
- **Phantom/formal-proof** (`witness = <phantom>`): fold it onto the site as
  `#[presents(AntigenType, proof = <expr>)]`.

The `requires =` / `proof =` grammar is identical to the old `#[immune]` forms,
so substrate-tier and phantom-tier predicates migrate verbatim. Audit observes
these registrations and reports per-site verdicts (`defended` / `undefended` /
`substrate-gap`).

The [migration guide](immune-migration-guide.md) walks each case with
before/after code; see [`#[defended_by]`](#defended_byantigen_type) above and
[`witness-tiers.md`](witness-tiers.md) for how witness tiers are now graded on
the `#[defended_by]` registration.

### See also

- ADR-029 (Immunity Is Observed, Not Declared)
- [migration guide](immune-migration-guide.md) — step-by-step `#[immune]` conversion
- `#[defended_by]` — the code-tier migration target
- `#[presents(requires=)]` / `#[presents(proof=)]` — substrate-tier / phantom-tier targets

---

## `#[descended_from(ParentAntigen)]`

Declare structural inheritance between failure-classes.

### Required arguments

- One positional argument: the parent antigen type

### Applies to

Unit structs declared with `#[antigen]` (extends an existing antigen
declaration).

### Example

```rust
use antigen::{antigen, descended_from};

#[antigen(
    name = "polarity-inverted-class-meet",
    fingerprint = r#"item = enum, name = matches("*Class"), has_method("meet", "(self, Self) -> Self")"#,
    summary = "Class enums must use max (not min) for lattice meet.",
)]
pub struct PolarityInvertedClassMeet;

#[antigen(
    name = "polarity-inverted-class-join",
    fingerprint = r#"item = enum, name = matches("*Class"), has_method("join", "(self, Self) -> Self")"#,
    summary = "Class enums must use min (not max) for lattice join.",
)]
#[descended_from(PolarityInvertedClassMeet)]
pub struct PolarityInvertedClassJoin;
```

### Behavior

`cargo antigen scan` propagates presentations from parent to descendant
through the inheritance chain. Witnesses on the parent may apply to the
descendant if structurally compatible (audit validates).

The `inherited_from` field on each `Presentation` carries a `ProvenanceEntry`
recording the chain (ADR-018 Option C). Diamond inheritance is dedup'd
correctly (multiple paths to the same parent produce one presentation,
not duplicates).

### Discipline

- **Cycles are caught**: A descended-from B descended-from A is detected
  via DFS white/gray/black coloring (ADR-005 Amendment 3 crash-resistance;
  ATK-A3-002)
- **Depth limit**: 64 levels by default, configurable via
  `[package.metadata.antigen]`
- **Orphaned references**: parents that no longer exist surface via
  `orphaned_lineage_edges()` query method

### See also

- ADR-008 (descended_from carrier)
- ADR-018 (propagation semantics + ProvenanceEntry + diamond dedup)
- [`usage-patterns.md`](usage-patterns.md) — inheritance patterns

---

## `#[antigen_tolerance(antigen_type, rationale = "...", until = "...")]`

Explicitly tolerate a fingerprint match the team has reviewed.

### Required arguments

- First positional: the antigen type the match was flagged against
- `rationale = "..."` (required) — narrative justification; required at parse time

### Optional arguments

- `until = "..."` (optional) — expiry date or condition (e.g., `"2026-12-31"`, `"v1.0"`)
- `see = [...]` (optional) — open-vocabulary cross-references (links to ADR, PR, issue)

### Applies to

Any Rust item that has been flagged by a fingerprint match the team
deliberately wants to retain.

### Example

```rust
use antigen::antigen_tolerance;
use crate::antigens::PolarityInvertedClassMeet;

// This test fixture deliberately constructs the inverted-polarity case
// to verify the fingerprint catches it.
#[antigen_tolerance(
    PolarityInvertedClassMeet,
    rationale = "Test fixture that deliberately constructs the inverted case \
                 to verify the fingerprint catches it. Vulnerability is the point.",
    until = "2026-12-31",
)]
#[test]
fn test_fingerprint_detects_inverted_meet() {
    // ... test body
}
```

### Behavior

`cargo antigen scan` recognizes tolerances and reports tolerated sites
separately from unaddressed presentations. The audit tracks tolerance
status, including stale tolerances (where the underlying fingerprint
match no longer surfaces).

### Discipline

- **Rationale is required at parse time** — an empty or missing rationale
  is a compile error (ADR-011)
- **Until clauses are not enforced automatically** — they
  surface in audit output for human review (future tooling may surface
  expired tolerances structurally)
- **Tolerance ≠ immunity**: tolerance acknowledges the failure-class is
  present and accepted; immunity claims the failure-class is structurally
  prevented. They are different lifecycle dispositions.

### See also

- ADR-011 (antigen_tolerance carrier with required rationale)
- [`usage-patterns.md`](usage-patterns.md) — when to tolerate vs defend
- [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) — tolerance placement

---

## Prescriptive / work-orchestration family

Eight macros that express code-site-local work-needs directly in the type system.
The thesis: a TODO comment rots; a `#[panel(needs = [...], filled_by = [...],
reviewed_by = [...], due = "...")]` stays current or emits a loud verdict when it
doesn't. Code IS the coordination board. `cargo antigen audit` renders per-site
verdicts (`Pending` / `Fulfilled` / `Overdue` / `OutOfFrame`) as a live-projected
board section — the same evaluator as defense, no parallel mechanism. (ADR-033)

### The four structural shapes

The eight macros route to four structural shapes (ADR-033 §Decision 1):

| Shape | Macros | What it models |
|---|---|---|
| **S1 Role-workflow** | `#[panel]`, `#[rx]`, `#[refer]`, `#[biopsy]` | An ordered set of who-steps to fill + review |
| **S2 Elimination** | `#[ddx]` | A set of alternatives to rule out one by one |
| **S3 Ordering** | `#[triage]` | A re-validatable priority order over code-site references |
| **S4 Frame-only** | `#[culture]`, `#[quarantine]` | A temporal window with an expiry |

Satisfaction uses the same witness leaves as defense: `signers()` / `signed_trailer()`
with `allowed_types`, fingerprint-pinned via NFA-21. Step-presence is verified (who-refs
are checked; satisfaction is order-agnostic).

### Verdict lattice

`WorkVerdict` is isomorphic to the defense tri-state with `false` temporally
partitioned:

| Verdict | Meaning |
|---|---|
| `Fulfilled` | Satisfaction met at current fingerprint |
| `Pending` | Declared and evaluable; satisfaction not yet met. Expected state — not loud |
| `Overdue` | Past the `due` frame and unsatisfied (and evaluable). Loud |
| `OutOfFrame` | Un-evaluable in the current substrate — unknown who-ref, unresolvable code-site reference, or unparseable date. NOT the same as `Overdue`; different intervention required |

---

### `#[panel(...)]` — ordered review battery (S1)

Marks a code site as carrying a diagnostic battery — a checklist the site's
reviewers must close. Biology: a clinical panel (a battery of tests ordered together,
closed by the reviewing clinician).

#### Arguments

| Field | Required | Purpose |
|---|---|---|
| `needs = ["...", ...]` | Yes (non-empty) | The battery's checklist items |
| `filled_by = ["who", ...]` | No | ADR-020 who-refs that fill the needs |
| `reviewed_by = ["who", ...]` | No | Who-refs that review the fills |
| `ordered_by = "who"` | No | Who-ref that ordered the battery |
| `due = "YYYY-MM-DD"` | No | ISO-8601 temporal frame |

#### Example

```rust
use antigen::panel;

#[panel(
    needs = ["review null-handling path", "verify error message copy"],
    filled_by = ["alice"],
    reviewed_by = ["bob"],
    due = "2026-09-01",
)]
pub fn parse_user_input(raw: &str) -> Result<Input, ParseError> {
    // ...
}
```

---

### `#[rx(...)]` — treatment prescription (S1)

Marks the remedy a site must carry out before it ships. Biology: a prescription — a
treatment ordered for a diagnosis, filled and reviewed.

#### Arguments

| Field | Required | Purpose |
|---|---|---|
| `treatment = "..."` | Yes (non-empty) | What must be done |
| `diagnosis = "..."` | No | Opaque diagnosis label |
| `filled_by = ["who", ...]` | No | ADR-020 who-refs |
| `reviewed_by = ["who", ...]` | No | Who-refs that review |
| `due = "YYYY-MM-DD"` | No | ISO-8601 frame |

#### Example

```rust
use antigen::rx;

#[rx(
    treatment = "Replace unwrap() with proper error propagation before v1 tag",
    filled_by = ["alice"],
    due = "2026-10-01",
)]
fn legacy_parse(s: &str) -> MyType {
    s.parse().unwrap()  // intentional tech-debt, now structurally tracked
}
```

---

### `#[refer(...)]` — specialist referral (S1)

Hands a work-need to an owner outside this site's immediate responsibility and anchors
the referral at the code site. Biology: a specialist referral — the referring clinician
hands off and awaits a response.

#### Arguments

| Field | Required | Purpose |
|---|---|---|
| `to = "who"` | Yes | ADR-020 who-ref for the external owner |
| `response_due = "YYYY-MM-DD"` | No | ISO-8601 frame for the response |

#### Example

```rust
use antigen::refer;

#[refer(to = "security-team", response_due = "2026-08-15")]
fn compute_signature(data: &[u8]) -> Vec<u8> {
    // security team needs to review the algorithm choice
    todo!()
}
```

---

### `#[biopsy(...)]` — deep investigation request (S1)

Marks a request to investigate a specific sub-site in depth. Biology: a biopsy —
sampling a specific location for deep analysis.

#### Arguments

| Field | Required | Purpose |
|---|---|---|
| `location = "..."` | Yes | Sub-site pointer (opaque label) |
| `request_text = "..."` | Yes (non-empty) | What to investigate |
| `deep_investigation_by = "who"` | No | ADR-020 who-ref |

#### Example

```rust
use antigen::biopsy;

#[biopsy(
    location = "line 47: hash collision path",
    request_text = "Confirm collision probability under adversarial input distribution",
    deep_investigation_by = "cryptography-reviewer",
)]
fn hash_input(data: &[u8]) -> u64 {
    // ...
}
```

---

### `#[ddx(...)]` — differential diagnosis (S2)

Marks a site where a symptom has multiple candidate causes, each to be independently
eliminated. Biology: differential diagnosis — the list of conditions to rule out.
Each alternative carries its own closing attestation; the set is closed when all are
eliminated.

#### Arguments

| Field | Required | Purpose |
|---|---|---|
| `symptom = "..."` | Yes (non-empty) | The observed problem |
| `rule_out = ["...", ...]` | Yes (non-empty) | The candidate causes to eliminate |
| `investigator = "who"` | No | ADR-020 who-ref |
| `reviewer = "who"` | No | ADR-020 who-ref |

#### Example

```rust
use antigen::ddx;

#[ddx(
    symptom = "intermittent deadlock observed in load test",
    rule_out = [
        "lock ordering inversion between MutexA and MutexB",
        "thread starvation from unbalanced work queue",
        "signal handler re-entering a non-reentrant path",
    ],
    investigator = "alice",
)]
fn acquire_resources(a: &Mutex<A>, b: &Mutex<B>) {
    // ...
}
```

---

### `#[triage(...)]` — priority ordering (S3)

Marks a site that carries a re-validatable priority order over a set of code-site
references. Biology: triage — ranking by urgency, re-assessed each round. Distinct
from `#[triage_commit]` (ADR-026 VCS-rollback classification) — names rhyme, surfaces
are unrelated (ATK-PRES-10).

#### Arguments

| Field | Required | Purpose |
|---|---|---|
| `priority_order = ["...", ...]` | Yes (non-empty) | Code-site refs in priority order |
| `triaged_by = "who"` | No | ADR-020 who-ref that attested the order |
| `re_triage_due = "YYYY-MM-DD"` | No | ISO-8601 staleness frame; standing order re-earned each cycle |

`priority_order` entries are **code-site references** (file/item-path). Unresolvable
entries are `OutOfFrame`, never silently satisfied.

#### Example

```rust
use antigen::triage;

#[triage(
    priority_order = [
        "src/auth.rs::validate_token",
        "src/session.rs::refresh_session",
        "src/audit_log.rs::flush_batch",
    ],
    triaged_by = "team-lead",
    re_triage_due = "2026-09-01",
)]
fn security_remediation_order() {}
```

---

### `#[culture(...)]` — time-boxed observation (S4)

Marks a site that must stay green within a temporal observation window. Biology:
a culture — incubate for a fixed period and read the result.

#### Arguments

| Field | Required | Purpose |
|---|---|---|
| `test_kind = "..."` | Yes (non-empty) | What is being observed/cultured |
| `duration = "..."` | No | Duration string |
| `runs_until = "YYYY-MM-DD"` | No | ISO-8601 expiry |

#### Example

```rust
use antigen::culture;

#[culture(
    test_kind = "soak test: memory usage stable under 24h continuous load",
    runs_until = "2026-07-15",
)]
fn process_stream(stream: impl Iterator<Item = Event>) {
    // ...
}
```

---

### `#[quarantine(...)]` — isolated region under a hold (S4)

Marks a region deliberately isolated until a named condition lifts. The `reason` is
required per ADR-005 Amendment 2 (rationale-as-required for every
suppression-shaped primitive). Biology: quarantine — isolate until cleared.

#### Arguments

| Field | Required | Purpose |
|---|---|---|
| `scope = "..."` | Yes | The isolated-region pointer |
| `until = "YYYY-MM-DD"` | No | ISO-8601 expiry |
| `reason = "..."` | Yes (non-empty) | Why the hold (ADR-005 Amd2) |

#### Example

```rust
use antigen::quarantine;

#[quarantine(
    scope = "experimental feature flag path",
    until = "2026-08-01",
    reason = "Awaiting RFC-0042 decision before stabilizing the API surface",
)]
mod experimental {
    // ...
}
```

---

### Compile-time validation (prescriptive family)

All eight macros validate their input at compile time for structural requirements:
required fields must be non-empty (a panel with no `needs` is a vacuous work-need;
a quarantine with no `reason` is a silent hold — both are compile errors). Unknown
fields are rejected at compile time.

Date fields (`due`, `until`, `runs_until`, `response_due`, `re_triage_due`) are
accepted as raw strings at compile time and validated at runtime by
`FrameState::classify()` during `cargo antigen audit`. A malformed date produces
`OutOfFrame` (the un-evaluable state) — it does not silently pass or fail; the
audit surface it explicitly.

### See also

- ADR-033 (prescriptive work-orchestration family — full decision)
- ADR-020 (who-refs and their resolution semantics)
- [`output-formats.md`](output-formats.md) — how WorkVerdict appears in audit output

---

## Compile-time validation

All five core macros validate their input at compile time:

| Failure | Detected by | Result |
|---|---|---|
| Missing required field | proc-macro parser | Compile error with span pointing to the macro invocation |
| Non-kebab-case `name` | proc-macro parser | Compile error |
| Empty fingerprint string | proc-macro parser | Compile error |
| Unknown attribute field | proc-macro parser | Compile error (forward-compat: unknown fields rejected) |
| Empty `rationale` on tolerance | proc-macro parser | Compile error |
| Macro applied to wrong item kind (e.g., `#[antigen]` on a non-unit struct) | proc-macro parser | Compile error |

Span-aware error messages point at the specific token that's wrong (W4
substrate; ADR-005 Amendment 3 §Mechanics §3 crash-resistance).

trybuild fixtures in `antigen-macros/tests/ui/` codify the compile-error
contracts.

---

## Common patterns

### Marking a site and registering its defense

```rust
// On the vulnerable site: declare only the shape it presents.
#[presents(MyAntigen)]
fn risky_function() { /* ... */ }

// On a test elsewhere: register the code-tier defense.
#[defended_by(MyAntigen)]
#[test]
fn my_test() { /* exercises the invariant */ }
```

Under ADR-029's observe-not-declare model the presents-marker and the defense are
*separate*: the site declares only the vulnerable shape, the defense evidence lives
on the witness, and `cargo antigen audit` cross-references them to report the per-site
verdict (`defended` / `undefended` / `substrate-gap`). The site never claims its own
immunity. (The removed `#[immune(..., witness=)]` form co-located a claim on the
site — see the removed-API section above.)

### Substrate-tier defense on the site

```rust
#[presents(MyAntigen, requires = signers(required = ["reviewer"]))]
fn governed_function() { /* ... */ }
```

When the defense is substrate state a test cannot execute — sidecar signers,
freshness, ratified docs — attach it as a `requires=` predicate on the presents-site
instead of a code-tier witness. See
[`usage-patterns.md`](usage-patterns.md#antigens-at-composition-boundaries).

### Tolerance for fixture-constructed cases

```rust
#[antigen_tolerance(
    MyAntigen,
    rationale = "Fixture constructs the vulnerable shape to test detection.",
)]
fn fixture_for_antigen_detection_test() { /* ... */ }
```

### Inheritance chain

```rust
#[antigen(...)] pub struct Generic;
#[antigen(...)] #[descended_from(Generic)] pub struct Specialized;
```

Witnesses on `Generic` may apply to `Specialized`; audit validates.

---

## What macros do NOT do

To prevent confusion:

- **They don't emit runtime code.** Identity transforms only. Zero runtime
  overhead; binary size and compile time are unaffected.
- **They don't validate witness existence.** Compile-time validation is
  syntactic only. Witness resolution happens at `cargo antigen audit` time.
- **They don't enforce naming consistency across crates.** Cross-crate
  identity uses `canonical_path` at `name@version` (ADR-017); the macros
  themselves only validate per-declaration syntax.
- **They don't expand into helper code.** No generated traits, no
  generated tests, no generated impls. The macros are pure pass-through.

---

## See also

- [`tutorial.md`](tutorial.md) — your first 15 minutes
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL
- [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) — placement conventions
- [`usage-patterns.md`](usage-patterns.md) — pattern recipes
- [`witness-tiers.md`](witness-tiers.md) — WitnessTier gradient semantics
- [`output-formats.md`](output-formats.md) — scan/audit output reference
- [`troubleshooting.md`](troubleshooting.md) — error diagnostics
- [`decisions.md`](decisions.md) — ratified ADRs

The macro implementations live in `antigen-macros/`. The crate-level
doc-comments in `antigen-macros/src/lib.rs` provide an alternative
view of the same surface oriented to `cargo doc` consumers.
