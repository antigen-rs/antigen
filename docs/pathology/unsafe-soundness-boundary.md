# Pathology case-file — Unsafe-Soundness-Boundary

> The deep tier of the catalog row. Read the catalog entry first:
> [`../stdlib-families.md` § Unsafe-Soundness-Boundary](../stdlib-families.md#unsafe-soundness-boundary).
> The source docstring in
> [`../../antigen/src/stdlib/unsafe_soundness.rs`](../../antigen/src/stdlib/unsafe_soundness.rs)
> is ground truth; this file mirrors it.

- **Family**: Unsafe-Soundness-Boundary
- **Category**: `FunctionalCorrectness`
- **Members**: `TransmuteSizeOrLifetimeMismatch` (**named**),
  `UninitMemoryAssumedInit` (**named**),
  `UnvalidatedFromUtf8Unchecked` (**named**)
- **Scan fixture** (no runnable example — see below):
  [`../../antigen/tests/fixtures/family_unsafe_soundness/lib.rs`](../../antigen/tests/fixtures/family_unsafe_soundness/lib.rs)

---

## Presentation

Three `unsafe`-primitive call-shapes, reachable from safe-looking code, where a
wrong invariant is **Undefined Behavior** rather than a panic:

- **`TransmuteSizeOrLifetimeMismatch`** — a `mem::transmute` / `transmute_copy`
  reinterprets bytes as another type. With a wrong layout, a shortened lifetime, or
  an added `mut` (`&T → &mut T`), it's instant UB. The developer sees nothing wrong
  at the call; the corruption surfaces elsewhere.
- **`UninitMemoryAssumedInit`** — `MaybeUninit::assume_init` / `mem::uninitialized`
  reads memory as a valid value before it's initialized. Treating uninitialized
  bytes as a valid value is instant UB.
- **`UnvalidatedFromUtf8Unchecked`** — `str::from_utf8_unchecked` / `_mut` builds a
  `str` from bytes without the UTF-8 validity check. A `str` holding invalid UTF-8
  is UB, and *every downstream `str` operation may misbehave*.

In all three, the symptom is the soundness-hole signature: no panic, no immediate
error, just UB that the safe/unsafe boundary was supposed to keep out.

## Etiology

The mechanism: each primitive crosses the safety membrane with an unchecked
invariant. `unsafe` is the explicit "I'm crossing the safety membrane, trust me" —
and these are the call-shapes where a wrong "trust me" is UB, not a recoverable
error. The defect is the *wrong invariant*, not the call itself: a correct
transmute / `assume_init` / `from_utf8_unchecked` is sound; a wrong one is UB.

Biology cognate (as real mechanism): the **breached self/non-self membrane**. The
cell membrane (and the MHC markers on it) is how the body distinguishes self from
non-self; `unsafe` is the deliberate membrane crossing. A wrong `unsafe` invariant
is a **forged MHC marker** — a foreign cell passing as self, admitted through the
membrane it should have been excluded by. This rhymes hard with the
Mucosal-Boundary family: it is mucosal-boundary applied to the *memory-safety*
membrane.

## Epidemiology

Real-world recorded references — cite only what the source actually references:

- **`TransmuteSizeOrLifetimeMismatch`** — rustc **`mutable_transmutes`**
  (deny-by-default — `&T → &mut T` is UB) and **`wrong_transmute`**; clippy
  **`unsound_collection_transmute`** and **`transmute_null_to_fn`**. Std reference:
  `https://doc.rust-lang.org/std/mem/fn.transmute.html`.
- **`UninitMemoryAssumedInit`** — clippy **`uninit_assumed_init`** and
  **`uninit_vec`**; `mem::uninitialized` is **deprecated *because* it is almost
  always UB**. Std reference:
  `https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.assume_init`.
- **`UnvalidatedFromUtf8Unchecked`** — rustc **`invalid_from_utf8_unchecked`**. Std
  reference: `https://doc.rust-lang.org/std/str/fn.from_utf8_unchecked.html`.

These are rustc/clippy lint names and std docs — the prior-art the source cites.
No RUSTSEC/CVE advisory IDs are claimed for this family (none would be honest to
invent); the source's prior-art is "RUSTSEC's informational=unsound advisories +
rustc's deny-by-default unsafe lints" as a *category*, plus the named lints above.

## Histology — the fingerprints, annotated

### `TransmuteSizeOrLifetimeMismatch`

```text
any_of([
    body_calls("transmute"),
    body_calls("transmute_copy"),
])
```

- Either last-segment call (`transmute` / `transmute_copy`) suffices. The presence
  is the named tell — `transmute` is `mem`-specific, a rare self-anchor no domain
  type collides with. The fingerprint does not claim the precise
  size/lifetime/mutability check that separates a sound `transmute` from an unsound
  one; that deeper check is a recorded graduation path — see
  [`../roadmap.md`](../roadmap.md).

### `UninitMemoryAssumedInit`

```text
any_of([
    body_calls("assume_init"),
    body_calls("uninitialized"),
])
```

- Both are rare/std-specific self-anchors with **no common safe namesake** (you
  don't name a safe method `assume_init`), so they spare the namesake-clean case.

What was **dropped**, and why (ADR-039 §C Amendment 1, the spares-namesake
sub-test) — both genuinely out, not hidden:

- **`zeroed` → DROPPED at every tier.** The `zeroed` last-segment fires on
  `bytemuck::zeroed()` / `Zeroable::zeroed()` — the **recommended-safe**,
  trait-gated replacement for `mem::zeroed`. A needle that flags the recommended
  remediation is inadmissible at any tier (the clean-sibling collision), so the arm
  is dropped, not demoted.
- **`set_len` → DROPPED from this named member.** `set_len` fires on any domain
  buffer/builder's `.set_len(n)`, not only the unsafe `Vec::set_len`-on-uninit, and
  there is **no AST-feasible discriminator**: risky-vs-safe turns on the **receiver
  type** (`Vec` vs a domain buffer) *and* the **arg value** (`new_len ≤
  initialized`), neither of which is syntactic. The recall hole (an unsafe
  `Vec::set_len`-on-uninit is not flagged here) is documented, not silently absorbed
  (a dedicated `set_len` member is a recorded charter — see the prognosis below).

### `UnvalidatedFromUtf8Unchecked`

```text
any_of([
    body_calls("from_utf8_unchecked"),
    body_calls("from_utf8_unchecked_mut"),
])
```

- Either last-segment call suffices. A rare/std-specific self-anchor; the member
  fires on the call's presence, not on a "were the bytes validated?" check.

## Differential — why all three are named

- **The effective-codomain rule.** Every needle here is a **rare / std-specific**
  unsafe primitive (`transmute`, `assume_init`, `from_utf8_unchecked`, …) — a domain
  type will not define a method by that name, so the needle alone restricts the
  codomain to the defect population (the self-anchor rule). That's *why all three
  are named*: the presence of the call is itself the high-confidence signal.
- **Named = call-presence, not the semantic check.** The presence of the call is
  what the scanner reads; the member does **not** claim to have verified the precise
  size/lifetime/validity invariant. The tier reflects the *fingerprint shape* (a rare
  self-anchoring call), not a claim that antigen has verified the invariant.
- **Where the line is drawn (`UninitMemoryAssumedInit`)**: the `zeroed` and
  `set_len` arms were considered and rejected precisely because they break the
  effective-codomain rule (clean-sibling collision / no AST-feasible discriminator).
  Keeping them out is what preserves the named tier's honesty.

## Treatment — the witness

`present ≠ vulnerable`. Each call presents a soundness assertion in this family's
territory; the witness proves the invariant.

- **`TransmuteSizeOrLifetimeMismatch`** — a documented layout guarantee
  (`#[repr(...)]`) + a miri run, OR the transmute is replaced by a checked
  conversion.
- **`UninitMemoryAssumedInit`** — a `// SAFETY:` proving full initialization before
  the read, OR miri/kani.
- **`UnvalidatedFromUtf8Unchecked`** — the bytes were validated (or are a known-UTF-8
  constant), proved by a `// SAFETY:` + a check / miri.

## Prognosis

All three members are at the **named** tier. The fingerprint fires on the *presence*
of the unsafe call; it deliberately does not claim the precise size/lifetime/validity
check that distinguishes a sound use from an unsound one. That deeper check, and the
dropped `set_len` risky form (a separate documented recall hole), are recorded
graduation paths — see [`../roadmap.md`](../roadmap.md).

> **Why a scan fixture, not a runnable example** (same reason as async-soundness):
> every tell is a real `unsafe` primitive (`transmute` / `assume_init` /
> `from_utf8_unchecked`), which `unsafe_code = "forbid"` blocks from a compiled
> crate — so the three affinity-pairs live in a fixture you **scan**:
>
> ```sh
> cargo run --bin cargo-antigen -- antigen scan --root antigen/tests/fixtures/family_unsafe_soundness
> ```
>
> Scan reports **six** presentations — the bad path **and** the safe sibling of each
> of the three members — because this fixture `#[presents]`-marks *both* siblings
> (unlike the async fixture's un-marked spare). Read the BAD/GOOD pairs in the
> *source* for the fingerprint difference (`transmute` vs an `as` cast;
> `assume_init` vs a plain value; `from_utf8_unchecked` vs the checked `from_utf8`);
> the console lists both, per the "word on spared" caveat in the catalog.

---

## See also

- [`../stdlib-families.md`](../stdlib-families.md) — the catalog row
- [`async-soundness.md`](async-soundness.md) — the other scan-fixture family
- [`panic-on-index.md`](panic-on-index.md) — `get_unchecked` belongs to both that
  family and this soundness boundary
- [`../fingerprint-grammar.md`](../fingerprint-grammar.md) — the fingerprint DSL
- [`../witness-tiers.md`](../witness-tiers.md) — the tier gradient
- [`../../antigen/src/stdlib/unsafe_soundness.rs`](../../antigen/src/stdlib/unsafe_soundness.rs)
  — the source docstring (ground truth)
