# Pathology case-file — Deserialization-Trust-Boundary

> The deep tier of the catalog row. For the at-a-glance summary, tier badge, and
> the family-table context, read the catalog entry first:
> [`../stdlib-families.md` § Deserialization-Trust-Boundary](../stdlib-families.md#deserialization-trust-boundary).
> The source docstring in
> [`../../antigen/src/stdlib/deserialization.rs`](../../antigen/src/stdlib/deserialization.rs)
> is ground truth; this file mirrors it.

- **Family**: Deserialization-Trust-Boundary
- **Category**: `FunctionalCorrectness`
- **Members**: `UnboundedDeserialization` (**named**),
  `DeserializeWithoutDenyUnknownFields` (**suspected**)
- **Runnable example**:
  [`../../antigen/examples/deserialization.rs`](../../antigen/examples/deserialization.rs)

---

## Presentation

What the developer sees when this class bites:

- **`UnboundedDeserialization`** — a service that parses JSON/CBOR/etc. straight
  off a socket or file handle. In testing it parses well-formed inputs and is
  fine. In production an attacker sends a deeply-nested document or an enormous
  flat one, and the process either blows the stack (recursive descent runs out of
  frames) or allocates unboundedly until the OOM killer fires. The symptom is a
  denial of service: one crafted request takes the service down. The std library
  itself warns that a `from_reader` over a *non-terminating* stream "will not
  return."
- **`DeserializeWithoutDenyUnknownFields`** — nothing crashes. A config, auth, or
  payment payload arrives with an extra field the type does not declare, and serde
  **silently drops it**. The developer never sees an error; the masked symptom is
  later — API drift goes unnoticed, or a smuggled field that *should* have been
  rejected is quietly discarded (or, worse, accepted on a different path).

The first is loud-in-prod / quiet-in-tests; the second is quiet everywhere until
an audit asks "why did that field disappear?"

## Etiology

The mechanism, with the biology cognate as real mechanism (not decoration):

Deserialization is *the* canonical place where untrusted bytes cross into
typed-Rust land — the **gut mucosa**: the largest and busiest trust surface in the
body, where the outside world is admitted under control. The biology cognate is
exact: `#[serde(deny_unknown_fields)]` is the **tight-junction** — the protein
seal between epithelial cells that decides which molecules may cross the gut wall.
Its absence is a **leaky gut**: uncontrolled admission, where material crosses that
should have been stopped.

- For `UnboundedDeserialization`, the mechanism is that a *streaming* entrypoint
  (`from_reader`) has no a-priori bound on how much it will read or how deep it
  will recurse. The deserializer faithfully follows the input's structure; a
  malicious input dictates unbounded structure. The harm is resource exhaustion,
  not a wrong value.
- For `DeserializeWithoutDenyUnknownFields`, serde's **default** is to ignore
  fields the target type does not name (serde issue #44 is the origin). That
  default is convenient but, at a trust boundary, it means the type cannot enforce
  its own contract — unknown input is admitted and discarded rather than rejected.

## Epidemiology

Real-world recorded harm — cite only what the catalog and source actually
reference; nothing is invented here.

- **Recursion / stack-exhaustion DoS** — recorded harm across **≥3 RUSTSEC
  advisories spanning 2022→2026**, named in the source docstring:
  - **RUSTSEC-2024-0012** (serde-json-wasm) — stack overflow on deeply-nested
    input, fixed with a `remaining_depth` counter.
  - **RUSTSEC-2022-0004** (rustc-serialize).
  - **RUSTSEC-2026-0009** (time).
  This is **survivor-bias-exempt** evidence: the harm is recorded, not inferred
  from a hypothetical.
- **Silent unknown-field drop** — prior art is serde's own issue tracker rather
  than a CVE: serde **#44** (the silent-drop origin), serde **#2634** (why users
  reach for `deny_unknown_fields`: "they want notified as soon as the format
  changes"), and the known-caveat issues **#2283 / #1600** (`#[serde(flatten)]`
  bypasses the check).

## Histology — the fingerprint, annotated

### `UnboundedDeserialization`

```text
body_calls("from_reader")
```

- `body_calls("from_reader")` — matches any call whose **last path segment** is
  `from_reader` (no path resolution), so it catches `serde_json::from_reader` and
  any `Foo::from_reader`. This is the streaming entrypoint — the form std warns
  "will not return" on a non-terminating stream.

What is **deliberately excluded**, and why (the spares-namesake sub-test,
ADR-039 §C Amendment 1):

- `from_slice` is **dropped** — a slice is a *bounded* source, so `from_slice` is
  not an unbounded read; and the `from_slice` last-segment fired on the
  bounded-slice fix itself plus safe constructors like `GenericArray::from_slice`
  (a clean-sibling collision).
- `from_str` is **dropped** — it would collide with every `i32::from_str`.
- A `not(take)` guard is **deliberately not used** — see Treatment / Differential.

### `DeserializeWithoutDenyUnknownFields`

```text
all_of([
    derives("Deserialize"),
    not(serde_arg("deny_unknown_fields")),
])
```

- `derives("Deserialize")` — the type carries `#[derive(Deserialize)]` (reads the
  derive attribute tokens). **Presence** half.
- `not(serde_arg("deny_unknown_fields"))` — the type does **not** carry
  `#[serde(deny_unknown_fields)]` (reads the serde attribute tokens). **Absence**
  half. This is the cleanest attribute-presence-AND-absence tell in the stdlib:
  the presence of the *safe* argument spares the sibling.

## Differential — why this tier, not another

A diagnostic decision tree for the two members:

- **Is the needle's effective codomain the defect population?**
  - `from_reader` is **rare / std-specific** — a domain type rarely defines a
    `from_reader` method, so the needle self-anchors onto the defect population.
    "If it doesn't fire, you're covered." → **named**.
  - `derives("Deserialize")` is *common* — not every `Deserialize` type sits at a
    trust boundary. The shape co-occurs with the defect but also fires on
    idiomatic-correct internal types that never touch untrusted bytes. A labeled
    recall hole is acceptable here → **suspected**.
- **Known within-tier caveat** (kept honest, not hidden): for the suspected
  member, `#[serde(flatten)]` re-opens the boundary in a way the syntactic tell
  cannot see (serde #2283 / #1600). The member sits at suspected because not every
  `Deserialize` is at a trust boundary — pairing it with an explicit trust-boundary
  marker is what would earn the named tier.

## Treatment — the witness

`present ≠ vulnerable`. A presentation is a site *in this failure-class's
territory*; the witness is what proves the defense at audit.

- **`UnboundedDeserialization`** — the std anti-DoS idiom is a `.take(limit)`-capped
  reader (`from_reader(reader.take(n))`). The key design point (the **surface-flag
  / witness-proof split**, ADR-019/029): the bounded form **still presents** the
  surface — the risky `from_reader` *is* present — so the fingerprint does **not**
  spare it; the **witness** proves the defense at audit via `#[defended_by]`. A
  `not(take)` guard would silently suppress real DoS sites whenever an unrelated
  `Iterator::take` appeared in the body — a silent false-negative that would break
  the named tier's promise.
- **`DeserializeWithoutDenyUnknownFields`** — set `#[serde(deny_unknown_fields)]`,
  OR carry a documented "lenient-by-design" tolerance, OR a validating wrapper.

> **A precise note on what `audit` shows today** (the gap between the design
> principle and the console). The example marks *both* `load_unbounded` and
> `load_bounded` with `#[presents]` and supplies a single `#[defended_by]` witness.
> Run `cargo antigen audit --root antigen/examples` and you'll see **both** sites
> reported defended, credited to that one witness — because the current audit
> credits a `#[defended_by]` witness at the **antigen-type** granularity, not
> per-site. The surface-flag / witness-proof *split* is a real, durable design
> principle; this example does not *visibly* separate the two sites at the console.
> (A finer **site-granular** model is a recorded graduation path — see
> [`../roadmap.md`](../roadmap.md).) To
> *see* the fingerprint bind/spare directly, read the guard tests
> ([`../../antigen/tests/stdlib_family_fingerprints.rs`](../../antigen/tests/stdlib_family_fingerprints.rs)),
> not the console.

## Prognosis

`UnboundedDeserialization` stands alone at **named** on its honest core
(`from_reader`); `DeserializeWithoutDenyUnknownFields` sits at **suspected** (not
every `Deserialize` is at a trust boundary, and `#[serde(flatten)]` is a known blind
spot the syntactic tell can't see). Both are at the honest tier their current
fingerprint earns. The graduation paths — an in-memory deep-nesting
`#[descended_from]` depth-member, and the tier-promotion of the suspected member —
are recorded in [`../roadmap.md`](../roadmap.md).

---

## See also

- [`../stdlib-families.md`](../stdlib-families.md) — the catalog (this case-file's
  summary row)
- [`../fingerprint-grammar.md`](../fingerprint-grammar.md) — the fingerprint DSL
- [`../witness-tiers.md`](../witness-tiers.md) — the confidence/tier gradient
- [`../../antigen/src/stdlib/deserialization.rs`](../../antigen/src/stdlib/deserialization.rs)
  — the source docstring (ground truth)
