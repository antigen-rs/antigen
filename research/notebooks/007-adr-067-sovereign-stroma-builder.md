# Staging draft — ADR-067 (The Stroma: a Sovereign Immune Lattice over Composed Resolution)

> **Staging/ceremony notebook, not the canonical record.** Status **Proposed (v3 — post-council design-pair
> draft)**. Verify the live ADR number before ratifying (ADR-066 landed; 067 expected — rev-parse
> `decisions.md`). Two registers: ratified **spine** (Decision — invariant + process) + drift-allowed **map**.

## Revision log

**v2 → v3 (post 7-lens council + design-pair, 2026-06-24/25).** The council (aristotle · contrarian ·
adv-break · adv-feasibility · practitioner · observer · systems) hardened v2. Bones held (base-as-structural-self,
induced-views, change-as-danger-signal, substrate-then-lenses); the fixes:

- **The headline BLOCKER (3 lenses + substrate): `syn` does NOT resolve call/data-flow/trait edges.** `syn` is
  syntactic; resolving which `foo()` a call binds to needs name+type resolution — rust-analyzer's reason to
  exist. v2's "sovereign roll-our-own resolver, cheap because the parse is sunk" was the **Amd3 *overdose***
  (reinventing battle-tested universal infra). **Fix: resolution is the *herd's*.** Name/type resolution of Rust
  is herd-immunized universal substrate (the whole ecosystem watches rust-analyzer) → Amd3 **clause-1
  compose-freely**, in its safest *require-installed / user-invoked, zero-cascade-surface* form. The **sovereign
  part is the immune lattice** (attributes, contracts, field, sheaf); the **resolution is composed from the
  user's r-a.** (NOT a granuloma — that's a severe last-resort for *non-herd* forced-composes; r-a is herd, and
  we *want* to track it, not wall it off.)
- **Edge-provenance carries the honesty** — a syntactically-derived edge is `provenance: syntactic` (approximate
  → `dread`-grade); an r-a-resolved edge is `provenance: resolved` (`presents`-grade). The decidability ceiling,
  on edges.
- **Opinionated contracts** — the semantic invariants types can't express (aliasing, ordering, protocol,
  panic-freedom) become *declarable*: structural contracts are inferred free; semantic ones are opt-in via a
  declaration discipline. Graceful degradation (undeclared → `dread`; declared → `presents`). This ADR ratifies
  *that* contracts are first-class node-data with provenance; the syntax + sheaf + field are incremental
  follow-ons.
- **Snapshot-vs-live keystone (systems):** maintenance computes against a *frozen* snapshot and publishes
  atomically; detection reads only fully-published versions, never torn — lifted from "outcome" into a ratified
  invariant (gives the wavefront its own termination + makes three loops virtuous by construction).
