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

**Block-staleness question**: Routed to navigator (camp question `952ae25e`).

---

## Step 14: Post-Navigator-Update — Coverage Terrain Audit

**Date**: 2026-05-26 (second wake, after navigator message)  
**Git HEAD**: `8408838`  
**Commits since sleep**: 10 new commits (f2270fe through 8408838)  
**Test count**: 819 passed, 48 ignored — all green

### Summary of Commits Since Sleep

| Commit | Content |
|---|---|
| `f2270fe` | dogfood: #[immune] AuditFingerprintSelfReferential + fix string-literal witnesses |
| `9ab47bc` | fix(dx): DSL roles= field, witness string-literal guard, example drift (F3+F5+F7 in commit message — F7 here = DSL capability, not per-leaf diagnostics) |
| `21e7687` | fix(macros): reject string-literal #[immune] witness with a clear error |
| `57cf56e` | test(macros): trybuild fixture for witness string-literal rejection guard |
| `259c13d` | test(dx): mark f3 audit-warning test ignored pending fix |
| `8bb3a4d` | fix(dx): Finding 3 — warn when code-witness site has orphan substrate sidecar |
| `2d0718e` | dogfood: declare SilentSemanticMismatchAtTrustBoundary antigen (#14) |
| `f21ba1f` | docs(readme): explicit antibody→witness bridge + requires= row in metaphor map |
| `3200ad5` | docs(adr): draft ADR-028 Amendment 5 — encounter-status axis |
| `8408838` | docs(macros): mark #[polyclonal]/#[adcc] audit hints as planned, not emitted |

### Key State Change vs Prior Session

**F3, F5 now CLOSED** (per campsite substrate verification):
- `findings/sidecar-witness-disconnect-warning` — COMPLETE (2/2)
- `findings/signers-name-vs-role-and-example-drift` — still PARTIAL (2/3); pathmaker needs to sign

**New antigens declared**: #14 `SilentSemanticMismatchAtTrustBoundary`

**New campsite**: `findings/dsl-signers-capability-omission`, `findings/eval-leaf-not-evaluated-arm` (explicit completion, navigator note only)

### Coverage Terrain Assessment

**Zero production `#[immune]`/`#[presents]` usage**: Verified by grep — no actual antigen markers in any non-example, non-dogfood source file. The comprehensive-coverage goal is at 0%.

**Working tree state**: `parser.rs` (120 lines) + `audit.rs` (6 lines) + `docs/immune-system-primitive-map.md` in progress. The parser.rs changes are `signature_allow`/`signature_prefer` DSL extension (dsl-signers-capability-omission follow-up), not EvalNode/F7.

**Coverage gaps verified from scout terrain notes**:

1. **`antigen-macros/src/lib.rs`**: Zero immune/presents markers. `polyclonal()` and `adcc()` functions present `DeclaredCapabilityWithNoProductionPath` — the promised audit hints never fire. Macro rustdoc fixed in `8408838`, but NO `#[presents]` marker yet. The `DeclaredCapabilityWithNoProductionPath` antigen (dogfood.rs:763) exists to be pointed at; no one is pointing at it.

2. **`antigen/src/audit.rs:443-448`**: `PolyclonalInsufficientLineages` and `AdccSingleMechanismOnly` enum variants have doc comments describing active behavior, zero construction sites. The macros doc was fixed in `8408838` but audit.rs doc was deferred. These variants now also deserve `#[presents(DeclaredCapabilityWithNoProductionPath)]`.

3. **`antigen-macros/src/parse.rs:339`** (`ImmuneArgs::validate()`): No `#[immune(WitnessClaimWithoutImplementation)]` despite the scout identifying this as the defense site for string-literal witnesses. The `immune_witness_string_literal` trybuild fixture (`57cf56e`) is the witness, but the `#[immune]` pointing to it from `ImmuneArgs::validate()` doesn't exist.

4. **`antigen/src/supply_chain/evaluate.rs:27-42`**: Path construction from user-supplied `crate_name` without sanitization. `dep_attest_path()`, `content_hash_path()`, `maintainer_path()` all format `crate_name` directly into paths. Crate names from crates.io can't contain `/` by convention, but the function signature doesn't enforce this. A `#[presents]` on the path-building functions would make this constraint explicit.

5. **`signature_allow`/`signature_prefer` DSL extension** (in working tree): The parser.rs working tree change adds these fields. Once committed, they enable the parser coverage that was previously missing for Leaf::Signers. The corresponding `#[immune]` on `parse_signers()` or `to_leaf()` for `CapabilityOmissionAtLowering` isn't drafted yet.

### Observer Finding: EvalNode/F7 Status Clarification

The "7" in commit `9ab47bc`'s "findings 3+5+7" refers to the DSL capability omission (aristotle's F7 internal numbering = DSL fields missing from signers DSL). This is DIFFERENT from my F7 = `attest-check-per-leaf-diagnostics` (EvaluatedPredicate per-leaf traces). The per-leaf diagnostics campsite (`findings/attest-check-per-leaf-diagnostics`) is still at 0/2 — not started.

