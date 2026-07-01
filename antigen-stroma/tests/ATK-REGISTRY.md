# ATK Registry — the-frame (antigen-stroma, FRAME epoch)

> The unbroken ID-chain: each adversarial attack is traceable charter → substrate-note → born-red
> test → forever-guard, years later. A passing ATK is **never deleted** — it lives as a regression
> guard with its ID recorded in-file. This registry is the by-hand version of what a future platform
> automates: per claim-kind, which test-class defends it, at what strength, and what is undefended.

## How a born-red ATK lives in this crate (the repo idiom, matched)

The `antigen` workspace has a settled convention this suite follows exactly:

- **Runtime born-red** (`tests/atk_*.rs`): a `#[ignore]` pre-implementation contract that asserts what
  SHOULD be true. It compiles against the **frozen type-signatures** the skeleton provides, and the
  body it calls is `todo!("frame epoch: ...")` — so the test **panics (RED)** until the builder fills
  the body. When the builder lands the fill, the test author **removes `#[ignore]`, confirms the test
  was RED against the stub, then confirms GREEN against the fill**, and it becomes a forever regression
  guard. (Same discipline as `antigen-fingerprint/tests/atk_w6a_fingerprint_contracts.rs`.)
- **Compile-state born-red** (`tests/ui/*.rs` + `tests/compile_fail.rs`, trybuild): the unsound
  interleaving is asserted to be a **compile error**. The `.stderr` is blessed once the type-state
  lands. The `tests/ui_pass/*.rs` companion proves the *sound* path still compiles (the NC). (Same
  harness as `antigen-macros/tests/compile_fail.rs`.)
- **Static fixtures** (`tests/fixtures/atk_*/`): readable specimens a stranger can study — the named
  failure-mode kept as a scannable tree. (Same layout as `antigen/tests/fixtures/atk_a2_*/`.)

**The teeth-check (negative-control class):** every ATK ships with a paired NC. A green test proves
nothing unless it can go red — the NC is the same-module companion that the degenerate-but-discriminating
implementation FAILS. A trivially-vacuous impl (e.g. "always `Unreconstructible`") passes the ATK but
FAILS the NC. If both ATK and NC can pass against a non-implementation, the test is toothless — a finding.

## The registry (FRAME epoch — 9 entries)

| ID | Defends (the claim-kind) | Class | Born-red against | Negative control (teeth) | Source |
|----|--------------------------|-------|------------------|--------------------------|--------|
| ATK-FRAME-IDENTITY | FQ-identity is CONSTRUCTED — `foo::bar` ≠ `baz::bar` (the bare-name `diff.rs` defect is closed) | runtime `#[ignore]` | `node::path::syntactic_fq_path` todo! | two identical items in the SAME module collide (proves only the cross-module case is rejected) | ADR-070 §4.1/§4.2, §8 |
| ATK-FRAME-DIGEST-TIER | `IdentityDigest` is collision-RESISTANT (BLAKE3) — two FNV-colliding items get distinct digests | runtime `#[ignore]` | `node::digest::IdentityDigest::of_tokens` todo! | `ShapeDigest` (FNV) MAY collide near-misses (proves the test targets the identity tier only) | ADR-070 §4.3, §6, §8 |
| ATK-FRAME-DIGEST-STRIP | Forging/editing a LOAD-BEARING antigen attr changes `IdentityDigest` (tamper-evident) | runtime `#[ignore]` (boundary form) | `IdentityDigest::of_tokens` + the strip-set | editing a PURE-ANNOTATION attr does NOT change `IdentityDigest` | ADR-070 §4.3 (A10), §8 |
| ATK-FRAME-DIGEST-STRIP-E2E | the §4.3 come-apart end-to-end on the REAL path (`IdentityDigest::of_item`): forging `#[presents]`/`#[defended_by]` changes identity; toggling `#[diagnostic]`/`#[antigen]` does not | runtime (GREEN now) — **DISAGREEMENT-SETTLER** | the `canonical_identity_tokens` §4.3 seam | the pure-attr direction (stability) is the built-in teeth | ADR-070 §4.3, the builder-vs-scout §4.3 dispute |

