# Capture — Adversarial Self-Attack on Discipline-Witnesses v2

> **Date**: 2026-05-18
> **Author**: single-instance Claude, immediately after v2 draft landed
> **Relation to v2**: this capture attacks v2 to find what's brittle
> *before* the team adversarial pass. Survivors are listed as
> "refinements for team-time folding into v3" at the end. Per memory
> note `feedback_adversarial_seed_from_naturalist.md`: substantive
> thinking first; adversarial seeded with that material.
> **Status**: append-only capture

> **What this is for**: when the team adversarial agent runs against v2,
> they should attack at the frontier — not rediscover known weaknesses.
> This capture is the seed material. Refinements that survive should be
> folded into v3 by the team after their pass; the team adversarial agent
> can attack v3 with confidence the easy attacks were already considered.

---

## Posture

Attacking my own claims with the same posture I'd want a hostile
reviewer to use: no benefit-of-the-doubt, every "obviously" is suspect,
every "by construction" gets pushed on, every "edge case" gets explored
rather than waved off.

Eight attacks below. For each: the attack, my response, the verdict
(core-survives / refinement-needed / attack-lands). Core survives in
all eight; six produce refinements; two are absorbed without change.

---

## Attack 1 — "Audit-as-verifier" is circular reasoning

**Attack**: v2 argues substrate-witnesses can reach Execution because
"the audit IS the verifier" for substrate-predicates. But the audit is
also what reports tiers. By that logic, *any* work the audit does could
be promoted to Execution by appealing to "the audit is the verifier".
What stops Reachability work (just identifier-resolution) from being
relabeled "audit completed its verification work" and tier-bumped?

**Response**: The distinguishing factor isn't "audit did some work" but
"audit completed the verification work the *witness* encodes." Tier
names *the fraction of the witness's encoded verification work the
audit actually completed*:

- **Reachability** = identifier-resolution-completed + nothing-else
- **Execution** = full-witness-encoded-verification-work-completed

For a test-witness, the encoded verification work is "run this test."
v0.1 audit doesn't do that work; it does the strict subset of
identifier-resolution; reports Reachability. For a substrate-witness,
the encoded verification work IS predicate-evaluation-against-substrate.
The audit does that work in full; reports Execution.

The line: does the audit complete the verification work *the witness
encodes*, or only a strict subset? Substrate-witnesses are the case
where the audit's work IS the witness's encoded work — they aren't
distinct.

**Verdict**: Core survives. Refinement: v3 should explicitly name
"tier = fraction-of-witness-encoded-work-completed" rather than just
"tier = depth-of-verification-work-done." Sharper framing.

---

## Attack 2 — Audit reading a file is NOT verification; it's just reading

**Attack**: The strongest version of the counter-argument. The audit
reading a sidecar doesn't *do* the verification work — humans did the
verification work when they signed, completed oracles, ratified the
doc. The audit just *reads* their attestation. By analogy, reading a
test result file isn't running the test. Substrate-witnesses should
cap at Reachability because all the audit did was read.

**Response**: The audit reading a sidecar IS more than just reading —
it parses JSON, validates schema, checks signer-name presence against
required-set, checks fingerprint-currency (SHA comparison), checks
freshness (date arithmetic), composes leaf results via combinators.
That's evaluating a predicate. The predicate IS what the witness
encodes; the audit evaluates it.

But the deeper point in the attack: the predicate is over substrate
that's *human-attested*. The audit verifies "alice's name appears in
signers"; it cannot verify "alice actually reviewed the code and her
judgment is sound." The latter is what the discipline-antigen *wants*.

**Hmm — does that mean substrate-witnesses should be lower-tier?**

No, because every other tier has the same structure. Phantom-type
witnesses at FormalProof don't actually verify truth either — per
`witness-tiers.md`:

> "behavioral verification that the constructor IS sealed is the
> developer's responsibility — the audit recognizes the shape but
> cannot prove the constructor's soundness"

Even FormalProof tier verifies structure-of-evidence, not truth. The
pattern is consistent: **every tier verifies structure of evidence;
truth verification is always developer's responsibility**.

By that pattern, substrate-witnesses at Execution-when-predicate-passes-
and-current is parallel to other tiers: the audit verifies the
structure of evidence (predicate passes against current substrate);
truth of "alice's judgment is sound" is developer's responsibility.
Same pattern as phantom-type at FormalProof.

**Verdict**: Core survives. Refinement: v3 should explicitly include
the "every tier verifies structure-of-evidence, not truth" framing in
the tier-honesty section. This is the deepest argument for why
substrate-witnesses at Execution is consistent with the existing tier
discipline.