Navigator confirmed pathmaker is "building EvalNode" — but the working tree shows `signature_allow`/`signature_prefer` DSL work, not EvalNode. Either EvalNode work hasn't started yet, or it's in a different location. Observer notes: the `findings/eval-leaf-not-evaluated-arm` campsite (seeded by navigator at 03:10) documents that when EvalNode lands, it needs a `NotEvaluatedHere` third arm to avoid supply-chain leaves lying about failure vs not-evaluated. This dependency is documented in the campsite.

### Highest-Value Observer Actions

1. **Watch for EvalNode landing** — verify it has `NotEvaluatedHere` arm per `eval-leaf-not-evaluated-arm` campsite
2. **audit.rs dead variants doc update** — low-friction, unblocked; observer can note to pathmaker
3. **`#[presents]` marking coverage** — when pathmaker completes F7, the presenting sites are: `polyclonal()`/`adcc()` functions, `PolyclonalInsufficientLineages`/`AdccSingleMechanismOnly` enum variants, `dep_attest_path()` and siblings, `ImmuneArgs::validate()` site
4. **`dogfood/comprehensive-antigen-coverage`** — observer should NOT sign until #[presents] markers are added to at least some production code and at least one Execution-tier substrate-witness claim exists

---

## Step 15: Critical Gap — Ratified-But-Unimplemented Antigen Family

**Hypothesis**: The team's camp notes contain ratified antigen declarations that have not been committed to dogfood.rs.

**Verification**:
- grep for `SilentIntentNullification`, `ActiveArgumentDiscard`, `CapabilityOmissionAtLowering`, `DeferredIntentNullification`, `AntigenFingerprintDivergesFromClassExtension` in dogfood.rs → **zero matches**
- `SilentArgumentDiscard` still present at line 415 (not renamed to `ActiveArgumentDiscard`)
- Total declared antigens in dogfood.rs: **15** (verified via pub struct count)

**Ratified-but-uncommitted** (all from aristotle F5/F8 notes + naturalist convergence, 02:18-02:32 UTC):

1. **`SilentIntentNullification`** — parent antigen for the nullification family; rational: the summary in dogfood.rs for `SilentArgumentDiscard` already described this parent scope before anyone named it explicitly
2. **`ActiveArgumentDiscard`** — rename of existing `SilentArgumentDiscard`; `#[descended_from(SilentIntentNullification)]`; behavioral witness
3. **`CapabilityOmissionAtLowering`** — new child; `#[descended_from(SilentIntentNullification)]`; site = `parse_signers()` → `to_leaf()` lowering; structural parity-test witness
4. **`DeferredIntentNullification`** — predicted 3rd child; vaccinated state; held awaiting first instance
5. **`AntigenFingerprintDivergesFromClassExtension`** — meta-antigen; rename from `AntigenFingerprintUnderCoversItsOwnClass` (which itself was never declared); scope-comparison witness

**Conclusion**: Camp substrate ratification records a design decision; code substrate is the authoritative source. Until these are committed, the ratification is fragile — it lives only in camp notes, not in git history.

**Observer action**: Camp note dropped on `dogfood/fingerprint-extension-not-instance-shape` and `dogfood/antigen-on-antigen-continuous` to surface this to pathmaker. Navigator notified via camp question `b5e405d0` (convergence pattern note).

**Sequencing dependency**: `CapabilityOmissionAtLowering` requires the F2 parity-test witness to exist first (the test needs to be written before the `#[immune]` marker can be added to `to_leaf()`).

**Observer sign-gate update**: Observer will NOT sign `dogfood/fingerprint-extension-not-instance-shape` until at minimum items 2-3 are committed (parent + the two concrete children). Items 4-5 can lag.

**Block-staleness question**: Routed to navigator (camp question `952ae25e`) — what's the protocol for adversarial to clear stale blocks?

---

## Step 16: Wake Audit — Post-F7 Commit State

**Time**: 2026-05-26 ~03:35 UTC
**HEAD at wake start**: e51b247 (docs: WitnessKind::None disambiguation)
**HEAD discovered mid-audit**: e2fb8f9 (F7 committed + ADR-028 Amendment 5 ratified)

### Before

**Hypothesis**: F7 (per-leaf diagnostics) has landed in the working tree but not yet committed. The `NotEvaluatedHere` arm documented in `findings/eval-leaf-not-evaluated-arm` may or may not be present.

**Design**: Read `camp wake`, `camp activity`, then verify compile state and test suite. Check EvalNode definition for NotEvaluatedHere arm. Check dogfood.rs for SilentIntentNullification family.

### Results

**What happened**:

1. **Two new commits landed during audit** (not in git log at session start):
   - `7e5289a` — F7 per-leaf diagnostics via EvalNode tree
   - `e2fb8f9` — ADR-028 Amendment 5 ratified

2. **F7 design verified in evaluate.rs**:
   - `LeafOutcome` struct: `label: String`, `passed: bool`, `reason: String` — clean design
   - `EvalNode` enum: `Leaf(LeafOutcome)`, `AllOf(Vec<EvalNode>)`, `AnyOf(Vec<EvalNode>)`, `Not(Box<EvalNode>)` — 4 arms
   - `eval_leaf()` now returns `LeafOutcome` (was `bool`)
   - `eval_pred()` now returns `EvalNode` (was `bool`)
   - Single eval path: no separate explain-pass (RatifiedSpecDriftFromImpl shape avoided)