> **ATK-FRAME-DIGEST-STRIP-E2E settled the builder-vs-scout §4.3 disagreement EXECUTABLY (verdict: GREEN,
> no hole).** The builder flagged `lower_scan_report`'s `clone_without_antigen_attrs` (strips ALL antigen
> attrs) as a tamper-evidence hole; the scout called it not-a-gap. The strong ATK (against the real
> `IdentityDigest::of_item` path, not the `of_tokens` boundary) is GREEN: the constitute adapter routes
> identity through `canonical_identity_tokens` (strips PURE attrs only, KEEPS load-bearing), so forging
> `#[presents]` DOES change identity. The builder's worry was a correct read of the SHAPE digest (where
> all-strip is right) misapplied to identity — the exact §4.3 "two digests, different strip-sets" conflation
> the ADR warns against. **The weaker `#[ignore]` boundary-form DIGEST-STRIP is now SUPERSEDED by this
> E2E form** — keep both (the boundary one de-ignores when `of_tokens` is independently exercised), but
> the E2E is the load-bearing tamper-evidence guard. `PURE_ANTIGEN_ATTRS` (the strip set) correctly
> EXCLUDES presents/defended_by/descended_from/crossreactive.
| ATK-FRAME-TIER-CAP | A `source=syntactic` read CANNOT construct a `presents`-grade verdict (type-state) | compile-state (trybuild) + runtime | the type-state in `read::tier` / `read::answer` | a `source=resolved` read MAY construct `presents` (cap is tier-keyed, not a blanket ban) | ADR-070 §3.2, §8 |
| ATK-FRAME-TORN-READ | A torn read (detection over a half-published base) is a COMPILE error under `&db`/`&mut db` | compile-state (trybuild) | the `SnapshotHandle<'db>` borrow model | a correctly-sequenced read session (`&db` across detection+field+provenance) COMPILES + sees ONE revision | ADR-070 §3.5, §8 |
| ATK-SCIP-RECON-001 | SCIP reconstruction is TIER-HONEST: a macro-call-site is NEVER silently `Resolved`/`presents` | runtime `#[ignore]` (3-fixture) | `scip::ingest_scip` + `EdgeReconstruction` | **F2:** a plain call-site MUST reconstruct cleanly to `Resolved`. **F3:** a malformed symbol MUST fall through to syntactic, never construct a `StromaNodeId` with it | ADR-070 §5.2 (A5), island `frame-impl-scip-ingestion`, §8 |
| ATK-FRAME-FIDELITY | The freshness witness reads FILESYSTEM mtime, NEVER r-a output (no-self-witness at source) | runtime `#[ignore]` + structural | `fidelity::FidelityWitness::check` | the witness module has NO dependency on the resolution layer (a hung-but-cached r-a would report "fresh"; the fs-mtime path catches what an r-a-output path misses) | ADR-070 §4.9 (A7), §8 |
| ATK-FRAME-LASTCHANGED | Concurrent `last_changed` writes resolve by `max(mtime)` — order-independent monotone join | runtime `#[ignore]` | the maintenance `set` of `last_changed` | a single write with an OLDER mtime MUST NOT regress `last_changed` backward (proves `max`, not blind last-write-wins) | ADR-070 §4.5a (A4), §8 |
| ATK-FRAME-NONIDEM (inherited) | A non-idempotent (counting/blast) semiring without condensation is a COMPILE error | compile-state (trybuild) — **engine-epoch placeholder** | the `CondensedGraph` type-state (engine fill) | the same query on a `CondensedGraph` compiles; an idempotent detection query on a raw graph compiles | ADR-068 clause-3, ADR-070 §4.6 (A9), §8 |

## Gap-closing ATKs (post-floor — BINDING invariants the §8 seed-table did not enumerate)

The §8 table is the FLOOR. A registry cross-walk (every `INVARIANT (BINDS` in ADR-070 → its defending
ATK) surfaced BINDING invariants with NO seed ATK. Three closed here (`atk_frame_preimage_and_demotion.rs`):

