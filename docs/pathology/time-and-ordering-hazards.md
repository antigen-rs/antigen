# Pathology case-file — Time-and-Ordering-Hazards

> The deep tier of the catalog row. Read the catalog entry first:
> [`../stdlib-families.md` § Time-and-Ordering-Hazards](../stdlib-families.md#time-and-ordering-hazards).
> The source docstring in
> [`../../antigen/src/stdlib/time_ordering.rs`](../../antigen/src/stdlib/time_ordering.rs)
> is ground truth; this file mirrors it.

- **Family**: Time-and-Ordering-Hazards
- **Category**: `FunctionalCorrectness`
- **Member**: `SystemTimeUnwrapPanic` (**suspected**)
- **Runnable example**:
  [`../../antigen/examples/time_ordering.rs`](../../antigen/examples/time_ordering.rs)

---

## Presentation

The developer writes the obvious clock read:

```rust
let elapsed = SystemTime::now().duration_since(earlier).unwrap();
```

It passes every test. It works in dev. Then in production the **system clock runs
backwards** — an NTP correction, a manual clock set, a VM pause/resume — and
`duration_since` returns `Err`. The `.unwrap()` panics. The defining symptom: the
bug **panics in production but never in tests**, because test machines don't
NTP-skew mid-test. This is the textbook failure-class the test suite *structurally
cannot reach* — the silent-in-tests / panic-in-prod flagship.

## Etiology

The mechanism: `SystemTime` is a **wall clock**, and a wall clock is not monotonic —
it can jump backwards. `SystemTime::duration_since` returns a `Result` precisely
because "later minus earlier" can be negative when the clock moved back. An
`.unwrap()` / `.expect()` on that `Result` treats the fallible read as infallible —
fine on the happy path the tests exercise, fatal on the backwards-clock path they
never do.

Biology cognate (as real mechanism): **circadian / signaling-timing failure**. The
immune system depends on correctly-ordered signaling cascades — the right response
fires when the timing is right. A clock that runs backwards corrupts the cascade
timing, so the wrong response fires. The wall-clock skew is exactly that: a
timing source that runs backwards and corrupts everything ordered by it.

## Epidemiology

Real-world recorded reference — cite only what the source actually references:

- The standing reference is **std's `SystemTime::duration_since` documentation** —
  `https://doc.rust-lang.org/std/time/struct.SystemTime.html#method.duration_since`
  — which documents that the method returns `Err` when the clock has moved
  backwards. The source calls this "the canonical clock footgun."

No RUSTSEC/CVE advisory IDs are claimed for this family — the source's reference is
the std documentation, not a specific advisory. (An advisory ID here would be
invented.)

## Histology — the fingerprint, annotated

```text
all_of([
    body_calls("duration_since"),
    any_of([
        body_calls("unwrap"),
        body_calls("expect"),
    ]),
])
```

- `body_calls("duration_since")` — a clock-read call (last-segment match).
- `any_of([body_calls("unwrap"), body_calls("expect")])` — an `unwrap` or `expect`
  call somewhere in the same body.
- `all_of([...])` — **both** must co-occur.

This is the **co-occurrence** form: the *precise* tell is the method-chain
`x.duration_since(y).unwrap()`, but the shipped grammar has no relational/chain
leaf, so the member ships "a `duration_since` call AND an `unwrap`/`expect` in the
same body."

What is **excluded**, and why (the clean-sibling rule):

- `elapsed` is **excluded** from the anchor. An `elapsed` arm would fire on
  `Instant::now().elapsed()` — but `Instant` is monotonic and `Instant::elapsed()`
  returns `Duration` (not `Result`, can't panic-on-skew). That's the textbook *"use
  `Instant` instead of `SystemTime`"* **fix** — the member's own clean sibling. A
  needle that fires on the anti-correlated safe case (the fix) is dropped at **every**
  tier, not merely demoted.

## Differential — why suspected, not named

- **Does the shape prove the panic-chain, or only correlate with it?**
  - The co-occurrence *correlates* with the panic-chain but does not *prove* it — the
    `unwrap` could be guarding a different `Result` entirely. → **suspected**.
- **Known namesake false-positive (disclosed, not hidden)**: `duration_since` is
  *also* the name of the **infallible** `Instant::duration_since` (it returns
  `Duration`, not `Result` — no `unwrap` needed). So the co-occurrence form fires on
  a body that calls `instant_a.duration_since(instant_b)` *and* `unwrap`s something
  unrelated — a false positive on the `Instant` path, because the only discriminator
  is the **receiver type** (`SystemTime` vs `Instant`), which scan cannot resolve
  (`x.duration_since(y)` does not expose `x`'s type). This is exactly *why* the
  member is suspected, not named: a receiver-type-only discriminator is not an
  AST-feasible leaf, so this FP is honest within-tier recall noise at suspected (a
  named tier could not carry it).
- **Why `duration_since` is kept despite the FP, but `elapsed` is dropped**:
  dropping `duration_since` would leave the member with **no anchor at all**, and the
  `SystemTime`-vs-`Instant` ambiguity on it is resolved by the witness/tier. `elapsed`
  is dropped because its codomain *includes the clean fix* (`Instant::elapsed`) — an
  anti-correlation (drop), not the un-correlated namesake noise that merely demotes.

## Treatment — the witness

`present ≠ vulnerable`. A `duration_since` + `unwrap` co-occurrence is a *prompt to
look* at the suspected tier. The witness, either:

- the `Result` is handled (`.unwrap_or(Duration::ZERO)`, a `match`, …) — the
  backwards-clock case is given a value instead of a panic; OR
- `Instant` is used instead of `SystemTime` for the measurement (the monotonic
  clock, which can't skew backwards).

## Prognosis

`SystemTimeUnwrapPanic` sits at **suspected**: the shipped grammar has no
method-chain leaf, so the fingerprint is the `duration_since` + `unwrap`
*co-occurrence* form, and it shares a name with the infallible
`Instant::duration_since` (the receiver type — the only discriminator — is not
syntactically resolvable). Two honest costs are documented, not forgotten: the
dropped `SystemTime::elapsed().unwrap()` recall hole, and the `Instant`-namesake
false-positive. Both are recovered by the same receiver-type-resolution graduation,
recorded in [`../roadmap.md`](../roadmap.md).

---

## See also

- [`../stdlib-families.md`](../stdlib-families.md) — the catalog row
- [`../fingerprint-grammar.md`](../fingerprint-grammar.md) — the fingerprint DSL
- [`../witness-tiers.md`](../witness-tiers.md) — the tier gradient
- [`../decisions.md`](../decisions.md) — the namesake-FP disclosure, and the
  grammar leaves
- [`../../antigen/src/stdlib/time_ordering.rs`](../../antigen/src/stdlib/time_ordering.rs)
  — the source docstring (ground truth)