3. **NotEvaluatedHere arm**: NOT present in `EvalNode`. Supply-chain leaves return `LeafOutcome { passed: false, reason: "supply-chain leaves are not evaluated... drive cargo antigen verify instead" }`. The reason text is honest and redirecting, partially addressing the concern from `findings/eval-leaf-not-evaluated-arm`. A distinct enum arm was not added. The campsite remains open with explicit completion.

4. **Compile gap in F7 commit (7e5289a)**:
   - `ImmunityAudit::leaf_outcomes` field added to struct definition (audit.rs:653) and to `immunity_audit_from_evaluated` (line 1244)
   - **Missing** from code-witness construction site at audit.rs:1075
   - `cargo test --doc` fails with E0063 "missing field `leaf_outcomes`"
   - 818 regular tests pass; doctest suite fails
   - Camp note dropped; navigator alerted

5. **SilentIntentNullification family**: Still NOT in dogfood.rs at HEAD (e2fb8f9). 15 antigens total. No change since Step 15.

6. **ADR-028 Amendment 5**: Ratified (e2fb8f9). Naturalist noted anergy precision fix (non-volitional, not deliberate) was applied at commit 2859c0e before ratification.

7. **Camp state delta since sleep** (9 campsites moved):
   - `findings/idiotype-network-cognate-to-primitive-map` → COMPLETE (new + signed)
   - `findings/sidecar-witness-disconnect-warning` → COMPLETE (unblock + sign)
   - `findings/empty-fingerprint-guard` → COMPLETE (sign)
   - `findings/string-literal-witness-silently-unresolved` → COMPLETE (new + sign)
   - `witnesstier-duplication-drift` → OPEN (new — outsider finding)
   - `metaphor-map-api-vocab-gap` → PARTIAL (1/2)
   - `findings/dsl-signers-capability-omission` → OPEN (blocked then unblocked — adversarial fixed)
   - `findings/eval-leaf-not-evaluated-arm` → OPEN (new — observer/navigator finding)
   - `dogfood/new-antigen-declared-capability-no-production-path` → OPEN (new)

**Surprise**: F7 committed while I was in the wake audit pass. The compile gap (missing leaf_outcomes at code-witness path) is a clean example of the generation-inspection asymmetry antigen exists to prevent — the F7 work generated the new field in 3 places but missed 1 construction site. Exactly the kind of structural gap observer role exists to catch.

### Discussion

**What we learned**:
- F7 implementation is substantive and well-designed — single eval path, no drift between verdict and explanation, honest supply-chain redirect text
- The `NotEvaluatedHere` arm was consciously not added; the redirect reason inside `LeafOutcome` is the chosen approach. This is a design decision, not an oversight.
- The compile gap at audit.rs:1075 is a real post-commit regression. It blocks doctest suite.
- ADR-028 Amendment 5 ratification is complete — encounter-status axis is now ratified design.

**What changed**: F7 baseline from "in-progress working tree" to "committed with 1 compile gap." Amendment 5 from DRAFT to ratified.

**Next**: Watch for pathmaker to fix the audit.rs:1075 missing field. Also: SilentIntentNullification family still uncommitted — that remains the largest ratified-but-unimplemented gap. Consider whether observer should sign `dogfood/comprehensive-antigen-coverage` once the doctest gap is closed and production #[presents] markers exist.

---

## Findings Status — Updated 2026-05-26 03:40 UTC

| Finding | Campsite | State | Notes |
|---------|----------|-------|-------|
| F1 (dead_code) | binary-adopter-ergonomics | COMPLETE (2/2) | |
| F2 (AntigenCategory) | binary-adopter-ergonomics | COMPLETE (2/2) | |
| F2b (DSL signers capability) | dsl-signers-capability-omission | OPEN (0/1) | adversarial unblocked; pathmaker to sign |
| F3 (sidecar-witness silence) | sidecar-witness-disconnect-warning | COMPLETE (2/2) | 8bb3a4d |
| F4 (scan self-match) | scan-fingerprint-self-match | COMPLETE (1/1) | |
| F5 (signers name-vs-role) | signers-name-vs-role-and-example-drift | PARTIAL (2/3) | pathmaker pending |
| F6 (no fingerprint) | scan-emit-item-fingerprint | COMPLETE (3/3) | |
| F7 (per-leaf diagnostics) | attest-check-per-leaf-diagnostics | OPEN (0/2) | 7e5289a committed but has compile gap |
| F8 (empty fp signed) | empty-fingerprint-guard | COMPLETE (2/2) | |

**Compile state** (HEAD e2fb8f9 + working tree): cargo build --workspace CLEAN, cargo test --doc CLEAN (working tree fix applied at audit.rs:1087 leaf_outcomes: Vec::new()).

**Test regression FOUND AND FIXED**: `atk_w7_i_stacked_immune_no_false_positive_sidecar_ignored` FAILED (observer-caught). F3 fix created false-positive in stacked-immune case — `code_witness_sidecar_ignored` was unconditional. Observer surfaced via camp note + navigator alert; fixed in commit `19e018f` within minutes. METHODOLOGY CONFIRMATION: observer caught a real regression in a COMPLETE campsite via asymmetric-default audit pass (assume untracked-important = gap). 822 pass, 0 fail at HEAD `19e018f`.

---

## Step 17: Working Tree Audit — NotEvaluatedHere + CapabilityOmissionAtLowering Parity Test

