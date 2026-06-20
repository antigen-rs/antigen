# Case Study — the determinism-class polarity inversion

> The founding incident, told as a focused case study. A polarity-inverted
> lattice `meet` shipped in a computational-mathematics project, was caught and fixed, and then **arrived
> again, independently, months later** in a structurally-identical new enum. This
> is the bug antigen was invented to make un-repeatable — and the antigen that
> would have caught the second occurrence before it was written.
>
> For the full founding narrative (the originating observation, the frame shift to immune-
> system architecture), see [`origin.md`](origin.md). For the generic
> "same bug twice" pattern told with a simplified enum, see
> [`case-study.md`](case-study.md). This file is the **specific** determinism-class
> story, told in the current ADR-029 idiom.

---

## The codebase

The originating project is a Windows-native,
GPU-accelerated mathematical computing toolkit. Its operations are classified by
a **`DeterminismClass`** — how reproducibly an operation produces identical
bit-level results across hardware, compiler versions, and optimization levels:

- `BitExact` — bit-identical everywhere; the **strongest** claim
- `MathematicallyEquivalent` — equal at infinite precision; rounding may differ
- `ArchConditional` — depends on ISA features (e.g. FMA fused vs. separated)
- `ChoiceContingent` — depends on a runtime user choice; the **weakest** claim

These form a partial order under *strictness*, and the lattice supports a
**`meet`** operation: given two classes, `meet` returns the strongest class that
holds along **both** axes simultaneously — which is the **weaker** of the two
inputs. `BitExact ⊓ ArchConditional = ArchConditional`.

Here is the trap, in one sentence: **the enum's discriminant ordering is
strongest-first, so the strongest class has the *smallest* discriminant — which
is the reverse of the lattice ordering.**

```rust
enum DeterminismClass {
    BitExact,                 // discriminant 0 — STRONGEST
    MathematicallyEquivalent, // 1
    ArchConditional,          // 2
    ChoiceContingent,         // 3 — WEAKEST
}
```

If you glance at a strongest-on-top lattice diagram and reach for
`meet = std::cmp::min`, you get `BitExact` — the *strongest*, exactly backwards.
The lattice `meet` of strongest-first discriminants is **`max`**, not `min`.

---

## The first illness

An early implementation of `DeterminismClass::meet` shipped `meet = min`. It
looked correct ("smaller is weaker, right?"). Tests passed — they exercised some
cases but missed the boundary. Code review didn't catch it.

The failure is the worst kind: **silent over-promising of a safety property.**
`meet` returned the *strongest* of two classes when it should have returned the
weakest, so the system claimed *more* determinism than actually held. Adversarial
testing surfaced it.

The fix was one line — `meet = max` — and the enum was redesigned around it. The
lesson was learned, and it was specific and structural:

> When a class enum represents strength-of-claim, and discriminants are ordered
> strongest-first, the lattice `meet` is `max`, not `min` — because the lattice
> ordering is reverse-strictness while the discriminant ordering is
> forward-strictness.

That lesson lived in the team's memory, in the issue tracker, in the fix
commit, and in a docstring on `meet()`. **It did not live in any structural
artifact that would propagate to new code.**

---

## The second illness

Months later, the same project was ratifying a symbolic refinement-lattice that
introduced a *new* class enum, `CommutativityClass`:

```rust
enum CommutativityClass {
    Strict,             // 0 — strongest
    RoundingEquivalent, // 1
    ArchConditional,    // 2
    ChoiceContingent,   // 3 — weakest
}
```

**Structurally identical** to `DeterminismClass`: strongest-first discriminants,
lattice ordering reverse to discriminant ordering, the same `meet` polarity
question. The new draft — written by a *different* agent than the one who fixed
the original bug — specified `meet = std::cmp::min`.

Same shape. Same trap. **Independently arrived at, independently wrong.**

### The catch — expensive, by hand

What stopped the second illness was **not** memory of the first fix; that memory
was not structurally accessible. What stopped it was *re-deriving the lesson from
scratch*. The math-researcher traced a worked example by hand and noticed the
polarity was wrong. The implementer, about to write the code, paused: their
mental model said `min`, but the substrate-of-record said `max` — and they caught
the inversion before any code went down.

The fix shipped. Same lesson as the first time, re-derived months later by
different agents.

The catching took real engineering: a manual trace, a mental-model check against
substrate, a re-deconstruction of *why* polarity matters. It only worked because
the team was disciplined, multi-agent, and adversarially paranoid.

> In a less-disciplined team — or a fresh-context single agent, or a human team
> without the first incident in lived memory — **the inversion ships.**

The lesson had been learned *once*. The system had been healed *once*. The
illness came back because **the healing didn't propagate.** That observation —
*corrected designs don't carry the failure that motivated them* — is the
observation that became antigen.

---

## The antigen that would have caught it

If antigen had existed when the original bug was first fixed, the fix would have
*also* produced a structural declaration of the failure-class:

```rust
// src/antigens.rs
use antigen::antigen;

/// Class enums with strongest-first discriminants must use `max` (not `min`)
/// for lattice meet. The discriminant ordering convention determines whether
/// `meet` is `max` or `min`; misalignment silently over-promises safety.
#[antigen(
    name = "polarity-inverted-class-meet",
    family = "frame-translation",
    fingerprint = r#"
        item = enum,
        name = matches("*Class"),
        has_method("meet", "(self, Self) -> Self")
    "#,
    summary = "Class enums with strongest-first discriminants must use max for \
               meet; misalignment silently over-promises safety properties.",
    references = [
        "issue:#142",
        "issue:#318",
        "post-mortem:polarity-inversion",
    ],
)]
pub struct PolarityInvertedClassMeet;
```

The fingerprint structurally matches *any* enum named `*Class` with a `meet`
method of the right signature shape. When the second draft introduced
`CommutativityClass`, `cargo antigen scan` would have recognized the fingerprint
and **flagged the new enum automatically** — no human re-derivation, no
multi-agent rescue. The healing would have propagated.

---

## Defending the now-fixed sites (current ADR-029 idiom)

Once the antigen is declared, the team marks the fixed sites and registers the
witness that proves the polarity. Under ADR-029 (*observe-don't-declare*),
**immunity is observed by the audit, not declared at the code site** — so the
site carries `#[presents]` and the *test* carries `#[defended_by]`:

```rust
use antigen::presents;

// Each fixed enum presents the structural shape of the failure-class.
#[presents(PolarityInvertedClassMeet)]
pub enum DeterminismClass {
    BitExact, MathematicallyEquivalent, ArchConditional, ChoiceContingent,
}

#[presents(PolarityInvertedClassMeet)]
pub enum CommutativityClass {
    Strict, RoundingEquivalent, ArchConditional, ChoiceContingent,
}
```

```rust
use antigen::defended_by;
use proptest::prelude::*;

// One property test defends the whole class: meet must return the LARGER
// discriminant (the weaker class) across the full enum space. #[defended_by]
// registers what this test defends; the audit observes that it covers the sites.
proptest! {
    #[test]
    #[defended_by(PolarityInvertedClassMeet)]
    fn class_meet_returns_max(a in 0u8..4, b in 0u8..4) {
        let m = DeterminismClass::from(a).meet(DeterminismClass::from(b));
        prop_assert!((m as u8) >= a && (m as u8) >= b); // meet = max of discriminants
    }
}
```

`cargo antigen audit` then reports each presents-site as **defended** (at
Reachability tier — the witness is *wired* to the site, not run). One witness registered with
`#[defended_by]` covers every `#[presents(PolarityInvertedClassMeet)]` site of
that class — defense is class-granular, so the single property test defends both
enums.

---

## What changes for the third occurrence

Six months on, a developer (human or LLM) adds a third class enum:

```rust
pub enum NumericalStabilityClass {
    StablyBounded,          // 0 — strongest
    StableInPractice,       // 1
    PathologicallyUnstable, // 2 — weakest
}

impl NumericalStabilityClass {
    pub fn meet(self, other: Self) -> Self {
        // is this max or min?
    }
}
```

Three things happen automatically:

1. **`cargo antigen scan` flags it** — the new enum matches the
   `PolarityInvertedClassMeet` fingerprint and appears as a candidate site.
2. **The developer reads the antigen declaration** — the `summary` names the
   failure-class; the `references` link to the original issues and the
   post-mortem. They learn the lesson *before* writing the bug.
3. **An LLM agent reading the codebase sees the antigen surface** and either
   writes `meet` correctly the first time or surfaces the known pattern to its
   human.

The lesson — once tribal, then archived, then drifted — is now **structural**. It
survives the original author leaving, the Slack channel archiving, the wiki
drifting, and the LLM cycling through sessions.

---

## What this case does — and doesn't — demonstrate

**It does** demonstrate the most under-served move in software-engineering
practice: making the lesson learned in *one* fix transfer structurally to *all*
future structurally-similar sites, without relying on human memory or LLM context
to carry it.

**It does not** claim more than is true:

- Antigen didn't write the original boundary tests that would have caught
  the bug the *first* time. Better tests would have. Antigen names the
  pattern so the lesson persists.
- Antigen didn't catch `CommutativityClass` before it shipped — the *team* caught
  it, by hand. Antigen's value is preventing the *third* occurrence preemptively.
- Antigen will not catch a novel logic error that doesn't structurally resemble a
  *declared* failure-class. It catches *named* shapes; novel bugs are still
  tests' job.

The case is small. The pattern — *failure-classes recur structurally faster than
human memory and drifting documentation can catch them* — is universal. That is
why it matters, and why it is the case antigen was built around.

---

## See also

- [`origin.md`](origin.md) — the full founding narrative (the originating observation, the
  frame shift to immune-system architecture)
- [`case-study.md`](case-study.md) — the "same bug twice" pattern told generically
- [`immune-migration-guide.md`](immune-migration-guide.md) — converting old
  `#[immune]` sites to the ADR-029 idiom used above
- [`composition.md`](composition.md) — how the proptest witness composes under
  the shared antigen vocabulary
- [`glossary.md`](glossary.md) — `antigen`, `presents`, `defended_by`, `witness`,
  `fingerprint`

---

*The lesson was learnable once. The system was healed once. Antigen is how the
healing propagates — so the third illness is cured before it appears.*
