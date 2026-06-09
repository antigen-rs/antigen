# Antigen — A Case Study

> A real failure-class, narrated end-to-end: how it surfaced, how
> antigen names it, how the lesson lives on. This is the user-facing
> companion to [`tutorial.md`](tutorial.md) — less "follow these
> steps" and more "here's what actually happened, and what changed
> after antigen."

The codebase is **tambear**, a Rust mathematical-determinism library
that became antigen's origin project. The failure-class is real; the
fix was real; the lesson was nearly lost to commit history before the
project that became antigen captured it structurally.

For the internal post-mortem narrative (richer detail), see
[`origin.md`](origin.md).

---

## The codebase

Tambear is a library for determinism-class lattices in numerical
computation. Operations on values carry a *class* — a marker for the
strength of guarantee the operation provides. The lattice has
operations like:

- **`meet`** — combine two classes; returns the *weaker* of the two
  (lattice meet)
- **`join`** — combine two classes; returns the *stronger* of the two
  (lattice join)

A `meet` says: "if I combine a strict-determinism result with a
loose-determinism result, the combined result is loose-determinism."
The weaker bound wins.

The enum looks like this:

```rust
pub enum DeterminismClass {
    Strict,   // discriminant 0 — strongest determinism
    Loose,    // discriminant 1 — medium
    LooseFp,  // discriminant 2 — weakest determinism
}
```

The discriminant ordering goes *strongest-first*. Strict is 0; LooseFp
is 2.

Note where this is going.

---

## The first bug

Someone wrote `meet`:

```rust
impl DeterminismClass {
    pub fn meet(self, other: Self) -> Self {
        // wrong: returns the smaller discriminant
        if (self as u8) < (other as u8) { self } else { other }
    }
}
```

Tests passed (they tested some cases but missed the boundary). Code
review didn't catch it. The function shipped.

Months later, a bug report came in: numerical results were stronger
than they should be when combining strict and loose values. Production
behavior was *promising more determinism than actually held*. Subtle;
expensive to track down.

The root cause: the author used `min` of the discriminants, thinking
"smaller is weaker." But the discriminants were ordered
*strongest-first*, so `Strict` (0) was numerically smallest. `meet`
should return `max` of discriminants, not `min`, because the larger
discriminant is the *weaker* class.

One-line fix:

```rust
pub fn meet(self, other: Self) -> Self {
    if (self as u8) > (other as u8) { self } else { other }
}
```

Reviewed. Merged. Done. The bug had a CVE-shaped form (silent
overpromising of safety properties); the team's post-mortem was
thoughtful.

The lesson — **"discriminant ordering convention determines whether
`meet` is `max` or `min`; misalignment silently overpromises"** —
went into:

- The commit message (`fix: meet should be max not min — see #892`)
- A Slack thread
- A code-review comment template
- One paragraph in the team's wiki

---

## The second bug

Six months later, the team added `CommutativityClass`:

```rust
pub enum CommutativityClass {
    StrictlyCommutative,   // 0 — strongest
    LooselyCommutative,    // 1
    NonCommutativeWithFix, // 2 — weakest
}

impl CommutativityClass {
    pub fn meet(self, other: Self) -> Self {
        // exact same shape — exact same bug
        if (self as u8) < (other as u8) { self } else { other }
    }
}
```

The lesson from `DeterminismClass` did not transfer.

**Why?**

- The original author of `DeterminismClass.meet` had left the team
- The post-mortem from six months ago was in a Slack channel that had
  archived
- The wiki page describing the lesson had drifted as the codebase
  evolved
- The commit message was a few thousand commits back; nobody read
  ancient git logs when writing new enums
- Code review caught some things but didn't catch this specific
  structural pattern; the reviewer wasn't *primed* to look for it

The team caught the second bug in pre-release testing this time. But
they noticed something important: **the same structural failure-class
had landed twice**, and the team had no structural way to catch the
*third* occurrence (which would be a different enum with the same
strongest-first discriminant convention).

