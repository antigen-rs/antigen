# Encounters — proposal for a pre-V0+1 vocabulary tier

**Status**: PROPOSAL (not ratified). Awaiting aristotle Phase 1-8 +
team deconstruction + ratification per `process.md`.

**Proposed by**: Tekgy + team-lead in conversation, antigen-A3 launch
session 2026-05-09. Surfaced from substrate-currency reframe pressure.

---

## The shape of the proposal in one sentence

> Add a third tier *below* the existing postures.md V0+1 candidates
> for first-encounter formal capture, so subsequent encounters can be
> recognized as the second/third/Nth instead of treated as fresh
> first-recognitions each time. Encounters span patterns, vocabulary,
> structural moves, and frames — anything noticed that wants
> future-recognition. Encounters carry tracing (graph relationships
> between encounters), not just entry data.

---

## The bootstrap problem (why this proposal exists)

`postures.md` currently has two tiers:

1. **Ratified postures** (sections 1–7) — operationally normative,
   threshold cleared.
2. **V0+1 candidates** (the catalog at the end of `postures.md`) —
   threshold not yet cleared but on-track and being watched.

Below V0+1 there is no formal home. A pattern recognized for the *first
time* lands in: a campsite finding, a typed feedback file, a garden
entry, scattered ADR comments, conversation-context, or nowhere at all.

The structural problem this creates:

- The promotion-to-V0+1-candidate threshold is "three independent
  instances." Counting requires *recognizing the second and third
  instances as instances of the same pattern*, not as three independent
  first-recognitions.
- That recognition relies on the team finding the prior documentation
  of the pattern. If the prior documentation is in a campsite, a typed
  feedback file, or a garden entry — substrate that's not indexed as
  "watched patterns" — the second instance is likely treated AS a
  first-recognition.
- The result: legitimate patterns fail to accumulate instances across
  sessions because no formal tier exists to remember "this was already
  noticed once."

This is structurally substrate-currency at the recognition tier:
substrate-as-of-first-encounter-time ≠ substrate-as-of-second-encounter-
time, because the first encounter wasn't registered formally enough
to be matched against. Antigen-the-discipline applied to antigen-the-
project's own vocabulary is exactly this: capture failure-classes (or
patterns, or postures, or recognitions) at first-encounter so future
encounters can be recognized as recurrences, not novel.

---

## Tekgy's framing (2026-05-09)

> "i'm just proposing a separate formalism system like postures but
> just not yet promoted to a posture but formalized on a first instance"
>
> "encounter also implies 1st time we still encountered it, note it
> somehow so we can recognize a 2nd encounter"

The framing in compact form: **the body's first encounter with a
pathogen produces a generic innate response and a memory trace, before
adaptive immunity has produced specific antibodies. The trace is what
lets the body recognize the second encounter as a re-encounter rather
than a novel one.**

`encounters` (proposed name) captures this: the discipline of noting
first-encounters formally enough that second encounters are recognizable
as such.

Other candidate names considered and explicitly held for future use:

- `sensitizations` — real immunology term; tracks "developed memory
  but not specific antibody yet." May fit a different vocabulary
  pattern when one surfaces.
- `exposures` — medical term; "exposed to" framing.
- `innate-recognitions` — biology cognate; verbose but precise.
- `first-contacts` — narrative.

`encounters` was chosen because it's short, the verb form
("encountered") is exactly what the tier captures, and it pluralizes
naturally for catalog use.

---

## The biology cognate (informational)

The innate immune system has pattern-recognition receptors (PRRs) that
detect pathogen-associated molecular patterns (PAMPs) before adaptive
immunity has produced specific antibodies. The innate response is
*generic* — it doesn't yet know what the pathogen is — but it *registers
the encounter* and provides scaffolding for adaptive response to
develop specific recognition.

`encounters`-tier maps onto this: a pattern is registered formally
without being committed to a specific shape (which would be
posture-class) or a watched-candidate identity (which would be V0+1).
The registration alone enables future encounters to be matched against.

This is recognition-not-design (ADR-006) at the meta tier: the
encounter discipline doesn't *design* what the pattern eventually
becomes; it preserves enough structure that the team can *recognize*
when the pattern recurs.

---

## What can be encountered (expanded scope, 2026-05-09)

Initial draft framed encounters narrowly: first-recognition of
*patterns* that might eventually become postures. Tekgy expanded the
scope mid-drafting: the discipline applies to anything noticed that
wants future-recognition.

