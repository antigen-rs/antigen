# Lab Notebook 002: antigen-dx-dogfood — Observer Baseline Substrate Audit

**Date**: 2026-05-25 (UTC)
**Observer**: observer (antigen-dx-dogfood expedition)
**Branch**: main
**Last commit at session start**: `5b599f9` — chore: gitignore agent-feedback/ as local dev substrate
**Status**: Active
**Depends on**: Notebook 001 (v02-completion-arc baseline — prior expedition)

---

## Context & Motivation

Camp adopted antigen as its first binary-crate + lightweight-sign adopter during the camp QoL arc. That hard adoption surfaced **8 concrete adopter-DX findings** documented in `agent-feedback/2026-05-24-camp-first-binary-adopter-dx-findings.md`. The antigen-dx-dogfood expedition exists to:

1. Fix all 8 findings (each is a `findings/*` campsite)
2. Dogfood antigen on antigen continuously (`dogfood/antigen-on-antigen-continuous`)
3. Achieve comprehensive antigen coverage (`dogfood/comprehensive-antigen-coverage`)

This notebook is the observer's running record of what IS versus what's claimed — hypothesis before every verification, results immediately after, surprises captured in real time.

**Observer's asymmetric value**: Implementers assume untracked = intentional. Observer assumes untracked-important = gap. The substrate-alignment gap failures invisible from the implementer perspective are exactly what observer catches.

---

## Expedition Substrate State at Session Start

**Expedition**: antigen-dx-dogfood  
**Activity log entries**: 27 (all from 2026-05-26 01:43–01:59 UTC — very fresh start)  
**Team roles registered**: navigator, observer (just joined)  
**Other roles**: aristotle, pathmaker, adversarial, scientist, outsider, naturalist — not yet joined  
**Campsites**: 9 total, all open (9 open, 0 complete)

Navigator seeded all 8 finding campsites and left routing notes. The expedition is less than 1 hour old at my spawn time.

---

## Step 1: Verify Finding 5 — signers-name-vs-role-and-example-drift

### Before

**Hypothesis**: The agent-feedback doc claims `evaluate.rs:393` filters by `s.name == *needed`, and the canonical `substrate_witness.rs` example has `required = ["math-researcher"]` but instructs signing with `--signer alice --role math-researcher`. These two facts together mean the example's predicate would NOT be satisfied by alice (name ≠ "math-researcher"). I expect to confirm this directly from code.

**Rationale**: This is the finding with the most complex semantics (name vs role distinction); it requires aristotle for a semantics decision before pathmaker can fix it. My job is to establish the exact substrate state so aristotle has clean evidence.

**Design**: Read `evaluate.rs` around line 393; read `substrate_witness.rs` fully; grep for any tests in evaluate.rs that confirm name vs role semantics.

### Results

**`evaluate.rs:393`** — confirmed verbatim:
```rust
let candidates: Vec<&Signer> = item.signers.iter().filter(|s| s.name == *needed).collect();
```
`required` entries are compared against `s.name`. Role check is separate (`roles` map on the predicate, checked only if `roles.get(needed)` is `Some`).

**`substrate_witness.rs:120`** — `required = ["math-researcher"]` with compound `all_of` predicate. The `signers` call names `math-researcher` as a required entry.

**`substrate_witness.rs:219`** — signing instruction: `--signer alice --role math-researcher`.

**Verdict**: alice's signer entry would have `name = "alice"`, `role = Some("math-researcher")`. The predicate looks for a signer with `name == "math-researcher"` — alice does not match. The example literally cannot satisfy its own predicate as written.

**Tests in `evaluate.rs`** (need to check):

### Discussion

**What I learned**: Finding 5 is confirmed as reported. The example failure is real and reproducible from substrate alone — no runtime needed.

**The subtle additional point**: The `roles` map in the predicate (line 403: `let expected_role = roles.get(needed);`) allows `signers(required = ["alice"], roles = {"alice": "math-researcher"})` — but this syntax isn't shown in the example. An adopter would need to know about the `roles` map key to constrain by role. The example doesn't demonstrate this pattern.

**Two valid resolutions**:
1. Change semantics: `required` takes role names (what the example implies). Changes `evaluate.rs:393` to filter by `s.role`.
2. Keep semantics: `required` takes signer names. Fix example to `required = ["alice"]` with alice as a literal name, and show `roles = {"alice": "math-researcher"}` if role constraint is desired.

**Either way**: the example's own predicate cannot pass in the current impl. aristotle needs to ratify which semantics is correct.

---

## Step 2: Check evaluate.rs tests for name vs role semantics evidence