That noticing is what produced antigen.

---

## What antigen does here

The team declares an antigen for the failure-class:

```rust
// src/antigens.rs

use antigen::antigen;

/// Class enums with strongest-first discriminants must use max (not min)
/// for lattice meet. The discriminant ordering convention determines whether
/// `meet` is `max` or `min`; misalignment silently overpromises safety.
#[antigen(
    name = "polarity-inverted-class-meet",
    family = "frame-translation",
    fingerprint = r#"
        item = enum,
        name = matches("*Class"),
        has_method("meet", "(self, Self) -> Self")
    "#,
    summary = "Class enums with strongest-first discriminants must use max for meet; \
               misalignment silently overpromises safety properties.",
    references = [
        "issue:tambear/tambear#892",
        "issue:tambear/tambear#1247",
        "internal:post-mortem-2026-05-06",
    ],
)]
pub struct PolarityInvertedClassMeet;
```

The fingerprint structurally matches any enum named `*Class` that has
a `meet` method with the right signature shape. `cargo antigen scan`
walks the codebase and reports every such enum — including
`DeterminismClass`, `CommutativityClass`, and *any future enum
following the same pattern*.

For the now-fixed sites, the team records the defense in two parts: a
`#[presents]` marker on each vulnerable site, and a `#[defended_by]`
witness on the test that proves the fix. The marker declares the site
*presents* the failure-class; the witness registers the proof.

```rust
#[presents(PolarityInvertedClassMeet)]
pub enum DeterminismClass { Strict, Loose, LooseFp }

#[presents(PolarityInvertedClassMeet)]
pub enum CommutativityClass {
    StrictlyCommutative, LooselyCommutative, NonCommutativeWithFix
}

/// Proptest exercises both enums' `meet` across the full discriminant
/// space, asserting it returns the larger (weaker) class — the property
/// the polarity bug violated.
#[defended_by(PolarityInvertedClassMeet)]
#[test]
fn class_meet_returns_max_proptest() {
    // proptest body: for all (a, b), meet(a, b) == max-by-discriminant
}
```

`cargo antigen audit` now reports both presentations as **defended** —
the `#[defended_by]` witness binds the `PolarityInvertedClassMeet`
class, verified to exist in the workspace.

---

## What changes

Now consider what happens six months from now when a new developer
(human or LLM) adds a third class enum:

```rust
pub enum NumericalStabilityClass {
    StablyBounded,         // 0 — strongest
    StableInPractice,      // 1
    PathologicallyUnstable // 2 — weakest
}

impl NumericalStabilityClass {
    pub fn meet(self, other: Self) -> Self {
        // is this max or min?
        // ...
    }
}
```

Three things happen automatically:

1. **`cargo antigen scan` flags it.** The new enum matches
   `PolarityInvertedClassMeet`'s fingerprint. The scan output says:

   ```
   src/stability.rs:42  PolarityInvertedClassMeet on enum [fingerprint match]

   To acknowledge each site, use the antigen type shown above:
     #[presents(<class>)]                              (mark it explicitly)
     #[presents(<class>)] + #[defended_by(<test>)]     (record a defense)
     #[antigen_tolerance(<class>, rationale = "...")]  (accept it)
   ```

2. **The new developer reads the antigen declaration.** The `summary`
   tells them the failure-class: "must use max for meet; misalignment
   silently overpromises." The `references` link to the two prior
   issues + the internal post-mortem. They learn the lesson before
   writing the bug.

3. **An LLM agent generating code for `meet` sees the antigen surface.**
   If they read the antigen declaration (per the protocol in
   [`for-llm-collaborators.md`](for-llm-collaborators.md)), they
   recognize the pattern and either:
   - Write `meet` correctly the first time (using max)
   - Surface to the human: "this is a known failure-class pattern;
     here's the antigen declaration; here's how the prior sites
     defended"

