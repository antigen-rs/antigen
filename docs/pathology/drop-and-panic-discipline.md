# Pathology case-file — Drop-and-Panic-Discipline

> The deep tier of the catalog row. Read the catalog entry first:
> [`../stdlib-families.md` § Drop-and-Panic-Discipline](../stdlib-families.md#drop-and-panic-discipline).
> The source docstring in
> [`../../antigen/src/stdlib/drop_panic.rs`](../../antigen/src/stdlib/drop_panic.rs)
> is ground truth; this file mirrors it.

- **Family**: Drop-and-Panic-Discipline
- **Category**: `FunctionalCorrectness`
- **Member**: `PanicInDrop` (**named**)
- **Runnable example**:
  [`../../antigen/examples/drop_panic.rs`](../../antigen/examples/drop_panic.rs)

---

## Presentation

What the developer sees: a destructor that *can* panic. Most of the time it
doesn't, so the program runs fine. The failure surfaces only on the unwinding
path — an error is propagating, the stack is unwinding, and a `Drop::drop` body on
that path reaches a panic source. Now **two panics are in flight at once**, and
under `panic = unwind` the runtime's only safe move is to **abort the process**.
Worse, the destructor's own cleanup is skipped, so resources leak *even on the
path that was supposed to clean up*. The developer sees an abrupt `abort`, no
clean error, and leaked handles/locks — a teardown that turned a recoverable error
into a process kill.

## Etiology

The mechanism: a `Drop::drop` body contains a *reachable* panic source. When that
body runs during an **in-flight unwind** (another panic is already propagating), a
second panic is fatal — Rust cannot unwind two panics simultaneously, so it
aborts. The destructor's remaining cleanup never runs.

Biology cognate (as real mechanism): **apoptosis gone wrong**. Apoptosis is
*programmed cell death* — the clean, orderly teardown a cell performs when it's
time to go. `Drop` is exactly that: the type's programmed teardown. The pathology
is when programmed death itself triggers a catastrophic cascade — a double-panic
that aborts the whole organism — instead of the clean, contained teardown it was
meant to be.

The two panic *shapes* are why the v2 fingerprint exists:

- **call-shaped** panics — `.unwrap()` / `.expect()` in the drop body. This is the
  *more common* teardown panic and the older macro-only fingerprint missed it.
- **macro-shaped** panics — `panic!` / `unreachable!` / `todo!` /
  `unimplemented!`.

## Epidemiology

Real-world recorded reference — cite only what the source actually references:

- The authoritative reference is **std's own `Drop` documentation, the `#panics`
  section** — `https://doc.rust-lang.org/std/ops/trait.Drop.html#panics`. The
  double-panic-on-unwind → abort behavior is **documented std behaviour**, not a
  hypothetical; the source notes "std got this wrong repeatedly."

No RUSTSEC/CVE advisory IDs are claimed for this family — the standing reference is
the std documentation. (Asserting an advisory ID here would be invented; the source
does not.)

## Histology — the fingerprint, annotated

```text
all_of([
    item = impl,
    impl_of_trait("Drop"),
    any_of([
        body_calls("unwrap"),
        body_calls("expect"),
        body_contains_macro("panic"),
        body_contains_macro("unreachable"),
        body_contains_macro("todo"),
        body_contains_macro("unimplemented"),
    ]),
])
```

- `item = impl` — the matched item is an `impl` block (not a fn, struct, etc.).
- `impl_of_trait("Drop")` — the impl is a **real** `Drop` trait impl, not an
  inherent impl with a method merely *named* `drop`. This is the precision anchor:
  the example's `NotReallyDrop` (an inherent `drop` method) is correctly spared.
- `any_of([...])` — the body reaches a panic source by either shape:
  - `body_calls("unwrap")` / `body_calls("expect")` — call-shaped panics (last
    path segment match).
  - `body_contains_macro("panic" | "unreachable" | "todo" | "unimplemented")` —
    macro-shaped panics.

Covering **both** shapes is the point: the older `basic.rs` `PanickingInDrop` was
macro-only (it missed the `.unwrap()` form) and over-fired on non-`Drop` impls.
`PanicInDrop` is its v2 — the example's shipped fingerprint was tightened in
lock-step (CHANGELOG'd).

## Differential — why named

- **Is the anchor precise enough that "doesn't fire ⇒ covered" holds?**
  - `impl_of_trait("Drop")` is a **defect-slice anchor**: it matches the real
    `Drop` trait, restricting the codomain to actual destructors. Combined with a
    reachable-panic-source, the shape pinpoints the failure-class.
  - The double-panic-on-unwind class is **documented std behaviour** — the harm is
    real and named.
  - → **named**. A match is a real site to defend or tolerate.
- **Contrast with the older fingerprint** (why it was *not* named-honest): the
  `item = impl, body_contains_macro(...)`-only form lacked `impl_of_trait("Drop")`,
  so it fired on non-`Drop` impls (false positives) and was macro-only (missed
  `.unwrap()`, false negatives). Adding the Drop anchor *and* the call-shaped arm
  is what earns the named tier.

## Treatment — the witness

`present ≠ vulnerable`. A `Drop` impl presenting this shape is in the
failure-class's territory; the witness proves the teardown is panic-safe. Any of:

- the drop body is **panic-free** (no reachable panic source); OR
- the risky op is **wrapped to catch/log** (so a teardown error doesn't escalate
  to a panic); OR
- a `std::thread::panicking()` check **before** the risky op — so the destructor
  stays quiet during an in-flight unwind (the canonical guard).

## Prognosis — the graduation path

`PanicInDrop` is already at the named tier and is the v2 of the older
`PanickingInDrop`; no demotion or pending re-tier. The natural future increments
are sibling, not corrective:

- The **Resource-Lifecycle-Leak** family is the other half of the Drop-Lifecycle
  axis — *drop never-fires* — to this family's *drop fires-but-explodes*. They are
  kept distinct (distinct remedies) rather than merged; see
  [`resource-lifecycle-leak.md`](resource-lifecycle-leak.md).

---

## See also

- [`../stdlib-families.md`](../stdlib-families.md) — the catalog row
- [`resource-lifecycle-leak.md`](resource-lifecycle-leak.md) — the sibling family
  on the Drop-Lifecycle axis
- [`../fingerprint-grammar.md`](../fingerprint-grammar.md) — the fingerprint DSL
- [`../witness-tiers.md`](../witness-tiers.md) — the tier gradient
- [`../../antigen/src/stdlib/drop_panic.rs`](../../antigen/src/stdlib/drop_panic.rs)
  — the source docstring (ground truth)
