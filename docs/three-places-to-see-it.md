# Three places to see it — where antigen's thesis is actually visible

> Antigen's thesis is: *name a failure-class, catch its shape, separate the bad
> path from the clean one.* A newcomer naturally wants to **watch that happen** —
> and then asks "where?" The honest answer is that different parts of the thesis are
> visible in different places, each the right surface for its own question. This is
> the map.
>
> The short version:
>
> | You want to see… | Go here | What you'll see |
> |---|---|---|
> | a defense being **observed** (class-level) | **run an example** | both siblings present; the witnessed one reports `defended` |
> | a fingerprint **genuinely sparing** a site | **scan one fixture** | exactly one site; the un-marked safe sibling simply absent |
> | **every family's** bind-vs-spare, side by side | **read the guard tests** | the bad shape that binds next to the clean namesake that doesn't |

---

## 1. Class-level defense → **run an example**

The family examples teach **Lesson 1: `present` ≠ `vulnerable`.** Each one marks
*both* a risky site and its safe sibling with `#[presents]`, then defends what it
can. Run one and read what it prints:

```sh
cargo run --example drop_panic -p antigen
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

In `audit`, **both** the risky `PanickyGuard` and the safe `SafeGuard` list as
presentations — because both are `#[presents]`-marked (a site being *in the
territory* is not a vulnerability claim). The difference between them is carried by
the **witness**: a defended site reports `✓ defended`, an unwitnessed one
`✗ undefended`. This is defense observed at the level of the *class*, exactly as
ADR-029 intends — immunity is *observed from evidence*, never claimed at the site.

> What you will **not** see here is the safe sibling vanishing. Both are marked, so
> both surface. For the place a site genuinely disappears, read on.

---

## 2. The fingerprint genuinely sparing a site → **scan one fixture**

The cleanest "watch the fingerprint separate" demo in the repo is the
**async-soundness scan fixture** — Lesson 2:

```sh
cargo run --bin cargo-antigen -- antigen scan --root antigen/tests/fixtures/family_unsafe_send_sync
```

```
1 unaddressed explicit presentation(s):
  antigen/tests/fixtures/family_unsafe_send_sync/lib.rs:37  UnsafeSendSync on impl
```

**Exactly one site.** The bad `unsafe impl Send for RawHandle` is reported; the safe
sibling (`impl Clone for RawHandle`) is **un-marked** — no `#[presents]` — so the
fingerprint doesn't bind it and it simply **never appears**. *This* is what it looks
like when the fingerprint itself separates clean from dirty: the dangerous shape
surfaces, the safe shape is absent.

(It works here and not in the §1 examples because the §1 examples deliberately mark
*both* siblings to teach Lesson 1. Different lesson, different wiring — both
intentional.)

---

## 3. Every family's bind-vs-spare → **read the guard tests**

To see the bind/spare distinction proved **directly, for every family, side by
side** — read the guard tests. They're clearer than any example, because each
family's `_binds_` case sits right next to its `_spares_` case with plain-English
rationale:

- [`antigen/tests/stdlib_family_fingerprints.rs`](../antigen/tests/stdlib_family_fingerprints.rs)
  — a bind/spare pair for **every** shipped family. For example,
  `unbounded_deserialization_binds_from_reader_call` (the bad `from_reader` shape
  **matches**) right beside `unbounded_deserialization_spares_from_slice_namesake`
  (the clean `from_slice` namesake **doesn't**).
- [`antigen/tests/spares_namesake_contract.rs`](../antigen/tests/spares_namesake_contract.rs)
  — pins the harder namesake cases (the clean same-method siblings a fingerprint
  must *not* fire on).

```sh
cargo test --test stdlib_family_fingerprints
cargo test --test spares_namesake_contract
```

The verb is **read**, not run: `cargo test` confirms the separations still *hold*
(green/red), but **reading** the test bodies is where you *see* the bad shape and
the spared namesake next to each other with the why. No example re-engineering
needed — the thesis is demonstrable today, in code that runs green.

---

## Why three surfaces, not one

Because they answer three different questions, and forcing them into one example
would break one to serve another:

- mark both siblings `#[presents]` → you get **Lesson 1** (present ≠ vulnerable),
  but you *lose* the visible fingerprint-spare (both surface).
- leave the safe sibling un-marked → you get **Lesson 2** (the visible spare), but
  you *lose* the "a safe site can still present" lesson.

So each surface teaches its own thing cleanly. The guard tests then give you the
direct bind-vs-spare proof for every family at once. Three right tools, three real
questions — a division of labor, not three attempts at the same thing.

---

## See also

- [`reading-a-verdict.md`](reading-a-verdict.md) — what every scan/audit line means.
- [`i-scanned-and.md`](i-scanned-and.md) — "I scanned and the safe one didn't
  disappear" and other symptom-first answers.
- [`stdlib-families.md`](stdlib-families.md) — the catalog, incl. the
  "two lessons, two demos" framing and the per-family guard pointers.
- [`examples-guide.md`](examples-guide.md) — the runnable lesson per example.