| ID | Defends | § | Class | Teeth (NC) |
|----|---------|---|-------|------------|
| ATK-FRAME-PREIMAGE-TOKENS-ONLY | the IdentityDigest preimage is tokens-ONLY (cfg/path are sibling fields, not folded in — else identity couples to resolution-state) | §4.4 | runtime `#[ignore]` | distinct item tokens still produce distinct digests (tokens-only not collapsed to a constant) |
| ATK-FRAME-INGEST-DEMOTION | staleness demotes resolved→dread AT INGESTION, so the STORED tier already reflects it; `corroborate` over two stale-stored edges cannot reach presents (closes the §5.3 corroborate-up window) | §5.3 | runtime `#[ignore]` | (built into the assert: a fresh edge would store Resolved — see ATK-FRAME-FIDELITY's NC) |
| ATK-FRAME-T3-SLOT-PRESENT | the read-frame ships a structurally-present T3Mir slot; a 2-axis frame / deleted T3 is code-drift | §3.1 | compile-level structural (not ignored) | the ladder must be ordered Syntactic < Resolved < T3Mir |
| ATK-FRAME-INJECT-FROM-OVERLAY | an `InjectedException` is constructible ONLY via `from_overlay` — "injected must derive from overlay" is unconstructible to violate (ADR-067 Open-seam-4) | compile-state (trybuild) — **BUILD-SURFACED** | `ui/injected_exception_needs_overlay.rs` fails with `E0451` (private field); `ui_pass/injected_from_overlay_compiles.rs` confirms the door | the `from_overlay` door must still type-check (not a blanket ban) |

> **ATK-FRAME-INJECT-FROM-OVERLAY is BUILD-SURFACED, not seed-derived.** The builder applied the C3
> (law-in-types) pattern to `write.rs` — a THIRD type-state the §8 table did not name (observer F-005).
> A newly-enforced invariant with NO guard is one refactor from silent regression (someone adds a
> public constructor). The test-architect defended it the same day. This is the registry posture live:
> when the build creates a new claim-kind, a test-class must rise to defend it — GREEN, verified teeth.

| ATK-FRAME-TIER-CAP-CONSTRUCTION | the `TieredAnswer` grade is DERIVED from the source tier at construction (`grade = tier.detection_ceiling()`), never caller-supplied — `{Syntactic, Presents}` is unconstructible | §3.2 | property (GREEN) + compile-state (`E0451`) | a Resolved source DOES reach Presents (tier-keyed); corroborate(Syntactic,_) refuses | `atk_frame_tier_cap_construction.rs` |

> **ATK-FRAME-TIER-CAP-CONSTRUCTION is BUILD-DISCOVERED + already GREEN.** The builder built a STRONGER
> TIER-CAP than the skeleton showed: `TieredAnswer::answered` derives the grade inside a private
> constructor, so the false-quiet `{Syntactic, Presents}` is unconstructible, not runtime-refused. My
> original TIER-CAP guarded the `PresentsVerdict` door; this guards the `TieredAnswer` CONSTRUCTION cap
> (the gap a future caller-supplied-grade refactor would reopen). TIER-CAP is now defended at THREE
> strengths: the no-ctor door (trybuild), the construction-level derive (property, GREEN), the
> field-privacy (trybuild). **Substrate-over-memory catch:** authored `#[ignore]` assuming
> `detection_ceiling` was a stub — but the read-CONTRACT fill had already landed, so they were
> silently-GREEN and de-ignored immediately (green-from-birth forever-guards, never born-red).

## v2-PATCH ATKs (SURVEY-DISCOVERED — the constitute synchronization node, post-certification)

The frame's SURVEY (cargo-mutants full sweep) located a real LOGIC bug + its enabling test-void at the
constitute adapter — the single point where a `ScanReport` becomes facts. Closed correct-before-build
(before the engine builds on these facts). This is the registry posture live: the survey named the
undefended claim-kind, a born-red defined "done", the builder fixed, the guard lives forever.

| ID | Defends (the claim-kind) | Class | Born-red against | Negative control (teeth) | Source |
|----|--------------------------|-------|------------------|--------------------------|--------|
| ATK-FRAME-V2-DEDUP-001 | the constitute dedup keys on IDENTITY (`StromaNodeId`), NOT `(file, ItemTarget)` — two items sharing an `ItemTarget` in one file but with distinct bodies (→ distinct `identity_digest`) BOTH survive as distinct nodes (§4.1: `ItemTarget` is NOT identity) | runtime born-red (RED@`eb1156b`) | the `(file, ItemTarget)` `NodeKey` dedup at `adapter.rs:179` (early-return BEFORE `identity_digest` computed) | two GENUINELY-identical records (same `(file, line, ItemTarget)` → same identity) must STILL dedup to one — proves the fix collapses true duplicates, not distinct-same-target items (a "never dedup" impl passes the ATK, fails this NC) | frame-v2#1 island, ADR-070 §4.1, `atk_frame_v2_dedup_identity.rs` |
| ATK-FRAME-V2-CONTAINMENT | `digests_at_line` matches an item by CONTAINMENT (`span.start().line ..= span.end().line`), NOT exact span-start — one item carrying two antigen attrs on DIFFERENT lines resolves BOTH records to the same containing item → same `IdentityDigest` → merges to ONE node | runtime born-red (RED@exact-match) | exact-span-start matching (`== target_line`): the N+1 attr record misses every item → `gap_digests` → distinct id → spurious 2nd node (unmasked by the identity-keyed dedup) | the SAME-line merge case (`same_item_across_two_vecs_merges_to_one_node`) passes under BOTH matchings, so it does NOT guard containment — the DIFFERENT-line case is the discriminating come-apart | frame-v2#1 (builder-deepened), `multi_attr_different_lines.rs`, `atk_frame_v2_dedup_identity.rs` |

> **ATK-FRAME-V2-DEDUP-001 — the teeth-of-record (RED→GREEN, empirical).** Force-run against the
> certified frame `eb1156b` (the `(file, ItemTarget)` dedup): `atk_frame_v2_dedup_two_impls_same_target_both_survive`
> FAILED `left: 1, right: 2` — the second `impl Foo` (distinct body, same `ItemTarget`) was dropped at
> the `node_map.contains_key(&key) → return` gate BEFORE its `identity_digest` was ever consulted. The
> NC (`nc_two_genuinely_identical_records_dedup_to_one`) PASSED at both revisions (true-dup → 1 node).
> The builder's fix (delete `NodeKey`; key `node_map` on `StromaNodeId` itself; compute the id BEFORE
> the dedup gate) turns the ATK GREEN: 2 distinct nodes sharing one `fq_path`
> (`…::same_target_distinct_identity::Foo`), differing only by `identity_digest`. That red→green IS the
> verification of record for this v2 patch. **The specimen** (`tests/fixtures/frame_v2_dedup/`) is a
> readable two-`impl Foo` tree a stranger can study; the test DISCOVERS the two blocks' lines by parse
> (never hard-coded), so an edit that collapses the specimen fails loud rather than going vacuously green.

