# Meta-findings: when a team's recurring drift becomes a typed antigen

> Audience: hardcore adopters running antigen on real work. This doc isn't
> needed for first-contact use — it's the discipline for the next layer up,
> once you've started noticing patterns *across* your antigens.

A **meta-finding** is a failure-class your team keeps catching across unrelated
places: not "this one code site has bug X," but "we keep finding the same
*shape* of bug in different parts of the codebase." Meta-findings are
opportunities. They're what antigen exists to capture, and the discipline of
turning a meta-finding into a typed antigen is replicable.

This doc teaches the loop.

---

## The loop

```
1. notice ── recurring drift between two surfaces (a comment promising
   │         enforcement nothing delivers; an enum mirrored as a const that
   │         drifted; a version string hand-copied across docs)
   │
2. name ──── articulate the shape: what *kind* of failure is this? what
   │         structurally do all the instances share?
   │
3. declare ─ author a #[antigen] with a category, a fingerprint that recalls
   │         the surface where the drift lives, and a name that captures the
   │         shape
   │
4. witness ─ design (or repurpose) a test that reads both drifted surfaces
   │         and asserts they agree — the test that *would have caught* the
   │         original drift if it had existed
   │
5. guard ─── the antigen now defends against future instances; the next
             time the shape recurs, the witness fails instead of drifting
             silently
```

Each step earns its place; skipping one collapses the loop:

- **Skip *notice*** and you don't have a meta-finding — just a single bug.
- **Skip *name*** and you can't tell whether the next case is the same class
  or a different one.
- **Skip *declare*** and the recognition is in tribal memory, which decays.
- **Skip *witness*** and the antigen is decorative — it names the class but
  doesn't defend.
- **Skip *guard*** is what happens when a witness exists but doesn't run in
  CI; the discipline is "the witness is structural, not aspirational."

> **The loop is linear on first encounter; iterative in practice.** A
> declaration's fingerprint typically gets refined over multiple instances —
> when a second adopter, a scout pass, or an adversarial sweep finds a site
> the original fingerprint missed, you're not failing the loop, you're in its
> *affinity-maturation* phase. See
> [`AntigenFingerprintDivergesFromClassExtension`](decisions.md) for the
> under-coverage / over-coverage refinement discipline.

---

## A worked example: `ParallelStateTrackersDiverge`

Antigen itself ran this loop in May 2026. Here's the full arc.

### Notice (the recurring drift)

The team noticed the same shape across three apparently-unrelated places:

- A **const** in a test file (`ADR025_AUDIT_HINTS`) listed kebab-case strings
  that "must match the `AuditHint` enum's serde keys" — but the test only
  checked the const against itself, and the enum had silently grown one
  variant past the const.