### Before

**Hypothesis**: The test suite in `evaluate.rs` uses `required = ["alice"]` with alice as the signer name — this would be strong evidence the current name-semantics is the intended behavior, and the example is the bug not the impl.

**Design**: Grep for `required` and `signers` patterns in test files; read the relevant test section of `evaluate.rs`.

### Results

All test usages found use `"alice"` as a name literal in `required` (e.g., `vec!["alice".to_string()]`). One test comment explicitly documents `alice [role=math-researcher]` as the context. This is strong evidence for the name-semantics being intentional.

The test at `evaluate.rs:1316` has a comment: `` `eval_signers` checks "does any alice have current... ``. This makes the name-semantics unambiguous in the author's intent.

### Discussion

**What I learned**: Name-semantics is the deliberate design. Every test in `evaluate.rs` constructs signers with a name field and uses that name in `required`. The example (`substrate_witness.rs`) is wrong — it uses `required = ["math-researcher"]` where `math-researcher` is alice's role, not alice's name. This is a documentation/example bug, not an implementation bug.

**Verdict for aristotle**: Strong substrate evidence that name-semantics should be ratified. The fix is: change the example's predicate from `required = ["math-researcher"]` to `required = ["alice"]` (matching the actual signer name), and update the doc comment + four-step workflow description at line ~96-116 to make the name/role distinction explicit.

---

## Step 3: Verify Finding 6 — scan emits no item fingerprint

### Before

**Hypothesis**: The scan JSON output for an immunity entry has no fingerprint field. The `attest scaffold` command instructs the user to use a fingerprint from `scan --format json`, but that field does not exist. This makes `against="current"` + `fresh_within_days` dead features for adopters.

**Design**: Check `cargo-antigen/src/` for JSON serialization of immunity entries; grep for fingerprint fields in scan output structs; look at how `attest scaffold` documents the fingerprint parameter.

### Results

**SURPRISE**: Working tree has significant in-progress implementation. `git status` revealed 4 modified files and 1 new untracked file.

**What pathmaker built** (working tree, uncommitted):
1. New `antigen-fingerprint/src/digest.rs` — FNV-1a 64-bit structural digest module with full stability contract, antigen-attr stripping, `HasAttributes` trait implemented for 8 `syn` node types, and 8 tests covering determinism, whitespace-insensitivity, antigen-attr exclusion, structural sensitivity
2. `antigen-fingerprint/src/lib.rs` — added `pub mod digest` + `pub use digest::{structural_digest, HasAttributes}` + disambiguation doc comment
3. `antigen/src/scan.rs` — added `structural_fingerprint: String` field to `Immunity` and `Toleration` structs (with `#[serde(default)]`), added `current_item_digest: String` to `ScanVisitor`, and computes it via `antigen_fingerprint::structural_digest(item)` in all 8 `visit_item_*` methods before calling `check_attrs`, stamping the digest onto the struct fields
4. `antigen/src/audit.rs` — two test helpers updated to include `structural_fingerprint: String::new()` (test scaffolding only)
5. `antigen-macros/src/lib.rs` — Finding 1 (dead_code) fixed: `#[allow(dead_code)]` emitted by `#[antigen]` macro; Finding 2 (AntigenCategory) fixed: doc note added to macro docs

