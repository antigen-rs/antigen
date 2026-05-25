# Camp adoption — four adopter-DX findings from antigen's first binary + lightweight-sign adopter

**Date**: 2026-05-24
**Severity**: medium — all four are developer-experience (DX) gaps, not correctness bugs. None block adoption; each adds avoidable friction for the next adopter.
**Observed on**: camp (`R:/camp`) adopting antigen as a path dependency during camp's QoL arc. Camp is antigen's **second** adopter (after tambear) and its **first binary-crate** adopter and **first "I just want to sign it, not write a `requires=` predicate"** adopter — so it exercises paths antigen-on-antigen dogfooding never hits.

## What I was trying to do

Adopt antigen into camp: declare camp's real (mostly substrate-alignment) failure-classes in `R:/camp/src/antigens.rs`, mark the defending code with `#[immune(..., witness = <test>)]`, scaffold + sign `.attest/` sidecars as a lightweight `text-stamp` signer (role: camp-maintainer), and run `cargo antigen scan` / `audit` to see how it looks.

The adoption works end-to-end (5 declared antigens, 4 immune sites, 3 signed sidecars, scan+audit run clean-enough). These four findings are the friction encountered along the way. The live reproduction is camp's tree: `src/antigens.rs`, the immune sites in `src/schema/campsite.rs` / `src/store.rs` / `src/slug.rs`, and the sidecars under `src/schema/.attest/` + `src/.attest/`.

## Finding 1 — marker structs trip `dead_code` in a binary crate

**Symptom**: `#[antigen(...)] pub struct VacuousCompletionFalseGreen;` triggers `warning: struct ... is never constructed`. Under a `-D warnings` clippy gate this is a hard error. Required `#![allow(dead_code)]` on the antigens module.

**Why antigen-side**: the `#[antigen]` / `#[immune]` macros consume the marker type *name as a token* and emit no code that constructs or references the type. In a **library** crate `pub` suppresses `dead_code` (public API); in a **binary** crate there's no external API surface, so `pub` doesn't exempt it and the marker reads as unused. Antigen-on-antigen never sees this because antigen is a lib.

**Suggested fix**: have `#[antigen]` (or `#[immune]`) emit a zero-cost use-token next to the marker, e.g. `const _: fn() = || { let _: VacuousCompletionFalseGreen; };`, so the type counts as used and binary adopters need no allow.

## Finding 2 — importing `AntigenCategory` trips `unused_imports`

**Symptom**: `use antigen::AntigenCategory;` (the natural instinct when writing `category = AntigenCategory::SubstrateAlignment`) yields `unused import: AntigenCategory` → hard error under `-D warnings`. You must *not* import the type.

**Why antigen-side**: `category = AntigenCategory::X` is parsed as a *token path* by the macro, not resolved as a real type reference, so the import is genuinely unused from the compiler's view. Counterintuitive: the argument looks like it needs the type in scope.

**Suggested fix**: document prominently ("do not import `AntigenCategory`; the macro takes it as a token") in the `#[antigen]` macro docs + the quick-start. Optionally accept a bare `category = SubstrateAlignment` to remove the misleading qualified-path form.

## Finding 3 — a signed `.attest/` sidecar is silently ignored when the immune site uses `witness=` (the big one)

**Symptom**: I scaffolded + signed `.attest/` sidecars (`text-stamp`, signer "Claude") for three `SubstrateAlignment` antigens whose `#[immune]` used `witness = <test>`. `attest list` shows the sidecars + signatures correctly, but `audit` is **completely unchanged** — it still reads `witness = <test>` from the macro, still reports the sites at Reachability tier, and still emits `antigen-category-claim-inconsistent-with-predicate-type` ("SubstrateAlignment needs a `requires = ...` immunity"). The sidecar and the immune site are talking past each other and **nothing says so**.