**Time**: 2026-05-26 ~04:10 UTC
**HEAD**: `507cc12` (observer lab notebook commit)

**Working tree state** (2 modified files):
- `antigen-attestation/src/evaluate.rs` — 92 additions: `evaluated: bool` field on `LeafOutcome` + supply-chain leaf label change + ATK test
- `antigen-attestation/src/parser.rs` — 62 additions: `atk_dsl_signers_every_field_reachable_and_lowered_no_omission` parity test

**Both working-tree changes verified**: `cargo test --workspace` → **824 pass, 0 fail** (2 new tests added: the ATK eval-leaf + the signers parity test).

**NotEvaluatedHere design (final)**: Pathmaker chose `evaluated: bool` on `LeafOutcome` (option b from the ATK test) rather than a new `EvalNode::NotEvaluatedHere` variant. This is simpler and backward-compatible (`#[serde(default = "default_evaluated")]` where default = true for deserialized pre-fix records). Supply-chain leaves now:
- `label: "supply-chain-leaf (not-evaluated)"`
- `passed: false` (honest-tier-naming)
- `evaluated: false` (the distinguishing field)
- `reason: "Not a failure — the check was deferred, not run"`

The ATK test asserts `leaf.label.contains("not-evaluated")` — passing now.

**Camp status update**: `findings/eval-leaf-not-evaluated-arm` was BLOCKED by adversarial's ATK test; the block will clear when pathmaker commits. `findings/attest-check-per-leaf-diagnostics` is COMPLETE.

**CapabilityOmissionAtLowering parity test**: The parser.rs parity test witnesses every `Leaf::Signers` field surviving DSL→Leaf lowering. This is the prerequisite for declaring `CapabilityOmissionAtLowering` as an antigen in dogfood.rs. **Not yet committed**.

**SilentIntentNullification family status**: Still zero commits in dogfood.rs. Both working-tree changes are prerequisites for declaring the family (parity test = witness for CapabilityOmissionAtLowering). The family can land once pathmaker commits these.

---

## Convergence Assessment — 2026-05-26 04:20 UTC

Three expedition-wide patterns converging from multiple independent observations:

### Pattern A — Generation leads inspection at every scale

Evidence:
- F7 commit: `leaf_outcomes` field added to 3/4 construction sites, 1 missed (code-witness path at audit.rs:1075) — caught by observer, fixed within minutes
- F3 fix: orphan-sidecar check covered primary case, missed stacked-immune case — caught by observer via test suite, fixed by pathmaker (`19e018f`)
- Scanner blind spot: `visit_item_enum` checks enum-level attrs, never calls `check_attrs` on variant attrs — scout found, observer confirmed
- SilentIntentNullification family: ratified in camp notes at aristotle+naturalist level, zero commits in dogfood.rs — 5 antigens in camp substrate but not in git substrate

The pattern: scope expands, coverage doesn't keep up. Exactly the generation-inspection asymmetry antigen exists to address — turned inward on antigen itself.

### Pattern B — Complete campsites have latent scope gaps

Evidence:
- `findings/sidecar-witness-disconnect-warning` COMPLETE → F3 stacked-immune regression found afterward
- `findings/attest-check-per-leaf-diagnostics` COMPLETE → `eval-leaf-not-evaluated-arm` BLOCKED (supply-chain leaves not distinguished from genuine failures)

Implication: "complete" records fix coverage at time T for scope S. Neither T nor S is perpetual. Observer's role includes post-completion audits on completed campsites, especially when new adjacent cases emerge.

### Pattern C — Biology predictions are accurate substrate predictions

Evidence:
- `findings/eval-leaf-not-evaluated-arm` seeded from observer's biology analysis ("bare false = lie at finer granularity once F7 lands") → adversarial added ATK test → fix landed (`evaluated: bool` field)
- `dogfood/fingerprint-extension-not-instance-shape` seeded from naturalist's recognition-requires-a-second-body framing → connected to original-antigenic-sin → real antigen nomination pending
- `encounter-status axis` from naturalist's biology reading → ADR-028 Amendment 5 ratified

The biology metaphor is operating as a predictive instrument, not just post-hoc labeling.

---

## Step 18: Final Arc Assessment — 2026-05-26 04:45 UTC