---

## Attack 3 — Fingerprint-currency check is gameable

**Attack**: Alice signs against fingerprint X. Someone changes the
code, regenerates fingerprint Y. They run a CLI command that updates
`signed_against_fingerprint` to Y *for alice's entry*. The audit
reports current; alice never re-reviewed. The discipline is gone.

**Response**: The CLI sketch in v2 doesn't include any
`fingerprint --update` command for exactly this reason — but I never
made the discipline EXPLICIT in v2. Filling the gap:

`cargo antigen attest sign` is the ONLY command that writes
`signed_against_fingerprint`, and it does so:
- ONLY for the signer doing the signing (identity from `git config
  user.name`/`user.email`)
- With timestamp = now
- With fingerprint = current
- Refuses if `git config user.name` not set (so anonymous bypass
  blocked)

Other signers' entries stay pinned to whatever they signed against.
The predicate evaluates against the current state of all signer entries.

But there's still a gap — what if the discipline requires ALL signers
to be against current fingerprint? The `signers` predicate needs an
explicit parameter:

```rust
signers(required = ["alice", "bob"], against = "current")
// All signers must have signed_against_fingerprint == current
```

Default: `against = "current"`. `against = "any"` is opt-in for
disciplines where stale signatures are explicitly tolerated.

Combined refinement: tier-honesty for staleness is also tightened.
The v2 mapping had:
- predicate passes + ≥1 signer stale → Reachability
- predicate passes + all signers current → Execution

That's the right structure for `against = "current"`. For
`against = "any"`, predicate-passes-regardless-of-staleness → Execution.
Tier-honesty preserved because the predicate explicitly opts out of
currency-checking.

**Verdict**: Refinement needed. v3:
- (a) `signers(against = "current"|"any")` parameter; default current
- (b) CLI sketch makes explicit: only `attest sign` modifies
  `signed_against_fingerprint`, only for the signer's own entry, only
  via git-config identity
- (c) No `attest fingerprint --update` or similar bypass command

---

## Attack 4 — Git-trust is trivially bypassable

**Attack**: Sidecar `signers[].name` is just a string. Git author
identity is `name <email>`. Alice in sidecar might be "Alice Smith
<alice@example.com>" in git config — or any other identity if alice
changes her config. The audit checks string equality on names;
trivially bypassable by signing as someone else.

**Response**: This is real. v0.1 git-trust IS trivially bypassable.
The discipline doesn't pretend otherwise — v2 explicitly says
`Signer.signature` slot is reserved per ADR-007 for crypto-strong
verification in v0.4+.

But v2 didn't make the *git-trust caveat* explicit in the audit
output. Tier-honesty requires the consumer to know what strength of
evidence they have. v0.1 substrate-witnesses report:
- Tier: Execution (predicate passed)
- Hint: `discipline-substrate-validated-and-current`

This doesn't surface the git-trust-only caveat. Consumers reading
"Execution" might think they have stronger evidence than they do.

Two options:
- (a) Append qualifier to the hint:
  `discipline-substrate-validated-and-current-git-trust-only`
- (b) Add a separate `signature_strength` field on the audit entry:
  `"git-trust"` | `"crypto-signed"`

I lean (b). The hint name stays stable across versions; the strength
qualifier rides on a parallel field. When crypto-signing lands in
v0.4+, the field becomes more useful; the hint name doesn't change.

**Verdict**: Refinement needed. v3 adds `signature_strength` field to
audit output for substrate-witness entries. v0.1 reports
`"git-trust"`; v0.4+ reports `"crypto-signed"` for entries with
populated `Signer.signature`.

---

## Attack 5 — Empty-list edge cases in combinators

**Attack**: `not(any_of([]))` = `not(false)` = `true`. Trivially passes.
Does the audit accept this as a valid passing witness? Should it?
Same for `all_of([])` = `true` vacuously. And other vacuous compositions.

**Response**: This is a schema-vs-semantics gap. The audit's predicate
evaluator would correctly compute these as `true` per standard
boolean logic. The question is whether the *schema* should reject them
before evaluation reaches that point.