Concrete shapes encounters can take:

- **Pattern-encounters** — first-recognition of a pattern that might
  eventually warrant V0+1 candidate status or posture-class. (The
  original proposal scope.)
- **Vocabulary-encounters** — when the team generates candidate names
  (e.g., during the encounters-naming conversation: `sensitizations`,
  `exposures`, `innate-recognitions`, `first-contacts`) and holds
  some for future-use, the holding IS an encounter. Future-readers
  encountering a *pattern* that fits one of these names recognize
  the candidate without having to re-derive it. Substrate-grounded
  example: the immunology-vocabulary-pool generated tonight should
  be registered so future vocabulary work can match against it.
- **Structural-move-encounters** — when a Phase 1-8 produces a move
  shape that might generalize (e.g., "depth-shift recursion at the
  verifier-self-correction tier"), the move-shape is an encounter
  even before it has a name. Subsequent recurrences recognize each
  other as instances.
- **Frame-encounters** — when a framing landed in conversation that
  felt load-bearing but didn't yet have substrate (e.g., Tekgy's
  "lingering felt-shape that more axes will surface" from the
  substrate-currency reframe), the frame is an encounter. The
  framing's *next* substrate-grounded instance recognizes the prior
  frame.

The list is open-ended. The discipline is not "what counts as an
encounter" but "is this thing worth noting so a future encounter
recognizes it?" If yes, register.

---

## Encounter tracing — encounters as graph, not list

An encounter is rarely isolated. Encounters connect to each other:

- An encounter of pattern X surfaces *during* an encounter of frame Y.
- Vocabulary-encounter Z is *held for* hypothetical future
  pattern-encounter W.
- Two encounters at different moments turn out to be the same
  underlying recognition (recognized retroactively).
- An encounter of move-shape M may *trace through* multiple
  pattern-encounters that share the move.

This is structurally identical to the contact-graph framework already
in antigen substrate (`docs/contact-graph-and-recognition-tiers.md`):
3-tier cross-reactivity × 7-mode transmission = 21-cell matrix. The
contact-graph captures which antigen-recognitions connect to which
others.

Encounters get the same shape: each encounter entry carries (a) entry
data — what was encountered, when, where, by whom; (b) trace data —
which other encounters it connects to, by what relationship type.

Relationship type taxonomy is itself an open question for aristotle
Phase 1-8 (Q8 below). Candidate types from biology cognate:

- `surfaces-during` — encounter A appeared in the context of encounter B
- `held-for` — encounter A is candidate vocabulary or scaffold for
  hypothetical future encounter B
- `recognized-as` — encounter A retroactively recognized as the same
  underlying thing as encounter B
- `traces-through` — encounter A is one of several instances of move-shape B

The graph property enables recognition that flat-list lookups can't
support: "show me encounters that connect to anything in the
substrate-currency cluster" produces the relevant candidate vocabulary
and pattern-encounters at once.

This is also why the encounters tier formalism is structurally
generative — it doesn't just store first-encounters, it stores the
*relationships* between them. The relationships themselves can become
posture-class candidates over time.

---

## Three-tier picture (proposed)

Following the proposal:

1. **Postures** (`postures.md` §1–§7+) — operationally normative;
   threshold cleared; ADR-citations grounded.
2. **V0+1 candidates** (`postures.md` end-section) — threshold not
   met but accumulating; watched for promotion to posture.
3. **Encounters** (new tier — `encounters.md`? section in postures?
   sibling doc? — *open question*) — first-encounter formal capture;
   not yet candidates but registered so subsequent encounters
   recognize each other.

Promotion criteria *between* tiers (open questions):

- **Encounter → V0+1 candidate**: threshold? Tentatively: second
  independent instance of the same encounter pattern. Drives the
  encounter into the watched-candidate state.
- **V0+1 candidate → posture**: existing trigger (typically three
  independent temporally-distinct instances + shape stability +
  ADR-006 satisfaction).

These thresholds are *proposed*; aristotle Phase 1-8 should test them.

---

## Structural placement (open question)

Three options for where encounters live:

A. **New section in `postures.md`** at the end — three-tier document,
   one file. Ratified / candidates / encounters all in one place.
   Pros: single source of truth; cross-references easy. Cons: file
   grows; visual rhythm of catalog stresses with very-different
   tier-shapes.

