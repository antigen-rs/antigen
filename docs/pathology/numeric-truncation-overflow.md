# Pathology case-file — Numeric-Truncation-Overflow

> The deep tier of the catalog row. Read the catalog entry first:
> [`../stdlib-families.md` § Numeric-Truncation-Overflow](../stdlib-families.md#numeric-truncation-overflow).
> The source docstring in
> [`../../antigen/src/stdlib/numeric_truncation.rs`](../../antigen/src/stdlib/numeric_truncation.rs)
> is ground truth; this file mirrors it.

- **Family**: Numeric-Truncation-Overflow
- **Category**: `FunctionalCorrectness`
- **Member**: `SizeOfInElementCount` (**suspected**)
- **Runnable example**:
  [`../../antigen/examples/numeric_truncation.rs`](../../antigen/examples/numeric_truncation.rs)

---

## Presentation

The developer writes a raw-memory copy and passes a count that *looks* right:

```rust
ptr::copy_nonoverlapping(src, dst, n * size_of::<T>())
```

It compiles. It runs. In testing with small `n` it may even appear to work. But
`copy_nonoverlapping`'s count argument is in **elements**, not bytes — so
multiplying by `size_of::<T>()` over-copies by a factor of `sizeof(T)`. The copy
runs off the end of the destination: an out-of-bounds read/write → **UB**. The
symptom is the silent-corruption signature — *a* value comes out, the program keeps
running, and the damage surfaces far from the call or not visibly at all.

## Etiology

The mechanism: an off-by-a-factor on the count argument of an unsafe raw copy. The
API takes an **element** count; the developer reasons in **bytes** (the
size-of-buffer intuition) and multiplies by `size_of`. The two unit systems collide
silently because both are just `usize` — the type system can't catch it.

Biology cognate (as real mechanism): **silent mutation**. A single base-pair flip
can produce a protein that still folds and still functions *enough* to compile and
run — but it's the wrong protein. There's no immediate symptom; the corruption
propagates downstream. The miscounted copy is exactly that: it produces *a* result
(it compiles, runs, returns), but the wrong one, and the corruption spreads.

## Epidemiology

Real-world recorded reference — cite only what the source actually references:

- The standing reference is **clippy's correctness lint
  `size_of_in_element_count`** —
  `https://rust-lang.github.io/rust-clippy/master/index.html#size_of_in_element_count`
  — which exists for **exactly this** pattern. That a correctness lint targets it is
  the empirical confirmation the class recurs and the harm is memory corruption / UB.

No RUSTSEC/CVE advisory IDs are claimed for this family — the source's prior-art is
the clippy lint, not a specific advisory. (An advisory ID here would be invented.)

## Histology — the fingerprint, annotated

```text
all_of([
    body_calls("copy_nonoverlapping"),
    body_calls("size_of"),
])
```

- `body_calls("copy_nonoverlapping")` — a raw-memory copy call (last-segment match).
- `body_calls("size_of")` — a `size_of` call somewhere in the same body.
- `all_of([...])` — **both** must be present (the co-presence / co-anchor form).

This is the coarse **co-presence** form, deliberately so: the shipped grammar has no
arg-position or type-resolution leaf, so the fingerprint anchors on the two calls
co-occurring in one body, not on `size_of::<T>()` appearing *in the count argument*.

## Differential — why suspected, not named

This member is the catalog's **worked example of tier-honesty**: it was over-claimed
at named, then corrected to suspected with the fingerprint **unchanged** — the fix
was the *tier*, not the *shape* (ADR-039 §C Amendment 1).

The decision tree:

- **Does the shape pinpoint the defect, or merely correlate with a dangerous
  region?**
  - The co-presence *correlates* with the dangerous region (an unsafe raw copy near
    a `size_of`) but **cannot pinpoint** the defect — it fires on idiomatic-correct
    both-calls code too. A named tier ("if it doesn't fire, you're covered") could
    not carry those false positives. → **suspected**.
- **Is the member's own fix spared (demote), or does the needle fire on the fix
  (drop)?**
  - The anti-correlated **fix** — `copy_nonoverlapping(s, d, n)` with an element
    count and **no** `size_of` — *is* spared, because the `all_of` co-anchor needs
    both calls. So the needle does **not** fire on its own fix → **demoted, not
    dropped**.
  - But two **correct** both-calls siblings still fire (un-correlated, not
    anti-correlated): (1) a copy by element count whose body *separately* computes
    `size_of` for a bounds check, and (2) the legitimate single-element byte-copy
    `copy_nonoverlapping(p, q, size_of::<u32>())` on `*u8` pointers. Those firings
    are honest labeled-recall noise at suspected.
- **Discriminator**: anti-correlate ⇒ drop; un-correlate ⇒ demote. Here the fix is
  spared (no anti-correlation) but correct siblings fire (un-correlation) → demote
  to suspected.

## Treatment — the witness

`present ≠ vulnerable`. A `copy_nonoverlapping` + `size_of` co-presence is a *prompt
to look* at the suspected tier, not a verdict. The witness, any of:

- the count is an **element count** (no `size_of` multiplier on the count arg); OR
- a `// SAFETY:` argument that the byte/element units are correct; OR
- miri.

## Prognosis — the graduation path

Graduation to named is **type-aware, not a near-term syntactic leaf** (the source is
explicit about not over-promising an operator-leaf here). Pinpointing the defect
needs **both** arg-position introspection (`size_of::<T>()` *in the count argument*)
**and** the **pointee type** of the copy — because the arg-structure leaf *alone* is
insufficient: the correct `*mut u8` byte-buffer idiom
(`copy(dst: *mut u8, n * size_of::<T>())`) carries the very same `n * size_of` shape
and would still false-positive; sparing it requires knowing the destination is `*u8`
(a byte buffer), which is **resolved-type** information not available at macro/scan
time. So this graduates only at the **v0.4 type-aware tier** (arg-position AND
pointee-type), never at a syntactic operator-leaf.

The family's other shapes — `LossyNumericCast` (an `as`-cast type-signature tell) and
the arithmetic-overflow / float-equality members — are operator-shaped (no shipped
leaf) → **charter**. This family ships the clean call-co-presence member now.

---

## See also

- [`../stdlib-families.md`](../stdlib-families.md) — the catalog row (the
  tier-honesty worked example)
- [`../fingerprint-grammar.md`](../fingerprint-grammar.md) — the fingerprint DSL
- [`../witness-tiers.md`](../witness-tiers.md) — the tier gradient
- [`../decisions.md`](../decisions.md) — ADR-039 §C Amendment 1 (the named→suspected
  correction)
- [`../../antigen/src/stdlib/numeric_truncation.rs`](../../antigen/src/stdlib/numeric_truncation.rs)
  — the source docstring (ground truth)
