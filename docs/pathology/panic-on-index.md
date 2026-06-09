# Pathology case-file — Panic-on-Index

> The deep tier of the catalog row. Read the catalog entry first:
> [`../stdlib-families.md` § Panic-on-Index](../stdlib-families.md#panic-on-index).
> The source docstring in
> [`../../antigen/src/stdlib/panic_on_index.rs`](../../antigen/src/stdlib/panic_on_index.rs)
> is ground truth; this file mirrors it.

- **Family**: Panic-on-Index
- **Category**: `FunctionalCorrectness`
- **Member**: `GetUncheckedWithoutProof` (**named**)
- **Runnable example**:
  [`../../antigen/examples/panic_on_index.rs`](../../antigen/examples/panic_on_index.rs)

---

## Presentation

The family name says "panic," but the build-now member is the *worse* form. The
developer reaches for `get_unchecked` / `get_unchecked_mut` to skip a bounds check
for performance. In testing the indices are in range and everything works. In
production an index goes out of range — and there is **no panic**. Instead the
read/write lands in memory it shouldn't, which is **Undefined Behavior**: silent
memory corruption, a value read from the wrong place, or a subtle miscompile-style
misbehavior far from the actual bug. The symptom is *not* a clean crash with a
stack trace; it's a soundness hole that may surface arbitrarily later, or never
visibly at all while still being unsound.

## Etiology

The mechanism: `slice::get_unchecked` / `get_unchecked_mut` are the unchecked
indexing escape hatch. The safe `[i]` operator and `.get(i)` both verify the index
against the length; `get_unchecked` **skips that check by contract**, and the
caller takes on the obligation to prove the index is in-bounds. When that proof is
wrong, an out-of-bounds index is UB — not a panic. This member belongs to **both**
this family and the unsafe-soundness boundary, because the failure is a soundness
hole, not a controlled `DoS`.

Biology cognate (as real mechanism): **proprioception / spinal-reflex failure**.
Proprioception is the body's sense of where its limbs are; the protective
spinal reflex (e.g. the withdrawal reflex) fires *before* conscious thought to keep
a movement inside the body's safe range. The bounds check is exactly that reflex.
`get_unchecked` is acting past the valid range with the protective reflex
disabled — the movement completes, but into territory that injures the system.

## Epidemiology

Real-world recorded reference — cite only what the source actually references:

- The standing reference is **std's `slice::get_unchecked` documentation** —
  `https://doc.rust-lang.org/std/primitive.slice.html#method.get_unchecked` —
  which states the safety contract (the caller must guarantee the index is
  in-bounds; violating it is UB). The class is **miri-catchable**: miri detects the
  out-of-bounds access at runtime, which is the empirical confirmation the class is
  real.

No RUSTSEC/CVE advisory IDs are claimed for this family — the source references the
std documentation, not a specific advisory. (An advisory ID here would be invented.)

## Histology — the fingerprint, annotated

```text
any_of([
    body_calls("get_unchecked"),
    body_calls("get_unchecked_mut"),
])
```

- `body_calls("get_unchecked")` — a call whose last path segment is
  `get_unchecked`.
- `body_calls("get_unchecked_mut")` — likewise for the `_mut` form.
- `any_of([...])` — either call suffices.

Both are slice/`Vec`-specific method names with **no stdlib collision** — a clean
call-shape. There is no `not(...)` guard and no co-anchor: the presence of the call
*is* the tell.

## Differential — why named

- **Is the needle's effective codomain the defect population?**
  - `get_unchecked` / `get_unchecked_mut` are **rare / std-specific** method names
    with no stdlib collision — a domain type won't define a method by that name, so
    the needle alone self-anchors onto the defect population.
  - The class (UB on OOB) is **real and miri-catchable**.
  - → **named**. A match is a real site to defend (prove the index) or tolerate.
- **What stays out of scope (honest defect-slice)**: the **panic** form — `expr[i]`
  indexing a `Vec`/slice with an input-derived index
  (`UncheckedIndexOnDynamicCollection`) — is an Index-**operator** tell
  (`ExprIndex`), not a call leaf, so the call-only grammar can't express it (a
  recorded charter — see the prognosis below). The deref-coercion compile-vs-runtime
  gem (`(&arr)[OOB]` compiles where `arr[OOB]` doesn't) is a specimen-garden exhibit,
  not a fingerprint member.
  This family ships the clean call-shaped member now and labels the operator-shaped
  recall hole rather than over-reaching the grammar.

## Treatment — the witness

`present ≠ vulnerable`. A `get_unchecked` call is in this failure-class's
territory; the witness proves the index obligation is met. Either:

- a `// SAFETY:` comment proving the index is in-bounds **plus a miri run** (the
  proof is stated *and* checked); OR
- the checked `.get(i)` with a handled `None` (the call is replaced by the safe,
  bounds-checking form).

## Prognosis

`GetUncheckedWithoutProof` is at the **named** tier; the fingerprint fires on the
*presence* of the `get_unchecked` call. This member deliberately claims only the
unchecked-call shape — not the precise in-bounds check (which needs resolved index
values) nor the panic-operator form `expr[i]` (which needs the operator-leaf the
call-only grammar lacks). Those are *additional* coverage, not a re-tier of this
member, and are recorded as graduation paths in [`../roadmap.md`](../roadmap.md).

---

## See also

- [`../stdlib-families.md`](../stdlib-families.md) — the catalog row
- [`unsafe-soundness-boundary.md`](unsafe-soundness-boundary.md) — the sibling
  soundness-boundary family (`get_unchecked` belongs to both)
- [`../fingerprint-grammar.md`](../fingerprint-grammar.md) — the fingerprint DSL
- [`../witness-tiers.md`](../witness-tiers.md) — the tier gradient
- [`../../antigen/src/stdlib/panic_on_index.rs`](../../antigen/src/stdlib/panic_on_index.rs)
  — the source docstring (ground truth)
