# Pathology case-file — Async-Soundness

> The deep tier of the catalog row. Read the catalog entry first:
> [`../stdlib-families.md` § Async-Soundness](../stdlib-families.md#async-soundness).
> The source docstring in
> [`../../antigen/src/stdlib/async_soundness.rs`](../../antigen/src/stdlib/async_soundness.rs)
> is ground truth; this file mirrors it.

- **Family**: Async-Soundness
- **Category**: `FunctionalCorrectness`
- **Member**: `UnsafeSendSync` (**named**)
- **Scan fixture** (no runnable example — see below):
  [`../../antigen/tests/fixtures/family_unsafe_send_sync/lib.rs`](../../antigen/tests/fixtures/family_unsafe_send_sync/lib.rs)

---

## Presentation

The developer writes `unsafe impl Send for T {}` (or `Sync`) to make a type cross
a thread boundary that the compiler refused to allow automatically — usually
because the type holds a raw pointer (`*mut`), an interior non-`Sync` field, or a
foreign handle. It compiles. The author has *asserted* cross-thread safety, but the
compiler **did not verify it**. The symptom, if the assertion is wrong, surfaces
only under concurrency: a data race, torn reads/writes, or UB across threads —
nondeterministic, hard to reproduce, and often invisible until a specific
interleaving hits in production. A nearby symptom from the source: "some mutex
crates implement `Send` for their `MutexGuard`s … compiles, deadlocks."

## Etiology

The mechanism: `Send` and `Sync` are **auto-traits** — the compiler derives them
structurally and refuses them when a field makes the type unsound to share or send.
A hand-written `unsafe impl Send/Sync` **overrides** that refusal with an unchecked
promise. If the promise is wrong (the pointee really isn't thread-safe), the type
is moved/shared across threads and the soundness the auto-trait rules exist to
guarantee is broken.

Biology cognate (as real mechanism): the **innate barrier of the concurrency
boundary**, and specifically a **mislabeled self/non-self marker**. The immune
system's first line is the innate barrier that admits "self" and excludes
"non-self"; an `unsafe impl Send/Sync` declares "this is safe to cross the thread
boundary" *without the receptor that proves it* — a forged marker waved through
the barrier. When the marker is wrong, hostile material (a non-thread-safe value)
is admitted into a context that assumed it was safe.

## Epidemiology

Real-world recorded reference — cite only what the source actually references:

- The standing reference is the **Rustonomicon's Send/Sync chapter** —
  `https://doc.rust-lang.org/nomicon/send-and-sync.html` — which documents exactly
  when a hand-written `unsafe impl` is/ isn't sound.
- The source records the population-level signal: **~40% of unsound RUSTSEC
  advisories root in a wrong `unsafe impl Send/Sync`** (raw pointers, `*mut`,
  interior non-`Sync`). That is a *category* frequency, stated in the source — not a
  specific advisory ID, and none is invented here.

## Histology — the fingerprint, annotated

```text
all_of([
    item = impl,
    is_unsafe,
    any_of([
        impl_of_trait("Send"),
        impl_of_trait("Sync"),
    ]),
])
```

- `item = impl` — the matched item is an `impl` block.
- `is_unsafe` — the impl carries the `unsafe` qualifier (the G1 leaf reads the
  `unsafe` keyword on the impl).
- `any_of([impl_of_trait("Send"), impl_of_trait("Sync")])` — the impl is of the
  `Send` or `Sync` trait (the G3 `impl_of_trait` leaf reads the implemented trait).

A pure impl-presence + `unsafe`-qualifier + trait tell — fully syntactic. The
fixture confirms the affinity-pair: the **bind** is
`unsafe impl Send for RawHandle` (line `:37`), and the **spare** is the un-marked
`impl Clone for RawHandle` — `is_unsafe` = no match and it isn't `Send`/`Sync`, so
the fingerprint doesn't bind it on either count.

## Differential — why named

- **Is the needle's effective codomain the defect population?**
  - A hand-written `unsafe impl Send/Sync` is an **explicit soundness assertion** —
    its mere presence is a strong signal, because the safe path is for the compiler
    to derive `Send`/`Sync` automatically; reaching for `unsafe impl` is itself the
    tell.
  - RUSTSEC-backed (~40% of unsound advisories root here).
  - → **named** ("named/confident" in the source).
- **What stays out of scope (honest defect-slice)**: `LockHeldAcrossAwait`
  (liveness of a typed binding across a suspension point) needs a new control-flow
  grammar dimension → **charter**. `BlockingCallInAsyncFn` (`is_async` + a heuristic
  blocking-API name-list) is build-now at the **suspected** tier — a next-wave
  candidate. `SpawnedFutureNotAwaited` (`let _ = spawn()` binding-tell) → **charter**.
  This family ships the clean, named `unsafe impl Send/Sync` member now.

## Treatment — the witness

`present ≠ vulnerable`. An `unsafe impl Send/Sync` presenting is a soundness
assertion in this family's territory; the witness proves the assertion. Either:

- a **documented safety argument** — a `// SAFETY:` comment the sensor layer reads,
  laying out why the type really is safe to send/share; OR
- a **kani proof** of the `Send`/`Sync` invariant.

## Prognosis

`UnsafeSendSync` is at the **named** tier — no demotion or pending re-tier. It is the
family's one shipped member; other async-soundness shapes (a blocking call in an
`async fn`, a lock held across an `await`, a spawned future never awaited) need
grammar dimensions the call-only fingerprint lacks — control-flow and binding-tells —
before they can ship honestly. See [`../roadmap.md`](../roadmap.md) for the
grammar-edge these wait on.

> **Why a scan fixture, not a runnable example.** The member's tell is a real
> `unsafe impl Send`, and the workspace sets `unsafe_code = "forbid"` (an
> un-overridable forbid an inner `#[allow]` cannot lift), so it can't live in a
> compiled example crate. The scanner reads source as **text** (it does not compile
> it), so the affinity-pair lives in a fixture you **scan**:
>
> ```sh
> cargo run --bin cargo-antigen -- antigen scan --root antigen/tests/fixtures/family_unsafe_send_sync
> ```
>
> This is the catalog's **Lesson-2 showcase** — what it looks like when the
> fingerprint **genuinely spares a site at the console**: scan reports **exactly
> one** site (the bad `unsafe impl Send for RawHandle`), because the safe sibling
> (`impl Clone for RawHandle`) is *un-marked*, so the fingerprint doesn't bind it
> and it never appears. (Contrast the L1 family examples, which deliberately
> `#[presents]`-mark *both* siblings so both surface — the "present ≠ vulnerable"
> lesson.)

---

## See also

- [`../stdlib-families.md`](../stdlib-families.md) — the catalog row (incl. the
  "two lessons, two demos" note)
- [`unsafe-soundness-boundary.md`](unsafe-soundness-boundary.md) — the other
  scan-fixture family
- [`../fingerprint-grammar.md`](../fingerprint-grammar.md) — the fingerprint DSL
- [`../witness-tiers.md`](../witness-tiers.md) — the tier gradient
- [`../../antigen/src/stdlib/async_soundness.rs`](../../antigen/src/stdlib/async_soundness.rs)
  — the source docstring (ground truth)