- Two **enums** named `WitnessTier` — one in `antigen-attestation`, one in
  `antigen::audit` — kept in manual lock-step "for serde stability." Their
  derives had silently drifted (one had `Hash`, the other didn't); the
  comment promising the lock-step enforced nothing.
- A **version string** (`=0.1.0-rc.2`) hand-copied across `README.md`,
  `docs/quickstart.md`, and `docs/tutorial.md`. The README pinned `rc.2`
  while the other two said `rc.3`. A reader hitting two of the docs got
  inconsistent install instructions.

### Name (the shape)

After the third instance, the shape was clear: *two hand-maintained
representations of the same fact, kept "in sync" by a comment rather than a
mechanism, drift silently.* A comment promising "must stay in sync" is a
promise of enforcement that delivers none. The drift is invisible until a
reader trusts the stale copy.

The team called the class **`ParallelStateTrackersDiverge`**.

### Declare (the antigen)

```rust
#[antigen(
    name = "parallel-state-trackers-diverge",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("lock-step")"#,
    family = "dogfood",
    summary = "Two hand-maintained representations of the same fact (a \
               mirrored enum, a const shadowing an enum's serde keys, a \
               version string copied across docs, a doc cross-reference) \
               are kept in sync by a comment rather than a mechanism, and \
               drift silently. A comment promising 'must stay in sync' \
               enforces nothing; only a test reading both sides catches \
               the divergence.",
    references = ["ADR-004", "ADR-028"]
)]
pub struct ParallelStateTrackersDiverge;
```

A few decisions are worth pulling out:

- **`category = SubstrateAlignment`** — the failure is a *representation*
  diverging from another *representation*. Per ADR-028, that picks the
  `requires =` witness channel rather than a code-witness. See
  [`witness-tiers.md`](witness-tiers.md) for the category↔witness mapping.
- **Fingerprint is broad and recall-tuned** — `doc_contains("lock-step")`
  matches the *comment surface* where the in-sync promise lives, not the
  specific drift sites. It's the same fingerprint-only pattern as
  `BiologyGroundingClaimDrift`: the *recall* is the comment phrase; the
  *precision* lives in the witness.
- **Family `"dogfood"`** — this antigen guards antigen's own substrate.
  Real adopter projects will declare their own equivalents in their own
  families.

### Witness (the test that would have caught it)

The canonical instance — the `ADR025_AUDIT_HINTS` const ↔ `AuditHint` enum
drift — gets the witness that closes that specific gap: a **bijection
test** that reads both sides and asserts they agree.

```rust
#[test]
fn adr025_audit_hints_const_matches_enum_serde_keys() {
    let supply_chain_variants: &[AuditHint] = &[
        AuditHint::UnpinnedDependency,
        AuditHint::UnpinnedTransitiveDependency,
        // … all sixteen
    ];

    // Forward: every variant's serde key is in the const.
    let mut variant_keys = Vec::new();
    for variant in supply_chain_variants {
        let key = serde_json::to_string(variant).unwrap();
        let key = key.trim_matches('"').to_string();
        assert!(
            ADR025_AUDIT_HINTS.contains(&key.as_str()),
            "variant serializes to '{key}' but the const lacks it"
        );
        variant_keys.push(key);
    }

    // Reverse: every const entry corresponds to a real variant key.
    for hint in ADR025_AUDIT_HINTS {
        assert!(
            variant_keys.iter().any(|k| k == hint),
            "const has '{hint}' but no variant serializes to it"
        );
    }

    // Bijection: the lengths must be equal — couples the count to the live
    // variant list so it can't drift on its own.
    assert_eq!(ADR025_AUDIT_HINTS.len(), supply_chain_variants.len());
}
```

Three assertions, three directions of drift each one closes:
- Forward catches **rename** (enum variant renamed, const has the old string)
- Reverse catches **dead entries** (const accumulated a string nobody emits)
- Bijection catches **add/remove** (variant count changed, const didn't follow)

The witness is the precision that the broad `doc_contains("lock-step")`
fingerprint hands off to.

### Guard (the antigen now defends)

With the antigen declared and the bijection test running in CI:

- A future variant added to `AuditHint` without being added to the const
  fails the forward direction at the next test run.
- A future variant *removed* without removing the const entry fails the
  reverse direction.
- The `count == 15` brittle assert is gone — replaced by the bijection,
  which can't drift independently.

The recurrence is structurally guarded. The next time something in the
codebase grows the `doc_contains("lock-step")` shape, the scan surfaces it
as a candidate site to consider for the same witness pattern.

---

## When *not* to declare

The cost of declaring an antigen for every observed bug is high — both in
substrate clutter and in adopter cognitive load. Recognition-not-design
(ADR-006) is the threshold:

- **Single instance ≠ meta-finding.** One bug of a shape is a bug. Two is a
  coincidence. Three or more, in genuinely-distinct places, is a class.
  Don't promote to a typed antigen on instance #1.
- **The shape must be *describable* in a fingerprint.** If you can't write
  even a broad recall fingerprint that catches the surface where the drift
  appears, the class isn't sharp enough to declare yet — keep observing.
- **The witness must be *writable*.** If you can describe the drift but
  can't write a test that catches it, the antigen would be decorative.
  Either wait for the witness to become tractable, or pick a narrower
  class you *can* witness.
- **Beware over-coverage.** A fingerprint that matches a thousand
  unrelated sites is recall noise — it teaches adopters the tool cries
  wolf. Tighten before declaring.
  ([`AntigenFingerprintDivergesFromClassExtension`](decisions.md) covers
  the under-coverage / over-coverage trade in detail.)

A useful threshold question: *would a future contributor, hitting an
instance of this class, benefit from the fingerprint flagging it before
review?* If yes, declare. If the class is too vague to be helpful at the
moment of flagging, hold.

---

## The pattern tiles upward: code, then team coordination

`ParallelStateTrackersDiverge` was first noticed in code (consts, enums,
version strings). But the *same shape* recurs at the coordination layer —
when two views of the same fact are kept in sync by convention rather than
mechanism:

- A teammate's stated belief about what's committed vs the actual `git
  show` output.
- A routing decision's framing vs the substrate-recorded ruling.
- A "we agreed X" assertion vs the campsite log of who actually agreed.

Not by coincidence: this is the *generation-outpaces-inspection asymmetry*
tiling upward. The asymmetry is scale-invariant — its three faces (detection,
retention, verification, per [`vision-pitch.md`](vision-pitch.md)) recur at
every rung where generation outpaces inspection of itself. Code-defects need
structures outside the generating act because the generating act can't
inspect itself; multi-agent coordination needs structures outside the
coordinating-act for the same reason. The *same structural cause* produces
each rung's `ParallelStateTrackersDiverge`, which is why the same loop
applies: notice, name, design a substrate-level check (a campsite review,
a git-log gate, a pre-routing substrate-grep), guard. The witness moves from
"test that reads two surfaces" to "discipline that reads two surfaces" — but
the shape, and the loop, are the same because the asymmetry is the same.

This is part of why `ParallelStateTrackersDiverge` lives in the dogfood
family: it's a class that crosses the code/coordination boundary, and the
discipline that catches it crosses with it.

---

## See also

- [`decisions.md`](decisions.md) — ADR-006 (recognition-not-design),
  ADR-028 (antigen-category taxonomy)
- [`testing-patterns.md`](testing-patterns.md) — witness conventions, the
  "test that reads both sides" pattern
- [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) —
  placement rules for the marker (type-level only)
- The `ParallelStateTrackersDiverge` declaration itself, in
  `antigen/src/stdlib/dogfood.rs`