Yes — the schema should reject zero-leaf compositions at the
producer-consumer trust boundary (per ADR-005 sub-clause F at parse
time). Specifically:
- `all_of([])`: schema-reject ("compound witness with no leaves is
  meaningless; use a single leaf or omit the predicate")
- `any_of([])`: schema-reject (same reason; vacuously false anyway)
- `not(<expr>)` where `<expr>` evaluates to empty composition:
  schema-reject (composition rejected first)
- `not(<single_leaf>)` is fine

This is a schema-lock concern, not a predicate-language ceiling concern.
The combinator grammar stays closed; the schema enforces non-vacuousness.

**Verdict**: Refinement needed. v3 specifies: schema validation rejects
zero-leaf compositions; predicate evaluator never reaches them.

---

## Attack 6 — Cross-crate descended_from with substrate-witnesses

**Attack**: Crate A declares discipline-antigen X. Crate B uses X via
`#[descended_from = "A::X"]` on a function. Where does the `.attest/`
folder live? Crate B (code-locality) or crate A (declaration-locality)?
If in crate B, then crate B asserts compliance — but crate A's
required_signers might be ["bob", "carol"], who don't work in crate B.
Alice signs in crate B; what does the audit make of that?

**Response**: v2 doesn't address this. Real gap. Three options:

**Option (a) Per-consumer ratification**: sidecar lives in consuming
crate (crate B). Predicate evaluates against consuming crate's
substrate. The required_signers list might be different per consumer
(alice for crate B's adoption; bob+carol for crate A's adoption).
Each consumer ratifies in their own substrate; discipline is
per-consumption-site.

**Option (b) Declaration-bound ratification**: sidecar lives in
declaring crate (crate A). Predicate evaluates against declaring
crate's substrate. All consumers inherit the answer.

**Option (c) Hybrid**: declaring crate's `.attest/` provides default;
consuming crate can override via local `.attest/`.

Option (a) is the most honest. Discipline doesn't transfer — alice's
review in crate B doesn't satisfy crate A's "math-researcher must
ratify" because crate A's discipline is about crate A's
math-researchers. Each consumption site requires its own ratification.
This respects code-locality and respects that discipline judgment
isn't transferable.

Option (b) is what people will WANT (inherit attestation from upstream)
but it's wrong — it would let crate B claim compliance with crate A's
discipline without crate B's signers actually doing the discipline work.

Option (c) is appealing as a v0.2 extension (consuming crate inherits
DEFAULT unless overridden) but introduces complexity that v0.1 doesn't
need to resolve.

Position for v3: **option (a) is the answer for v0.1**. Cross-crate
discipline-antigens require per-consumer ratification. The consuming
crate's substrate is authoritative for THEIR adoption of the
discipline. Document this explicitly — adopters will expect (b) and
need to be told why (a).

**Biology rhyme**: this is the same as **immune competence vs
inherited immunity**. Maternal IgG transfers from mother to infant
(inherited immunity); but the infant's own adaptive immunity must
develop independently — they don't permanently inherit the mother's
B-cell repertoire. The maternal antibody provides short-term passive
protection; long-term, the infant generates their own. Inheritance of
attestation is like passive protection — useful as a starting state
maybe (v0.2 default mechanism), never a permanent substitute for
the consuming crate's own discipline.

**Verdict**: Refinement needed. v3 documents per-consumer ratification
as v0.1 position; flags v0.2 default-inheritance as future
extension; uses passive-vs-active immunity rhyme.

---

## Attack 7 — Macro-expanded code with discipline antigens

**Attack**: A discipline antigen on a function whose impl is generated
by a macro. The macro EXPANSION site is generated; the macro
INVOCATION site is authorable. Where does the `.attest/` folder live?
v2 says "generated `.rs` files are out of scope for `.attest/` sidecars
in v0.1" but macro-expanded code is a gray case — the user wrote the
invocation; the compiler generates the impl.

**Response**: v2's "input layer" framing applies here. The macro
invocation site IS the input layer for macro-expanded code. The
discipline antigen and `.attest/` folder live at the file containing
the macro invocation, not the file containing the expansion.

This is consistent with v2's tonic example (discipline antigens for
gRPC error handling live next to `.proto`, not next to the generated
`service.rs` in OUT_DIR). For macros, the macro INVOCATION site is the
`.proto` analog.

What if a discipline antigen is emitted BY the macro itself? Then the
macro is authoring discipline-antigen declarations on behalf of the
user. The `.attest/` folder lives at the file containing the macro
invocation; the user (or the macro itself, scaffolding-style) creates
the sidecar there.

Edge case: a macro that emits MULTIPLE impls, each presenting different
antigens. The `.attest/` folder at the macro invocation site needs
sidecars for each emitted antigen. CLI scaffolding (`attest scaffold`)
should support this: `cargo antigen attest scaffold --file
src/foo.rs --line 42` discovers all macro-emitted antigens at that
invocation and creates sidecars for each.