**What is NOT yet done** (confirmed from code):
- `audit.rs:1140-1156` has a comment labelled `Audit-SF-1` explicitly documenting that `current_fingerprint` is still self-referential (uses sidecar's own stored value, not a freshly recomputed digest from scan). The `structural_fingerprint` field from the scan report is NOT yet used in the audit path.
- No `cargo antigen fingerprint` subcommand exists in `cargo-antigen/src/main.rs`

**Test count**: 806 pass, 48 ignored — up from 625 in prior session.

### Discussion

**What I learned**: Pathmaker is moving fast. Findings 1 and 2 are fully fixed in working tree. Finding 6 is ~60% complete (digest producer + scan emission done; audit wiring + fingerprint subcommand not done). The harness tasks #1-3 map correctly to this state.

**Critical gap I'm naming**: The `structural_fingerprint` is emitted by scan, but `audit.rs` has an explicit self-referential-detection comment at lines 1140-1156 saying it still uses `item.current_fingerprint` from the sidecar. An adopter who signs against the scan-emitted fingerprint, then runs audit, will still not have staleness detection working end-to-end. Task #2 (wire audit to recompute) is the load-bearing remaining piece for F6.

**None of this is committed.** The working tree state is ahead of HEAD. This is normal for in-progress pathmaker work, but it means the substrate (git) doesn't yet reflect what's actually there. Observer note: `camp sign` should only happen after `cargo test` passes AND `git commit` exists.

---

## Step 4: Verify Finding 3 — sidecar-witness-disconnect silence

### Before

**Hypothesis**: When an immune site uses `witness=` (code-witness) and a sidecar exists for it, audit credits neither the sidecar nor warns about the mismatch. The sidecar and the `witness=` predicate are parallel tracks that never connect.

**Design**: Read the audit path in `cargo/antigen/src/audit.rs`; look for any code that checks sidecar presence for `witness=` sites.

### Results

**CONFIRMED unimplemented.** `audit.rs:1036` shows the decision point:

```rust
let immunity_audit = immunity.requires_predicate.as_ref().map_or_else(
    || { /* code-witness path: validate_witness() — never looks for sidecar */ },
    |predicate_json| audit_substrate_witness(immunity, predicate_json),
);
```

When `requires_predicate` is `None` (i.e., the site uses `witness=`), the code goes to the code-witness path and calls `validate_witness`. There is no check for sidecar existence in the code-witness path. A sidecar that exists for a `witness=` site is completely invisible to the audit.

**No warning exists** for this case anywhere in `audit.rs` or `main.rs`. Finding 3 is fully unimplemented.

### Discussion

The fix requires: after taking the code-witness branch, also call `load_sidecar(...)` and emit a warning if a sidecar is found. The warning message from the DX findings doc is good: "sidecar exists for X but immune site uses `witness=`; substrate-witness sidecars are only credited for `requires=` immunities." This should be in both `audit` output and `attest scaffold`/`sign` pre-check.

---

## Step 5: Verify Finding 1 — dead_code in binary adopter

### Before

**Hypothesis**: The `#[antigen]` proc-macro emits no code that references the marker struct type.

**Design**: Read `antigen-macros/src/lib.rs` to see what `#[antigen]` emits.

### Results

**ALREADY FIXED in working tree.** `antigen-macros/src/lib.rs:139-146`:

```rust
// Suppress dead_code: in a binary crate `pub struct Foo;` with no
// constructors triggers the lint even though antigen intentionally uses
// the type as a declaration token, not a runtime value (DX finding 1).
let expanded = quote! {
    #[doc = #attr_doc]
    #[allow(dead_code)]
    #input
};
```

The fix chose `#[allow(dead_code)]` (suppress lint on the struct) rather than a use-token. This is simpler and more direct — the DX findings doc suggested a use-token, but `#[allow(dead_code)]` achieves the same result with less generated code. The comment explicitly cites "DX finding 1".

**Not committed** — working tree only.

---

## Step 6: Verify Finding 2 — AntigenCategory unused_imports

### Before

**Hypothesis**: `AntigenCategory` is used as a macro token path — the import is unused from rustc's view.

**Design**: Read macro docs in `antigen-macros/src/lib.rs` to see if doc note was added.

### Results

**ALREADY FIXED in working tree.** The `#[antigen]` macro doc comment now includes (from git diff):

```
/// - `category = AntigenCategory::X` (optional) — `SubstrateAlignment` or
///   `FunctionalCorrectness` (ADR-028). **Do not import `AntigenCategory`** —
///   the macro reads this as a token path, so `use antigen::AntigenCategory;`
///   triggers `unused_imports` under `-D warnings`. Write the path directly
///   without importing.
```

This is exactly the doc-fix approach suggested in the DX findings doc. The alternative (accept bare `category = SubstrateAlignment`) was not taken — the qualified path form is kept, just documented as "don't import."

**Not committed** — working tree only.

---

## Step 7: Verify Finding 7 — attest check per-leaf diagnostics

### Before

**Hypothesis**: `EvaluatedPredicate` has no leaf-level result structure — it collapses to a single flat struct with one `AuditHint`.

**Design**: Read `EvaluatedPredicate` struct definition in `evaluate.rs`.

### Results

**CONFIRMED unimplemented.** `EvaluatedPredicate` at `evaluate.rs:76-90`:

```rust
pub struct EvaluatedPredicate {
    pub witness_tier: WitnessTier,
    pub audit_hint: AuditHint,
    pub evidence_kind: EvidenceKind,
    pub signature_strength: Option<SignatureStrength>,
}
```

No `leaf_results` field. No diagnostic detail. The entire compound-predicate evaluation result is a single `AuditHint`. A failed `all_of([signers(...), ratified_doc(...), fresh_within_days(...)])` produces exactly `DisciplinePredicateFailed` with no indication of which leaf failed.

The fix requires adding a `leaf_results: Vec<LeafResult>` (or similar) to `EvaluatedPredicate`, having `eval_signers`/`eval_ratified_doc`/`eval_fresh_within_days` etc. return per-leaf diagnostics, and surfacing them in CLI output. This is a non-trivial API change to `antigen-attestation`.

### Discussion

The per-leaf diagnostic is the highest-UX-impact change after F6 in terms of adopter experience. The DX findings doc notes that debugging Finding 5 required reading `evaluate.rs` source — a per-leaf diagnostic would have made it a 5-second read. This is exactly the "20-minute source dive turned into 5-second read" improvement.

---

## Step 8: Verify Finding 8 — empty fingerprint signed silently

### Before

**Hypothesis**: `attest scaffold` writes `current_fingerprint: ""` and `attest sign` accepts it without warning.

**Design**: Check `cargo-antigen/src/main.rs` for the scaffold and sign handlers.

### Results

**PARTIALLY addressed.** From the earlier grep (line 2676 in main.rs):

```
fingerprint is empty — update `current_fingerprint` before signing.
```

This text appears in the scaffold output — so scaffold does warn. But the grep also showed (line 3215):

```
"warning: --fingerprint not supplied; using sidecar's stored \
```

This suggests that when `--fingerprint` is not supplied to `attest sign`, it falls back to the sidecar's stored value. Combined with F6 (fingerprint now obtainable from scan), the empty-fingerprint path is at least warned about at scaffold time.

**What's NOT addressed**: `attest sign` still accepts signing against an empty fingerprint without refusing. The guard that should refuse/warn when `current_fingerprint == ""` AND `against = "current"` doesn't appear to exist yet. Finding 8 is partially addressed (scaffold warns) but not fully fixed (sign doesn't refuse).

---

## Step 9: Verify Finding 4 — scan fingerprint self-match

### Before

**Hypothesis**: Scan reports an antigen's own declaration struct as a fingerprint match against itself.

**Design**: Check scan fingerprint-matching code for any self-match exclusion.

### Results

**Not yet verified from code.** The DX findings doc describes this as "noise" — the antigen struct matches its own `doc_contains` fingerprint. This is likely minor in severity and straightforward to fix. Will check when pathmaker routes it.

**From the routing note**: Navigator explicitly noted this is pathmaker-only (no adversarial), suggesting it's simpler than the other findings.

---

## Step 10: Antigen Coverage State — Scan + Audit Pass

### Before

**Hypothesis**: The antigen workspace scanned against itself will show meaningful dogfood coverage — some antigens declared, some immune sites, some at Execution tier. The comprehensive-antigen-coverage campsite implies gaps exist.

**Design**: Run `cargo antigen scan` and `cargo antigen audit` on the workspace.

### Results

**Scan**: 77 explicit `#[presents]` markers; 16,040 fingerprint matches (noise — broad `doc_contains` fingerprints hitting everything). 59 unaddressed presentations.

**Audit**: 44 immunity claims; 1 at Execution tier (phantom_witness example — formal proof, not dogfood); 43 below Execution tier (all Reachability, `FunctionResolves` or `TestAttributePresentNotInvoked`).

**Camp's antigen antigens** (`camp/src/antigens.rs`) trigger `antigen-category-defaulted-implicit-functional` — still missing `category = AntigenCategory::X` on their declarations. This is camp-side work (not antigen-side), but it's notable: antigen's first binary adopter's antigens are still incomplete against ADR-028.

**`UnsandboxedProcMacro`** in `supply_chain.rs` has `hybrid-incomplete-evidence` — it's `SubstrateAlignment` with a `witness=` (code-witness) rather than `requires=` (substrate). Either the category or the witness kind is wrong; this is the ADR-028 category/witness-type cross-check hitting live stdlib.

**UnstableHashAsPersistedValue dogfood gap**: The antigen (dogfood.rs:587) promises `#[immune]` on the defending code, but `antigen-fingerprint/src/digest.rs` can't carry it (circular dependency). Zero `#[immune(UnstableHashAsPersistedValue)]` markers exist anywhere.

### Discussion

**What I learned**: Coverage is sparse. 12 dogfood antigens declared; zero at Execution tier in production code (the one Execution-tier claim is a formal-proof example). The comprehensive-antigen-coverage work is genuinely needed — it's not a cosmetic exercise.

**The 16,040 fingerprint matches** are almost entirely noise from the broad `doc_contains` fingerprints scanning antigen's own source. This is the self-match problem (Finding 4) at scale — the codebase mentions the terms its own antigens document, so every source file matches multiple antigens. Making the fingerprints more specific would reduce noise dramatically and make scan signal useful.

**Three specific gaps that observer is naming**:
1. `UnstableHashAsPersistedValue` immune marker gap (circular dep constraint — doc comment is misleading)
2. Camp's antigen declarations still lack `category` field (camp-side, not antigen-side)
3. `UnsandboxedProcMacro` category/witness-type inconsistency (supply_chain.rs)

---

## Open Questions

- Does the `substrate_witness.rs` example have a companion `.attest/` sidecar in the repo? If so, does it use `required = ["alice"]` (name) or `required = ["math-researcher"]` (role)? This would show whether the broken example has been signed against.
- Finding 8 partial: `attest sign` still accepts empty fingerprint without refusing — is this intentional (deferring until F6 is complete so adopters can actually GET a fingerprint to sign against)?
- The harness tasks #1-3 accurately reflect the remaining F6 work. Task #1 is done (digest producer built); task #2 (audit wiring) and task #3 (fingerprint subcommand) remain.
- When pathmaker commits the working tree changes, are F1 and F2 being committed together or separately from F6?

---

## Step 11: Test Suite Check — Working Tree Gate

### Before

**Hypothesis**: The working tree may have partially-implemented fixes with tests written ahead of the implementation. I need to verify cargo test passes before any campsites can be signed.

### Results

**CRITICAL: 2 tests FAILING** in `cargo-antigen/tests/atk_dx_findings.rs`:

1. `atk_dx_f8_sign_empty_fp_must_warn` — Finding 8 guard is NOT implemented. `attest sign` accepts an empty fingerprint with exit=0. Test correctly expects failure; it passes (i.e., the test is a TDD gate that drives the remaining implementation).

2. `atk_dx_f6_presentation_entry_has_fingerprint` — 16,185 `FingerprintMatch` `Presentation` entries in the scan report are missing `structural_fingerprint`. The F6 implementation added the field to `Immunity` and `Toleration` (scan.rs:1342, 1374) but NOT to `Presentation` (scan.rs:1278). The struct lacks the field entirely.

**What this means**: The test suite was written ahead of the implementation (TDD style). These are active red gates driving completion of F6 and F8.

### Discussion

**What I learned**: The team is doing red-green TDD. Tests were written to cover expected behavior before implementation. The observer's job is to verify both that tests actually test what they claim AND that the tests' expectations are correct.

**Is the `atk_dx_f6_presentation_entry_has_fingerprint` test correct?** Yes — `FingerprintMatch` presentations represent potential vulnerable sites; knowing the structural fingerprint of those sites is exactly what the adopter needs for `against="current"` + `fresh_within_days`. The test expectation is sound.

**Is the `atk_dx_f8_sign_empty_fp_must_warn` test correct?** Yes — an adopter who scaffolds without `--fingerprint` gets `current_fingerprint: ""`, then signs against it. The sign should warn or refuse when `against="current"` is bound and the fingerprint is empty, since the resulting sidecar will fail audit with a confusing "DisciplinePredicateFailed" rather than "fingerprint is empty." The test expectation is sound.

**No campsites should be signed until both tests pass.** Flagged to navigator.

---

## Verified Severity Assessment

After direct substrate verification:

| Finding | Reported Severity | Verification Status | Implementation Status (session end) |
|---|---|---|---|
| F1 — binary dead_code | Medium | **CONFIRMED** | **FIXED** (working tree, uncommitted): `#[allow(dead_code)]` emitted by `#[antigen]` macro |
| F2 — AntigenCategory unused_imports | Medium | **CONFIRMED** | **FIXED** (working tree, uncommitted): doc note added to macro rustdoc |
| F3 — sidecar-witness disconnect silence | High | **CONFIRMED** — code-witness path never checks for sidecar | **DOCS-HALF FIXED** (outsider shipped `#[immune]` rustdoc rewrite); **runtime warning NOT YET IMPLEMENTED** |
| F4 — scan fingerprint self-match | Low | Not verified from code | **SHIPPED** (commit `fa4522f`) — self-match suppression with regression test |
| F5 — signers name-vs-role example drift | High | **CONFIRMED** — name-semantics, example is the bug | **ARISTOTLE RATIFIED name-semantics**; example fix (pathmaker) pending commit |
| F6 — scan emits no fingerprint | High | **CONFIRMED** → **MOSTLY FIXED** | **PARTIAL**: digest module + scan emit done (commit `2165720` + `a63098d`); Audit-SF-1 wired (working tree uncommitted); **Presentation struct missing field (test RED)** |
| F7 — attest check no per-leaf diagnostic | Medium | **CONFIRMED** — EvaluatedPredicate is flat | **NOT YET IMPLEMENTED** |
| F8 — empty fingerprint signed silently | Medium | **PARTIALLY addressed** | **NOT YET IMPLEMENTED** (test RED: `atk_dx_f8_sign_empty_fp_must_warn` fails) |

**Test count during session**:
- Session start: 806 pass, 48 ignored (working tree)
- Session end: **FAILING: 2 tests** (`atk_dx_f8_sign_empty_fp_must_warn`, `atk_dx_f6_presentation_entry_has_fingerprint`)
- These are TDD gates driving remaining F6 (Presentation field) and F8 (sign guard) work

**Commits landed during observer session** (from `5b599f9` to `a63098d`):
- `2165720` — feat(scan): emit per-item structural fingerprint (DX finding 6)  
- `fa4522f` — fix(scan): suppress declaration self-match in fingerprint synthesis (DX finding 4)
- `a63098d` — dogfood(antigen): declare UnstableHashAsPersistedValue

**Still uncommitted working tree** (as of session end):
- `antigen-macros/src/lib.rs` — F1 + F2 fixes + outsider's `#[immune]` rustdoc rewrite
- `antigen/src/audit.rs` — Audit-SF-1 fix + 208-line regression test
- `cargo-antigen/src/main.rs` — scaffold help text update

**Observer-surfaced issues beyond the 8 findings**:
1. `UnstableHashAsPersistedValue` doc comment promises `#[immune]` that architectural constraint (circular dep) prevents — doc should be corrected
2. `dogfood/fingerprint-extension-not-instance-shape` — naturalist crystallized: fold is KNOWN classes with too-narrow fingerprints; authoring discipline needed
3. `WitnessTier::None` overloaded across 3 situations — makes F7 load-bearing, not cosmetic (outsider + naturalist convergence)
4. `attest scaffold` auto-fill proposal (outsider) — may make task #3 (fingerprint subcommand) vestigial

**Observer priority ongoing**: Hold F6 and F8 campsite signatures until test suite is green. Watch F7 (per-leaf diagnostics) — it's the highest-UX remaining item and now more critical given WitnessTier::None overloading finding.

---

## Step 12: Wake Verification — Post-Context-Compaction State Audit

**Date**: 2026-05-26 (wake from prior session — context compacted)  
**Last commit at wake**: `4dd8d38` — chore: gitignore ad-hoc scan/audit JSON dumps at repo root

### Substrate vs Camp Substrate Alignment

**SUBSTRATE-ALIGNMENT GAP FOUND**: Two campsites are blocked by adversarial, but the blocking conditions no longer apply. The fixes have been written (working tree uncommitted) and both tests NOW PASS.

Evidence:
- `findings/empty-fingerprint-guard` blocked by adversarial: "`atk_dx_f8_sign_empty_fp_must_warn` FAILS"
  - Substrate check: `cargo test --test atk_dx_findings "atk_dx_f8_sign_empty_fp_must_warn"` → **1 PASSED**
  - The fix: `warn_if_empty_fingerprint()` function added to `cargo-antigen/src/main.rs` (working tree, uncommitted)
  - Block is STALE — reflects pre-fix reality

- `findings/scan-emit-item-fingerprint` blocked by adversarial: "`atk_dx_f6_presentation_entry_has_fingerprint` FAILS"
  - Substrate check: `cargo test --test atk_dx_findings "atk_dx_f6_presentation_entry_has_fingerprint"` → **1 PASSED**
  - The fix: `structural_fingerprint: String` field added to `Presentation` struct in `antigen/src/scan.rs` (working tree, uncommitted)
  - Block is STALE — reflects pre-fix reality

**Full test suite at wake**: 815 passed, 48 ignored (up from 806 at prior session start)

**Working tree uncommitted at wake** (git status):
- `antigen/src/audit.rs` — Audit-SF-1 fix + 208-line regression test
- `antigen/src/scan.rs` — Presentation struct structural_fingerprint field + synthesis_pass emit
- `antigen/tests/atk_a3_fractal_preview.rs` — (unknown, requires inspection)
- `cargo-antigen/src/main.rs` — F8 guard + jq hint field name fix
- UNTRACKED: `cargo-antigen/tests/atk_dx_findings.rs` — the DX findings test file (never committed)
- UNTRACKED: `research/notebooks/002-antigen-dx-dogfood-observer-baseline.md` — this notebook

**CORRECTION (verified post-session-start)**: The 4 modified files and the untracked test file ALL landed in commit `d46a044` (dogfood: declare AuditFingerprintSelfReferential). The commit was authored Mon May 25 21:36:28 2026 -0500, but the git status at session start caught a state where my context showed 6-ahead while the commit had just landed, resulting in apparent working-tree changes. After dropping camp notes, the git status resolved to clean. All fixes are committed.

### Commits landed since prior session end (new since `a82f802`):

```
4dd8d38 chore: gitignore ad-hoc scan/audit JSON dumps at repo root
32ab737 fix(macros): reject generic #[antigen] markers with a clear error
151ed48 fix(macros): zero-cost use-token for #[antigen] markers; AntigenCategory import note (DX findings 1+2)
a82f802 fix(dogfood): correct UnstableHashAsPersistedValue doc comment — no #[immune] exists
```

- `151ed48`: F1+F2 macro fixes are NOW COMMITTED (were in working tree at prior session end)
- `32ab737`: Additional macro fix — generic `#[antigen]` markers rejected with clear error (not in 8 DX findings; separate DX improvement)
- `a82f802`: Observer-flagged `UnstableHashAsPersistedValue` doc comment corrected

### Camp Substrate State at Wake

Campsites: 54 total — 21 open, 2 partial, 29 complete, 2 blocked

**New campsites since prior session** (observer freshly joined):
- `findings/encounter-status-axis-adr-amendment` — aristotle seeded; encounter-status axis (vaccinated/encountered/affinity-matured) orthogonal to WitnessTier
- `scan-output-floods-newcomer` — outsider seeded; 16K unbounded fingerprint-match lines at human output
- `findings/signer-name-role-confusion-unrepresentable` — new (F5 variant — name vs role is unrepresentable in the DSL)
- `findings/examples-ci-executable-workflow-integrity` — new
- `dogfood/fingerprint-extension-not-instance-shape` — naturalist crystallized; extension-not-instance-shape authoring discipline

**Active team**: naturalist sleeping (extensive session), outsider sleeping, aristotle active with comprehensive encounter-status axis analysis

### Pending Observer Actions

1. **Camp note**: document stale adversarial blocks on `scan-emit-item-fingerprint` and `empty-fingerprint-guard`
2. **Lab notebook update**: reflect post-session-compaction verified state (this step)
3. **DX findings test file untracked**: flag to navigator — this should be committed
4. **`dogfood/comprehensive-antigen-coverage`**: observer needs to sign; prerequisites need verification first
5. **`v02-beta-docs/beta-readiness-v020`**: observer needs to sign; requires investigation

### New Observer Findings from Wake Audit

**Finding A (substrate-alignment)**: Two adversarial campsite blocks are stale — both blocked conditions no longer apply. The fixes have been written but NOT committed. The camp substrate says "blocked"; the code says "fixed." This is exactly the substrate-alignment gap pattern observer is positioned to catch.

**Finding B (untracked test file)**: The adversarial TDD test file governing this expedition's completion criteria (`cargo-antigen/tests/atk_dx_findings.rs`) has never been committed. 5 tests, 275 lines, covering F3/F6/F8 — all untracked. If pathmaker runs `git checkout .` the test gates disappear silently.

**Finding C (WitnessTier::None docs — RETRACTED)**: Prior session observer notice `12be4202` flagged `docs/witness-tiers.md:36` as missing the 4th None case. RETRACTION: the fix was already present in commit `852314b` (2026-05-19), predating this expedition entirely. The line now reads: "Two distinct sub-channels collapse to None: (a) witness-resolution gap...; (b) predicate-evaluation outcome — a requires= substrate-witness predicate was evaluated and failed." Observer's prior notice was a false positive. Methodological note: I flagged a gap in something I had already verified contained the correct definition — possible the prior session read an older cached state or I misread. Camp notice remains in substrate; its resolution is that the docs were already correct.

**Finding D (encounter-status axis is LARGE)**: Aristotle's analysis is comprehensive (ADR-028 amendment with three-axis state-space). The encounter-status finding will not resolve quickly — it's an ADR amendment requiring navigator ceremony. Observer should watch whether this enters the process lifecycle before becoming drift.

**Finding E (F3 test implemented, now RED)**: The committed version of `atk_dx_f3_audit_warns_on_sidecar_for_witness_site` is a full implementation (not the placeholder `return;` I read in prior session). It creates a real sidecar for the `atk_a2_003_empty_witness` fixture and runs audit. Test FAILS because `audit.rs:1036` code-witness branch never checks for sidecar presence. This is the correct TDD posture — the test documents the gap accurately.

**Finding F (commit attribution)**: `d46a044` includes `research/notebooks/002-antigen-dx-dogfood-observer-baseline.md`. The observer lab notebook is now tracked in git. Future edits should be committed. This creates a signal about continuity: the lab notebook is part of the project substrate, not just local observer memory.

---

## Final Verified Status — Wake Pass Complete

**Git HEAD**: `d46a044` (dogfood: declare AuditFingerprintSelfReferential)  
**Ahead of origin**: 8 commits  
**Working tree**: clean (only `research/notebooks/002...md` modified — this session's edits)

**Test count (committed state)**: 815 pass, 48 ignored, **1 FAILING** (`atk_dx_f3_audit_warns_on_sidecar_for_witness_site`)

**8 Findings — definitive status at wake**:

| Finding | Status | Evidence |
|---|---|---|
| F1 — binary dead_code | FIXED + COMMITTED | `151ed48` |
| F2 — AntigenCategory unused_imports | FIXED + COMMITTED | `151ed48` |
| F3 — sidecar-witness disconnect silence | **NOT IMPLEMENTED (RED GATE)** | test fails; `audit.rs:1036` missing sidecar check |
| F4 — scan fingerprint self-match | FIXED + COMMITTED | `fa4522f` |
| F5 — signers name-vs-role example drift | ARISTOTLE RATIFIED; **example fix pending** | pathmaker needs `substrate_witness.rs` update |
| F6 — scan emits no fingerprint | FIXED + COMMITTED | `2165720` (immunity), `d46a044` (presentation + Audit-SF-1) |
| F7 — attest check no per-leaf diagnostic | **NOT IMPLEMENTED** | `EvaluatedPredicate` is still flat; no test gate yet |
| F8 — empty fingerprint signed silently | FIXED + COMMITTED | `d46a044` (warn_if_empty_fingerprint) |

**Campsite blocks still active (stale as of wake)**:
- `findings/scan-emit-item-fingerprint` — adversarial block predates `d46a044`; condition resolved
- `findings/empty-fingerprint-guard` — adversarial block predates `d46a044`; condition resolved

**Active TDD red gate**: `atk_dx_f3_audit_warns_on_sidecar_for_witness_site`

**Observer pending signatures** (not signed, reasons documented):
- `dogfood/comprehensive-antigen-coverage`: too early; coverage work not complete
- `v02-beta-docs/beta-readiness-v020`: gate conditions not met (0/6 supporting campsites complete)

---

## Step 13: Findings Campsite Definitive Status Audit

**Timestamps**: All campsite.json timestamps verified directly from substrate (not from camp activity log).

| Campsite | State | Signers | Key Evidence |
|---|---|---|---|
| `findings/binary-adopter-ergonomics` (F1+F2) | **COMPLETE** | pathmaker `02:25`, adversarial `02:37` | commit `151ed48` |
| `findings/scan-fingerprint-self-match` (F4) | **COMPLETE** | 1/1 signed | commit `fa4522f` |
| `findings/scan-emit-item-fingerprint` (F6) | **COMPLETE** | scientist `02:10`, pathmaker `02:12`, adversarial `02:34` | adversarial cleared block after verifying `d46a044` |
| `findings/empty-fingerprint-guard` (F8) | **PARTIAL** (1/2) | adversarial `02:34` | commit `d46a044`; awaiting pathmaker |
| `findings/sidecar-witness-disconnect-warning` (F3) | **BLOCKED** | 0/2 | test `atk_dx_f3_audit_warns_on_sidecar_for_witness_site` FAILS; `audit.rs:1036` gap |
| `findings/signers-name-vs-role-and-example-drift` (F5) | **PARTIAL** (2/3) | aristotle `02:06`, scientist `02:12` | R3 implementation pending pathmaker |
| `findings/attest-check-per-leaf-diagnostics` (F7) | **OPEN** | 0/2 | not started |
| `findings/signer-name-role-confusion-unrepresentable` | **OPEN** | 0/1 | F5 companion |
| `findings/examples-ci-executable-workflow-integrity` | **OPEN** | 0/1 | aristotle-seeded |
| `scan-output-floods-newcomer` | **OPEN** | 0/2 | outsider-seeded |

**Active TDD red gate**: `atk_dx_f3_audit_warns_on_sidecar_for_witness_site`

**Peer-review flag dropped**: F5's `required=NAMES` decision was inherited from impl, not deliberate design. Observer note on campsite points this out for the ADR record.

**Block-staleness question**: Routed to navigator (camp question `952ae25e`) — what's the protocol for adversarial to clear stale blocks?
