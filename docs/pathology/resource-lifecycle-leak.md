# Pathology case-file — Resource-Lifecycle-Leak

> The deep tier of the catalog row. Read the catalog entry first:
> [`../stdlib-families.md` § Resource-Lifecycle-Leak](../stdlib-families.md#resource-lifecycle-leak).
> The source docstring in
> [`../../antigen/src/stdlib/resource_lifecycle.rs`](../../antigen/src/stdlib/resource_lifecycle.rs)
> is ground truth; this file mirrors it.

- **Family**: Resource-Lifecycle-Leak
- **Category**: `FunctionalCorrectness`
- **Member**: `DeliberateLeakNotDocumented` (**suspected**)
- **Runnable example**:
  [`../../antigen/examples/resource_lifecycle.rs`](../../antigen/examples/resource_lifecycle.rs)

---

## Presentation

The developer calls an explicit-leak primitive — `mem::forget`, `Box::leak`, or
`Vec::leak` — which deliberately skips `Drop`. Sometimes this is exactly right (a
`Box::leak` to obtain a `'static` reference to a known singleton). Sometimes it's a
silent leak: a file handle, socket, lock guard, or allocation whose `Drop` would
have released it never runs, and the resource accumulates. Nothing crashes; the
symptom is a slow leak — climbing memory, exhausted file descriptors, a lock never
released — with no error at the call site. The question antigen asks isn't "is this
a leak?" but "is the rationale *documented*?"

## Etiology

The mechanism: `forget` / `leak` **suppress the destructor**. Rust's resource
discipline is RAII — `Drop` runs at end of scope and releases. `mem::forget`
consumes a value *without* running its `Drop`; `Box::leak` / `Vec::leak` convert an
owned allocation into a leaked `'static` reference. Both are legitimate tools, but
both move the resource off the automatic-cleanup path, so correctness now depends on
a human reason that may or may not exist.

Biology cognate (as real mechanism): **failure of apoptosis / efferocytosis**.
Apoptosis is programmed cell death; efferocytosis is the clearance of the dead cell.
Cells that should die and be cleared but instead persist are the senescent-cell
accumulation behind aging and disease. `mem::forget` is **explicitly suppressing the
death signal** — telling the resource not to die when it should.

This family is the **sibling of Drop-and-Panic** on one Drop-Lifecycle axis:
- `drop_panic` = drop **fires-but-explodes** (the destructor runs and panics);
- `resource_lifecycle` = drop **never-fires** (the destructor is skipped).
They are **not merged** — the remedies are distinct (panic-free teardown vs
document-the-leak) — but the kinship is real and recorded.

## Epidemiology

Real-world recorded reference — cite only what the source actually references:

- The standing reference is **std's `mem::forget` documentation** —
  `https://doc.rust-lang.org/std/mem/fn.forget.html` — and the std guidance "don't
  use `std::mem::forget` unnecessarily." `Box::leak` is documented as legitimate for
  `'static` upgrades but silently leaking if misused.

No RUSTSEC/CVE advisory IDs are claimed for this family — the source's prior-art is
the std documentation and guidance, not a specific advisory. (An advisory ID here
would be invented.)

## Histology — the fingerprint, annotated

```text
any_of([
    body_calls("forget"),
    body_calls("leak"),
])
```

- `body_calls("forget")` — a call whose last path segment is `forget`
  (`mem::forget` → last-segment `forget`).
- `body_calls("leak")` — a call whose last path segment is `leak` (`Box::leak` /
  `Vec::leak` → last-segment `leak`).
- `any_of([...])` — either call suffices.

Note the grammar fact that drives the tier: `body_calls` matches by **last
segment**, and a path-qualified needle (e.g. `mem::forget`) is parse-rejected — so
the effective needle is the bare segment `forget` / `leak`.

## Differential — why suspected, not named

- **Is the needle's effective codomain the defect population, or wider?**
  - `forget` / `leak` are **bare common last-segments with no narrowing anchor**.
    Because `body_calls` matches by last segment, the effective codomain *includes*
    a domain `cache.forget()` / `permissions.leak()` that isn't a leak at all. A
    **positive** tell at the named (loud) tier would overclaim precision — and the
    dial cannot soften at named — so the honest tier is **suspected**.
  - Contrast the named members whose effective codomain *is* the defect population:
    a rare/std-specific self-anchoring needle (`get_unchecked`, `from_reader`), a
    defect-slice anchor (`impl_of_trait("Drop")`), or a rare co-anchor
    (`copy_nonoverlapping` + `size_of`). The leak primitives *are* `forget` / `leak`
    — no rarer companion exists at the leaf — so there's nothing to narrow on.
- **Can the witness rescue named?** No. A `cache.forget()` false-positive is a
  *different method*, so no `#[defended_by]` reaches it — the precision gap is in the
  fingerprint, not coverable by a witness.
- **Provenance vs dial-tier (orthogonal axes, ADR-039).** The leak *class* is
  trivially `provenance = Constructable` — `mem::forget` demonstrably skips `Drop`,
  the affinity-pair holds. But *this instance's* dial sits at **suspected** because
  of the naming-breadth precision issue. A solid class can carry a low-precision
  fingerprint: provenance = how solid the class is; dial-tier = how loud this
  instance is.

## Treatment — the witness

`present ≠ vulnerable`. A `forget` / `leak` call is a *prompt to look* at the
suspected tier — many are entirely legitimate. The witness antigen asks for is the
**documented rationale**, any of:

- a documented rationale — `Box::leak` for a known-`'static` singleton is fine, *if
  said*; OR
- the resource is not actually leaked (the call is on a domain type, or the value's
  cleanup happens another way).

## Prognosis

`DeliberateLeakNotDocumented` sits at **suspected**: `forget` / `leak` are bare
common last-segments, so a domain `cache.forget()` fires too. This family ships the
clean leak-call-presence member now; its tier-promotion — path / semantic resolution
narrowing the codomain to the real leak primitives — is a recorded graduation path,
see [`../roadmap.md`](../roadmap.md).

---

## See also

- [`../stdlib-families.md`](../stdlib-families.md) — the catalog row
- [`drop-and-panic-discipline.md`](drop-and-panic-discipline.md) — the sibling
  family on the Drop-Lifecycle axis (drop fires-but-explodes)
- [`../fingerprint-grammar.md`](../fingerprint-grammar.md) — the fingerprint DSL
- [`../witness-tiers.md`](../witness-tiers.md) — the tier gradient (provenance vs
  dial-tier)
- [`../../antigen/src/stdlib/resource_lifecycle.rs`](../../antigen/src/stdlib/resource_lifecycle.rs)
  — the source docstring (ground truth)