**Verdict**: Refinement needed. v3 clarifies: macro-invocation site
IS the input layer for macro-expanded code; `.attest/` folder lives
there; CLI scaffolding handles multi-emitting macros.

---

## Attack 8 — Workspace-vs-package boundary for `.attest/` placement

**Attack**: v2 assumes per-file `.attest/` subfolders. What about
workspace-wide disciplines that apply across multiple packages? Or
package-wide disciplines that apply to all files in a package?
Per-file granularity might be the wrong default for some discipline
types.

**Response**: v2's three-layer granularity (file → folder, antigen →
sidecar, item → entry) is the *finest* granularity. Coarser-grained
disciplines can be expressed by:

- **Package-wide discipline**: a `.attest/` folder at the package root
  (next to `Cargo.toml`) with sidecars for package-level antigens (e.g.,
  applied to the package's `lib.rs` `#[immune]` declaration that
  presents at the crate level).
- **Workspace-wide discipline**: same pattern at workspace root.

The granularity follows the antigen presentation site. If an antigen
is presented per-function, the sidecar lives at the function's file.
If presented at the crate root (a top-level `#[immune]` on the crate),
the sidecar lives at the package root. If presented via workspace
attribute (does this exist? probably not), workspace root.

For v0.1, document per-file as the default; coarser-grained as
"presented at the coarser scope, sidecar at the coarser substrate."
No new CLI machinery; the existing `attest scaffold --file <path>`
takes whatever path the antigen is presented at.

**Verdict**: Core survives unchanged — the granularity pattern v2
established naturally supports coarser-grained disciplines because
sidecar-location follows antigen-presentation-location. v3 should add
a brief note clarifying this.

---

## Attacks considered briefly but not fully developed

These are real but don't change v2's core; flagging for team
adversarial pass to pick up:

**Attack-A**: `cargo antigen attest gc` false-positive case. What
happens if an item is temporarily removed during refactor (e.g., a
function is being moved between modules; mid-refactor, the function
doesn't present the antigen anywhere)? `gc` would delete the sidecar;
post-refactor, the developer has to re-ratify. CLI behavior spec needs
`--dry-run` default + explicit `--commit` flag.

**Attack-B**: Schema-version migration with mixed-state workspaces.
v1 → v2 migration; some sidecars migrated, some not (PR in flight).
What does the audit do in mixed state? Position: audit accepts both v1
and v2 during migration window; CLI `attest migrate` is per-sidecar
idempotent; explicit migration window declared in workspace config.

**Attack-C**: Role collisions in signer list. Alice has roles
["math-researcher", "code-reviewer"]; Bob has ["math-researcher"].
Required: 2 math-researchers. Does Alice cover one slot? Schema needs
to specify: roles are a set (not a list); required count is per
role-membership; one signer satisfies at most one role-slot per
required entry.

**Attack-D**: `.attest/` naming collision risk. Does any existing
Rust project convention use `.attest/`? Probably not (it's a
hidden-directory pattern; project-owned namespace). But cargo's
`target/` and `.cargo/` are conventions; we should verify `.attest/`
is unclaimed. Quick check.

**Attack-E**: `signed_trailer` parsing of git log. Git commit
messages are unbounded text. Can a malicious commit-message inject
content that bypasses the trailer-check? Position: use git's
canonical trailer parsing (`git interpret-trailers`), not naive
string-search. Trailer-grammar is well-defined.

---

## Refinements that survive — to fold into v3

Listed in priority order (highest-impact first):

### R-A1 — Tier framing: "fraction of witness-encoded work"
Replace "tier = depth of verification work done" with **"tier =
fraction of the witness's encoded verification work the audit
actually completed."** This sharpens the argument for why
substrate-witnesses can reach Execution. (From Attack 1.)

### R-A2 — Tier framing: "every tier verifies structure-of-evidence, not truth"
Add explicit framing: **all tiers (Reachability, Execution,
FormalProof) verify structure-of-evidence; none verifies truth.
Truth-verification is always developer's responsibility.** This is the
deepest tier-honesty argument and applies uniformly across witness
families. (From Attack 2.)

### R-A3 — `signers` predicate `against` parameter
Add to predicate language: **`signers(required = [...], against =
"current" | "any")`**, default `"current"`. Audit reports
`discipline-substrate-stale` hint when any signer is pinned to stale
fingerprint and `against = "current"` was required. (From Attack 3.)

### R-A4 — CLI: only `attest sign` writes `signed_against_fingerprint`
Make explicit in CLI sketch: **`attest sign` is the only command that
writes `signed_against_fingerprint`; writes only for the signer's own
entry; identity from git config; refuses if git config unset. No
bypass command (no `fingerprint --update` or equivalent).** (From
Attack 3.)

### R-A5 — `signature_strength` field on audit output
Add to audit output for substrate-witness entries: **`signature_strength:
"git-trust" | "crypto-signed"`** field on each signer's audit entry.
v0.1 always reports `"git-trust"`; v0.4+ reports `"crypto-signed"` for
entries with populated `Signer.signature`. Tier-honesty extends to the
strength of attestation, not just the existence of attestation. (From
Attack 4.)

### R-A6 — Schema rejects zero-leaf compositions
Schema validation rejects: **`all_of([])`, `any_of([])`, and `not(<expr>)`
where the inner expression is a vacuous composition.** Predicate
evaluator never reaches these states. Trust-boundary check at
parse-time, not evaluation-time. (From Attack 5.)

### R-A7 — Per-consumer ratification for cross-crate descended_from
**Cross-crate discipline-antigens require per-consumer ratification.**
Sidecar lives in consuming crate; predicate evaluates against
consuming crate's substrate. Discipline judgment doesn't transfer.
v0.2 default-inheritance via local override is a future extension.
Biology rhyme: passive immunity (maternal IgG) vs active immune
competence — passive transfer is short-term protection, not
substitute for own immune development. (From Attack 6.)

### R-A8 — Macro-expanded code: invocation site is input layer
Clarify: **macro-invocation site IS the input layer for
macro-expanded code; `.attest/` folder lives at the file containing
the macro invocation, not at the expansion site.** CLI `attest
scaffold --file <path> --line <line>` handles multi-emitting macros
by discovering all macro-emitted antigens at that invocation. (From
Attack 7.)

### R-A9 — Granularity pattern naturally supports coarser scopes
Add brief note: **sidecar location follows antigen presentation
location.** Per-function antigens → file-adjacent `.attest/`.
Crate-level antigens → package-root `.attest/`. Workspace-level
antigens → workspace-root `.attest/`. No new CLI machinery; existing
`attest scaffold --file <path>` adapts. (From Attack 8.)

---

## What doesn't change

The core shape of v2 survives all eight attacks:

- Substrate-witnesses CAN reach Execution tier (refined: when
  predicate-completes-encoded-work AND substrate is current per
  predicate's `against` parameter)
- FormalProof unreachable from substrate-witnesses
- Three coupled pieces (predicate language + Ratification schema + CLI)
- Per-antigen-per-file granularity with `.attest/` subfolder
- ONE ADR position
- Closed combinator grammar with sealed leaf primitives
- Recognition-not-design ratio is healthy
- Code-locality (germinal-center pattern) over doc-locality
  (central-registry pattern)

The attacks landed refinements (sharpenings, gap-fillings,
edge-case-specifications), not invalidations. This is what
adversarial pre-pass should produce — the core gets harder to attack
because the easy attacks were already considered.

---

## What the team adversarial pass should attack

With these refinements absorbed into v3, the team's adversarial agent
should attack at the new frontier:

1. **The "fraction of witness-encoded work" framing**: does it
   actually hold uniformly, or are there witness families where it
   breaks down?

2. **The `against` parameter**: are there compound staleness states
   not covered by `"current" | "any"`? E.g., "all critical signers
   must be current; non-critical can be stale" — is that a real
   discipline need? Does it require a third value?

3. **The `signature_strength` field**: does this leak abstraction —
   making the audit output strength-aware when consumers should
   probably just consume tier + hint? Or is it actually load-bearing?

4. **Cross-crate ratification, deeper**: per-consumer is the v0.1
   position, but the v0.2 default-inheritance mechanism is
   under-specified. What does that look like? Does it have a
   tier-honesty escape hatch?

5. **Predicate-language ceiling, deeper**: closed combinator grammar
   is the position, but the LEAF SET is open (we ship 5 leaves; more
   can be added via amendments or witness-provider crates). What
   prevents leaf-set bloat from being its own Turing-tarpit at a
   different layer?

6. **CLI workflow gaps**: `attest sign` is the only write-path; does
   that handle PR workflows where alice reviews bob's code but only
   bob can run `attest sign` locally? Does the CLI need a
   `attest sign --on-behalf-of` flag, and does that re-introduce the
   bypass risk Attack 3 closed?

These are the frontier questions. The eight attacks above are settled
or refined; the frontier is open.
