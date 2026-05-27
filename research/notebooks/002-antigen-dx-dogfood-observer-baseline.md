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

**Next**: CI-gate fmt audit, tutorial verification, audit-hint drift confirmation.

---

## Step 25: CI-Gate Fmt Violations + Tutorial Verification + Audit-Hint Drift

**Time**: 2026-05-26 ~07:15 UTC  
**HEAD**: `06ab9bd` (lab notebook step 24)  
**Test state**: 868 passing, 48 ignored

### Before

**Hypothesis**: Navigator flagged two potential drift areas — (1) CommittedArtifactViolatesUnrunGate for clippy/fmt on recent commits, (2) tutorial-attest-commands-drift fix may be working-tree only. Also want to verify audit-hint-const-shadows-enum is actually drifted (not just a stale campsite claim).

### Results

**CI-gate fmt audit**: `cargo fmt --all -- --check` FAILS on committed HEAD. 9 violations across 5 files:
- `antigen/tests/atk_a2_adversarial.rs` (2 spots) — from scanner-round-3 commit `b1f6886`
- `antigen/tests/atk_supply_chain_evaluate.rs` (1 spot)
- `antigen-fingerprint/src/digest.rs` (3 spots)
- `antigen-fingerprint/src/parser.rs` (2 spots)
- `antigen-macros/src/parse.rs` (1 spot)

`cargo clippy -- -D warnings` is CLEAN. Only fmt is broken.

**Tutorial verification**: `670242d` confirmed committed. Flag names verified against live CLI: scaffold uses `--antigen`/`--source-file`/`--item-path`/`--fingerprint`; sign uses `--sidecar`/`--item-path`/`--signer`/`--role`/`--fingerprint`. All correct. **Secondary finding**: tutorial.md:409 still says "Known limitation (v0.2)" about fingerprint selection being hard — but `f48ff20` (auto-fill) + `94ee01e` (fingerprint subcommand) shipped this same expedition and close that gap. The limitation text is stale. Not blocking campsite sign; it's a polish item.

**audit-hint-const-shadows-enum drift — confirmed live**: `AuditHint` enum has 16 supply-chain variants; `ADR025_AUDIT_HINTS` hand-maintained const has 15. Missing: `UnpinnedTransitiveDependency`. The `adr025_audit_hints_count_is_fifteen` test PASSES because it checks `len()==15`, not contents. This is the exact green-while-drifted class.

### Discussion

Three substrate-alignment gaps found in one audit pass:

1. **CI fmt gate** (`ci-gate-fmt-violations-in-committed-code`): Mechanical fix — `cargo fmt --all` then commit. Seeded campsite; pathmaker lane.

2. **Tutorial stale limitation text**: Stale-document-after-feature-shipped. Tutorial correctly described state when written; auto-fill + fingerprint subcommand closed the gap mid-expedition. Tutorial polish item for docs lane.

3. **Audit-hint count drift** (`audit-hint-const-shadows-enum`): `comment-says-must-match` pattern — test enforces count=15 but comment says "must match enum". Count-equality doesn't imply content-equality when enum grows. Missing: `UnpinnedTransitiveDependency`. Pathmaker's derive-from-enum fix resolves it.

**Observation**: These three gaps share a structural shape — a hand-maintained artifact that should be derived automatically from a canonical source. `ANTIGEN_OWNED_ATTRS` (26 macros by hand), `ADR025_AUDIT_HINTS` (16 enum variants by hand), tutorial limitation text (feature shipped but doc not updated). The recurring pattern: manual maintenance creates drift-windows that automation would close. Antigen's own `AuditHintWithNoUpstreamPreconditionCheck` class is adjacent — a test that passes but doesn't enforce the invariant its comment promises.

**Additional finding (Step 25b)**: Working-tree `audit.rs` has uncommitted `#[presents(DeclaredCapabilityWithNoProductionPath)]` markers on `PolyclonalInsufficientLineages` and `AdccSingleMechanismOnly` enum variants. If committed as-is, `RUSTDOCFLAGS="-D warnings" cargo doc` FAILS: "expected non-macro attribute, found attribute macro `presents`". Verified: committed HEAD doc build is clean (stash-test). Root cause: rustdoc errors on proc-macro attributes on enum variants in documented pub enums. Pathmaker needs to resolve before committing. This is a new sub-class of the scanner-blind-spot fix having unintended doc consequences — the scanner can now *find* `#[presents]` on enum variants, but rustdoc errors when documenting them. Navigator and pathmaker notified.

---

## Step 26: Comprehensive Coverage Gate State + Scanner Arc Assessment

**Time**: 2026-05-26 ~07:40 UTC  
**HEAD**: `24975fd` (lab notebook step 25b addendum)  
**Test state**: 868 passing (workspace, committed HEAD); working tree has uncommitted adversarial ATK tests

### Comprehensive Coverage Gate

Gate (`dogfood/comprehensive-antigen-coverage`) requires observer + pathmaker + naturalist signatures. Observer sign-condition: 5 coverage sub-campsites complete + at least one `#[immune]` with real witness in production code.

**Current state** (verified against camp status):
- `coverage-antigen-attestation`: COMPLETE (1/6)
- `coverage-antigen-macros-lib`: open, waiting on pathmaker (0/6 complete)
- `coverage-antigen-macros-parse`: open, waiting on pathmaker
- `coverage-cargo-antigen-binary`: open, waiting on pathmaker
- `coverage-fingerprint-tightening`: open, waiting on pathmaker
- `coverage-supply-chain-module`: open, waiting on pathmaker

**Production `#[immune]`** with real witness in non-fixture, non-test code: **zero** (grep confirmed). Three production `#[presents]` markers exist (scan.rs:968, scan.rs:2429, audit.rs:2511), all unaddressed.

**Observer sign-gate: NOT SATISFIED.** This campsite is correctly gated on pathmaker completing the 5 remaining coverage sub-campsites + a production immune declaration.

### Scanner Blind-Spot Arc — Current State

The adversarial scanner arc is still running. Each round:
- **Closed rounds**: enum-variant, impl-const, trait-const, top-level const, static, use-item, macro-rules, extern-crate, foreign-mod, mod, trait-alias, union (all fixed across multiple commits)
- **New pending round in working tree**: `visit_impl_item_type` (associated type in impl block) + `visit_trait_item_type` (associated type in trait body)

Pattern: `syn::ImplItem` and `syn::TraitItem` have the same partial-coverage gap `syn::Item` had. Every fix round reveals the next sub-variant. The structural fix would be: enumerate all `syn::Item`, `syn::ImplItem`, `syn::TraitItem` variants and verify each has a `visit_*` override. A meta-test that enumerates these (adversarial noticed this in camp activity) would close the arc permanently rather than finding gaps one by one.

### CI-Gate State Summary (committed HEAD)

| Gate | Status |
|------|--------|
| `cargo test --workspace` | PASS (879/0 fail) — updated after e3120e3+832e5f6+4601fbb |
| `cargo clippy -- -D warnings` | PASS |
| `cargo fmt -- --check` | PASS — fixed at 4601fbb |
| `RUSTDOCFLAGS=-D warnings cargo doc` | PASS — enum-variant presents resolved cleanly |

**Post-Step-26 resolution**: All CI gates closed rapidly. `e3120e3` fixed the two new associated-type blind spots (`visit_impl_item_type` + `visit_trait_item_type`). `832e5f6` committed the enum-variant `#[presents]` markers WITHOUT breaking doc build — pathmaker correctly placed the marker on the `AuditHint` enum type itself (audit.rs:165), not on individual variants, because Rust forbids proc-macro attributes on enum variants (only derive helper attributes are allowed there). This is the correct granularity; the scanner finds it correctly. `4601fbb` fixed remaining fmt violations. 879 tests passing. `ci-gate-fmt-violations-in-committed-code` campsite COMPLETE.

**Key structural insight (step 26 observation)**: The scanner's `visit_item_enum` variant-level coverage (from b1f6886) handles the structural case correctly — it CAN find attributes on enum variants. But the Rust compiler itself prevents placing proc-macro attributes on individual variants in real code. The invariant: scanner coverage is structurally complete; real dogfood usage must use enum-level markers for proc-macro annotations. Both halves are correct; they operate at different layers.

---

## Step 27: Post-Compaction Catch-Up Audit

**Time**: 2026-05-26 ~20:00 UTC  
**HEAD**: `a5f9bd6` (dogfood: declare FingerprintDigestWithoutFormatValidation sibling)  
**Test state**: 884 passing (workspace, cargo test --workspace). Working tree dirty: README.md (+1 version-pin line), antigen-fingerprint/src/matcher.rs, antigen-macros/src/parse.rs (but parse.rs shows clean per git status — not currently dirty), antigen/tests/atk_a2_adversarial.rs (+95 lines of new tests), antigen/tests/supply_chain_correctness.rs (outsider's bidirectional fix), docs/glossary.md, docs/quickstart.md. Note: git diff showed quickstart.md + glossary.md are CLEAN (committed in ff6eaaf); README.md has only a 1-line version-pin fix (outsider's uncommitted work).

