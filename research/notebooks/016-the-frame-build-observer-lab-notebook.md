# Lab Notebook 016: The-Frame Build Observer

**Date**: 2026-06-30
**Role**: build-observer (the-frame expedition, Phase-2: BUILD)
**Branch**: 0.6.1-self-non-self (antigen-stroma, workspace member #6)
**Status**: Active
**Depends on**: ADR-070 (da92290), Notebooks 011 (design), 013 (stroma-remembers observer)

---

## Context & Mandate

The-frame expedition Phase-2: BUILD. ADR-070 is law (ratified da92290). The builder has the frame
spec and the born-red ATK suite (deposited by the prior converge wave). My mandate:

- Maintain the forensic decision-trail that makes the builder's self-closes accountable
- Record in real time: what was built at each ADR-070 build-step, what the builder self-closed
  (Hypothesis tier), the reasoning, what is genuinely built vs stubbed vs deferred
- Surface any C3 gaps, methodology concerns, or premature self-closes
- The trail the SURVEY wave inherits to certify

I do NOT certify or build -- I witness only.

---

## BUILD-ORDER invariant (ADR-070)

Read-CONTRACT frozen FIRST -> CONSTITUTE base -> read-IMPL stubs -> write.

---

## 8 Load-Bearing Invariants (I1-I8)

| # | Name | ADR ref | Mechanism |
|---|------|---------|-----------|
| I1 | FQ-identity constructed | 4.1/4.2 | syntactic_fq_path builds module::item path |
| I2 | Two digests, different strip-sets | 4.3 | IdentityDigest (BLAKE3, tokens-only); ShapeDigest (FNV, delegate) |
| I3 | Tokens-only preimage | 4.4 | cfg/path are sibling fields, not folded into digest |
| I4 | Stable locator, changing digest | 4.5 | Locator = (fq_path, cfg_set); IdentityDigest changes on token-edit |
| I5 | Tool-independent fidelity | 4.9 | FidelityWitness reads filesystem mtime, never r-a output |
| I6 | Staleness demotion at ingestion | 5.3 | stale node -> demote tier before storing |
| I7 | Three-way SCIP reconstruction | 5.2 | EdgeReconstruction = Resolved|Ambiguous|Unreconstructible |
| I8 | Open base schema | 4.8 | EdgeKind non_exhaustive, Perspective mirrors it |

---

## C3 Pattern Inventory

| # | Invariant | Mechanism | ADR ref |
|---|-----------|-----------|---------|
| C3-1 | Syntactic read cannot mint PresentsVerdict | PresentsVerdict { earned_at } private field; corroborate_presents sole door | 3.2 |
| C3-2 | Injected exception must derive from overlay | InjectedException { _private } private field; from_overlay sole constructor | 4.7 |
| C3-3 | Three-way SCIP distinction | EdgeReconstruction exhaustive enum; 4th variant breaks compile | 5.2 |
| C3-4 | Constituted, not authored | SourceWitness sole input to constitute(); no authored-bypass constructor | 4.7/WATCH-C2 |

---

## ATK Tracking Table

| ATK ID | Class | Status | Defends |
|--------|-------|--------|---------|
| ATK-FRAME-IDENTITY | runtime, born-red | DE-IGNORED (Step 2) -- GREEN in working tree | I1 FQ-identity |
| ATK-FRAME-DIGEST-TIER | runtime, born-red | DE-IGNORED (Step 2) -- GREEN in working tree | I2 BLAKE3 + ShapeDigest |
| ATK-FRAME-DIGEST-STRIP | runtime, born-red | DE-IGNORED (partial -- preimage boundary only; end-to-end form pending canonical_identity_tokens seam) | I2 tamper-evident |
| ATK-FRAME-TIER-CAP | compile-state + runtime | GREEN (day 1) | C3-1 |
| ATK-FRAME-TORN-READ | compile-state | GREEN (day 1) | SnapshotHandle borrow model |
| ATK-SCIP-RECON-001 | runtime, born-red (3 fixtures) | RED (correct, engine-epoch) | I7 three-way SCIP |
| ATK-FRAME-FIDELITY | runtime + structural | ALL DE-IGNORED (Step 5 fill complete) -- GREEN in working tree | I5 fidelity |
| ATK-FRAME-LASTCHANGED | runtime, born-red | DE-IGNORED (Step 2) -- GREEN in working tree | I4 monotone join |
| ATK-FRAME-NONIDEM | compile-state, engine-epoch placeholder | GREEN (structural) | non-idempotent semiring |
| ATK-FRAME-PREIMAGE-TOKENS-ONLY | runtime, born-red | DE-IGNORED (Step 2) -- GREEN in working tree | I3 tokens-only |
| ATK-FRAME-INGEST-DEMOTION | runtime, born-red | RED (correct, fidelity epoch) | I6 staleness demotion |
| ATK-FRAME-T3-SLOT-PRESENT | structural | GREEN (day 1) | T3Mir slot structurally present |
| ATK-FRAME-INJECT-FROM-OVERLAY | compile-state | GREEN (day 1) | C3-2 |
| ATK-FRAME-TIER-CAP-CONSTRUCTION | runtime property | GREEN (day 1) | TieredAnswer grade derived |
| ATK-FRAME-LOCATOR-RENAME | runtime property | DE-IGNORED (F-010 fixed, Locator::new) -- GREEN in working tree | I4 stable locator |

---

## Hypothesis Tracker

| Hypothesis | Status | Evidence |
|-----------|--------|----------|
| H1: ADR-070 C3 prediction | CONFIRMED | 4 independent C3 patterns, none redundant |
| H2: Born-red ATKs before self-closes | MET | ATK suite in ba7d0ab |
| H3: ba7d0ab coverage complete | CONFIRMED | All 54 files verified |
| H4: Single commit will carry Steps 2-5 | PENDING | All 18 files staged; commit not landed yet |

---

## Findings Log

| ID | Type | Finding | Status |
|----|------|---------|--------|
| F-001 | Note | born-red idiom confirmed | DOCUMENTED |
| F-002 | Note | salsa borrow split gives atomic-publish FREE | DOCUMENTED |
| F-003 | Note | no_lifetime on salsa::interned | DOCUMENTED |
| F-004 | RESOLVED | Tier-cap seam: PresentsVerdict private field | RESOLVED |
| F-005 | GEM | InjectedException C3 type-state -- build-surfaced | DOCUMENTED |
| F-006 | GEM | TieredAnswer grade derived-not-caller-supplied | DOCUMENTED |
| F-007 | RESOLVED | lower_scan_report stub missing source_root | RESOLVED -- SourceWitness.source_root present in working tree |
| F-008 | WATCH | atk_frame_locator_rename struct-literal -- breaks on salsa wiring | ACTIVATED -> F-010 |
| F-009 | GEM | SourceWitness fourth C3 type-state | DOCUMENTED |
| F-010 | RESOLVED | F-008 activated: struct-literal broke on salsa wiring; builder fixed all 4 tests to Locator::new pattern | RESOLVED in working tree |
| F-011 | OPEN | §4.3 canonical_identity_tokens seam gap: strip_antigen_attrs (clone_without_antigen_attrs) strips ALL antigen attrs including load-bearing ones (presents, defended_by, etc.) -- forging #[presents] is INVISIBLE to IdentityDigest as currently wired | Design deposited by builder (ea14d90f, 95d8bfc0); seam not yet typed; pending main's routing decision |

---

## Build-Step Table

| Step | Task | Description | Self-Close? | Assessment | Commit |
|------|------|-------------|-------------|-----------|--------|
| 0 | 16 | StromaDb + Cargo.toml + workspace wiring | YES, Hypothesis | HONEST -- DURABLE (ba7d0ab) | ba7d0ab |
| 1 | 17 | read-CONTRACT: tier.rs / coord.rs / answer.rs | YES, Hypothesis | HONEST -- DURABLE (ba7d0ab) | ba7d0ab |
| 2 | 18 | Node-identity + salsa shape + path-construction | YES, Hypothesis | HONEST -- fills verified; F-010 resolved; F-011 OPEN (seam gap) | pending commit |
| 3 | 19 | Relational base facts (NodeFacts/EdgeFacts/ContractFacts salsa inputs + fact tuples) | YES, Hypothesis | HONEST -- NodeFact has shape_digest; EdgeFact tier-stamped; EdgeKind #[non_exhaustive] + 8 variants | pending commit |
| 4 | 19 | CONSTITUTE adapter (lower_scan_report) | YES, Hypothesis | HONEST -- field-names correct, MarkedUnknown skipped (honest), lineage edges deferred (honest); adapter.rs now 305 lines | pending commit |
| 5 | 20 | FidelityWitness::check + StromaFidelityUnwitnessed | YES, Hypothesis | HONEST -- COARSE_FS_MARGIN=1; saturating_add; demotion at ingestion; all 4 ATKs green | pending commit |
| 6-8 | 21 | Query stubs + write stubs + deferred engine-epoch | NOT YET | -- | -- |

---

## Observations (Summary Form)

### 001 -- Baseline orientation
ADR-070 law; C3 prediction is the key claim to track; 8-ATK seed-table is the floor.

### 002 -- Pre-commit state survey (~20:00 UTC)
52 files staged; HEAD da92290; 23 source files in working tree; ATK suite scaffolded.

### 003 -- read/tier.rs audit (Step 1 Phase A)
COMPLETE (not stub). C3-1 confirmed: PresentsVerdict private earned_at; corroborate_presents sole door. F-004 RESOLVED. E0599 confirmed in trybuild.

### 004 -- read/answer.rs + read/coord.rs audit (Step 1 Phase B)
answer.rs COMPLETE: TieredAnswer private fields; grade derived from tier. F-006 GEM: grade derivation makes {Syntactic, Presents} unconstructible. coord.rs COMPLETE: ReadCoord + Perspective(non_exhaustive) + Polarity.

### 005 -- Compile-state ATK suite verification
4 fail-fixtures (E0502/E0599/E0451/E0451) + 3 pass-fixtures -- ALL GREEN before commit.

### 006 -- constitute/ sweep (PARTIAL)
INCOMPLETE: read adapter.rs only, missed mod.rs. F-009 corrected this in Obs 011.

### 007 -- write.rs audit (C3-2 discovered)
InjectedException._private field: struct-literal unconstructible; from_overlay sole constructor. F-005 GEM. Test-architect added ATK-FRAME-INJECT-FROM-OVERLAY same day.

### 008 -- scip.rs + base/facts.rs audit (I7, I8)
scip.rs: EdgeReconstruction exhaustive enum -- C3-3 confirmed. base/facts.rs: EdgeKind 8-variant open registry (#[non_exhaustive]). IDEMPOTENT gate shape documented.

### 009 -- Full source inventory (23 files)
All 23 src/ files audited. Split: 6 fully implemented, 12 frame-epoch stubs, 5 engine-epoch stubs.

### 010 -- Task 16+17 self-close assessment (pre-commit)
Task 16 HONEST at Hypothesis. Task 17 HONEST at Hypothesis. DURABILITY RISK: uncommitted. Substrate-over-memory catch: test-architect de-ignored detection_ceiling tests when disk said already filled.

### 011 -- SourceWitness sweep (C3-4 found)
constitute/mod.rs: SourceWitness sole input to constitute(); no authored-bypass constructor; WATCH-C2 discharged. C3 count = FOUR. ADR-070 S0 prediction CONFIRMED.

### 012 -- Commit ba7d0ab durability audit (post-compaction, first action)
Commit landed: 54 files, 2407 insertions. Coverage COMPLETE. Tasks 16+17 DURABLE. Constitute signature decision LEAVES-OPEN (build-scout d8412fb8).

### 013 -- Build-scout constitute terrain map (partial correction)
Finding (3) syn dep: initially assessed stale; corrected -- syn was NOT in ba7d0ab, added in Step 2 (working tree). Build-scout was CORRECT. Finding (1) API gap: real but not a hard blocker (HasAttributes public, 3-step preimage sequence viable). Finding (2) ShapeDigest reuse trap: real but ShapeDigest::of_item already routes correctly.

### 014 -- Step 2 working tree audit (Task 18 in progress, ~21:42 UTC)
8 fills confirmed: of_tokens (BLAKE3), of_item (syn dispatch), CfgSet::canonical, syntactic_fq_path, refine_with_scip, is_well_formed_scip_symbol, Revision::merge, salsa::interned live on Locator. 5 test files de-ignored (13 tests). F-010 noted (locator rename test struct-literal break).

### 015 -- Multi-writer collision + coordination resolution (~21:45 UTC)
Build-scout applied salsa::interned to locator.rs concurrently with builder filling pure functions. Tree not compiling. Captain resolved: builder = sole src/ author; scout = map-only. F-010 attribution error: scout was the concurrent actor, not builder forgetting.

### 016 -- Step 3 base/node.rs wiring (Task 19 in progress, ~21:50 UTC)
base/facts.rs: #![allow(missing_docs)]; NodeFacts/EdgeFacts/ContractFacts salsa::input live; NodeFact + ContractFact plain structs. node.rs: salsa::input live; Revision::merge. ShapeDigest lives on Node entity handle, NOT NodeFact tuple -- correct design: closure reads flat NodeFact, per-entity queries use Node.shape_digest(&db).

### 017 -- Step 2 self-close + concurrent-actor correction (~21:52 UTC)
Builder note d5aaf284: STEP 2 COMPLETE. CORRECTION to Obs 015: the concurrent actor on locator.rs was NOT the build-scout but an AUTOMATED WIRING-ASSIST. Builder confirmed via SendMessage (no teammate). Captain's attribution was incorrect; resolution (builder owns src/) was still correct. DESIGN CONFIRMED: NodeFacts holds plain NodeFact value-tuples (not Node handles). #![allow(missing_docs)] = clean salsa-generated method solution.

### 018 -- Constitute adapter fill -- ATTRIBUTION ERROR (see 019)
[CORRECTION: This observation analyzed the IMPLEMENTER's WIP, not the builder's fill. See Obs 019.]
The adapter.rs I read was the-frame--implementer's WIP (camp-pathmaker), not builder's fill. Patterns described (extract_item_at_line, strip_antigen_attrs, blake3_identity, module_chain_from_path, NodeKey dedup) survived to the final version. Field-name errors (MarkedUnknown.item_target, LineageEdge fields) were real in the WIP; builder's review (7e4c5435) provided corrections.

### 019 -- Second thrash event: implementer overlap + attribution correction (~22:00 UTC)
Builder's Step 2 background commit FAILED the gate because the-frame--implementer (camp-pathmaker) was in the shared tree with mid-flight constitute/adapter.rs WIP. REAL FIELD-NAME ERRORS confirmed by builder review (7e4c5435). Tree multi-writer again, this time from implementer violating captain's Step 2 role boundary. This is a cascade of Obs 015 -- the boundary enforcement didn't reach the implementer before they started on adapter.rs.

### 020 -- Implementer stand-down + tree returns to single-writer (~22:00 UTC)
Camp notice [02967418] 21:58 UTC: the-frame--implementer (camp-pathmaker) STOOD DOWN explicitly: "find-wave implementer ceasing ALL edits to antigen-stroma... The crate is YOURS solo now." Critically: implementer held the correct protocol (no checkout/reset/stash -- would silently wipe builder's concurrent untracked work). Implementer slept [a87712a4] 21:58 UTC. Tree is now SINGLE-WRITER again (builder solo). The builder's corrected adapter.rs (field names fixed, MarkedUnknown correctly skipped, lineage edges deferred) is now in the working tree.

STATUS: All 18 files staged. git diff --cached HEAD == git diff HEAD (staging = working tree). Single-writer state confirmed.

ADAPTER.RS FINAL VERIFICATION (post-implementer-corrections, post-builder-review):
- MarkedUnknown: CORRECTLY SKIPPED (no item_target field; comment explains engine-epoch reverse-lookup TODO)
- LineageEdge: CORRECTLY SKIPPED at frame epoch (child/parent are antigen-declaration names, not ItemTarget pairs; frame-epoch = no edges)
- proc-macro2 + quote: both in imports AND in Cargo.toml (Cargo.toml line 38-39)
- constitute() return: handled in mod.rs (lower_scan_report returns Vecs; constitute() wraps with NodeFacts::new/EdgeFacts::new)
- quote::ToTokens: `use quote::ToTokens;` at top of adapter.rs
- All ScanReport field names verified correct vs antigen/src/scan/types.rs

### 021 -- Step 5 fill: FidelityWitness::check + StromaFidelityUnwitnessed (~22:05 UTC)
fidelity.rs (52 lines) is filled (Step 5 complete in working tree):
- COARSE_FS_MARGIN = 1u64 (the FAT32 same-second false-FRESH guard band, ADR-070 §4.9 attack A7)
- FidelityWitness::check(source_mtime: u64, index_mtime: u64, claimed: ResolutionTier) -> ResolutionTier
  - stale = source_mtime.saturating_add(COARSE_FS_MARGIN) >= index_mtime
  - stale -> ResolutionTier::Syntactic (demotion to floor)
  - fresh -> claimed (pass-through)
- Tool-independence enforced by SIGNATURE: takes raw u64s, no r-a handle parameter possible
- StromaFidelityUnwitnessed { reason: UnwitnessedReason } + enum UnwitnessedReason { StaleIndex | ScipUnverified }

ATK-FRAME-FIDELITY (atk_frame_fidelity.rs, 4 tests, ALL ACTIVE -- no #[ignore]):
1. atk_frame_fidelity_stale_index_demotes_resolved_to_dread -- source(t=2000) > index(t=1000) -> Syntactic
2. atk_frame_fidelity_coarse_mtime_guard_band_closes_same_second_false_fresh -- source==index -> Syntactic (guard band)
3. nc_frame_fidelity_fresh_index_keeps_resolved -- source(t=500) < index(t=2000) -> Resolved (negative control)
4. atk_frame_fidelity_witness_signature_is_tool_independent -- fn-pointer coercion proves (u64,u64,ResolutionTier)->ResolutionTier is the signature

ASSESSMENT: HONEST fill. The demotion rule (source_mtime + 1 >= index_mtime -> stale) is conservative in the correct direction. The saturating_add avoids overflow on u64::MAX source_mtime. The 4-test suite has real teeth: test (1) confirms stale-old; test (2) closes the same-second tie; test (3) proves it's freshness-keyed not blanket-cap; test (4) is compile-level. The born-reds were de-ignited -- the 3 lost lines in the diff stat are the removed #[ignore] annotations.

### 022 -- §4.3 canonical_identity_tokens seam design deposited -- OPEN FILL (F-011)
Builder deposited the §4.3 seam design in camp notes [95d8bfc0] + [ea14d90f] 21:59 UTC. The seam is NOT yet typed in digest.rs.

THE GAP (F-011): The current adapter.rs uses strip_antigen_attrs (clone_without_antigen_attrs dispatch) which strips ALL antigen attrs. This defeats tamper-evidence for LOAD-BEARING attrs: forging #[presents] on an item is INVISIBLE to IdentityDigest (the presents attr is stripped before hashing). The ATK-FRAME-DIGEST-STRIP currently tests at the preimage bytes level (of_tokens(WITH_PRESENTS) != of_tokens(WITHOUT_PRESENTS)) but not end-to-end through the canonicalizer.

THE DESIGNED FIX: a public `canonical_identity_tokens(item: &syn::Item) -> Vec<u8>` seam in node/digest.rs that:
- Strips ANTIGEN_OWNED_ATTRS \ LOAD_BEARING (pure-annotation out, load-bearing kept)
- LOAD-BEARING keep-set (forgeable claims): presents, defended_by, descended_from, crossreactive, antigen_tolerance, anergy/immunosuppress/poxparty/orient, mucosal/mucosal_delegate/mucosal_tolerant, dread/aura/red_flag, quarantine, triage_commit
- PURE-ANNOTATION strip-set: diagnostic, itch, recurrence_anchor, crystallize, chronic, saturate, strand, panel, rx, refer, biopsy, ddx, culture, triage, polyclonal/monoclonal/adcc/clonal/igg (witness-classification labels), antigen/immune, antigen_generates
- Borderline: witness-classification labels -- default-KEEP if uncertain (KEEP-a-load-bearing = churn; STRIP-a-load-bearing = invisible forgery; keep is safer)
- ATK-FRAME-DIGEST-STRIP retargets onto end-to-end form: forge #[presents] on a real syn item -> identity CHANGES; toggle #[diagnostic] -> identity STABLE

OPEN STATUS: builder is holding this seam pending main's routing decision (worktree isolation vs route-to-implementer). The seam design is complete (deposited); only typing it and ATK-retargeting remain.

ASSESSMENT: F-011 is REAL. The strip_antigen_attrs approach in the current adapter.rs is correctly identified as a hole by the builder. The gap is: what currently exists (step 4 adapter) STRIPS ALL antigen attrs for the identity preimage, meaning tamper-evidence for grade-level claims (presents, tolerance grants, deferred-defense grants) is NOT currently enforced end-to-end. The atk_frame_digest_strip.rs test exists but is at the preimage-bytes level -- it proves of_tokens correctly differentiates WITH_PRESENTS from WITHOUT_PRESENTS bytes, but it does NOT prove the canonicalizer preserves this distinction. The end-to-end ATK retargeting is the remaining work.

PEER-REVIEW NOTE: This is the right finding to surface. A survey-wave reviewer will need to verify that canonical_identity_tokens both:
(a) strips pure annotations (so identity is stable on documentary-marker edits), and
(b) keeps load-bearing attrs (so the tamper signal fires).
The two requirements are in TENSION -- they have to be verified INDEPENDENTLY, not just as "is the digest different?"

---

## Open Questions

1. §4.3 canonical_identity_tokens seam (F-011): builder designed, not yet typed. Who types it? When? Main's routing decision pending.
2. ATK-FRAME-DIGEST-STRIP retarget: currently tests preimage boundary only. Needs end-to-end retarget once canonical_identity_tokens seam lands.
3. Engine-epoch C3 prediction: will IDEMPOTENT gate (Assert<{ S::IDEMPOTENT }>: IsTrue) be C3? (currently preview comment in facts.rs -- build it at engine epoch)
4. ShapeDigest DefaultHasher fallback in of_item: unstable across toolchains, acceptable for clustering not signing -- future improvement flag.
5. Steps 6-8 not yet filled: query stubs, write stubs, deferred engine-epoch. When does builder move to these?
6. When does the Step 2-5 commit land? (18 files staged but not committed -- builder holding for main's routing on the §4.3 seam)

---

## Artifacts

| File | SHA/location | Description |
|------|-------------|-------------|
| antigen-stroma crate | ba7d0ab | First build-wave commit, Steps 0-1 |
| ATK-REGISTRY.md | ba7d0ab | 9 seed + 4 gap-closing ATKs (13 total) |
| Camp build-observer trail | the-frame/frame-build-observer-trail | Camp notes: F-001 through F-010, Observations 001-022 |
| ADR-070 | da92290 | The Frame Build-Spec |
| Step 2-5 working tree | HEAD ba7d0ab + 18 modified files staged | Steps 2-5 fills uncommitted; waiting for routing on §4.3 seam |
| Step 2 backup | scratchpad/step2-staged-builder.patch | 32KB, 14 files -- builder's pre-thrash backup |
| Garden entry | C:\Users\bfpcl\.claude\garden\2026-06\the-frame-wave-multi-writer-as-mirror.md | Reflection on multi-writer collision pattern |