> **ATK-FRAME-V2-CONTAINMENT — a fix that UNMASKED a latent bug (the deepening).** The DEDUP-001 fix
> (key on identity) had a second-order consequence: it turned a previously-HARMLESS `digests_at_line`
> imprecision into a real double-node bug. One item with `#[presents]`@N + `#[immune]`@N+1 (routine in
> antigen's own dogfooding) emits records at N and N+1; `syn` folds both attrs into the item span
> (`[N ..= close]`). Under exact-span-start matching the N+1 record missed every item → `gap_digests` →
> and *because the dedup now keys on identity* it SPLIT into a spurious second node. (Under the OLD
> `(file, ItemTarget)` dedup this stayed masked — both records shared the key and collapsed regardless.)
> The builder went deeper than the surface dedup bug and fixed `digests_at_line` to match by CONTAINMENT.
> This guard LOCKS that semantic: it is born-RED against exact-match (2 nodes: N real + N+1 gap), GREEN
> under containment (1 node). **Why it was REQUIRED, not decorative:** the existing same-line merge test
> passes under BOTH matchings, so a silent revert to exact-match would pass every other test — only the
> DIFFERENT-line come-apart catches it. The registry posture live: a fix that changes a system's coupling
> can convert a dormant imprecision into a live defect; the test that pins the NEW invariant is owed the
> same day. Specimen `tests/fixtures/frame_v2_dedup/multi_attr_different_lines.rs`; the test discovers the
> span by parse (`FixtureFile::struct_span`) and asserts the span is multi-line, so it can't go vacuous.

> **frame-v2#4 — the enabling test-void, closed (`frame_v2_constitute_lowering.rs` + the reusable
> `ScanReport` fixture).** The survey's sweep found **13 mutants** surviving across the constitute
> lowering — `module_chain_from_path` (lib/main/mod/regular-file boundary logic, ~10), `digests_at_line`
> (line-match + gap-fallback, 2), `lower_scan_report` (body→empty, 1) — ALL because no `ScanReport`
> fixture existed to reach them. The reusable fixture (`tests/support/scan_report_fixture.rs`:
> `ScanReportBuilder` + file-backed `FixtureFile`) is the missing vehicle; 7 integration tests over a
> real module-tree specimen (`tests/fixtures/frame_v2_modtree/src/{lib,main,node/mod,node/locator,a/b/c}.rs`)
> exercise the two private helpers THROUGH the pub `lower_scan_report` — asserting the produced `fq_path`
> (encodes the module chain) and whether the identity digest is a real content digest vs the traceable
> gap-fallback. This is a NEW test-class the suite lacked: **integration-over-the-synchronization-node**,
> reaching private lowering logic through its one public seam. Mutation-verified — see the teeth-sweep.