**Why both-sided**:
- *Adopter-side (camp's, deferred):* `witness = <test>` is the wrong witness-*kind* for a `SubstrateAlignment` antigen; the correct form is `requires = <predicate>`. Confirmed cleanly: the one `FunctionalCorrectness` antigen using `witness =` (`SlugPathTraversalUnvalidated`) is **not** flagged — only `SubstrateAlignment`-with-`witness=` are.
- *Antigen-side (the gap):* even granting the adopter's mistake, antigen let me scaffold + sign a sidecar for a `witness=` site and gave **no signal** that the sidecar can never be credited because the immune syntax is `witness=` not `requires=`. A reasonable adopter concludes "I signed it, so it's attested" — and is silently wrong.

**Suggested fix**: when `cargo antigen attest scaffold`/`sign` targets an antigen whose `#[immune]` site uses `witness=` (or when `audit` finds a sidecar with no `requires=` immune to credit it), emit a warning: *"sidecar exists for X but its immune site uses `witness=`; substrate-witness sidecars are credited only for `requires=` immunities."* Close the silent seam between the attestation surface and the macro-declared witness kind.

## Finding 4 — fingerprint `doc_contains` self-matches add scan noise

**Symptom**: `scan` reported 11 "fingerprint match (unmarked site)" advisories, several of which were each antigen's **own declaration struct** matching its own `doc_contains(...)` fingerprint (e.g. `VacuousCompletionFalseGreen on struct` at `antigens.rs:33`).

**Why mostly adopter-side, with an antigen sliver**: our fingerprints are broad (`doc_contains("slug")` matches every fn whose docs mention "slug") — camp should tighten them. But antigen could suppress the trivial case where a declaration's own struct matches its own fingerprint; that match carries no signal.

**Suggested fix (antigen sliver)**: exclude the declaring struct from its own antigen's fingerprint-match report.

## Finding 5 — `signers(required = [...])` matches signer NAME, but the canonical example reads as ROLE (example ⇄ impl drift)

**Symptom**: `requires = signers(required = ["camp-maintainer"])` with a sidecar signed `--signer Claude --role camp-maintainer` → `DisciplinePredicateFailed`. The predicate never matched because `required` is compared against the signer's **name**, not role.

**Why antigen-side**: `antigen-attestation/src/evaluate.rs:393` filters `item.signers.iter().filter(|s| s.name == *needed)` — `required` is a list of **names**. Role is a *separate, optional* constraint via a `roles` map. But the canonical `examples/substrate_witness.rs` declares `signers(required = ["math-researcher"])` and instructs signing with `--signer alice --role math-researcher` — which, by the impl, would **not** match (no signer *named* `math-researcher`). The example's own four-step workflow can't satisfy its own predicate. This is antigen's own `RatifiedSpecDriftFromImpl` / example-vs-impl drift, eating itself again.

**Suggested fix**: decide the intended semantics and align example + impl + docs. If `required` is meant to be roles (which the example implies and which reads more naturally for "a math-reviewer signed"), change the evaluator; if it's names, fix the example's predicate to `required = ["alice"]`. Either way, name-vs-role must be unambiguous at the DSL surface — right now an adopter copying the example gets a silent failure.

## Finding 6 — scan never emits the item's structural fingerprint, so the documented `against="current"` workflow is uncompletable

**Symptom**: `scaffold`/`sign` want `--fingerprint`, and `against = "current"` + `fresh_within_days` both require `signed_against_fingerprint == current_fingerprint`. The example says "use the fingerprint from `cargo antigen scan --format json`." But the scan JSON's `immunities[]` entries carry **no fingerprint field**, presentations carry none, and there is no `cargo antigen fingerprint` subcommand. There is no adopter-reachable way to obtain the value to sign against.

**Consequence**: an adopter following the documented workflow signs against an empty placeholder, and every `against="current"` / `fresh_within_days` predicate then fails (`DisciplinePredicateFailed`) in audit. The only currency that can pass is `against = "any"` — which silently discards the staleness guarantee that is the whole point of the fingerprint. Camp reached Execution tier **only** by dropping `fresh_within_days` and using `against = "any"`.

**Suggested fix (high value)**: emit the computed per-item structural fingerprint in `scan --format json` (on the immunity entry and/or a fingerprints map), or add `cargo antigen fingerprint <file> <item-path>`. Without it, `against="current"` and `fresh_within_days` are effectively dead features for real adopters.

## Finding 7 — `attest check` gives no per-leaf diagnostic; a failed predicate is opaque

**Symptom**: a failing compound predicate reports only tree-level `audit_hint: DisciplinePredicateFailed`. It does not say which leaf failed, what was required, or what was found. Debugging Finding 5 (name-vs-role) required reading `evaluate.rs` source to discover that `required` matched name — the tooling gave zero signal.

**Suggested fix**: `attest check` (and audit) should report per-leaf pass/fail with expected-vs-found, e.g. `signers(required=["camp-maintainer"]): FAIL — no signer named "camp-maintainer" (found names: ["Claude"]; "camp-maintainer" is a role)`. This single improvement would have turned a 20-minute source dive into a 5-second read — exactly the onboarding cost that compounds for every adopter.

## Finding 8 — empty placeholder fingerprint fails silently instead of warning

**Symptom**: `attest scaffold` without `--fingerprint` writes `current_fingerprint: ""` and tells you to "update before signing," but `sign` happily signs against `""`, and audit then fails the predicate with no indication that the empty fingerprint is the cause.

**Suggested fix**: warn (or refuse) when signing a `against="current"`-bound sidecar with an empty `current_fingerprint`, naming the empty fingerprint as the reason the predicate will fail. Pairs with Finding 6 — once the fingerprint is obtainable, this guard makes the failure mode legible.

## What the hardcore `requires=` pass achieved (and why we did NOT defer it)

Initial instinct was to defer the substrate-witness pass. We reversed that: camp is antigen's hardcore adopter, and walking antigen's *full* discipline — not the test-witness shortcut — is the highest-value signal for antigen-in-antigen. Outcome:

- Camp's three `SubstrateAlignment` antigens now use real `requires = signers(...)` substrate-witnesses, are signed via `.attest/` sidecars (signer: Claude, role camp-maintainer, `text-stamp`), and audit credits them at **Execution tier** (`DisciplinePredicatePassedSubstrateCurrent`). The category-consistency hints are cleared.
- The one `FunctionalCorrectness` antigen (`SlugPathTraversalUnvalidated`) correctly stays on `witness = <test>` at Reachability tier — its ceiling until antigen's test-invocation (A4–A5) feature lands.
- Findings 5–8 above were ALL surfaced *by* doing the pass properly. None would have appeared from antigen-on-antigen dogfooding, because antigen-the-lib never walks the adopter's "I followed the example and it silently failed" path.

**Still genuinely camp-side (small):** tighten the broad `doc_contains` fingerprints (Finding 4 adopter-half). And revisit `fresh_within_days` once Finding 6 is fixed (camp dropped it only because the fingerprint was unobtainable).

## How critical

Medium. The adoption works and produced real structural memory in camp. But Findings 1–3 will each bite the *next* adopter (especially the next binary adopter, and anyone who reaches for the lightweight `attest sign` path instead of `requires=`). Finding 3 is the highest-value: a silent disconnect that lets an adopter believe they've attested when audit won't credit it.

## Related

- `R:/camp/src/antigens.rs` — the five camp antigen declarations (live reproduction)
- `R:/camp/src/schema/.attest/`, `R:/camp/src/.attest/` — the signed sidecars
- `docs/expedition/tambear-adoption-log.md` — the first-adopter (tambear) experience, for contrast (tambear is a lib; camp is the first binary + lightweight-sign adopter)
- ADR-028 §Schema — the category↔witness-kind rule that Finding 3 turns on
- ADR-019 — the `.attest/` sidecar / substrate-witness mechanism
