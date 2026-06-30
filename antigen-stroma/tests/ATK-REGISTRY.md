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
| ATK-FRAME-DIGEST-STRIP | Forging/editing a LOAD-BEARING antigen attr changes `IdentityDigest` (tamper-evident) | runtime `#[ignore]` | `IdentityDigest::of_tokens` + the strip-set | editing a PURE-ANNOTATION attr (in `ANTIGEN_OWNED_ATTRS`) does NOT change `IdentityDigest` (stable change-detection) | ADR-070 §4.3 (A10), §8 |
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
| DIGEST-TIER | name-sensitive raw-byte hash | ShapeDigest must stay name-INSENSITIVE |
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