B. **Sibling document** (`encounters.md` or `field-notes.md` etc.)
   — separate tier-doc, cross-referenced. Pros: cleaner separation;
   each doc has consistent shape; encounters can have their own
   metadata conventions. Cons: two files to maintain; cross-references
   risk drift.

C. **Section in `glossary.md`** — encounters as glossary annotations,
   later promoted to postures.md V0+1. Pros: glossary is already the
   vocabulary anchor; minimal new structure. Cons: glossary is for
   defined terms, not patterns-being-watched; conflates two functions.

Aristotle Phase 1-8 to recommend.

---

## Bootstrap: the first formal entries

The proposal has a self-referential property — the very act of proposing
a first-encounter formal capture system is itself a first encounter
that has nowhere formal to live until the system exists. The proposal
solves its own bootstrap by being the first entry.

Proposed first formal entries on ratification (in priority order):

### Entry 1 — *encounters-as-tier* (self-referential)

The proposal of the encounters tier is itself the first encounter of
"vocabulary-tier formalization at the pre-candidate level." Subsequent
encounters of similar formalization moves (for *other* vocabulary
patterns we haven't yet identified) will recognize this entry as the
prior instance.

### Entry 2 — *evolution-as-inoculation*

Surfaced 2026-05-09 during the substrate-currency reframe (see
`postures.md` V0+1 substrate-currency entry, the "Evolution of framing"
section). The pattern: when a vocabulary candidate's framing evolves,
preserve the past framing explicitly with the observations that forced
the reframe — the preservation itself is structural memory of the
failure-class "premature vocabulary closure on a still-extending
concept." Future readers develop antibodies against that closure-mode
by encountering the past framings.

This is *one* instance. Entry-as-encounter status: registered. If a
second instance surfaces (another vocabulary candidate evolves and the
team preserves the evolution explicitly), promote to V0+1 candidate.

### Entry 3 — *two-axis substrate-currency*

The substrate-currency taxonomy reframe (mechanism × substrate-domain;
two axes; project-substrate vs harness-substrate; persistence/registry
as fourth mechanism) is itself a first-encounter of "what we thought
was a single-axis taxonomy was actually a two-axis matrix." If similar
multi-axis discoveries surface for other vocabulary candidates,
promote.

### Entry 4 — *immunology-vocabulary-pool held for future patterns*

During the encounters-naming conversation (2026-05-09), the team
generated candidate names: `sensitizations`, `exposures`,
`innate-recognitions`, `first-contacts`, `encounters`. Tekgy ratified
`encounters` for this tier and explicitly held the others "for OTHER
things we are building."

Vocabulary-encounter status: registered. When a future pattern
surfaces that fits one of the held names (e.g., a discipline at the
team-coordination tier where prior contact creates partial readiness
matches `sensitization`; a discipline tracking specific exposures to
named substrate matches `exposures`), match against this entry.

Trace: held-for hypothetical future pattern-encounters; surfaces-during
the encounters-as-tier proposal-encounter (Entry 1).

### Future entries

The team adds entries when first-recognition patterns surface that
warrant formal note. Naturalist's gardening discipline naturally
produces these. Scout's exploration mode naturally produces these.
Aristotle's Phase 1-8 deconstructions occasionally surface them.
Vocabulary-encounters surface naturally during naming conversations.
Frame-encounters surface during conversation when a framing lands but
substrate is not yet present.

---

## Relationship to existing postures and substrate

- **Recognition-not-design (ADR-006)**: encounters operationalize
  ADR-006 *upstream* of V0+1 candidates. The discipline catches
  first-recognition before three independent instances accumulate, so
  the count from "1 recognized" to "3 ratifiable" can actually reach
  3 instead of getting reset by lost-prior-recognition.

- **Implicit-to-explicit elevation (ADR-004 / postures §5)**:
  encounters elevate the implicit pattern of "we noticed this somewhere
  but it didn't get tracked formally" to the explicit pattern of "we
  noticed this; it's registered; future-readers can match against it."

- **Sub-clause F at trust boundaries (postures §1)**: each encounter
  entry is a recognition-claim; the validation check is "did the pattern
  actually surface in real substrate, or is this speculative?" The
  encounter tier requires substrate-grounded first-recognition (a
  campsite finding, a typed feedback file, a real moment in the
  team's work) — not philosophical anticipation of patterns that
  *might* arise.

- **Substrate-currency itself**: encounters are an explicit response
  to substrate-currency failures at the recognition tier. The
  recognition tier IS one of the substrate-domains where currency
  drift happens; encounters are the operational fix.

---

## Open questions for aristotle Phase 1-8

Q1. **Structural placement**: section in `postures.md`, sibling
    document `encounters.md`, glossary annotations, or something else?

Q2. **Promotion criteria precision**:
    - First-encounter to V0+1 candidate: is "second independent
      temporally-distinct instance" the right threshold, or different?
    - Should encounters require ADR-006-style instance-grounding even
      at first-encounter (i.e., is *one* substrate-grounded instance
      required, vs allowing speculative encounter registrations)?

Q3. **Format / shape of an encounter entry**: what fields are required?
    Suggested minimum: name; date of first encounter; substrate location
    of first encounter (campsite, file, conversation reference);
    one-paragraph description; what would count as a second encounter.

Q4. **Bootstrap concern**: does the self-referential first-entry
    pattern (encounters-as-tier IS the first encounter of itself)
    introduce a circularity that compromises the discipline, or is it
    sound (analogous to how postures.md V0 was authored using the
    postures it documents)?

Q5. **Maintenance discipline**: who has authority to add an encounter
    entry? Suggested: any team-member, with the standard JBD
    discipline (substrate-grounded; not speculative). Naturalist
    likely produces the most given gardening + observational role.

Q6. **Pruning / archiving**: what happens to encounter entries that
    accumulate dust (no second encounter for N sessions)? Suggested:
    they stay; their dust IS the data (the pattern didn't recur, which
    is itself evidence about substrate shape).

Q7. **Phase 8 self-application**: does the proposal foreclose better
    alternatives we haven't surfaced? Specifically: is there a
    mechanism for first-recognition formalization that ISN'T a tier
    parallel to postures? E.g., automatic indexing of all campsite
    findings into a pattern-registry; tagging discipline; etc.

Q8. **Encounter tracing — relationship type taxonomy**: candidate
    types proposed (`surfaces-during`, `held-for`, `recognized-as`,
    `traces-through`); is this set complete? Are there relationships
    that the contact-graph framework already in antigen substrate
    suggests we should reuse? The 21-cell matrix in
    `docs/contact-graph-and-recognition-tiers.md` may have direct
    mappings worth absorbing rather than parallel-defining.

Q9. **Encounter scope coherence**: the proposal expanded mid-draft
    from pattern-encounters only to (patterns + vocabulary + structural-
    moves + frames + open). Is this scope too broad? Does the
    discipline lose coherence when it tries to register too many shapes?
    Or is the broader scope correct because the underlying pattern
    ("noticed something we want future-recognition for") is one thing
    that takes many shapes?

---

## Acknowledgment

This proposal is itself substrate-grounded in the following operational
moments — these are the actual encounters that surfaced the need:

- Substrate-currency vocabulary evolution (A2 day-2 → A3 launch
  session): three-layer framing reframed as two-axis; the
  reframe-record itself surfaced as worth preserving (inoculation
  pattern); preservation discipline had no formal home below V0+1.
- Multiple typed-feedback files in `~/.claude/projects/R--antigen/memory/`
  describe disciplines that each represent first-encounters of
  patterns the team has not yet ratified (e.g.,
  `feedback_metaphor_silence_at_boundary_is_the_evidence`,
  `feedback_shape_fit_per_instance_at_consolidation_time`). These are
  *encounter-tier* artifacts that currently live in role-memory rather
  than project-substrate.
- Tekgy's lingering-felt-shape that more axes/mechanisms surface
  during substrate-currency reframe — the pattern of "we keep finding
  this" requires a tier where "found this once" is registered.

---

## Routing

This proposal routes to navigator → aristotle for Phase 1-8 → team
deconstruction → ratification per `process.md`.

If the team ratifies the proposal, the next moves are:

1. Resolve Q1 (structural placement) → create the file or section
2. Resolve Q3 (entry shape) → write the schema
3. Migrate the three proposed first entries to the ratified location
4. Update `process.md` to include encounters in the ADR lifecycle
5. Update `glossary.md` if encounters becomes a defined term

If the team rejects or substantially modifies the proposal, the
proposal stays in `docs/expedition/` as a substrate document and the
discipline remains informal.

---

*Proposal authored 2026-05-09 during antigen-A3 launch session by
team-lead (in conversation with Tekgy). Preserves the conversation
substrate that produced it. Open for Phase 1-8.*