## Newly-undefended watch (BINDING invariants still WITHOUT an ATK — surfaced, not hidden)

These are BINDING but not yet defended by a test-class. Named here so the gap is a finding with a name,
not a silent hole. Candidates for the next architect or a build-time fill:

- **§4.1a schema-expressibility** — the base schema must be able to EXPRESS antigen's own efferent
  outputs as first-class nodes (open `kind` + open attr-map; no new `ItemTarget` variant needed).
  Hard to make born-red until the attr-map shape lands; a test that constructs an efferent-output node
  via the open schema would defend it. *Undefended.*
- **§4.8 active-cfg capture** — capture the ACTIVE cfg-set (build-config fact), NOT the cfg-gates that
  EXIST in source syntax; single-active-config-per-snapshot. Needs the cfg-capture seam (cargo-metadata
  read) to test against. *Undefended.*
- **§4.5 rename = delete+create** — ✅ **NOW DEFENDED** by ATK-FRAME-LOCATOR-RENAME (property class,
  `atk_frame_locator_rename.rs`, GREEN against the frozen `Locator` type): a rename/move changes the
  Locator value (delete+create); a body edit leaves it stable; cfg is part of the key. 4 tests pass.

## Strength ledger (the registry posture — held by hand; live state 2026-06-30 T+contract-freeze)

- **GREEN against the frozen contract (built, forever-guard — NOT born-red):**
  - **TORN-READ** — `ui/torn_read_does_not_compile.rs` fails to compile with `E0502` (cannot borrow
    `db` mutably while the `SnapshotHandle` `&db` borrow is live). The atomic-publish GEM proven: a
    torn read does not type-check. `ui_pass/serialized_session_compiles.rs` confirms the sound path.
  - **TIER-CAP** — `ui/syntactic_cannot_construct_presents.rs` fails to compile with `E0599`
    (`PresentsVerdict` has no `new` — no public constructor). `ui_pass/resolved_corroborate_mints_presents.rs`
    confirms `corroborate_presents(Resolved, Resolved)` mints. The cap is a compile-time impossibility.
  - **3 structural day-one guards** pass: `*_witness_signature_is_tool_independent` (fidelity is
    `fn(u64,u64,ResolutionTier)->ResolutionTier` — tool-dependence is a compile error),
    `*_t3_slot_is_structurally_present`, `*_three_way_distinction_exists_in_contract`.
  - Both `.stderr` snapshots verified to fail for the RIGHT reason (not a missing-item false-green).