- **Two wavefronts, not one** (adv-break): freshness = *forward*; detection = *backward*; they are different
  traversals (v2's "rides the same diffusion the field uses" was wrong).
- **Digest hardening** (BLAKE3 for the identity/signing use; FNV-1a for shape-clustering only), **proc-macro-use
  edges** (the macro-expansion blind spot), **cfg-collision** named, **rebuild-cadence on lifecycle-events not a
  clock**, **persistence crash-consistency → process**, **`GlobalConsistencyObstruction` deferred** (a dangling
  self-antigen here — its detector, the sheaf lens, isn't built in this ADR).

---

## ADR-067 — The Stroma: a Sovereign Immune Lattice over Composed Resolution

**Status**: Proposed (v0.6.1, 2026-06-25). Design-pair draft; pending refinement + a re-run council on v3.

**Implements / depends-from**: ADR-066 §2 (the stroma-builder — collision-free identity, lifecycle, digest,
graph-integrity); §11 (the persisted map = the output / distributed-cognition substrate); §4 + notebook 006
(the graph as the keystone substrate for the §4 maths). **ADR-002 Amendment 3**: resolution (r-a) is
**herd-immunized substrate, composed freely via require-installed (clause 1, user-invoked, zero cascade
surface)**; the **immune lattice is the sovereign own-capability**. **Subsumes** the `LineageEdge` DAG as one
edge-kind. **Defers** to follow-ons: the sheaf-lens ADR (incl. `GlobalConsistencyObstruction`), the §4
field/maths, the contract-declaration syntax, the live-editor surface.

**Related**: notebook 006 (math-feeder verdicts), the SZZ miner, the no-self-witness invariant, *JBD*.

---

### Finding

ADR-066 deferred the substrate its soundness rests on; two threads (binding-identity §2, the §4 keystone graph)
converge on it. **antigen does not have it** — `scan/parse.rs` is an attribute-only single-file `syn` walk; the
only typed edge (`LineageEdge`, `types.rs:1244`) is *human-declared* via `#[descended_from]`, not derived; the
team's own `tools/extract_edges.py` is a Python regex name-matcher (the heuristic the council correctly named
the seed of the cost/drift/detection failure-trinity).

The deeper finding the council forced: **`syn` resolves nothing.** Call / data-flow / trait edges — the ones
that make the stroma a *detector*, not a fancier import list — require name + type resolution antigen does not
have and should not reinvent. That capability is **the herd's** (rust-analyzer). So the honest design is *not*
"sovereign because cheap"; it is **"sovereign in the immune lattice, composed in the resolution."**

---

### The thesis: a sovereign immune lattice, built over the herd's resolution

The stroma is antigen's **structural-self** — a maintained, full-AST, attributed graph of the host codebase,
the base from which topology/sheaf/manifold/field are *induced views*, over which bindings, the field, and the
map all live. What antigen **owns** (sovereign) is the **immune lattice**: the node attributes, the contracts
(structural + declared-semantic), the field, the sheaf, the opinionated discipline — antigen's cognition-
substrate, the *lattice it lives from*, with no translation layer. What antigen **composes** (the herd's) is
**resolution** — which `foo()` binds where — from the user's installed rust-analyzer, the universal Rust
substrate the whole ecosystem watches. r-a is the herd-provided *eyes*; antigen is the immune *cognition*.

It is also **opinionated**: where Rust's types can't express a contract (aliasing, ordering, protocol,
panic-freedom), antigen lets you *declare* it — graceful degradation when you don't (infer the structural, flag
the rest as `dread`), full field/sheaf power when you do. *We make the ideal work by imposing it — and degrade
honestly where it isn't imposed.*

It is the first object where antigen's **two disconnected selves** (the identity-self = the digest; the
tolerance-self = the clean corpus) become addressable **in one namespace** — the place the long-open
"unify-the-selves" question can finally land as code.

---

### Decision

**The invariant: antigen maintains a sovereign immune lattice over a full-AST attributed base graph, whose
*resolution* is composed from the user's rust-analyzer (require-installed) and whose *immune meaning* is
antigen's own; maintenance computes against a frozen snapshot and publishes atomically; the load-bearing immune
identity is never delegated.** Everything below is method.

**A. The base.**

1. **Full-AST, attributed; the *computation* is never coarsened** (detail is dialed downstream at the lens/view
   layers; the *at-rest representation* is a separate, drift-allowed concern — these are two different things v2
   conflated under one word). Nodes are items at AST grain, each carrying: a **fully-qualified, collision-free
   identity** `(qualified-path, identity-digest)` — never bare item-name, never a line number; a **local
   contract** (provides/requires) with **provenance** (structural-inferred vs declared — below); an **open
   attribute set**. **`cfg`-collision is named and handled**: `#[cfg(unix)] fn f` and `#[cfg(windows)] fn f`
   share path *and* can share a structural digest — the identity must incorporate the active `cfg`-set (or mark
   the node cfg-multiplexed), never a positional `#N` hack (which churns on reorder).

2. **The identity-digest is collision-resistant; the shape-digest is fast.** Two digests, two jobs: the
   **identity/signing digest** (the change-and-tamper signal `StromaIncrementalDrift` rests on) is **BLAKE3 /
   SHA-256** — FNV-1a's 64-bit space is engineer-collidable by macro/codegen/malicious-dep, which would let a
   tampered node pass "same digest" with mathematical credibility. **FNV-1a stays** for the non-security
   shape-clustering use (near-miss matching).

