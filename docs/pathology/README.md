# Pathology case-files — the deep tier of the stdlib catalog

Each shipping stdlib failure-class family has a **catalog row** in
[`../stdlib-families.md`](../stdlib-families.md) (what it catches, its tier, its
fingerprint, a one-line summary). This directory expands each row into a full
**pathology case-file** — the deep tier the catalog summarizes — written in a
clinical register:

> **Presentation** (the symptom the developer sees) → **Etiology** (the mechanism,
> with the biology cognate as real mechanism) → **Epidemiology** (the recorded
> real-world advisories/lints) → **Histology** (the fingerprint, annotated
> token-by-token) → **Differential** (the tier reasoning as a diagnostic decision
> tree — why named vs suspected vs chartered) → **Treatment** (the witness) →
> **Prognosis** (the member's current tier; graduation paths live in
> [`../roadmap.md`](../roadmap.md)).

Ground truth is always the per-family source docstring in
[`../../antigen/src/stdlib/`](../../antigen/src/stdlib/); these case-files mirror it
and invent nothing (no fabricated fingerprints, tiers, or advisory IDs).

> **Read a tier as confidence, not severity.** `named` = high-confidence (the
> fingerprint's effective codomain *is* the defect population); `suspected` = a
> correlator (a prompt to look, not a verdict); `chartered` = the class is real but
> no honest fingerprint exists yet. And `present ≠ vulnerable`: a presentation marks
> a site *in the failure-class's territory*, not a confirmed bug — the witness proves
> the defense at audit. The full gradient is in
> [`../witness-tiers.md`](../witness-tiers.md).

## The case-files

| Family | Member(s) | Tier | Case-file |
|---|---|---|---|
| Deserialization-Trust-Boundary | `UnboundedDeserialization`, `DeserializeWithoutDenyUnknownFields` | named, suspected | [deserialization-trust-boundary.md](deserialization-trust-boundary.md) |
| Time-and-Ordering-Hazards | `SystemTimeUnwrapPanic` | suspected | [time-and-ordering-hazards.md](time-and-ordering-hazards.md) |
| Drop-and-Panic-Discipline | `PanicInDrop` | named | [drop-and-panic-discipline.md](drop-and-panic-discipline.md) |
| Panic-on-Index | `GetUncheckedWithoutProof` | named | [panic-on-index.md](panic-on-index.md) |
| Resource-Lifecycle-Leak | `DeliberateLeakNotDocumented` | suspected | [resource-lifecycle-leak.md](resource-lifecycle-leak.md) |
| Async-Soundness | `UnsafeSendSync` | named | [async-soundness.md](async-soundness.md) |
| Numeric-Truncation-Overflow | `SizeOfInElementCount` | suspected | [numeric-truncation-overflow.md](numeric-truncation-overflow.md) |
| Unsafe-Soundness-Boundary | `TransmuteSizeOrLifetimeMismatch`, `UninitMemoryAssumedInit`, `UnvalidatedFromUtf8Unchecked` | named, named, named | [unsafe-soundness-boundary.md](unsafe-soundness-boundary.md) |
| Crypto-Misuse | *(none — chartered)* | chartered | [crypto-misuse.md](crypto-misuse.md) |

**The count**: 8 families ship members (11 members total), and crypto-misuse is
chartered (no member ships yet) — so 8 shipping + 1 chartered = 9 case-files, 11
documented members. This matches the catalog exactly.

## See also

- [`../stdlib-families.md`](../stdlib-families.md) — the catalog (the at-a-glance
  tier)
- [`../fingerprint-grammar.md`](../fingerprint-grammar.md) — the full fingerprint DSL
- [`../witness-tiers.md`](../witness-tiers.md) — the confidence/tier gradient
- [`../examples-guide.md`](../examples-guide.md) — a runnable walkthrough per family
  example
- [`../decisions.md`](../decisions.md) — ADR-027, ADR-028, ADR-039 §C, ADR-040,
  ADR-041
