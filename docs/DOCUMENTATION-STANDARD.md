# The Documentation Standard

The rulebook for every documentation surface in this project. Apply it; do not improvise. It consolidates the
discipline we already run and the best-in-class prior art (Diátaxis, the Rust API Guidelines, the docs-as-tests
literature, the Google and Microsoft developer style guides, GOV.UK's content design, Tom Johnson's
known-limitations work).

A surface passes when a stranger — someone who has never seen this project, cannot ask us, and has only what is in
front of them — can understand and use the thing, and everything they read is true when they run it.

---

## §0 — The surface-class gate (check this first, every time)

The rules below are not uniform: they depend on **which kind of surface you are editing.** Name the class before
you write a word.

| Class | Examples | Governing rule |
|---|---|---|
| **Reference / teaching** | rustdoc (`//!`, `///`), README, help/`--help` text, guides, tutorials | The full standard. Teaches **what IS**, present tense, no versions, no roadmap. |
| **Source not-rendered** | plain `//` comments, identifiers, string literals, test comments | **No-leakage (P4) still applies** — people read the source directly. Accuracy applies. Version/roadmap notes are tolerated only as honest *implementation* notes, never as user-facing teaching. |
| **Decision record** | ADRs (`docs/decisions.md`) | Legitimately historical. Append-only, supersede-not-erase. The "teach what IS / no versions" rule does NOT apply — a decision record IS a record. |
| **Changelog / migration** | `CHANGELOG`, migration notes | Legitimately historical/versioned. Deprecations and version deltas live here, not in reference surfaces. |
| **Roadmap** | a dedicated `ROADMAP` surface | Legitimately aspirational. Forward-looking plans live here, not in a reference README. |

If a piece of content is on the wrong class of surface, **move it to the right class** — do not delete
information that has a legitimate home; relocate it. (A roadmap section in a README → the ROADMAP surface. A
deprecation note in a README → the CHANGELOG/migration surface.)

---

## The nine principles

**P1 — Teach what IS.** Reference surfaces are written in the timeless present tense. No version numbers
(`v0.x`, "epoch"), no "used to be," no "will someday be," no "not yet," no "planned," no TODO. A reader arriving
today learns the current truth — not the history, not the plan.
> *Present-tense is a **durability property**, not a style choice: a doc with no time-anchor cannot go stale as
> the code moves around it. That is the whole reason for the rule.*

**P1a — The insidious version-leak (the hard, valuable class).** The *overt* leaks are greppable — role names,
"the crew," explicit "TODO", a bare "v0.x". The *insidious* ones are camouflaged as legitimate technical prose
and slip straight past a keyword scan: *"X refines at the engine epoch"*, *"the SCIP symbol supersedes this"*,
*"the current approach"*, *"for now"*, *"eventually"*, *"at a later stage"*, *"the 90% case … the real version
will …"*. They read like ordinary API docs, yet they leak our roadmap, our internal phasing, AND a quiet
"this is only temporary" undertone that undermines the reader's confidence in what actually ships — three
teach-what-IS violations wearing a technical disguise. **Hunt them by READING for future / comparative /
provisional framing, not by grepping keywords.** Tells: a comparison to a not-yet-existing better version
("supersedes", "refines", "the real one"), a temporal hedge ("for now", "currently", "eventually"), a named
future phase, or a stated approximation that implies a promised replacement. The cure is the same: state what the
code DOES now, and move any genuine limitation into honest-scope (P8) as a **present-tense fact** ("does not
handle inline `mod` blocks") — never as a roadmap ("will handle them in a later phase").

**P2 — Accuracy is absolute.** Every behavioral claim ("returns X", "run Y → see Z", "this does W") is **RUN
against the real binary** and reflects what actually happens. A claim that produces something else is a defect,
full stop. Behavioral lies are hunted by *running*, not by reading.

**P3 — Derive over assert (priority-ordered).** Prefer, in order: (1) **derive** the doc from the code/data so it
cannot drift; (2) **run-capture** — generate the example output by running and pasting the real result; (3)
**hand-build** — only when neither is possible, and then it must be RUN to verify. Never hand-assert a behavioral
claim from memory or from the changelog.

**P4 — No leakage (governs source too, not just rendered surfaces).** No reader-visible surface exposes our
internal state, process, protocol, roles, or authorial anxiety. **This applies to the source itself** — plain
`//` comments, identifiers, string literals — because people read the code on GitHub, in review, in their editor,
even when nothing "prints" it. Scrub: role/process references ("co-captain", lens/agent names, "the crew
decided," "pending review," "the survey found"), internal phasing ("epoch," our wave/expedition structure),
methodology chatter, hedging, meta-commentary about how it was made. A comment explains *the code*, never *our
process*.

**P5 — Mode purity (Diátaxis).** The four documentation modes — **tutorial** (learning-oriented, must be safe and
must work), **how-to** (task-oriented, cannot promise safety, assumes competence), **reference** (information-
oriented, dry and complete), **explanation** (understanding-oriented) — are not interchangeable. Conflating them
is the failure. Know which mode a surface is in and keep it pure.

**P6 — Rustdoc conditional-mandatory sections.** A public item that can panic carries a `# Panics` section; that
returns `Result`/errors carries `# Errors`; that is `unsafe` carries `# Safety`. These are mandatory *when the
condition holds* — not optional prose.

**P7 — One canonical noun per concept.** Each concept has exactly one name across all surfaces; no synonym drift.
Maintain the glossary; a second noun for an existing concept is a defect.

**P7a — Define terms where the reader lands (the three-way decision).** A reader enters through ANY document —
via search, a link, a code-review diff — not "document 1." So **every document is self-sufficient for the terms
it uses**: a term defined once, in the first doc, fails everyone who arrives anywhere else. For each specialized
term a competent stranger landing on THIS page wouldn't already know (acronym, initialism, jargon, project-
concept), **decide** which of three mechanisms fits — do not default to one:
- **Inline-expand on first use** — when the meaning fits in a phrase. Best for standard acronyms/initialisms:
  *"SCIP (SCIP Code Intelligence Protocol)"*, *"FNV (the Fowler–Noll–Vo hash)"*, *"salsa (the incremental-
  computation framework)"*. Each document expands its own acronyms on first use — not just the project's first doc.