3. **Local contracts are first-class node-data with provenance** (the part this ADR ratifies; the *declaration
   syntax* + the sheaf detector are follow-ons). Provenance tiers: **structural** (inferred free from types +
   r-a's resolution — signatures, types, trait bounds — decidable, `presents`-capable) and **declared-semantic**
   (the opinionated discipline — aliasing/ordering/protocol/panic-freedom contracts the dev *declares*, since
   types can't express them). **Graceful degradation:** a node with only structural contracts gets structural
   detection (and `dread` where a semantic break is *suspected* but undeclared); a node whose semantic contracts
   are declared gets `presents`-grade semantic detection. *The sheaf's reach = structural-free + semantic-opt-in,*
   stated honestly (not "semantic detection for everyone").

**B. The induced-views architecture (honest).**

4. **Structures are induced views over the base** — topology (from conductance), sheaf (from contracts + edge
   consistency), manifold/atlas, field — derived at dial-able resolution; the base is **rich enough to derive
   them**, experimentation lives in the views. **Honest caveats (aristotle):** it is a *co-determining fixpoint*,
   not a one-way functor — the sheaf *requires* a node-contract the base carries *for* it (A3), so the base is
   made rich *for* the lenses ~as much as the lenses are derived *from* it; and **"accrete, never migrate" has a
   trapdoor** — a genuinely new lens-kind may demand a new base-attribute, which *is* a migration. Named, not
   hidden. **Richness IS detection** (god-object=high-degree, cycle=SCC, hub=betweenness=blast-radius) — *but
   only as sound as the edges* (§E): wrong-richness = wrong detection, which is why resolution is composed, not
   heuristic.

**C. The dynamics — frozen-snapshot, change-driven, two-directional.**

5. **Maintenance computes against a frozen snapshot and publishes atomically; detection always reads a
   fully-published version, never a torn one** (the systems keystone — lifted from outcome into the spine). This
   gives the maintenance wavefront its **own** termination guarantee (compute the could-have-changed closure
   once against the pre-change graph, re-evaluate that *fixed* set, no mid-wavefront re-triggering → storms
   impossible by construction over a cyclic graph where attributes are edge-triggers). The wavefront's
   convergence is *earned here*, not borrowed from the field's `diag(d)≻0` fixed-point (a different recurrence
   with no decay term).

6. **Two wavefronts, opposite directions.** **Freshness** = *forward* (what did the changed node's dependencies
   change?). **Detection** = *backward* (who *depends on* the changed node and may now be broken?) — with
   **receptor-selectivity** (a dependent lights up only if its requirement matches the change-type). They are
   **different traversals**; v2's "rides the same diffusion the field uses" conflated them.

7. **Persist-as-single-source-of-truth; conservative change-driven increment.** The change is the danger signal
   (git-diff + mtime + Cargo.lock/toml-diff + lifecycle). Conservatism fails toward noise (over-propagate) never
   silence (under-propagate = stale = false-quiet, the cardinal sin inward). The **full-rebuild witness**
   (`StromaIncrementalDrift`) is triggered on **lifecycle danger-events** (rename/split/merge) *and* on demand —
   **not a clock** (a clock-cadence is an eroding-goals trap: under cost pressure it stretches and blinds the
   very drift-witness). **Persistence is a gitignored, per-machine cache, never committed** (an invariant — not
   a repo artifact), and its **crash-consistency / atomic-write / corruption-detection contract is *process*,
   not outcome** (a partial/corrupt persisted base under-propagates → false-quiet; the *number* drifts, the
   *contract* cannot). **The macro-expansion blind spot is closed:** the digest is over pre-expansion tokens, so
   a proc-macro behavior change (via a Cargo.lock bump) changes the *expanded* code with no source edit — so
   **proc-macro-use edges** are a named edge-kind, and a lock-diff re-digests the items that use the changed
   macro.

**D. Detection + defense on the lattice.**

8. **Edge-bindings + edge-tests (architectural here; live-running deferred).** An **edge-test is an antibody on
   an edge** — the ADR-066 antibody model extends from node-bindings to edge-bindings (observed/asserted,
   proof-of-emission, the field all apply); honest by tier (structural edge-break = lint-grade `presents`;
   semantic = `dread`, routed). *That* is ratified here. The **live-running / "coding-with-antigen" / LSP-grade
   companion is current-map** (a follow-on surface), with a named **hub-node carve-out**: "only touched edges
   re-fire" is *not* constant — a hub-node change re-fires O(its-degree) ≈ workspace; live mode needs a
   budget/rate governor, and a `presents`-tier **zero-false-positive invariant** (the certain tier must never
   cry wolf — one false `presents` mutes the whole surface).

9. **The "am I OK?" change-review lens** (the product): never "OK", but **"nothing lit up downstream"** (no
   immune response — over *resolved* edges, so trustworthy) or **"check exactly these N — here, and here"** (the
   backward-dependent set, precisely located). Scopes the human's verification to the affected set — the 7th
   caller nobody tested. *Requires resolved (r-a) edges to be trustworthy — over heuristic edges it gives a
   confidently-wrong located answer, worse than none, which is why §E composes r-a.*

**E. Resolution + sovereignty.**

10. **Resolution is composed from the user's rust-analyzer (require-installed); the immune lattice is
    sovereign.** Name/type resolution is herd-immunized universal substrate → Amd3 **clause-1 compose-freely**,
    in its *user-invoked, zero-cascade-surface* form: **r-a is required-installed** (minimum versions enforced;
    a Rust dev already has it). antigen's **`antigen-stroma` crate is an *adapter*** over r-a's interface —
    version-check, wrap breaking-API changes, serve old-and-new shapes across r-a releases; handling / traversal
    / wrapping is its job. **Edge-provenance carries the resolution honesty:** `provenance: resolved` (from r-a —
    `presents`-grade), `provenance: syntactic` (from `syn` alone, e.g. import-as-written — approximate,
    `dread`-grade), `provenance: declared` (human — `#[descended_from]` and the contract discipline). **Graceful
    degradation:** no r-a / too-old → the syntactic subgraph (import + inheritance + co-change + cargo-metadata
    deps) still ships, and percolation runs on it; the *resolved* edges (call/data-flow/trait) and the detection
    that rests on them light up when r-a is present. **The sovereign claim, re-derived honestly:** we don't own
    Rust resolution (the herd does, better); we own the *immune meaning* layered on it — and that layer has no
    translation surface, which is the sovereignty that matters. *(The interface to r-a — its LSP protocol vs
    building on its analysis — is the build's call; the adapter pattern holds either way.)*

**Self-antigens:** `StromaIncrementalDrift` (incremental base diverges from a full rebuild — stale-stroma
inward; its detector, the full-rebuild, IS this ADR — minted here). `GlobalConsistencyObstruction` (a sheaf
gluing-failure) is **deferred to the sheaf-lens ADR** — minting it here would be a dangling self-antigen (its
detector isn't built in this ADR — the `defended_by(undefined-class)` pattern).

---

### Biology grounding

**Class-1:** the **thymic stroma** (the scaffold presenting self for tolerance — antigen's structural-self); the
stroma as **tissue** (the manifold the field traffics across); **the danger model** (Matzinger — respond to
change, locally); **receptor selectivity** (a signal reaches all neighbors, only matching receptors respond);
**herd immunity** (composing the ecosystem-watched substrate — r-a — is the herd protecting the universal
capability, Amd3). **Honest-invention:** the r-a adapter + provenance scheme, the frozen-snapshot/atomic-publish
rule, the digest split, the contract-declaration discipline, the incremental-update + rebuild-witness.

---

### The current map (orientation — drift-allowed)

**The keystone convergence** (one build → §2 sound bindings, §4 percolation/manifold/membrane, §11 map).
**Build-sequencing (notebook 006):** (1) the base + **syntactic** edges (cheap, no r-a) + **percolation** on
them; (2) **r-a-resolved** call/data-flow edges via the adapter; (3) the **field** representation; (4)
**quasispecies** population layer; (5) **Hill** combine (`n=1`); (6) **reaction-diffusion**; (7) the
**membrane**; alongside, the **sheaf lens** (with `GlobalConsistencyObstruction`) and the **contract-declaration
syntax**, and the **live-editor surface** ("coding with antigen"). **The two-selves-unify** insight (the stroma
as the one namespace both selves address) is the deeper *why* under "the lattice we live from" — and where the
unify-the-selves work can land. **Practitioner reality:** ship the **syntactic-subset + batch `--review`**
first (genuinely cheap, no resolver), earn live/resolved incrementally; the smallest adoptable unit is "one real
edge-kind, one real query (who-depends-on-this-I-changed), proven fresh, silent by default."

---

### Process not outcome

- **Invariant (durable):** sovereign immune lattice over a full-AST attributed base; identity fully-qualified +
  collision-free + collision-resistant-digest + cfg-aware; contracts first-class with provenance (structural +
  declared); resolution composed from require-installed r-a (immune lattice never delegated); **maintenance on a
  frozen snapshot, atomic publish, detection never reads torn**; two-directional conservative increment,
  rebuild-witnessed on lifecycle-events; persistence a gitignored per-machine cache with a crash-consistency
  contract; edge-provenance carries resolution-confidence → detection-tier.
- **Process (durable):** built from the sunk `syn` parse (syntactic edges) + composed r-a (resolved edges) +
  cargo-metadata (dep edges); the adapter wraps r-a versions; change is the danger signal; freshness=forward /
  detection=backward; the sheaf re-checks only change-touched edges.
- **Outcome (must drift — NOT decreed):** the node/edge schema + edge-kinds + conductance weights; the **field
  representation**; the **at-rest persistence format** (the *contract* is process; the *format/number* drift);
  the **r-a interface** (LSP vs library); the digest's exact canonicalization; the contract-declaration syntax;
  the live-editor surface.

---

### What this ADR does NOT do

- Does **not** build the §4 field/maths, the sheaf lens, the contract-declaration syntax, or the live editor
  surface (follow-ons; this ADR ships the base + the resolution composition + *that* contracts are first-class).
- Does **not** roll its own Rust resolver (the herd's r-a does it better) **nor** vendor/fork/granuloma r-a
  (require-installed, clause-1 herd-compose).
- Does **not** coarsen the *computation* (the *at-rest representation* is a separate, drift-allowed concern).
- Does **not** mint `GlobalConsistencyObstruction` (deferred — its detector isn't here).
- Does **not** claim semantic detection for everyone (structural-free + semantic-opt-in via declaration).

### Open seams (for refinement + a re-run council)

- **The r-a interface** (LSP protocol vs building on its analysis) — the adapter holds either way; the build
  picks, and an **author-distinct witness** confirms it yields the resolved call/data-flow edges we need
  (the Amd3 clause-3 experiment, now a pre-ratification *precondition*, not a footnote).
- **The contract-declaration discipline** — what's declarable, how, and how the sheaf consumes it (its own ADR).
- **Persistence crash-consistency mechanism** + the gitignored-cache lifecycle (first-build cost, CI runners).
- **Hub-node budget** + the `presents`-tier zero-false-positive invariant for the live surface (its own ADR).

### Glossary (delta)

- **stroma** — antigen's maintained full-AST attributed graph; the sovereign immune lattice; built over the
  herd's resolution.
- **immune lattice vs composed resolution** — antigen owns the *meaning* (attributes/contracts/field/sheaf);
  the herd's r-a supplies the *resolution* (which `foo()` binds where).
- **contract** — a node's provides/requires, with **provenance** (structural-inferred vs declared-semantic).
- **edge-provenance** — `resolved` (r-a, `presents`-grade) / `syntactic` (`syn`, `dread`-grade) / `declared`
  (human) — carries the honesty onto every edge.
- **frozen-snapshot / atomic-publish** — maintenance computes against a frozen graph and publishes whole;
  detection never reads a torn version.