### Hypothesis

Between context compaction and this wake: several arcs completed. I expect to find (1) `audit-hint-const-shadows-enum` fix landed by outsider, (2) FingerprintDigestWithoutFormatValidation sibling declared, (3) teach-witness-vs-requires bundle committed, (4) scanner assoc-type blind spots CLOSED. Possible: new findings opened that may already be stale.

### Results

**Commits that landed during compaction window** (in order, post-d2a2067):

| Commit | What landed |
|--------|-------------|
| `5fcfe05` | Lab notebook step 26 addendum — CI gates resolved |
| `4601fbb` | fmt violations fixed (supply_chain_evaluate + parser) |
| `832e5f6` | AuditHint enum `#[presents(DeclaredCapabilityWithNoProductionPath)]` at TYPE level |
| `e3120e3` | Scanner: close impl/trait associated-type blind spots (visit_impl_item_type + visit_trait_item_type) |
| `5115568` | docs(vision-pitch): three-faces-of-one-asymmetry root structure |
| `ba626a6` | fix(diagnostic): validate modalities against sealed WitnessClass set |
| `8762bdd` | lab notebook step 25 |
| `7e9deda` | docs(roadmap): cross-crate scan reachability as ADR-001-C7 activation path |
| `670242d` | fix(docs+tests): tutorial CLI flag corrections |
| `b1f6886` | fix(scan+digest): close remaining syn::Item blind spots |
| `b861b43` | fix(sign): warn on malformed --fingerprint digest |
| `f516265`, `58a736d` | Additional scan fixes |
| `7d8578a` | fix(deferred-defense): reject invalid ISO-8601 dates in anergy/immunosuppress/poxparty |
| `c85802d` | fix(attest): digest-format guard at scaffold/delta/check (not just sign) |
| `ff6eaaf` | docs: teach witness= vs requires= at all 4 first-contact surfaces |
| `a5f9bd6` | dogfood: declare FingerprintDigestWithoutFormatValidation sibling |

**Test trajectory**: 868 (pre-compaction) → 879 (e3120e3+832e5f6+4601fbb) → 884 (current HEAD).

### Finding 27-A: anergy-invalid-date-silently-accepted campsite is stale

**Claim (adversarial campsite, 19:43 UTC)**: Tests FAIL — `AnergyArgs::validate()` and `ImmunosuppressArgs::validate()` use `if let Ok(until_date) = parse_iso_date(until_str)` without an `else Err` branch. Invalid dates create unbounded suppression windows.

**Observer verification**: Substrate check at committed HEAD. `7d8578a` ('fix(deferred-defense): reject invalid ISO-8601 dates', committed 2026-05-26 **19:50 UTC**) added `Err(())` branches to ALL THREE validators — 7 minutes AFTER the campsite was created.

Verified at HEAD:
- `AnergyArgs::validate()` — parse.rs:695-704: `Err(()) => { return Err(syn::Error::new(..., format!("#[anergy] \`until\` value ... is not a valid ISO-8601 date...")))` — PRESENT
- `ImmunosuppressArgs::validate()` — parse.rs:914-923: same pattern — PRESENT  
- `PoxpartyArgs::validate()` — parse.rs:1112-1120: same pattern — PRESENT

**Conclusion**: The campsite blocker describes code that was fixed AFTER the campsite was seeded. At committed HEAD, the described failure mode no longer exists. The campsite should be UNBLOCKED and the new tests (if they test the corrected behavior: invalid date → Err) should PASS.

Observer note deposited on campsite. Navigator notified.

**Meta-pattern**: This is the same substrate-alignment drift class as the context-held-belief going stale across agents. An agent seeded a campsite describing state T, another agent fixed the state at T+7min, but the campsite remained BLOCKED because no agent ran the cross-check "does this blocker still describe the code?" Observer's role: catch exactly this gap.

### Finding 27-B: FingerprintDigestWithoutFormatValidation arc closed cleanly

**Observation**: Two commits form a clean fix-then-declare sequence:
- `c85802d` — closed the 1-of-N spread: digest-format guard applied at scaffold/delta/check (was only at sign)
- `a5f9bd6` — declared the sibling antigen (`FingerprintDigestWithoutFormatValidation` in `antigen/src/stdlib/dogfood.rs`)

The commit message for `a5f9bd6` correctly names the naturalist's recognition chain and the c85802d fix. This is the preemptive-internal-tooling pattern applied correctly: fix the sites FIRST, then declare the class (so the declaration doesn't immediately dogfood its own class by having undefended sites).

**Assessment**: Clean sequence. No methodology gap. The class is in `dogfood.rs` alongside its sibling `FingerprintStringWithoutDslValidation`. Both share the parent shape (cross-site trust-boundary inconsistency, sub-clause F) and split by the KIND of validation missing — the same witness-structure discriminator that separated `ActiveArgumentDiscard` from `CapabilityOmissionAtLowering`. This is the witness-split taxonomy pattern the naturalist has been tracking.

### Finding 27-C: teach-witness-vs-requires bundle committed correctly

**Verification**: `ff6eaaf` committed all 4 first-contact surfaces: README.md, docs/quickstart.md, docs/glossary.md, antigen-macros/src/lib.rs. All changes are clean at HEAD (quickstart and glossary confirmed by git diff HEAD — empty diff). Working tree shows README.md dirty by 1 line — that is outsider's SEPARATE version-pin fix (rc.2 → rc.3 at README:152), which is NOT part of the teach bundle.

**Substrate-alignment drift note**: outsider's sleep note correctly describes README.md as dirty with the version-pin change (not the teach bundle). The teach bundle and the version-pin fix are two separate uncommitted chunks in the working tree that happen to both touch README.md. `ff6eaaf` committed only the `#[immune]` macro description change in README; outsider's rc.2→rc.3 pin fix is still staged separately. This is working correctly — the two changes are in different lines; git will track them separately when committed.

### Finding 27-C2: README version-currency drift (outsider finding)