The lesson — once tribal, then archived, then drifted — is now
structural. It survives the original author leaving, the Slack channel
archiving, the wiki drifting, the LLM cycling through sessions.

---

## What this actually demonstrates

The case is small. The pattern matters.

**Before antigen**: the team relied on developer memory + git archaeology
+ tribal knowledge + maintained wiki pages. All carriers are
maintenance-tier; all drift. The same bug shipped twice.

**After antigen**: the lesson lives in code, alongside the failure-class
shape that distinguishes it. The carrier is structural-tier; drift is
caught at scan time, not weeks later in production.

**The vocabulary travels**: the antigen declaration is itself a kind of
documentation. An LLM agent six months from now (or a human team
member who joined yesterday) can read the declaration, follow the
references, and inherit the team's accumulated failure-class memory
in seconds. No fine-tuning required. No specialized training.

**Witness pluralism does the verification work**: the proptest witness
checks the actual `meet` behavior across the enum space. Antigen
doesn't reinvent verification; it threads existing verification
(proptest, tests, clippy, kani, phantom types) under a shared
vocabulary of *what's being defended against*.

---

## What antigen does NOT do here

It's worth being honest about what's outside antigen's reach:

- **Antigen didn't write the original tests** that would have caught
  the bug. Better tests would have. Antigen named the pattern after
  the fact so the lesson persists.
- **Antigen didn't catch the second occurrence** (`CommutativityClass`)
  *before* the bug shipped. The team caught it via pre-release
  testing, then declared the antigen to prevent the third.
- **Antigen will not catch novel logic errors** that don't structurally
  resemble a declared failure-class. If a future enum has a bug in
  some completely different shape, antigen won't surface it until
  someone declares an antigen for that shape.

What antigen *does* is the most under-served category in software-
engineering practice: **make the lesson learned in one fix transfer
structurally to all future structurally-similar sites**, without
relying on human memory or LLM context to carry it.

---

## What you can do now

If your codebase has had "the same bug twice" — write the antigen
declaration. The smallest viable form takes two fields:

```rust
#[antigen(
    name = "my-failure-class",
    fingerprint = r#"item = fn, name = matches("dangerous_*")"#,
)]
pub struct MyFailureClass;
```

Then add `#[presents(MyFailureClass)]` at the vulnerable site and
`#[defended_by(MyFailureClass)]` on the test that proves it safe.
Run `cargo antigen scan` and `cargo antigen audit`. The lesson is now
structural.

For the full walkthrough, see [`tutorial.md`](tutorial.md).
For pattern recipes, see [`usage-patterns.md`](usage-patterns.md).

---

## The deeper claim

Tambear's `DeterminismClass` / `CommutativityClass` pair is one
specific failure-class. But the *pattern of the pattern* — "the same
structural failure shipped twice, and would have shipped a third
time without intervention" — recurs across every long-lived codebase
the world has ever seen.

Antigen names this pattern: **failure-classes recur structurally
faster than human memory + tribal knowledge + drifting documentation
can catch them**. The recurrence is statistical. The defense, until
antigen, was discipline (which decays) or tooling specific to each
pattern (which doesn't scale).

Antigen ships the vocabulary that lets each team capture their own
failure-class memory structurally. The vocabulary is universal; the
declarations are specific to your domain.

The case study is small. The pattern is universal. That's why this
matters.

---

## See also

- [`tutorial.md`](tutorial.md) — your first 15 minutes (follow-along)
- [`concepts.md`](concepts.md) — what antigen IS, architecturally
- [`origin.md`](origin.md) — the full founding-incident narrative
  (richer detail than this case study)
- [`usage-patterns.md`](usage-patterns.md) — common patterns
- [`composition.md`](composition.md) — antigen + your existing tools
- [`scope.md`](scope.md) — comprehensive vision
- [`for-llm-collaborators.md`](for-llm-collaborators.md) — the LLM
  protocol that lets an AI agent read antigen declarations and
  respect them natively

---

*The case study is real. The pattern is real. The lesson is now
structural.*