- **born-red NOW (frame-epoch, RED against `todo!()`, awaiting the builder's fill):** IDENTITY,
  DIGEST-TIER, DIGEST-STRIP, SCIP-RECON-001 (F1/F2/F3 runtime), FIDELITY, LASTCHANGED, PREIMAGE-TOKENS-
  ONLY, INGEST-DEMOTION. Verified RED: forcing `--ignored` panics at the exact target (`path.rs:18`,
  the BLAKE3 digest, the fidelity check). Each goes GREEN + de-ignored when its function is filled.
- **born-red PLACEHOLDER (engine-epoch, named-not-built):** NONIDEM. The frame ships the born-red test
  + the type-state SHAPE; the engine epoch fills `CondensedGraph` and the closure. The placeholder is
  the way ADR-067 names `GlobalConsistencyObstruction` deferred-to-sheaf rather than minting a dangling
  defense — the obligation is recorded forward, not forgotten.

## Newly-undefended watch (what NOTHING defends yet — surfaced, not hidden)

- **G10 accrete/migrate** (ADR-070 §7): the base-schema-evolution distinction has NO build-time firing
  mechanism. Frame's initial schema is all pure-accretion onto an empty set, so it is not a frame
  blocker — but it is an **undefended claim-kind** the moment a second lens reinterprets a first lens's
  attribute. Named as a born-red PLACEHOLDER obligation, sequenced to the schema-evolution expedition.
- **Parity-surveillance** (`ParityOracleSharesComposedSource`): deferred oracle; the frame ships only
  the hook signature. Undefended until the oracle lands — named, attachment-point reserved.
- **SovereignMerge** (`SovereignCatalogMergedByReDerivation`): RELOCATED to `antigen-fingerprint`, not
  evaporated (ADR-070 §7 / A6). The frame's base is compose-only by construction; the sovereign-merge
  born-red is owed by the sovereign-generation expedition. Tracked here so the relocation is visible.

## Teeth-sweep status (the negative-control class — held by hand, mutation-testing deferred)

A green test proves nothing unless it can go red. Each ATK above ships a paired NC that a
vacuous/degenerate implementation FAILS. The design-time teeth-sweep (by-reasoning, recorded here) is
complete — for every ATK there is a degenerate impl that passes the ATK but fails its NC:

| ATK | the vacuous impl that would false-pass | the NC that catches it |
|-----|-----------------------------------------|------------------------|
| IDENTITY | every path made unique (counter suffix) | same-module-same-item must COLLIDE |
| DIGEST-TIER | name-sensitive raw-byte hash | ShapeDigest must stay name-INSENSITIVE (all 8 item kinds — v2#2) |
| DIGEST-STRIP | hash raw source (no strip) | pure-annotation edit must NOT move identity |
| TIER-CAP | forbid ALL presents verdicts | `corroborate(resolved,resolved)` must still mint |
| TORN-READ | forbid ALL publishing | the serialized drop-then-publish session must compile |
| SCIP-RECON-001 | always `Unreconstructible` | F2 plain call-site must reconstruct to `Resolved` |
| FIDELITY | always demote | a genuinely-fresh index must KEEP `Resolved` |
| LASTCHANGED | blind last-write-wins | an older observed mtime must NOT regress |
| NONIDEM | forbid ALL queries | condensed + idempotent paths must compile |

**SURVEY-wave obligation (the industrial teeth-check):** once the crate compiles, run **mutation
testing** (`cargo-mutants`) over `antigen-stroma` — a mutant the suite does NOT kill is a coverage-hole
with a name, not a number that dipped. The by-hand sweep above is the design spec for that automated
run. Treat any surviving mutant as a finding routed to the test-architect, not a metric.

### v2-PATCH teeth-check — the constitute adapter, mutation-verified (2026-07-01)

The obligation above, discharged for the constitute seam. `cargo-mutants --file adapter.rs`:

- **First run (against the SURVEY baseline, before the v2 tests):** the constitute lowering was
  integration-test-dark — 13 mutants across `module_chain_from_path` / `digests_at_line` /
  `lower_scan_report` survived (the finding that opened frame-v2#4).
- **After the v2#4 fixture + 8 integration tests landed:** `25 mutants tested: 18 caught, 5 unviable,
  2 missed`. The 2 survivors were BOTH `replace && with ||` at `adapter.rs:119` / `:123` — the lib/main
  special-cases. Their come-apart is a MULTI-segment path whose FIRST segment is `lib`/`main`
  (`src/lib/sub.rs`, `src/main/sub.rs`): with `&&` correct → `["lib","sub"]`; with `||` mutant →
  collapses to `[]`. The existing specimens (`src/lib.rs`, `src/main.rs`, both `len == 1`) could not
  distinguish them — a real teeth-gap, surfaced by the sweep, NOT hidden.
- **Closed:** added the `src/lib/sub.rs` + `src/main/sub.rs` specimens and
  `lib_and_main_special_cases_require_whole_path_not_first_segment` — the come-apart that kills both.
  (The 5 unviable are `Default`-based return-replacements the type does not admit — not coverage holes.)

The lesson (for the strength ledger): a coverage-void and a logic-bug CO-LOCATE — the same test-dark
lowering seam that hid frame-v2#1's dedup bug also hid these boundary-logic survivors. The fixture that
gives one its born-red gives the whole seam its teeth.

### v2#2 + v2#3 teeth-check — the digest.rs keystone, mutation-verified (2026-07-01)

Two teeth-holes the survey's `cargo-mutants` sweep surfaced on `node/digest.rs`, closed in
`atk_frame_digest_tier.rs` (tests-only; the frozen src is untouched):

- **v2#2 — `ShapeDigest::of_item` name-insensitivity untested for 7 of 8 item kinds.** The pre-existing
  `nc_frame_shape_digest_is_name_insensitive` tested ONLY the Struct arm; deleting any of the
  Enum/Union/Trait/Type/Const/Static/Fn arms (digest.rs:217–223) SURVIVED — the kind fell through to the
  name-SENSITIVE raw-DefaultHasher `_` branch, unguarded. **Closed:** `SHAPE_CASES` (one entry per
  ident-bearing arm) drives two parametrized tests —
  `nc_frame_shape_digest_name_insensitive_across_all_item_kinds` (two same-shape/different-name items must
  share a digest → kills all 7 arm-deletion mutants) and
  `nc_frame_shape_digest_shape_sensitive_across_all_item_kinds` (a genuine structural change must MOVE the
  digest → the non-vacuity guard, so name-insensitivity can't pass against a collapse-everything digest).
  **Teeth proven (revert-and-restore):** deleting the `Fn` arm → `[fn]` case RED (two `raw:` digests);
  deleting the `Enum` arm → `[enum]` case RED. Remaining 5 arms structurally identical in the loop;
  cargo-mutants confirms the 8/8 sweep.
- **v2#3 — `pub fn is_load_bearing_antigen_attr` had zero teeth.** Both whole-body replacements
  (`-> true`, `-> false`) SURVIVED (digest.rs:140) — a public helper an organ trusts for a tamper-surface
  decision shipped with no regression guard. **Closed:** `atk_is_load_bearing_antigen_attr_contract_triple`
  pins the triple (`#[presents]`→true, `#[diagnostic]`→false, `#[derive(Debug)]`→false) — the third case
  encodes the SUBTLETY that is_load_bearing/is_pure are complements only WITHIN `ANTIGEN_OWNED_ATTRS`.
  **Teeth proven (revert-and-restore):** body `-> true` → case (b) RED; body `-> false` → case (a) RED.
  Both missed mutants killed.

Both defend clustering-quality / public-API contract, NOT the identity/tamper-evidence tier (that path
routes through `is_pure`, whose mutants were already CAUGHT). A regression here degrades clustering or a
helper's honesty — it does not break signing.