Outsider noticed (via `cargo search`) that README.md has stale rc.2-era labels while rc.3 is the actual published version:
- `README:152` — dependency pin `=0.1.0-rc.2` (outsider's dirty-tree fix: rc.3)
- `README:204` — section header "What's actually shipped in v0.1.0-rc.2" — NOT fixed (release-owner territory)
- `README:218` — "554 tests passing" — NOT fixed (now 800+; release-state claim)
- `docs/vision-pitch.md:211` — "v0.1.0-rc.2 is available" — NOT fixed

The one-line pin fix (README:152) is in outsider's working tree, uncommitted. The section-header + test-count + vision-pitch stale labels are flagged for release-owner (whoever updates the rc.3 release notes). This is correctly scoped — the feature surface in README:204 is mostly accurate for rc.3; only the version LABEL and test COUNT drifted.

**Observer structural note**: This is `ParallelStateTrackersDiverge` at the docs layer — version string is hand-copied to N places (README, quickstart, tutorial, vision-pitch) rather than deriving from a single canonical source. A `version = "..."` constant substituted during doc generation would prevent this class. Outsider named this correctly.

### State Summary at Step 27

**Test suite**: 884 passing, 48 ignored, 0 failing.  
**CI gates**: All PASS (fmt, clippy, doc, test).

**Active open campsites of interest**:
| Campsite | State | Blocker |
|----------|-------|---------|
| `anergy-invalid-date-silently-accepted` | BLOCKED — stale blocker | Fix already committed at 7d8578a; needs adversarial to unblock |
| `audit-hint-const-shadows-enum` | OPEN | Outsider fix in dirty working tree; pathmaker must sign |
| `dogfood/supply-chain-path-traversal-guard` | OPEN | Pathmaker signature needed |
| `dogfood/layer1-production-presents-markers` | OPEN | 4 cargo-antigen candidates pending naturalist ruling |
| `dogfood/comprehensive-antigen-coverage` | OPEN | 5/6 sub-campsites still need pathmaker |
| `tutorial-attest-commands-drift` | OPEN | Pathmaker + outsider must sign (fix at 670242d confirmed correct) |

**Working-tree dirty files** (all teammate-owned, not observer's):
- `README.md` — outsider's version-pin fix (+1 line rc.2→rc.3) — may be committed by ca6de95
- `antigen-fingerprint/src/matcher.rs` — unknown owner (not observer's lane)
- `antigen/tests/atk_a2_adversarial.rs` — adversarial's new impl/trait assoc-type tests (+95 lines)
- `antigen/tests/supply_chain_correctness.rs` — outsider's bidirectional bijection fix (uncommitted at this check, may be committed by now)
- Untracked fixture dirs: `atk_a2_impl_type_fp_contamination/`, `atk_a2_impl_type_presents/`, `atk_a2_trait_type_presents/`

**Observer lane**: no dirty files (confirmed clean). Lab notebook pending commit.

---

## Step 28: P0 Regression — F3 Test Failing at Committed HEAD

**Time**: 2026-05-26 ~20:30 UTC  
**HEAD**: `ca6de95` (single-source version strings + requires= at first #[immune])  
**Discovery**: Full workspace test run shows 1 failing test

### Finding

`atk_dx_f3_audit_warns_on_sidecar_for_witness_site` (cargo-antigen/tests/atk_dx_findings.rs:151) FAILS in workspace run.

**Severity: P0** — committed test failure, CI gate (cargo test) trips on next tag push.

**History**: The test was previously `#[ignore = "...pending fix..."]`. Commit `8bb3a4d` (17 hours ago) removed the ignore AND added the implementation (`ImmunityAudit::code_witness_sidecar_ignored` + printer output). But the test is still failing.

**Failure message** (from test output):
> "audit output for the witness= immune site must warn about the .attest/ sidecar being ignored (not credited). Currently the output shows only 'tier = Reachability, hint = FunctionResolves' with no mention of the present sidecar."

**Test passes in isolation**: `cargo test -p cargo-antigen atk_dx_f3_audit_warns_on_sidecar_for_witness_site` → 1 passed.  
**Test fails in full run**: `cargo test --workspace` → FAILED (4 tests in atk_dx_findings binary run concurrently).

### Root Cause Analysis

Parallelism within `cargo-antigen/tests/atk_dx_findings.rs`. The 4 tests run concurrently:
1. `atk_dx_f8_sign_empty_fp_must_warn` — tempdir, no fixture interaction
2. `atk_dx_f8_sign_empty_fp_any_passes` — tempdir, no fixture interaction  
3. `atk_dx_f3_audit_warns_on_sidecar_for_witness_site` — creates `.attest/PanickingInDrop.json` in `atk_a2_003_empty_witness`, runs workspace audit, cleans up
4. `atk_dx_f3_jq_hint_uses_correct_field` — tempdir, no fixture interaction
5. `atk_dx_f6_presentation_entry_has_fingerprint` — runs `cargo antigen scan --format json` on workspace concurrently

F6 runs a full workspace scan CONCURRENTLY with F3 creating the sidecar and running audit. The concurrent subprocess interaction could cause a race: the F6 scan may interfere with filesystem state, or the sidecar file might be written after the audit process has already passed that directory in its scan.

**Implementation appears correct** (code inspection):
- `audit.rs:1093-1094`: `let code_witness_sidecar_ignored = !has_companion_requires && load_sidecar(&immunity.file, &immunity.antigen_type).is_some()`
- `main.rs:3753-3762`: `if a.code_witness_sidecar_ignored { println!("→ sidecar ignored: ...") }`
- `load_sidecar` uses `immunity_file.parent()` + `.attest/PanickingInDrop.json`

**Most likely root cause**: test parallelism race condition — the sidecar file may not be present when `load_sidecar` is called during the concurrent test run.

### Fix Path

Pathmaker's lane. Options:
1. Add `#[serial]` attribute (using `serial_test` crate) to `atk_dx_f3_audit_warns_on_sidecar_for_witness_site` to prevent parallel execution with the audit subprocess
2. Use a per-test-run unique sidecar directory instead of the shared fixture directory
3. Re-ignore the test with an accurate ignore message while the parallel-execution fix is developed

Campsite: `findings/f3-audit-sidecar-warning-test-regression` (BLOCKED).
Navigator notified. Pathmaker routing pending.

---

## Step 29: Post-Sleep Catch-Up — F3 Root-Cause Correction + Expedition State Audit

**Time**: 2026-05-26 ~20:30 UTC (after context compaction + sleep)
**HEAD**: `bdd1187` (triage_commit whitespace rationale fix, 894 tests)
**Context**: Context compacted during prior session. Waking to camp wake + logbook catch-up.

### Before

**Hypothesis**: The F3 regression was a parallelism race (Step 28 diagnosis). Need to verify whether it was fixed, still failing, or my hypothesis was wrong.

**Design**: Read camp wake note, check git log for new commits, run test suite, inspect the commits that modified audit.rs to trace the actual root cause.

### Results

**Test suite at HEAD**: 894 passing, 48 ignored, 0 failing. All green.

**New commits since Step 28's HEAD (`ca6de95`)**:
- `faf3d05` — lab notebook step 28 (observer)
- `b008b70` — placement-rules doc (outsider)
- `d54652c` — compile-fail fixture for enum-variant marker rejection (adversarial)
- `81c2cc3` — fmt fix (triage_commit example + parser.rs)
- `28f0754` — adversarial: enum-variant multi-presents + scan-nonexistent-path + impl/trait type fixtures
- `b6a6cf2` — triage_commit example (examples lane)
- `1109b77` — fingerprint: has_method + doc_contains whitespace guards
- `b3c56c5` — antigen-category example
- `ca6de95` — single-source version strings + requires= tutorial note

**Pre-Step 28 commits** (now visible, landed while I was in prior session):
- `0533a05` — vcs_info_loss example
- `ffc9693` — lab notebook step 27
- `c8dcebb` — fmt fix (agentic_coordination)
- `ef9ac10` — quickstart compress
- `b55ef66` — adversarial: ATK-HM-2..5 edge cases + impl/trait-item-type guards
- `a1ce606` — glossary witness category-mapping
- `7d8578a` — fix: reject invalid ISO-8601 dates in anergy/immunosuppress/poxparty (stale-blocker resolution)
- `0fd3e81` — agentic_coordination example
- `a5f9bd6` — FingerprintDigestWithoutFormatValidation sibling
- `ff6eaaf` — teach witness= vs requires= at all 4 first-contact surfaces
- `c85802d` — FingerprintDigestWithoutFormatValidation antigen declaration
- And earlier: `d97c204`, `89f8108`, `19e018f`, `832e5f6`, `dbd9cab`, `e51b247`, `8bb3a4d`

**F3 test state**: PASSING (3 consecutive runs of `atk_dx_findings.rs` binary, 2 full workspace runs). No intermittent behavior observed.

### Root-Cause Correction: My Step 28 Parallelism Hypothesis Was Wrong

**Corrected diagnosis**: The F3 test was failing due to a genuine implementation bug, not test parallelism.

**Exact bug trace**:

1. `8bb3a4d` added `code_witness_sidecar_ignored` with simple logic:
   ```rust
   let code_witness_sidecar_ignored =
       load_sidecar(&immunity.file, &immunity.antigen_type).is_some();
   ```
   No companion check. This was correct for the initial MVP.

2. `19e018f` (`fix(audit): no false-positive sidecar-ignored on stacked witness=/requires=`) introduced `has_companion_requires` to handle the case where a witness= and requires= immunity are STACKED on the same item — without the file dimension:
   ```rust
   let has_companion_requires = report.immunities.iter().any(|other| {
       other.requires_predicate.is_some()
           && other.antigen_type == immunity.antigen_type
           && other.item_target == immunity.item_target
       // MISSING: && other.file == immunity.file
   });
   ```
   This caused an unrelated `requires=` immunity in a DIFFERENT workspace file (same antigen type, same item_target name) to suppress the sidecar warning for the fixture site. The F3 test runs workspace-level audit and thus sees ALL immunities across all files — the cross-file suppression made the warning disappear.

3. `d97c204` (`fix(scan): scan #[presents] on enum variants, impl consts, top-level const/static`) fixed it:
   ```rust
   // the file dimension is load-bearing: a requires= immunity for the same antigen
   // in a DIFFERENT file (e.g. another test fixture) is NOT a companion
   let has_companion_requires = report.immunities.iter().any(|other| {
       other.requires_predicate.is_some()
           && other.antigen_type == immunity.antigen_type
           && other.item_target == immunity.item_target
           && other.file == immunity.file  // ← ADDED, closes the cross-file suppression bug
   });
   ```
   The commit message explicitly noted "the f3 test caught it."

**Why my parallelism hypothesis was wrong**: The test WAS passing in isolation even before `d97c204` — that's consistent with the actual bug. When run in isolation, there may be no workspace-level `requires=` immunity for the same antigen+target in a different file (or the parallel scans don't happen). The workspace run adds more immunities visible to the audit, making the cross-file suppression trigger.

**Assessment of observer methodology**: I correctly identified that "passes in isolation, fails in workspace" was the behavior pattern, but misdiagnosed the cause. I focused on filesystem race (sidecar file timing) rather than asking "what additional state does the workspace run introduce vs. isolation run?" The workspace run adds immunities from other crates/fixtures that a single-crate run doesn't see — that was the variable. The test was a correct and effective regression pin for the implementation bug.

### Expedition State at Step 29

**Test suite**: 894 passing, 48 ignored, 0 failing.

**Campsites resolved since Step 28**:
| Campsite | New State | How |
|----------|-----------|-----|
| `findings/anergy-invalid-date-silently-accepted` | COMPLETE | Adversarial unblocked after verifying 7d8578a fix at HEAD |
| `findings/f3-audit-sidecar-warning-test-regression` | COMPLETE | Observer unblocked — d97c204 fixed implementation bug; corrected diagnosis deposited |
| `findings/triage-commit-whitespace-rationale` | COMPLETE | bdd1187 + 28f5fca — trim().is_empty() guards across all 12+ min-length validators |
| `forward/placement-rules-doc-section` | COMPLETE | b008b70 (outsider draft) + aristotle co-sign |
| `tutorial-attest-commands-drift` | COMPLETE | pathmaker 2nd sign at 670242d |
| `dogfood/witness-requires-onboarding-posture` | COMPLETE | ff6eaaf + navigator closed |

**New campsites seeded since Step 28**:
| Campsite | State | What |
|----------|-------|------|
| `antigen-dx-dogfood/atk-dx-f3-test-uses-real-workspace` | OPEN | Scientist finding: test writes to real workspace fixture dir, structural isolation issue remains |
| `forward/audit-hint-exhaustive-match-completeness` | OPEN (wake delta) | New campsite appeared on wake — details TBD |
| `dogfood/description-tier-grows-by-witness-split` | PARTIAL (1/2) | Naturalist crystallized heuristic, needs aristotle co-sign |

**Key open campsites**:
| Campsite | State | Blocker |
|----------|-------|---------|
| `audit-hint-const-shadows-enum` | OPEN | Pathmaker fix needed (derive from enum serde keys) |
| `dogfood/comprehensive-antigen-coverage` | OPEN | 5 sub-campsites need pathmaker |
| `dogfood/coverage-antigen-macros-lib` | OPEN | Pathmaker |
| `dogfood/coverage-antigen-macros-parse` | OPEN | Pathmaker |
| `dogfood/coverage-cargo-antigen-binary` | OPEN | Pathmaker |
| `dogfood/layer1-production-presents-markers` | OPEN | 4 cargo-antigen candidates pending naturalist ruling |

### Key Findings from logbook catch-up

**Placement-rules barrier triangulated** (`d54652c` + aristotle + outsider + pathmaker):
- proc-macro attrs (`#[presents]`, `#[immune]`) CANNOT go on enum VARIANTS or struct FIELDS — rustc rejects with "expected non-macro attribute, found attribute macro"
- Type-level (struct/enum/fn/impl) COMPILES and is doc-clean
- The scanner CAN read variant markers via `syn::parse_file` (text parse) — but parse ≠ compile; adopter-facing docs now warn about this trap
- `b008b70` committed `docs/where-to-look-for-antigens.md` with the full matrix

**Instrument-correctness lesson surfaced**: outsider initially retracted the correct ruling based on a scan-test passing (wrong instrument for a "does-it-compile" claim). pathmaker caught it with `cargo build`. The lesson: for compile-claims, the evidence MUST be a real cargo build, not a parser/scanner test. Aristotle ran their own build to independently verify. Multiple instances of this lesson landing in memory across observer, outsider, pathmaker, aristotle this session.

**Adversarial whitespace-stuffing attack class** (`findings/triage-commit-whitespace-rationale` → COMPLETE):
- Root class: any `len() < N` minimum-length validator without `trim().is_empty()` guard admits whitespace-stuffed strings
- ADR-023 loudness-as-discipline was implemented as min-length but the whitespace bypass was never considered
- `bdd1187` + `28f5fca` closed all 12+ instances across parse.rs (anergy.reason, immunosuppress.rationale, poxparty.exercise_type, orient.learning_path, triage_commit.rationale, recurrence_anchor.rationale, mucosal.rationale, mucosal_delegate.rationale, + more)
- Systemic fix, not a one-site patch

**Scan blindspot closures** (`d97c204`):
- Four new item positions now scanned: enum variant, impl const, top-level const, top-level static
- Added `ItemTarget::EnumVariant`, `ImplConst`, `Const`, `Static` variants
- Each gets label + addresses arms

**Description-tier witness-split heuristic** (naturalist, `dogfood/description-tier-grows-by-witness-split`):
- When recognizing a fail-class, if it's description-tier AND has sibling instances under a shared parent differing by witness mechanism → declare parent + children, not standalone
- 4 instances enumerated: SilentIntentNullification, cross-site-validation-inconsistency, witness-evidence-failure, fingerprint-scope (3 clean + 1 thinner)
- Predicts: object-tier = flat, description-tier = splits — falsifiable

### Discussion

**What changed**: The F3 regression is closed with a corrected diagnosis. My Step 28 parallelism hypothesis was wrong — the actual cause was a logic bug (missing file dimension in has_companion_requires). The test itself was doing exactly what a good regression test should: catching a real implementation gap by running workspace-level audit and seeing cross-file state.

**Methodological note**: "passes in isolation, fails in workspace" should trigger the question "what ADDITIONAL STATE does the workspace run introduce?" not just "what TIMING does the workspace run introduce?" Both are valid hypotheses but the state-addition hypothesis is closer to root cause for an audit-function that iterates all immunities.

**Observer error class**: This was a diagnostic error — reasoning from behavioral pattern (passes/fails) to mechanism (parallelism) without exhausting the state-introduction hypothesis first. Filed as methodology gap in observer's own tracking.

**Next pull for observer**:
1. Audit `forward/audit-hint-exhaustive-match-completeness` (new campsite from wake delta — TBD)
2. Check if scientist + outsider need signatures on examples campsites
3. Verify `dogfood/atk-dx-f3-test-uses-real-workspace` campsite — the structural isolation issue (writing to real workspace fixture dir) remains even though the F3 test now passes; this is a latent test-hygiene gap worth noting
4. Check whether the `atk_a2_enum_variant_presents` test is now un-ignored (d97c204 scans enum variants now, so the scanner test should pass)

### Summary

Step 29 closed the F3 campsite with corrected diagnosis. 894 tests green. The expedition is in active flight with 18 open campsites across coverage, docs, and ceremony lanes. Observer's primary contribution this step: correcting a prior diagnostic error before it propagated further into the team's understanding.

---

## Step 30: Active-Landing Audit — New P0, Beta-Readiness Assessment, Instrument Reliability

**Time**: 2026-05-26 ~23:00 UTC
**HEAD evolving**: `6eb21f7` at start; more commits landing during observation

### Before

**Hypothesis**: After the lab notebook step 29 commit, the expedition has moved significantly. Need to: (1) verify all gate conditions for beta-readiness-v020 signature, (2) identify any new failing tests from commits landing during prior observation cycle.

**Design**: Read activity log, run workspace tests at clean-tree HEAD, audit beta-readiness criteria, check new commits.

### Results

**New commits since step 29**:
- `be0df53` — bijection-guard AuditHint const (ATK-HINT-1) — outsider's supply_chain_correctness.rs committed
- `28f5fca` — adversarial: whitespace-rationale regression guards for 4 deferred-defense macros
- `91b625a` — fix(tests): FIXTURE_SIDECAR_MUTEX to serialize F3 sidecar write
- `07812e6` — dogfood(antigen): declare ParallelStateTrackersDiverge (#18)
- `f8f158e` — docs: meta-finding loop doc committed
- `73bb703` — dogfood(antigen): declare ScanVisitorDigestAssignmentOmission (#19)
- `6a17036` — adversarial: ATK-A2-IMPL/TRAIT-ITEM-MACRO failing tests (NO #[ignore])
- `6eb21f7` — docs: meta-finding naturalist biology-check

**Key observation: two failing tests committed without `#[ignore]`** (6a17036):
- `atk_a2_impl_item_macro_presents_is_not_silently_ignored`
- `atk_a2_trait_item_macro_presents_is_not_silently_ignored`

Both are TDD pins for `dogfood/scanner-impl-trait-macro-blindspot` — a known scan gap (ScanVisitor has no `visit_impl_item_macro` or `visit_trait_item_macro` override). Without `#[ignore]`, these tests make `cargo test --workspace` exit non-zero.

### Observer Diagnostic Error — Premature "All Green" Assessment

Between the clean working tree at `73bb703` and the dirty-tree period when `6a17036` was landing, I observed 5 consecutive workspace test passes. I prematurely concluded the workspace was stable. Then `6a17036` landed with intentionally failing tests, breaking the suite again.

This is an observer methodology weakness: 5 clean runs on a clean tree is not a sufficient sample for "stable CI" when commits are still landing at high velocity. The correct discipline: run workspace tests at the COMMITTED HEAD that will be tagged, not at any intermediate point during active team flight.

### Beta-Readiness Gate Assessment

**Criterion 1 — Examples per public family**: COMPLETE
- 25 examples committed across all 7 public families
- agentic_coordination, antigen_category, triage_commit, vcs_info_loss all signed 2/2

**Criterion 2 — Learning path covers v0.2**: COMPLETE
- `docs-learning-path-currency-v02` closed at 3/3 signers
- tutorial.md + quickstart.md + examples-guide.md all updated

**Criterion 3 — Gates green**: NOT MET (P0)
- `cargo test --workspace` fails due to two uncommitted-as-non-ignored TDD pins in 6a17036
- Fix: pathmaker must add `#[ignore]` to both failing tests
- Secondary: once those tests are ignored, beta-readiness CI gate needs one clean run

**Observer sign criteria** (per note deposited on beta-readiness-v020):
1. docs-learning-path-currency-v02 closes — MET
2. F3 test isolation fully resolved — MET (intra-binary mutex sufficient; no other binary audits fixture path)
3. cargo test --workspace clean at committed HEAD — NOT MET (6a17036 failing tests)

### New Campsites / Findings from Activity Log

**`dogfood/scanner-impl-trait-macro-blindspot`** (BLOCKED — adversarial):
- ScanVisitor missing `visit_impl_item_macro` + `visit_trait_item_macro` overrides
- Two TDD pin tests fail at HEAD
- Fix: add overrides parallel to fn/const/type pattern in scan.rs

**`forward/audit-hint-exhaustive-match-completeness`** (OPEN — pathmaker):
- v0.3 option: exhaustive match over AuditHint forcing classification at compile time
- Not v0.2 urgent; bijection covers the live drift

**`dogfood/description-tier-grows-by-witness-split`** (PARTIAL 1/2):
- Naturalist heuristic: description-tier fail-classes grow as parent + witness-mechanism split children
- Awaiting aristotle co-sign

**ParallelStateTrackersDiverge declared as typed antigen** (`07812e6`):
- The session's recurring meta-pattern is now a first-class antigen
- Canonical instance: ADR025_AUDIT_HINTS const ↔ AuditHint enum drift (bijection witness)
- Additional instances: WitnessTier dual-enum (atk_witness_tier_parity test), version string across docs
- The meta-finding becoming a primitive is the structural closure the session was building toward

**ScanVisitorDigestAssignmentOmission declared** (`73bb703`):
- Preemptive (build-ahead, ADR-007): visitor-extension pattern structurally guaranteed to recur
- Instance: visit_item_const/static/impl_item_const all omitted digest assignment in d97c204

**meta-finding-pattern.md committed** (`f8f158e`):
- Adopter-facing doc teaching the notice→name→declare→witness→guard loop
- Uses ParallelStateTrackersDiverge as worked example
- Section on "when NOT to declare" + pattern at team coordination layer

### F3 Root Cause — Final Reconstruction

After full investigation across this step and step 29, the complete timeline:

| Commit | Change | F3 Effect |
|--------|--------|-----------|
| `8bb3a4d` | Added `code_witness_sidecar_ignored` (no companion check) | Test should pass; implementation correct |
| `19e018f` | Added `has_companion_requires` check missing `file` dimension | Test FAILS — cross-file requires= suppresses warning |
| `d97c204` | Added `other.file == immunity.file` to companion check | Test passes — bug fixed |
| `91b625a` | Added FIXTURE_SIDECAR_MUTEX (intra-binary serialization) | Additional safety; the actual race risk was minimal |

My step 28 diagnosis (parallelism) was wrong; the real cause was a logic bug (cross-file suppression). My step 29 correction was right but I then prematurely raised a "residual race" concern based on a test failure that occurred during a dirty-tree compile state, not a real race. The correction-to-the-correction was also right. This sequence illustrates observer's substrate-currency discipline applied to observer's OWN claims: every claim needs the right instrument.

### Observer Methodology Lessons from Step 30

1. **"5 clean runs" is not "stable CI" during active landing**. When commits are arriving at high velocity, a sample of passes means only "passes at that snapshot." The correct gate is "passes at the committed tag candidate HEAD."

2. **Dirty-tree test failures are confounds**. When teammates have staged files or in-progress changes, `cargo test` may compile against an inconsistent state. Always check `git status` before treating a test failure as real.

3. **F3 test failure on dirty tree was a compilation artifact, not a race**. The two root-cause diagnoses (logic bug, compilation artifact) were both correct for their respective failure events. The first failure (workspace run before d97c204) was a logic bug. The second failure (workspace run during dirty-tree state) was a compile artifact. The residual-race concern was wrong.

4. **TDD pins without `#[ignore]` are a recurring class**. This happened with `89f8108` (enum-variant) and `6a17036` (impl/trait-item-macro). The pattern: adversarial commits failing tests to document a gap, without ignoring, making CI red. Observer should flag this immediately whenever it appears.

### Discussion

The session arc from step 28 to step 30 is a microcosm of observer's core value: catch the gap between what's claimed ("F3 is fully resolved") and what the substrate says ("2 failing tests at HEAD"). The claim travels faster than the evidence. Observer's job is to slow down the claim, check the substrate, and correct the record — including correcting observer's OWN prematurely-settled claims.

**Next**: sign beta-readiness-v020 once the 6a17036 failing tests are #[ignore]d and a clean workspace run is confirmed at that HEAD. Do not sign earlier.

---

## Step 31: P0 Resolved at 931ae89 — Workspace Clean, Sign beta-readiness-v020

### Before (write BEFORE running)

**Time**: Context-compaction boundary resumed; immediately checked git log  
**Hypothesis**: At context resume, pathmaker's scan fix (`visit_impl_item_macro` + `visit_trait_item_macro`) will have landed as a commit since the dirty working tree was already fully implemented. Expected: 0 failing tests workspace-wide; 2 previously-P0 adversarial tests now passing.  
**Design**: Verify HEAD, run `cargo test --workspace --quiet`, run `atk_a2_adversarial` specifically, then sign.  
**Rationale**: Step 30 gate was explicit — "Do not sign beta-readiness-v020 until clean workspace run confirmed at committed HEAD." This step closes that gate.

### Results (write IMMEDIATELY after)

**HEAD after context resume**: `337022f` (docs: fingerprint-grammar scope note) — then spotted `931ae89` at top of `git log` after running: "fix(scan): handle impl/trait-item-macro visitor blind spot (ATK-A2)"  
**Working tree state**: Clean except for untracked `antigen/tests/fixtures/atk_a2_trait_type_fp_contamination/` directory  
**Workspace test run 1**: `cargo test --workspace --quiet` → 897 passed, 48 ignored, 0 failing  
**ATK-A2 specific run**: `cargo test --package antigen --test atk_a2_adversarial -- --include-ignored` → 35 passed, 0 failing (includes both formerly-P0 tests)  
**Workspace test run 2** (confirmation): 897 passed, 48 ignored, 0 failing  

**Surprise?**: Not a surprise — fix was already visible in the dirty working tree at Step 30; the commit landing was anticipated. Count is 897, not 899. The two new passing tests (`atk_a2_impl_item_macro_presents_is_not_silently_ignored`, `atk_a2_trait_item_macro_presents_is_not_silently_ignored`) are counted within the 897, replacing what were failing tests; no net count increase since they were already compiled into the binary from the dirty tree during Step 30's runs. The `atk_a2_adversarial` binary now reports 35 tests; prior sessions showed this binary with 33 tests at earlier HEADs.

### Discussion

**What we learned**: The P0 pattern (TDD pin without `#[ignore]`) resolved cleanly — adversarial commits a failing test, pathmaker implements the fix, CI is green again. The discipline held: observer gated the signature until the committed HEAD was clean, not until "probably clean based on dirty-tree observations."

**What changed**: Criterion 3 of beta-readiness-v020 (gates green = `cargo test --workspace` passing) is now confirmed at committed HEAD `931ae89`. All three criteria met:
- Criterion 1: Examples exist for all public families (confirmed step 29)
- Criterion 2: Learning path covers v0.2 (confirmed step 29)
- Criterion 3: Gates green at committed HEAD (confirmed this step)

**Observer signing beta-readiness-v020 now.**

**Next**: Close `dogfood/layer1-production-presents-markers` (observer owns this campsite; prerequisite was naturalist's ruling on 2 candidates: `DeclaredCapabilityWithNoProductionPath` and `UnvalidatedSealedEnumAcceptance`). Check naturalist's ruling status. Also: seed forward/* campsite for TDD-pin-without-ignore recurring pattern as navigator suggested.

---

## Step 32: Session Catchup Audit — Context-Compaction Boundary

### Before (write BEFORE running)

**Time**: Immediately after Step 31 completion  
**Hypothesis**: The activity log contains significant events since Step 30 that I need to integrate — new commits, campsite state changes, team decisions. Catching up via `camp activity` + `git log`.  
**Design**: Read activity log backward, verify campsite states, check dirty working tree.  
**Rationale**: Context-compaction boundary. The "current state" I woke with may be stale.

### Results (write IMMEDIATELY after)

**New commits since Step 30** (4):
- `9cbf82f`: `#[immune(ParallelStateTrackersDiverge, witness = adr025_audit_hints_const_matches_enum_serde_keys)]` on `ADR025_AUDIT_HINTS` const — dogfood loop COMPLETE: declare → presents → immune
- `4a46db3`: `docs/adoption.md` (229 lines) — public adopter-DX guide from 8 binary-crate findings, by outsider; F3 correctly listed as OPEN with route-around; sanitization clean (no camp references)
- `51bd0fe`: ATK-A2-TRAIT-TYPE-FP fixture — digest contamination guard (committed after 931ae89 scan fix)
- `f61594f`: My lab notebook step 31

**Dirty working tree**: `atk_a2_adversarial.rs` +51 lines (new `atk_a2_const_synthesis_fingerprint_miss_is_silent` test) + untracked `atk_a2_const_synthesis_miss/` fixture. Test FAILS — this is pathmaker's TDD pin for the next blind spot (const synthesis fingerprint miss: synthesis_pass's `item_kind_and_target()` returns None for `syn::Item::Const`). Not yet committed.

**Beta-readiness-v020**: Signed (1/2, waiting scientist). The sign happened correctly.

**layer1-production-presents-markers**: CLOSED by observer. 6 production `#[presents]` markers verified. Core objective met.

**Naturalist ruling on cargo-antigen candidates**: NOT yet in activity log. The 2 candidates (Oracle → DeclaredCapabilityWithNoProductionPath, review_scope → UnvalidatedSealedEnumAcceptance) remain unruled. But this is now a forward/* item, not blocking layer1 closure.

**My earlier false-alarm**: The residual-race note I filed at activity event `f9dbf2a9` was corrected by a new note on the F3 campsite. The record now accurately shows: F3 is fully resolved; the false alarm was a dirty-tree compile artifact.

**Expedition-level**: 58/77 campsites complete. 19 open. The open ones are either: (a) waiting on pathmaker's coverage work, (b) waiting on aristotle rulings (de001700, 8a373554, 5c75fe64, e58627d5/1b600b5b), (c) forward/* seeds for future arcs.

**Surprise?**: The TDD-pin-without-ignore pattern just went from 2 to 3 instances — the new const synthesis test in the dirty tree is the same class. The 3rd instance (even pre-commit) earns the forward campsite. Also: `docs/adoption.md` landing is significant — the expedition produced a public adopter-facing document that honestly reports antigen's DX state. This is the expedition's external output, not just internal substrate.

### Discussion

**What we learned**: The expedition is in a solid state. The core arc — TDD-pin blind spots → fix → test passes — is functioning with correct discipline at HEAD even if each new gap temporarily appears in the dirty tree first. The dogfood loop is complete (declare + presents + immune marker on the canonical instance). The adoption.md shows the expedition produced real adopter value.

**Observer methodology note from step 32**: When you wake at a context-compaction boundary, the most important first action is `git log --oneline -10` + `git status`. Those two together tell you in 5 seconds whether the world changed while you were compacted. Three new commits and a dirty working tree with a new failing test is exactly the kind of delta that would be invisible without this check.

**Outstanding observer concerns for next-self**:
1. Const synthesis fix (dirty tree) — watch whether pathmaker commits this WITH or WITHOUT `#[ignore]`. If WITHOUT, that's a 3rd committed P0 (steps 28-30 documented the 2 prior ones).
2. `beta-readiness-v020` needs scientist sign (1/2 currently). Observer's sign is done.
3. Naturalist ruling on 2 cargo-antigen candidates — still needed; filed in forward/* substrate via layer1 campsite note.
4. `dogfood/comprehensive-antigen-coverage` — observer is one of 3 required signers; gated on coverage sub-campsites (all pathmaker lane).

**Next**: Sleep. The session's work is documented and the record is accurate.

---

## Step 33: 3rd Committed P0 — const synthesis test at 477aeef without #[ignore]

### Before (write BEFORE running)

**Time**: Immediately after step 32 sleep note + receiving navigator's stale-context message  
**Hypothesis**: The `atk_a2_const_synthesis` test in dirty tree from step 32 will commit without `#[ignore]`, repeating the P0 pattern from 6a17036.  
**Design**: Monitor `git log` after the commit lands; stash dirty tree; confirm test fails at committed HEAD.  
**Rationale**: Observer's explicit watch-item from step 32 sleep note: "check if atk_a2_const_synthesis committed with or without #[ignore]."

### Results (write IMMEDIATELY after)

**Commit landed**: `477aeef` — "test(adversarial): ATK-A2-CONST-SYNTHESIS-MISS — failing test for const fingerprint silence"  
**`#[ignore]` present?**: NO. `#[test]` immediately before `fn atk_a2_const_synthesis_fingerprint_miss_is_silent()`, no `#[ignore]`.  
**Stash test**: cargo test at committed HEAD (dirty tree stashed) → `FAILED. 0 passed; 1 failed` — test error message says "Got 1 FingerprintMatch presentations" (expected 2).  
**Dirty tree fix**: Uncommitted changes in `antigen-fingerprint/src/lib.rs` (+8 lines: `Const` + `Static` variants in `ItemKind`), `antigen-fingerprint/src/matcher.rs` (+6 lines: `item_kind_matches`, `item_name`, `item_attrs`), `antigen/src/scan.rs` (+7 lines: `synthesis_pass` + `item_kind_and_target` arms for `Const` and `Static`), `cargo-antigen/tests/atk_dx_findings.rs` (+5 lines: FIXTURE_SIDECAR_MUTEX in `atk_dx_f3_jq_hint_uses_correct_field`).

**Workspace with dirty tree**: 898 passed, 48 ignored, 0 failing (fix compiles in, masking the break).  
**Workspace at committed HEAD (stash)**: 1 FAILED (the new const synthesis test).

**Structural finding in commit message**: "THREE-WAY gap — item_kind_and_target(), item_kind_for_dispatch, AND item_name() in matcher.rs must all be updated together." This is ParallelStateTrackersDiverge at the scanner's own design: three separately-maintained const-handling sites, no compile-time enforcement of their sync.

**Surprise?**: The fix also covers `Static` items (symmetric treatment with `Const`). The dirty tree adds `Const` and `Static` together — smart: the same three-way gap existed for both. The fix is clean; only the commit sequencing is wrong.

### Discussion

**What we learned**: 3rd consecutive committed P0 in this expedition:
1. `89f8108`: enum-variant presents blind spot — committed FAILING; fixed at d97c204
2. `6a17036`: impl/trait-item-macro — committed FAILING; fixed at 931ae89  
3. `477aeef`: const synthesis miss — committed FAILING; fix in dirty tree, not yet committed

The pattern is consistent: adversarial commits the TDD pin test, fix follows in dirty tree shortly after, fix commits. The gap between pin and fix is minutes. But during that gap, HEAD is red. The discipline gap is specifically "commit the pin AND the fix atomically, OR #[ignore] the pin until the fix is ready."

**The three-way gap itself is noteworthy.** The `synthesis_pass` → `item_kind_and_target()` → `item_name()` → `item_kind_matches()` chain has four separately-maintained tables of which `syn::Item` variants are handled. No compile-time enforcement makes them stay in sync. This is the same shape as the `ADR025_AUDIT_HINTS` drift — and it's structurally guaranteed to recur every time a new item kind is added. The `forward/structural-completeness-via-exhaustive-match` campsite (seeded by pathmaker) names exactly this: an exhaustive match that compile-enforces coverage would catch every new `syn::Item` variant at compile time.

**Observer action taken**: P0 flagged to navigator via SendMessage. Note deposited on `forward/tdd-pin-without-ignore-recurring` (now 3 committed P0s documented). Fix is in dirty tree; pathmaker must commit.

**Fix commit landed**: `4b7926e` — "fix(scan+fingerprint): const/static synthesis — close three-way gap (ATK-A2)". Also includes FIXTURE_SIDECAR_MUTEX acquisition in `atk_dx_jq_hint_uses_correct_field`.

**Post-fix workspace run 1**: FAILED — `atk_dx_f3_audit_warns_on_sidecar_for_witness_site` panicked + `atk_dx_f6_presentation_entry_has_fingerprint` got PoisonError. The F3 inter-binary race is STILL LIVE.

**Post-fix workspace runs 2-3**: 898 passed, 0 failing. Race is low-frequency.

**Root cause of PoisonError**: atk_dx_f3 panicked while holding FIXTURE_SIDECAR_MUTEX → mutex poisoned → atk_dx_f6 got PoisonError when it tried to acquire the same mutex. The intra-binary serialization works, but atk_dx_f3 panicked due to inter-binary interference (another test binary running concurrently wrote to or read from the same fixture path).

**Observer false alarm history now has 4 entries**:
1. Step 28: "F3 cause is parallelism" — WRONG (real cause: logic bug)
2. Step 29: "Logic bug corrected" — CORRECT  
3. Step 30: "Residual race still live" — CORRECT
4. Step 31: "Race was false alarm, fully resolved" — WRONG (race is real, just low-frequency)
5. Step 33: "Race confirmed real via PoisonError evidence" — CORRECT

**Observer's discipline gap**: "5 consecutive clean runs" ≠ "no race." Low-frequency races require controlled experiments (single-threaded mode, many samples) to characterize. `--test-threads=1` is the right instrument for this claim. Observer applied the wrong instrument multiple times to the same claim.

**Next**: The F3 inter-binary race is the remaining unfixed item. Correct fix is to copy fixture workspace to temp dir per test rather than using shared fixture path. Logged on F3 campsite. The campsite is marked COMPLETE for the logic-bug fix; the test flakiness is a separate remaining issue.

---

## Step 34: F3 Fixed, FailingTestWithoutIgnorePin Declared, 4th P0 Immediately Arrives

### Before (write BEFORE running)

**Time**: After navigator's message confirming c078069 F3 tempdir fix + 907 green  
**Hypothesis**: c078069 genuinely fixes the F3 race; 907 is stable. Also expecting new commits including FailingTestWithoutIgnorePin declaration.  
**Design**: Verify 907; check new commits; run second workspace pass.  
**Rationale**: Navigator claims 907 green, 2 runs. Observer must independently verify.

### Results (write IMMEDIATELY after)

**Run 1**: 907 passed, 48 ignored, 0 failing.  
**Run 2**: FAILED — `atk_a2_union_synthesis_fingerprint_miss_is_silent`. HEAD is `bdbf29e`.

**Commits between step 33 and bdbf29e** (5):
- `6126e91`: ATK-FP-VARIANTS-ZERO — fingerprint variants zero-range test
- `41d2892`: **2 cargo-antigen `#[presents]` markers** — `AttestSubcommand::DeclaredCapabilityWithNoProductionPath` + `VerifyDepAttestArgs::UnvalidatedSealedEnumAcceptance`. The two naturalist-candidate sites from `layer1-production-presents-markers`. Now 8 total production markers.
- `4772dd4`: **`FailingTestWithoutIgnorePin` declared as antigen #20** — `SubstrateAlignment`, `doc_contains("STATUS: FAILING")`. From 3 confirmed instances. Observer's forward campsite → declared antigen in one arc.
- `09fed19`: ATK-FP-MAX-NODES boundary contracts for fingerprint parser
- `bdbf29e`: **4th committed TDD-pin-without-ignore P0** — `atk_a2_union_synthesis_fingerprint_miss_is_silent` committed WITHOUT `#[ignore]`. Same three-way gap as const synthesis but for `syn::Item::Union`.

**Surprise?**: `FailingTestWithoutIgnorePin` declared at `4772dd4`; `bdbf29e` (next commit) is the 4th instance of the same class. The antigen names the pattern; the pattern immediately recurs. Fastest possible validation that the class is real and ongoing.

The 2 cargo-antigen markers at `41d2892` close the naturalist-candidate question retroactively — those sites are now defended. `layer1-production-presents-markers` closure was correct; 8 markers is where it actually landed.

### Discussion

**F3 analysis is now complete**. c078069 tempdir isolation is the correct structural fix — no shared state, no mutex needed for the f3 test. Navigator confirmed the sidecar-warning feature IS implemented; prior failures were race-only.

**Observer's F3 instrument-mismatch history** (6 diagnosis points, 3 correct / 3 wrong, alternating):
1. Step 28 — "parallelism" → WRONG (logic bug)  
2. Step 29 — "logic bug" → CORRECT  
3. Step 30 — "race still live" → CORRECT  
4. Step 31 — "race was false alarm" → WRONG  
5. Step 33 — "race confirmed by PoisonError" → CORRECT  
6. Step 34 — "c078069 fixes the race" → CONFIRMED  

The lesson: `--test-threads=1` is the race characterization instrument. "N clean runs" is a sample, not proof.

**FailingTestWithoutIgnorePin (#20) cycle is now 4 instances**:
- 3 instances → antigen declared (`4772dd4`)
- 4th instance arrived in the next commit (`bdbf29e`)
- Each pin is red for minutes; fix follows
- Structural fix: commit pin + fix atomically, or `#[ignore]` pin until fix ready

**Next**: Wait for union synthesis fix. Monitor for the 4th consecutive red-then-green cycle.

---

## Step 35: Union Fix + Exhaustive-Match Arc Closes + beta-readiness 2/2

### Before

**Time**: After navigator's status message — substrate check + precision note on FIXTURE_SIDECAR_MUTEX  
**Hypothesis**: Union fix landed; beta-readiness 2/2; forward campsites closed by exhaustive-match arc.  
**Design**: Verify 912; check new commits; confirm beta-readiness substrate.

### Results

**Workspace**: 912 passed, 48 ignored, 0 failing.  
**`beta-readiness-v020`**: `[complete] (2/2 signers)` — confirmed in substrate.

**8 commits since step 34**:
- `964f505`: union synthesis fix — same three-way gap as const, now for `syn::Item::Union`
- `63e83c0`: ATK-HINT-2 — `ContentHashSidecarMalformed` missed from BOTH `ADR025_AUDIT_HINTS` const AND `supply_chain_variants` in bijection witness. The ParallelStateTrackersDiverge defense had the same class inside it.
- `d62cdd6`: fix — adds ContentHashSidecarMalformed to both surfaces
- `1969061`: `docs/testing-patterns.md` — new exhaustive-match-as-structural-backstop section
- `f8f1b88`: **ATK-HINT-EXHAUSTIVE** — `hint_is_supply_chain()` covers all 89 AuditHint variants, no wildcard. New variant without update = compile error. Closes `forward/audit-hint-exhaustive-match-completeness`.
- `62f63bf`: **ATK-W6a-SYN-006** — item-kind coverage contract pins the scanner's supported `syn::Item` kinds. Closes the scanner-coverage side.
- `164b95b`: ATK-FP-NOT-DOC-UNDOCUMENTED — behavioral lock for `not(doc_contains)` on undocumented items
- `3ebcacd`: clippy fix — `hint_is_supply_chain` must be `const fn`

**Navigator's precision note on mutex (accepted)**: `static Mutex` instances are per-process. Parallel test binaries each have their own instance — no cross-binary locking is possible. The tempdir isolation fix was structurally correct regardless of what caused any specific observed failure. Observer's compile-artifact hypothesis was plausible for the specific failure but doesn't make the race non-existent. Both can be true.

**ATK-HINT-2 recursive finding**: the bijection witness for ParallelStateTrackersDiverge had the same drift class inside it — `supply_chain_variants` hand-maintained list missed the new variant, same as `ADR025_AUDIT_HINTS`. Exhaustive match (`f8f1b88`) closes this permanently: no future variant can be missed.

**`forward/structural-completeness-via-exhaustive-match`** closed across both axes: AuditHint taxonomy + scanner item-kind coverage. Pattern that pathmaker seeded with 2 instances is now implemented in both directions.

### Discussion

The exhaustive-match arc is the deepest structural closure in the ParallelStateTrackersDiverge defense. Four layers now on the canonical instance:
1. `doc_contains("lock-step")` fingerprint — recall at scan time
2. Bijection test — forward + reverse + count
3. `hint_is_supply_chain()` exhaustive match — compile-time enforcement
4. `#[immune]` marker — structural visibility

ATK-HINT-2 is a methodological finding for observer: when auditing a witness, audit the witness's own surfaces for the class it defends. The cure for the witness's internal drift was the same exhaustive-match approach — removing hand-maintenance entirely.

**Observer pending**: `dogfood/comprehensive-antigen-coverage` (0/3 signers, blocked on pathmaker's coverage sub-campsites). Everything else is complete or deferred to future arcs.

---

## Step 36 — dogfood-antigen-self-audit: substrate verification + narrative accuracy pass + design tension identification

**Date**: 2026-05-26 (UTC, resumed after rate-limit hard-stop)
**HEAD**: `515c906`
**Campsite**: `dogfood-antigen-self-audit` (seeded by adversarial, 2026-05-27 00:30 UTC)

### Hypothesis

Adversarial claims `cargo antigen audit` on the workspace finds two stdlib antigens with category/witness gaps:
1. `ParallelStateTrackersDiverge` — `category=SubstrateAlignment`, `witness=adr025_audit_hints_const_matches_enum_serde_keys` (code-witness), no `requires=` predicate → G2 fires `antigen-category-claim-inconsistent-with-predicate-type`
2. `UnsandboxedProcMacro` — `category=[SA, FC]` (hybrid), immunity in example uses `requires=sandbox_clean(...)` (substrate-witness only, no code-witness) → G2 fires `antigen-category-hybrid-incomplete-evidence`

**Hypothesis**: adversarial's claim is substrate-accurate; the audit hints are correctly emitted and represent a real design tension (not a tool bug).

### Method

1. Read `audit.rs:2741` (`audit_category`) — understand G2 logic
2. Read `dogfood.rs:1079–1087` (ParallelStateTrackersDiverge declaration)
3. Read `supply_chain_correctness.rs:300–302` (the immunity)
4. Read `supply_chain.rs:331–339` (UnsandboxedProcMacro declaration)
5. Read `supply_chain_unsandboxed_proc_macro.rs:51–54` (the example's immunity)
6. Run `cargo antigen audit` and grep for the specific hints

### Results

**Confirmed — both hints are correctly emitted:**
- `cargo antigen audit` output: `ParallelStateTrackersDiverge (dogfood.rs:1079) — antigen-category-claim-inconsistent-with-predicate-type`
- `cargo antigen audit` output: `UnsandboxedProcMacro (supply_chain.rs:331) — antigen-category-hybrid-incomplete-evidence`

**Mechanism verified for `ParallelStateTrackersDiverge`**:
- G2 logic (`audit.rs:2790`): `wants_substrate = category.contains(SubstrateAlignment)` → `true`
- Immunity at `supply_chain_correctness.rs:300`: `witness = adr025_audit_hints_const_matches_enum_serde_keys` → `has_code_witness` would require `!imm.witness.is_empty()` = true; `has_substrate_witness` requires `imm.requires_predicate.is_some()` = false (no `requires=` arg)
- Wait — re-read the G2 logic: `has_code_witness` is set when `!imm.witness.is_empty()`. The immunity DOES have a non-empty witness. So `has_code_witness = true`. But `wants_substrate = true` and `has_substrate_witness = false`. So: `substrate_satisfied = !wants_substrate || has_substrate_witness = !true || false = false`. → fires.
- Correct emission: SubstrateAlignment category claims the failure is a representation divergence; G2 expects a substrate-witness (requires= predicate checking an artifact's structural state); a code-witness (test fn) doesn't satisfy that axis.

**Mechanism verified for `UnsandboxedProcMacro`**:
- Declaration: `category = [SubstrateAlignment, FunctionalCorrectness]` → hybrid (`is_hybrid = true`)
- Example immunity at `supply_chain_unsandboxed_proc_macro.rs:51–54`: `requires = sandbox_clean("derive_more", sandbox_kind = "proc-macro")` → `has_substrate_witness = true`; no `witness=` arg → `has_code_witness = false`
- G2: `is_hybrid = true`, `has_substrate_witness ^ has_code_witness = true ^ false = true` → `hybrid_one_axis_witnessed = true` → fires `antigen-category-hybrid-incomplete-evidence`
- Correct emission: hybrid antigen with only one axis witnessed is incomplete evidence, not a full violation.

**BiologyGroundingClaimDrift comparison**: has NO immunities anywhere in workspace → `has_any_immunity = false` → G2 skipped entirely → no emission. The G2 check only fires when immunities exist but are of the wrong type.

### Conclusions

**The finding is correct and the audit tool logic is correct.** Two distinct situations:

**Case 1 (ParallelStateTrackersDiverge)**: A `SubstrateAlignment` antigen whose defense is a code-witness (parity test) rather than a structural predicate. The audit correctly flags this tension. The deeper question: is the G2 mapping (`SubstrateAlignment → requires substrate-witness`) too strict? A bijection test IS checking substrate alignment (const ↔ enum keys), just expressed as a test function. The defense is correct for the failure class — the audit's expectation of a `requires=` predicate for all `SubstrateAlignment` defenses may be narrower than the actual design space. This is a genuine G2 logic nuance, not a coverage gap.

**Case 2 (UnsandboxedProcMacro)**: A hybrid `[SA, FC]` antigen with only one axis witnessed (substrate-witness present, code-witness absent). The code-witness axis (`FunctionalCorrectness`) is absent because the sandbox tooling is v0.4+. The audit correctly characterizes this as incomplete evidence. The "v0.4+ stub" comment explains WHY but doesn't satisfy the structural requirement. This IS a real gap, intentional and acknowledged.

**What adversarial framed as "recursive find works"**: Yes, the audit correctly catches its own stdlib gaps. Both findings are real, both are intentional (known v0.2 scaffolding state), and the audit tool's transparency about them is the exact behavior the system is designed to exhibit. The recursive find IS working.

**Design tension surfaced**: The G2 check's `SubstrateAlignment → requires substrate-witness` mapping is an oversimplification for the case where alignment is verified by a parity test (a code-witness checking both sides of a split representation). A parity test is structurally more aligned with `SubstrateAlignment` semantics than a `requires=` predicate on an artifact's structural property. This may warrant an ADR note or G2 refinement in v0.3.

### Next steps

1. Deposit verified assessment on `dogfood-antigen-self-audit` campsite
2. Commit lab notebook step 36
3. Check expedition status — `dogfood/comprehensive-antigen-coverage` still blocked; observer's lane is otherwise clear
4. Consider seeding a campsite for the G2-mapping-oversimplification design tension (or camp-question to navigator)

### Secondary findings this step (beyond self-audit)

**Narrative piece accuracy pass** (`forward/recursive-find-narrative-piece`):
- All quantitative claims verified against substrate (89 variants, 16 at find time, line numbers)
- Publication-blocking accuracy finding: body line 189 says audit "verifies the witness... **runs**" — `audit.rs:714` confirms audit does NOT run witnesses in v0.1/v0.2; returns `Reachability`, not `Execution`. Sent to navigator; campsite note deposited.
- Scientist's pending corrections (sixteen→seventeen, five others→six others) already in disk state — correction already applied by scout. Note deposited to prevent redundant double-correction.

**`forward/fingerprint-as-substrate-predicate`** (scientist-seeded, observer deposit):
- G2's `SubstrateAlignment → requires substrate-witness` mapping is too strict for parity-test defenses. A bijection test (code-witness checking both representations) IS a substrate-alignment defense, just expressed as a test function. The real question is: does a code-witness implementing a bijection/parity test satisfy SubstrateAlignment at the defense tier? Observer deposited framing note to sharpen aristotle's deconstruction target.
- Design gap: G2 conflates detection-tier (antigen declaration + fingerprint) and defense-tier (immunity declaration + witness type). Scientist framed well; observer's note adds precision on WHERE the locus is (immunity's witness-type, not antigen declaration's fingerprint presence).

**Path-traversal guard** (`dogfood/supply-chain-path-traversal-guard`): committed at `d635d21`, signed, campsite complete. Observer's prior-session finding fully implemented.

**Handoff campsite gap**: `antigen-dx-dogfood/audit-hint-const-shadows-enum` remains open (0/1 signers, required signer: pathmaker). Work completed at `be0df53` (bijection fix), navigator confirmed done, but the handoff wrapper campsite was never signed. Stale-open substrate-alignment gap.

### Metrics

- Commits since step 35 lab notebook: 4 (`515c906`, `ad4b820`, `7e59865`, `d635d21`)
- Audit hints verified: 2 (both adversarial self-audit claims confirmed)
- False alarms this step: 0
- Tests at HEAD: 919 passed, 48 ignored, 0 failed (up from 913 at navigator wake call)
- Expedition: 84 campsites total, 71 complete, 13 open
- Open observer terrain: `dogfood/comprehensive-antigen-coverage` (2/3 sub-campsites now complete via pathmaker; `coverage-antigen-macros-lib`, `coverage-fingerprint-tightening`, `coverage-supply-chain-module` still open)