**HEAD**: `aaea684` (`CapabilityOmissionAtLowering` #16 committed)
**Test state**: 824 pass, 0 fail

**Antigen count**: 16 declared in dogfood.rs (1-15 + CapabilityOmissionAtLowering)

**8 DX Findings** — final status:

| Finding | Status | Fix commit |
|---------|--------|-----------|
| F1 — dead_code in binary | COMPLETE | binary-adopter-ergonomics |
| F2 — AntigenCategory unused_imports | COMPLETE | binary-adopter-ergonomics |
| F3 — sidecar-witness silence | COMPLETE + regression fixed | 8bb3a4d + 19e018f |
| F4 — scan fingerprint self-match | COMPLETE | |
| F5 — signers name-vs-role | COMPLETE | 9ab47bc + c237101 |
| F6 — scan emits no fingerprint | COMPLETE | |
| F7 — per-leaf diagnostics | COMPLETE + not-evaluated arm | 7e5289a + 166d609 |
| F8 — empty fingerprint signed | COMPLETE | |

**Key expedition additions beyond 8 original findings**:
- ADR-028 Amendment 5 (encounter-status axis) — ratified
- `evaluated: bool` on `LeafOutcome` — supply-chain leaves properly distinguished
- `CapabilityOmissionAtLowering` antigen #16 committed
- Scanner enum-variant blindspot identified (visit_variant override missing)
- F3 stacked-immune regression caught + fixed by observer

**Observer sign-gates still holding**:
- `dogfood/comprehensive-antigen-coverage` — NOT signed. Still 0 production `#[presents]` markers. 16 antigens declared but no production use of the annotation vocabulary outside dogfood.rs itself.

**Remaining SilentIntentNullification family** (unresolved):
- `SilentIntentNullification` parent
- `ActiveArgumentDiscard` (rename of `SilentArgumentDiscard`)
- `DeferredIntentNullification` (3rd child)
- `AntigenFingerprintDivergesFromClassExtension` (meta-antigen)

---

## Step 19: Post-Sleep Wake Audit — #17 Landed

**Time**: 2026-05-26 ~05:00 UTC  
**HEAD at wake**: `89f8108` (`AntigenFingerprintDivergesFromClassExtension` #17 + WitnessTier parity)  
**Test state at wake**: 828 pass, 48 ignored  
**Camp state delta since sleep**: 4 campsites moved: `dogfood/scanner-enum-variant-blindspot` → COMPLETE (explicit), `dogfood/layer1-production-presents-markers` → OPEN (new), `findings/dsl-signers-capability-omission` → COMPLETE (1/1), `witnesstier-duplication-drift` → PARTIAL (1/2, block+unblock+sign)

### What Landed at 89f8108

Three bundled changes:

1. **`AntigenFingerprintDivergesFromClassExtension` (#17) committed** — meta-antigen at `dogfood.rs:829-896`. `SubstrateAlignment` category. Two divergence directions documented in docstring with biological cognate (original antigenic sin). Summary: "the fingerprint's match-set diverges from the class's true extension." References `ADR-006`, `ADR-010`, `docs/testing-patterns.md`.

2. **`audit::WitnessTier` gained `Hash` derive** — `antigen_attestation::WitnessTier` already derived `Hash`; `audit::WitnessTier` did not. `atk_witness_tier_parity.rs` was authored to FAIL until this fix. Now 828 pass. The `#[ignore]` on `atk_a2_enum_variant_presents_is_not_silently_ignored` means 48 ignored (was 1 fail, now properly ignored as TDD pin awaiting scanner fix).

3. **Testing infrastructure cleanup** — `docs/testing-patterns.md` fingerprint-authoring-discipline section committed (was in working tree), `atk_witness_tier_parity.rs` clippy warnings cleaned, `atk_a2_adversarial.rs` enum-variant test properly `#[ignore]`'d, `atk_a2_impl_const_presents` specimen fixture committed.

### Stale Regression Flag Resolution

The F3 stacked-immune regression I flagged in the lab notebook (Step 16/17) was caught and fixed at commit `19e018f` **before my sleep** — I wrote the flag from pre-fix context. Navigator confirmed both campsites (`findings/sidecar-witness-disconnect-warning`, `findings/stacked-immune-sidecar-false-positive`) stand as closed. The flag was correct at the time of writing; the fix crossed in flight. No action needed.

### Gate Audit: dogfood/fingerprint-extension-not-instance-shape

Gate condition requires: "(b) AntigenFingerprintDivergesFromClassExtension antigen in stdlib/dogfood.rs with **2 severity-discriminated hints** (under-covers=HIGH/false-negative, over-covers=advisory)."

**What shipped**: The committed #17 antigen documents two divergence directions in its docstring ("silent under-coverage", "noisy over-coverage") and summary ("producing silent under-coverage... or noisy over-coverage"), but does NOT use the explicit HIGH/advisory severity labels the gate condition specifies.

**Key severity asymmetry** (from naturalist's note at 02:43 in campsite story): UNDER-coverage = false negative (real instance escapes, HIGH severity — defeats the tool's purpose). OVER-coverage = false positive (noise, advisory — flags non-risk sites). The asymmetry IS documented in the campsite story and in naturalist's notes, but NOT reflected in the committed antigen's macro attributes or summary text as explicit severity labels.

**Observer finding**: The "2 severity-discriminated hints" gate condition uses "hints" in the sense of labeled severity discriminators, not the antigen macro's `hints=` field (which doesn't exist as a macro attribute). Whether the committed docstring language ("silent" vs "noisy") satisfies this gate is naturalist's judgment to make, not observer's to decide. Camp note dropped on campsite, routed to naturalist.

**Remaining unresolved**: `SilentIntentNullification` family (parent + `ActiveArgumentDiscard` rename + `DeferredIntentNullification`) still not in dogfood.rs. These 3 items remain the largest ratified-but-uncommitted gap.

### Antigen Count Update

| # | Name | Status |
|---|------|--------|
| 1-15 | (prior arc) | Committed |
| 16 | `CapabilityOmissionAtLowering` | Committed (`aaea684`) |
| 17 | `AntigenFingerprintDivergesFromClassExtension` | Committed (`89f8108`) |

**Total committed**: 17 antigens in dogfood.rs. SilentIntentNullification family (parent + 2 descendants + 1 holding for first instance) remains uncommitted.

### Observer Sign-Gate Status (unchanged)

`dogfood/comprehensive-antigen-coverage` — NOT signed. Still 0 production `#[presents]` markers in non-dogfood, non-example, non-test codebase. 17 antigens declared; none pointed at from production code. Gate condition: this campsite's name implies comprehensive coverage, which is not yet achieved.

---

## Step 20: Scanner Blindspot Fix Arc + Post-Fix Audit

**Time**: 2026-05-26 ~05:25 UTC  
**HEAD**: `83f26a5` (5th scanner blind spot: trait-associated consts)  
**Test state**: 831 pass, 48 ignored  

### Scanner Fix Arc — Five Blind Spots, Three Commits

While I was auditing the working tree (noting the fix as verified-clean), the arc completed in git:

| Commit | Content |
|--------|---------|
| `d97c204` | 4 overrides: visit_variant (EnumVariant), visit_impl_item_const (ImplConst), visit_item_const (Const), visit_item_static (Static preemptive) |
| `83f26a5` | 5th override: visit_trait_item_const (reuses ImplConst target, renders Trait::CONST) |

audit.rs gained `&& other.file == immunity.file` in `has_companion_requires` (F3 cross-file suppression gap — third-order F3 fix). 831 pass, 48 ignored (the formerly-ignored enum-variant ATK test is now un-ignorable: passing).

**Pattern — original-antigenic-sin on campsite name**: `dogfood/scanner-enum-variant-blindspot` named for the first-seen instance; the fix covered the full class (5 item kinds with missing visitor overrides). The campsite name fits instance #1, not the class extension — `AntigenFingerprintDivergesFromClassExtension` demonstrated on camp substrate nomenclature.

### Production #[presents] Markers — State Update

The "ZERO production #[presents] markers" from Steps 14/15 is stale. Current committed state at HEAD `83f26a5`:

| Location | Antigen |
|----------|---------|
| `antigen/src/scan.rs:968` | `VecCardinalityMasqueradingAsSet` |
| `antigen/src/scan.rs:2429` | `ScannerBoundaryFalseNegative` |
| `antigen/src/audit.rs:2511` | `DelegateCrossCrateResolutionGap` |

3 committed production markers. With the scanner fix committed, `audit.rs:446` (`PolyclonalInsufficientLineages`) and `audit.rs:451` (`AdccSingleMechanismOnly`) can now receive `#[presents(DeclaredCapabilityWithNoProductionPath)]` — both have "Planned — not yet emitted at v0.2" in their doc comments.

**Observer sign-gate update for comprehensive-antigen-coverage**: Original gate ("zero markers") CLEARED. Revised gate: 5 coverage sub-campsites must reach complete AND at least one `#[immune]` with real witness in production code (not just `#[presents]`).

### stdlib Category Backfill Audit

All existing stdlib modules have complete category fields (supply_chain 11/11, vcs_info_loss 11/11, mucosal 3/3, recurrent 3/3, agentic_coordination 2/2). Campsite `v02-impl-stdlib-category-backfill` references `convergent.rs, ~8 antigens` which does not exist. Either the backfill is complete or the scope is misdescribed. Navigator notified.

### SilentIntentNullification Family — Still Uncommitted

`SilentArgumentDiscard` (#9) remains as original name. `SilentIntentNullification` parent, `ActiveArgumentDiscard` rename, `DeferredIntentNullification` — none committed. 5 audit steps (15, 16, 18, 19, 20) have noted this gap. The family is ratified in camp substrate, absent from git substrate. Camp question `2b8abbfd` routed to pathmaker.

---

## Step 21: Remaining Open Campsites Audit + Scan-Flood Severity Update

**Time**: 2026-05-26 ~05:45 UTC  
**HEAD**: `3cdcfe5` (lab notebook step 20)  
**Test state**: 831 pass, 48 ignored

### Schema-Lock Test — A New Structural Guard

Commit `6bcfef0` (doc: document new ItemTarget variants) added `atk_schema_lock.rs` — a test that scans each fixture and asserts every emitted `ItemTarget` discriminant appears in `docs/output-formats.md`. This is structurally significant:

- First test in the codebase that asserts *documentation matches behavior* (not just behavior matches spec)
- Removed "never-emitted" variants (Mod/Use/Type) from the doc — `DeclaredCapabilityWithNoProductionPath` applied to docs themselves
- The test is a CODE witness for the claim "docs/output-formats.md accurately describes the emitted schema"

This demonstrates antigen's dogfood reaching a new tier: substrate-alignment verified by a test that fails when docs drift from behavior.

### Scan-Flood Severity Update — WORSE After Scanner Fix

Verified `scan-output-floods-newcomer` claim independently. `print_fingerprint_matches` (cargo-antigen/src/main.rs:2340) confirms: `for p in &fp_matches` with NO cap. Adversarial's finding holds.

**Important amplification**: The scanner fix (d97c204, 83f26a5) added 5 new item kinds (enum variants, impl consts, top-level consts, static, trait consts) to fingerprint matching. The fingerprint match count on antigen's own codebase is now LARGER than outsider's ~16K measurement at f6e3846f. The flood problem is worse after the scanner correctness fix, not better. Pathmaker needs the cap before any public release — the correctness and UX concerns are now directly coupled.

### R3 Landed — R5 Now Unblocked

`findings/signer-name-role-confusion-unrepresentable` (R5: tagged constructors making name/role confusion a parse error) was blocked waiting for R3. Verified `roles=` field exists in parser at line 1034. R3 has landed. R5 is now unblocked in principle; pathmaker to implement when prioritized.

### Open Campsites Without Observer Action Needed

| Campsite | Status | Note |
|----------|--------|------|
| `dogfood/coverage-*` (5 campsites) | pathmaker lane | Waiting for implementation |
| `dogfood/fingerprint-extension-not-instance-shape` | gated | Naturalist gate-satisfy pending |
| `findings/examples-ci-executable-workflow-integrity` | pathmaker lane | Executable-workflow CI test |
| `findings/orient-field-optionality-ruling` | pathmaker lane | R5 unblocked per R3 landing |
| `findings/signer-name-role-confusion-unrepresentable` | pathmaker lane | R5 design (tagged constructors) |
| `scan-output-floods-newcomer` | pathmaker lane | Adversarial gate (failing test first) |
| `antigen-dx-dogfood/v02-impl-stdlib-category-backfill` | navigator routing | Scope misdescribed |

---

## Step 22: Navigator-Routed Field Entry Verifications

**Time**: 2026-05-26 ~06:10 UTC

### Item 1 — quickstart.md Version (field 04f766eb)

**Hypothesis**: quickstart.md:26 may show a stale or incorrect version string.

**Verification**:
- `cargo search cargo-antigen` → `0.1.0-rc.3` (published on crates.io)
- Local `cargo antigen --version` → `cargo-antigen-antigen 0.2.0-alpha.4`
- `quickstart.md:26` shows `cargo-antigen-antigen 0.1.0-rc.3`

**Verdict**: Correct. The doc matches the published crates.io release. The double-antigen format (`cargo-antigen-antigen`) is not a typo — it is clap's output format for a cargo subcommand: `<binary-name>-<subcommand> <version>`. The workspace is at `0.2.0-alpha.4` (unpublished alpha); quickstart targets the published `rc.3`.

### Item 2 — Path Traversal Risk (field 5cd20a94)

**Hypothesis**: `dep_attest_path`, `content_hash_path`, `maintainer_path` concatenate user-supplied strings without validation, enabling path traversal.

**Verification**:
- `parse_crate_at_version` (main.rs:1541): checks non-empty + no double `@`. Does NOT validate character set.
- `format!("{crate_name}@{version}.json")` in path functions: no sanitization.
- `../../../etc/passwd@1.0` parses successfully; `dep_attest_path` would produce `.attest/supply-chain/dep-attest/../../../etc/passwd@1.0.json`.
- Audit evaluation path: `crate_name` comes from DSL-parsed `#[immune]` predicate (adopter's own code). Lower-risk but same code path.
- **NOT from Cargo.lock** — the evaluate functions consume DSL leaves, not lockfile entries.

**Verdict**: Real gap. CLI write path (`run_verify_dep_attest`, main.rs:939) has exploitable path traversal if user passes adversarial `crate@version` string. Seeded `dogfood/supply-chain-path-traversal-guard` (pathmaker). Fix: character-set guard in `parse_crate_at_version` (alphanumeric + `-_.`, reject `..`).

**Secondary observation**: `parse_crate_at_version` is a candidate for `#[presents(UnvalidatedSealedEnumAcceptance)]` — accepts any `@`-split string without type-level validation of the crate name format.

---

## Step 23: ATK-A2-PRES-FP Fix — Presentation Fingerprint Gap

**Time**: 2026-05-26 ~06:20 UTC  
**HEAD**: `d9c251f` (fix: emit structural_fingerprint in extract_presentation)  
**Test state**: 833 pass, 48 ignored

**Fix**: `extract_presentation` in `scan.rs` was emitting `String::new()` for `structural_fingerprint` while `extract_immune` correctly emitted `self.current_item_digest.clone()`. One-line change. ATK test `atk_a2_pres_fp_struct_explicit_marker_has_non_empty_fingerprint` caught it.

**Why this matters for F6**: F6 (scan-emit-item-fingerprint) was marked COMPLETE for immunity entries. The `against="current"` substrate-witness workflow needs structural fingerprints. If presentation entries from explicit `#[presents]` markers had empty fingerprints, any `fresh_within_days` delta comparison would have silently compared against an empty string. The fix closes F6 fully — both immunity and presentation entries now emit correct digests.

**Third instance of symmetric-function partial wiring** (Pattern A + B convergence):
1. `ImmunityAudit::leaf_outcomes` — 3/4 construction sites wired. Fixed `dbd9cab`.
2. `has_companion_requires` — `antigen_type + item_target` but not `file`. Fixed `d97c204`.
3. `extract_presentation` fingerprint — `extract_immune` wired, `extract_presentation` not. Fixed `d9c251f`.

Three instances from one expedition. The fail-class is: a symmetric refactoring applied to N of N+1 symmetric targets — exactly what `CapabilityOmissionAtLowering` antigen #16 names at the DSL level. The pattern generalizes: when two symmetric functions diverge on one feature, both need the same treatment.

---

## Step 24: Context Resumption After Compaction + ATK-DIGEST-1 Fix

**Time**: 2026-05-26 ~07:00 UTC  
**HEAD at wake**: `b861b43` (fix: warn on malformed --fingerprint digest, not just empty)  
**Previous HEAD (Step 23)**: `d9c251f`  
**Test state**: 1 failing (workspace), 867 passing before fix

### Before (orientation after compaction)

**Hypothesis**: Resuming from compaction — current substrate may differ significantly from the Step 23 snapshot. Several teammate work cycles happened during the sleep. Primary concern: the 3 failing adversarial tests (mod/union/foreign_mod) may have been fixed; new findings may have landed.

**Design**: Wake protocol — `camp wake`, `camp activity`, `git log`, then targeted substrate verification.

### Results: What Landed While Observer Slept

Key commits between `d9c251f` (Step 23 HEAD) and `b861b43` (current HEAD):

- `c6ae87a` — deferred-defense rejects past until dates (anergy/immunosuppress/poxparty `horizon_days < 0` check)
- `53d2bab` — orient rejects past `until` date
- `aeb39fc` — ADR-023 Option-A hard break (`learning_path + until REQUIRED`)
- `28a8f1a` — supply-chain path-traversal guard in `evaluate_maintainer_unchanged` (path-traversal gap partially fixed)
- `50ea6d2`, `f4165d0` — fmt fix + examples CI workflow integrity test
- `254fc63` — fingerprint: strip parameter names from `has_method` patterns at parse time
- `f48ff20` — scaffold auto-fill fingerprint from scan
- `9a18e4e` — schema-lock fingerprint regression guard + path-traversal test for maintainer
- `94ee01e` — `cargo antigen fingerprint` subcommand (new)
- `b2bac81` — scan output cap + reframe (P0 onboarding DX; outsider verified + signed)
- `f516265` — align summary-line framing with candidate-site reframe
- `b861b43` — warn on malformed `--fingerprint` digest (format guard)

**Three prior-session failing adversarial tests**: All FIXED. `25 passed` in `atk_a2_adversarial`. The `visit_item_mod`, `visit_item_union`, `visit_item_foreign_mod` overrides landed (verified: 25 pass in current suite vs 22 prior session).

**Active incomplete in working tree**: `atk_a2_adversarial.rs` (+180 lines), `antigen/src/scan.rs` (+91 lines), `antigen-fingerprint/src/digest.rs` (+73 lines), `antigen/tests/atk_schema_lock.rs` (+19 lines), `docs/tutorial.md` (+34 lines), `antigen/examples/deferred_defense_orient.rs`. Eight untracked fixture dirs. This is adversarial's scanner-round-3 in-progress work: new tests for use-item, union, foreign_mod, mod, trait_alias, extern_crate, plus `impl_has_attributes!` expansion.

### ATK-DIGEST-1: ANTIGEN_OWNED_ATTRS Incomplete

**Finding**: 1 test failing in the workspace: `digest::tests::all_antigen_macros_do_not_change_digest` in `antigen-fingerprint`. The adversarial-planted test reveals that `ANTIGEN_OWNED_ATTRS` (the list of macros excluded from structural digest computation) had 11 of 26 antigen macros. The missing 15: `mucosal`, `mucosal_delegate`, `mucosal_tolerant`, `polyclonal`, `monoclonal`, `adcc`, `clonal`, `igg`, `diagnostic`, `itch`, `recurrence_anchor`, `crystallize`, `chronic`, `saturate`, `strand`.

**Effect of the gap**: Adding any of these macros to a signed item would change the item's structural digest, silently invalidating the previously-recorded signature at audit time. The invariant "antigen attestation macros do not change the structural digest" was violated for 58% of the macro surface (15 of 26).

**Fix**: Added all 15 missing macros to `ANTIGEN_OWNED_ATTRS` in `antigen-fingerprint/src/digest.rs`, grouped by family (mucosal, witness/audit-classification, recurrent-pattern).

**After fix**: `868 passed, 48 ignored` across workspace. All suites green.

**Coordination note**: The fix is in the working tree alongside adversarial's scanner-round-3 in-progress work. Both touch `digest.rs`. Pathmaker should commit `antigen-fingerprint/src/digest.rs` together with the scanner-round-3 bundle (fixture dirs + `atk_a2_adversarial.rs` additions). Campsite seeded: `atk-digest-1-antigen-owned-attrs-incomplete`. Navigator notified.

### Discussion

This is a fourth instance of the **incomplete-completion** pattern (complement to the symmetric-function partial-wiring pattern):

- `ANTIGEN_OWNED_ATTRS` grew from the original 5 core macros to 11 as deferred-defense and triage-commit landed — but the additions were not systematic.
- The fail-class: when a new antigen macro ships, it must be added to `ANTIGEN_OWNED_ATTRS` or it will corrupt signed items. This is a missing **structural invariant enforcement** — the connection between "new macro exists" and "must appear in exclusion list" is nowhere enforced.
- Possible structural fix: derive `ANTIGEN_OWNED_ATTRS` from the proc_macro registrations rather than maintaining the list by hand. This is the same class of fix as `audit-hint-const-shadows-enum` (derive from enum variants rather than hand-maintaining a const).

**Next**: Assess `dogfood/layer1-production-presents-markers` campsite closure, then write Step 25 on comprehensive coverage gate status.
