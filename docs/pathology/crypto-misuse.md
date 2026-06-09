# Pathology case-file — Crypto-Misuse *(chartered — no shipped member yet)*

> The deep tier of the catalog row. Read the catalog entry first:
> [`../stdlib-families.md` § Crypto-Misuse](../stdlib-families.md#crypto-misuse-chartered).
> The source docstring in
> [`../../antigen/src/stdlib/crypto_misuse.rs`](../../antigen/src/stdlib/crypto_misuse.rs)
> is ground truth; this file mirrors it.

- **Family**: Crypto-Misuse
- **Status**: **chartered** — the failure-class is identified and tracked, but **no
  member ships** (no honest call-only fingerprint exists in the shipped grammar yet)
- **Category**: `FunctionalCorrectness` *(prospective — assigned to the future member)*
- **Example/fixture**: none — by design, there is nothing to scan yet

---

This is a **stub case-file**: a chartered family has a real, recorded failure-class
but no specimen to dissect, because shipping a fingerprint today would be
*dishonest*, not merely incomplete. The pathology below documents the class and —
crucially — *why it stays chartered*, since "why no member yet" is the load-bearing
content here.

## Presentation

The would-be flagship member is **`NonConstantTimeSecretComparison`**: a secret or
MAC compared in **non-constant time** — a timing-attack oracle. The developer
hand-rolls a comparison of a secret value (`a == b`, or a manual byte-loop with an
early return on the first mismatch). It's correct functionally, but the *time it
takes* leaks information: an attacker measures response latency to learn how many
leading bytes matched, and recovers the secret byte-by-byte.

## Etiology

The mechanism: `==` (and naive byte-loops) **short-circuit** — they return as soon as
a difference is found. On a secret, that timing difference is the leak. The safe form
is a **constant-time** compare (e.g. `subtle::ct_eq` / `constant_time_eq`) that always
examines every byte regardless of where the first mismatch is.

The recurring shape, per the load-bearing framing (arxiv 1806.04929, "How Usable are
Rust Cryptography APIs?"), is **developer misuse, not bad defaults**: Rust crypto
libraries mostly *avoid* insecure defaults, so the failure-class is reaching past the
safe API for the dangerous one (or omitting the safe step) — not the library shipping
something unsafe.

Biology cognate (as real mechanism): **using the immune machinery wrong**. Reading a
self/non-self marker in non-constant time is the timing leak — the pathogen learning
the antibody's exact shape by watching how long the recognition response takes.

## Epidemiology

Real-world recorded references — cite only what the source actually references:

- **GHSA-q7pg-9pr4-mrp2** — the httpsig-rs HMAC timing attack. This is the *no-call-tell
  hand-rolled defect* the class is about (a hand-rolled `==` / byte-loop on a MAC), and
  it is the substrate that proves the class is real and recurring.
- The **RUSTSEC `crypto-failure` category** includes non-constant-time operations (the
  developer-side view of that category).
- **arxiv 1806.04929** — the misuse-not-defaults framing.

These are the references the source cites; no advisory ID is invented.

## Histology — why no honest fingerprint exists *yet*

This is the heart of the case-file: the class is real, but the **shipped grammar
cannot express an honest fingerprint for it.** Two independent angles confirm it
(codomain-reasoning + empirical crate-API verification):

1. **Anchoring on a verify entrypoint anti-aligns with the defect.** A first attempt
   anchored on a crypto verify entrypoint (`verify` / `hmac_verify` / `verify_mac`)
   and fired on the *absence* of a constant-time compare —
   `not(body_calls("ct_eq"))`. That fires **loudest on the safe path**:
   - `ring::hmac::verify(key, msg, tag)` is the **correct** API and is constant-time
     **internally** (the constant-time work is inside `ring`, with *no visible
     `ct_eq` call*). So the fingerprint would **falsely bind a `ring::hmac::verify`
     call** — looks undefended, gets flagged — reporting a named crate's
     **recommended API as the bug**. This is the clean-sibling-collision shape (cf.
     the `Instant::elapsed` drop in [time-and-ordering](time-and-ordering-hazards.md)):
     the anchor's codomain *includes the clean path*.
   - `verify` / `hmac_verify` are the **names of the safe operation** (a crypto lib's
     `mac.verify(tag)` does the constant-time compare itself), so anchoring on
     verify-presence anchors on the safe pattern's *vocabulary*.

2. **The real defect has no distinctive call.** The vulnerable pattern
   (GHSA-q7pg-9pr4-mrp2) is a **hand-rolled `==` / manual byte-loop on a secret** — an
   **operator** (`==`) on a **secret-typed value**, with no crypto-entrypoint call at
   all. The honest fingerprint would be:

   ```text
   all_of([
       <security_sensitive_name anchor>,
       not(any_of([
           body_calls("ct_eq"),
           body_calls("constant_time_eq"),
       ])),
   ])
   ```

   and it needs **both deferred grammar leaves**: a `security_sensitive_name`
   name-leaf (the *data-context*: does this fn hold secret bytes it might
   hand-compare?) and an `==` operator-leaf (the precise positive tell, `ExprBinary`).
   `body_calls` sees only `ExprCall` / `ExprMethodCall` — **neither operators nor a
   data-context.** Neither leaf ships in the current grammar.

## Differential — why chartered, not suspected

A chartered family is one tier below suspected in confidence: a suspected member at
least has a *correlator* that fires more on the defect than off it. Here, the only
expressible call-only fingerprint fires **loudest on the safe path** (it would flag
`ring::hmac::verify`, the recommended API). That is *worse than no member* — it would
actively mislead. So the honest disposition is **chartered**: the class is named and
tracked, the graduation path is recorded, and nothing ships until the fingerprint can
be at least a correlator rather than an anti-correlator.

## Treatment — the fix for the class

The fix is the same whether or not a fingerprint can yet name the defect: use a
constant-time comparator (`subtle::ct_eq` / `constant_time_eq`) on secrets and MACs,
or call the library's own `verify` (which does the constant-time compare internally)
rather than hand-rolling `==`.

## Prognosis

The family stays **chartered** — better honest-deferred than dishonest-shipped, since
a shipped call-only form would actively mislead by flagging the *correct*
`ring::hmac::verify` as the bug. Shipping `NonConstantTimeSecretComparison` (even at
the suspected tier) needs grammar leaves the call-only `body_calls` lacks — a
`security_sensitive_name` data-context leaf and ideally the `==` operator-leaf. That
graduation path is recorded in [`../roadmap.md`](../roadmap.md).

---

## See also

- [`../stdlib-families.md`](../stdlib-families.md) — the catalog row (chartered)
- [`time-and-ordering-hazards.md`](time-and-ordering-hazards.md) — the
  `Instant::elapsed` clean-sibling-collision is the same shape that defeats the naive
  crypto anchor
- [`../fingerprint-grammar.md`](../fingerprint-grammar.md) — the fingerprint DSL (and
  the deferred name/operator leaves)
- [`../witness-tiers.md`](../witness-tiers.md) — the tier gradient (chartered =
  no honest fingerprint yet)
- [`../../antigen/src/stdlib/crypto_misuse.rs`](../../antigen/src/stdlib/crypto_misuse.rs)
  — the source docstring (ground truth)