- **Link to the glossary** — for a recurring project-concept with a canonical short definition
  (e.g. `StromaNodeId`, `IdentityDigest`, `ShapeDigest`). One definition, cross-linked from every use. Add the
  entry if it's missing (the glossary must actually cover the term you link to).
- **Link to a definitive file** — when a whole document/ADR/guide already covers the concept comprehensively
  (SCIP's tradeoffs → the spike-yard analysis; a design decision → its ADR). Link to the authority; do NOT
  re-summarize it badly in every doc. Centralizing such a topic into a one-line glossary entry would be *worse*
  than the file that already exists.

The judgment test: **would a competent stranger landing on THIS page, having read nothing else, be lost by this
term?** If no (near-universal-for-the-audience — `AST`, `ADR`, `cfg` for a Rust reader), do nothing. If yes, pick
the lightest mechanism that unblocks them.

**P7b — Reference material splits by purpose, not one overstuffed file.** A single glossary conflates readers who
arrive with completely different intents: someone learning the domain *metaphor* needs teaching; someone who hit a
project acronym needs a one-liner; someone who hit an external method needs a definition plus a link out. One file
serves none of them well. Split reference material by **who reads it, when, and why** — not alphabetically into one
bucket. Distinct reader-intents get distinct files (each pure in purpose, Diátaxis-honest about its mode), cross-
linked to each other (P7a) rather than merged. A domain-*grounding* resource is **education** (explanation/tutorial
mode), not a definition list — don't flatten teaching into a glossary line. And keep discipline/methodology entries
separate from term-definitions: a *discipline* ("recognition-not-design") is an explanation of how the system is
built; a *term* ("witness") is a definition — different modes, different files.

**P8 — Honest-scope as a named section.** State plainly what the thing does NOT do — its limits, the cases it
doesn't handle — in an explicit section. Silence about a limitation reads as a promise the code doesn't keep.

**P9 — Stranger-facing.** Write for someone who has never seen this and cannot ask. If they cannot understand or
use it from the surface alone, the surface is incomplete — that is a finding, not the reader's fault.

---

## The shippable checklist (run this on every surface before it lands)

- [ ] **Class named** (§0) — and content is on the right class of surface.
- [ ] **Present tense, no time-anchors** (reference surfaces): zero version numbers, "epoch", "used to be", "will",
      "not yet", "planned", "TODO", "soon".
- [ ] **Every behavioral claim RUN** — output reflects the real binary; derived or run-captured where possible.
- [ ] **Zero leakage** — no internal state/process/roles/anxiety, in rendered docs OR plain source comments OR
      identifiers/strings. A stranger sees only the *thing*, never *us*.
- [ ] **Mode pure** — one Diátaxis mode per surface, not conflated.
- [ ] **Rustdoc sections present** where conditional-mandatory (`# Panics` / `# Errors` / `# Safety`).
- [ ] **One canonical noun** — no synonym drift; glossary honored.
- [ ] **Honest-scope stated** — limits named, not implied by silence.
- [ ] **Usable by a stranger** — understandable from the surface alone.
- [ ] **No dead links** — every internal reference resolves.

---

*This standard governs every doc surface project-wide. It is a reference surface itself — so it, too, teaches what
IS. When a new lesson is learned, it is named here.*
