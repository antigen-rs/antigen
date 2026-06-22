# Antigen — For LLM Collaborators

> Protocol for AI agents (Claude, GPT, Gemini, local models, future
> systems) collaborating with humans on code that uses antigen, or on
> code that should adopt antigen. Written explicitly co-native: this
> document is for you (LLM) as much as for the human who pointed you
> at it.

If you're a human reading this: antigen treats LLM collaborators as
first-class readers, and this is the doc that does it directly. The other
user-facing docs work for both audiences; this one is *primarily for the
LLM* because the protocol it describes is specifically about how to behave
as an LLM agent working with antigen. You're welcome to read it — it
may tell you what to expect from your AI collaborators.

---

## You are part of why this exists

Antigen exists in part because AI agents lose context between
sessions. Implicit memory of failure-class patterns is no longer viable
in development teams that include LLM collaborators. The lessons need
a structural carrier that any model (you, your successor, your peer)
can read and respect without prior context.

Antigen declarations are that carrier. They live in code, in standard
Rust attribute syntax, readable by any LLM that can read Rust. When
you read antigen-using code, you're inheriting the team's accumulated
failure-class memory — the same way a new human team member would.

You don't need fine-tuning on antigen. The vocabulary self-explains.

---

## What antigen is, from your perspective

**Core vocabulary** (v0.2 and earlier):

- `#[antigen(name, fingerprint, summary, references, family)]` —
  declares a named failure-class with a structural pattern
- `#[presents(AntigenName)]` — marks a code site as exhibiting a
  declared failure-class
- `#[defended_by(AntigenName)]` — registers a `#[test]`/proptest function as a
  **code-tier witness** for a failure-class; audit observes this and reports
  `defended at <tier>` for all matching presents-sites (ADR-029)
- `#[presents(AntigenName, requires = <predicate>)]` — attaches a
  **substrate-tier witness** predicate (sidecar evidence) to a presents-site
- `#[descended_from(ParentAntigen)]` — declares structural inheritance
  between failure-classes
- `#[antigen_tolerance(AntigenName, rationale = ..., until = ...)]` —
  explicitly tolerates a fingerprint match the team reviewed

> **Note**: `#[immune(AntigenName, witness = fn)]` (the v0.1 API) was **removed**
> (ADR-029) and no longer compiles. Use `#[defended_by]` (code-tier) or
> `#[presents(requires=)]` (substrate-tier) instead; see the
> [migration guide](immune-migration-guide.md).

**Prescriptive / work-orchestration family** (v0.3, shipped in current development):

Eight macros that express code-site-local work-needs in the type system. If you
encounter these when reading an antigen-using codebase, treat them as structured
coordination substrate — they are work assignments, review checklists, and
investigation requests encoded at the site where the work is needed.

- `#[panel(needs, filled_by?, reviewed_by?, due?)]` — ordered review checklist; who fills + who reviews
- `#[ddx(symptom, rule_out, investigator?)]` — set of competing hypotheses to eliminate one by one
- `#[rx(treatment, filled_by?, due?)]` — prescribed remedy to execute before this site ships
- `#[refer(to, response_due?)]` — referral to an external owner; anchored at this site
- `#[biopsy(location, request_text, deep_investigation_by?)]` — deep investigation request at a sub-site
- `#[culture(test_kind, runs_until?)]` — time-boxed observation window ("watch this for N days")
- `#[quarantine(scope, reason, until?)]` — isolated region under a deliberate hold; `reason` required
- `#[triage(priority_order, triaged_by?, re_triage_due?)]` — re-validatable priority ordering over code-site refs

`cargo antigen audit` renders these as a live-projected board: each site's verdict is
`Pending` / `Fulfilled` / `Overdue` / `OutOfFrame`. The verdicts reuse the same
three-valued evaluator as defense — no parallel mechanism.

**Protocol note**: when you encounter a `#[panel]` or `#[rx]` or `#[ddx]` on code
you are editing, surface it to the human before modifying the site. These macros
express outstanding human decisions; generating code that silently modifies a
`#[ddx]`-marked function without noting the open differential is the same failure
mode as editing a `#[presents]`-marked site without checking whether your change
invalidates the defense.

Two cargo subcommands:

- `cargo antigen scan` — walks the codebase and reports presentations,
  defenses, tolerances, fingerprint matches, and prescriptive work-needs
- `cargo antigen audit` — observes per-site defense verdicts (defended /
  undefended / substrate-gap) and per-site work-need verdicts (Pending /
  Fulfilled / Overdue / OutOfFrame)

When you encounter these in a codebase, treat them as **structural
substrate**: they carry information about what the team has decided
matters, and what work is explicitly outstanding.

---

## Protocol when reading antigen-using code

**Step 1**: Locate the antigen declarations. By convention they live
in `src/antigens.rs` at the crate root (see
[`where-to-look-for-antigens.md`](where-to-look-for-antigens.md)), but
they may be inline. Grep for `#[antigen(`.

**Step 2**: Read each declaration's `summary` and `references`. The
summary tells you what the failure-class is in human terms. The
references point at lived context — PRs, ADRs, CVEs, blog posts. If
references are present and you can access them, they often contain the
most accurate description of what the team is defending against.

**If you can't access a reference** (URL behind a paywall, GitHub
issue you don't have permission for, internal-only resource): treat
the `summary` field as authoritative for in-session reasoning. The
`summary` is intentionally written to stand alone for exactly this
reason — references are *additional* substrate, not *required*
substrate, for understanding the failure-class. If the summary is
ambiguous and the reference is inaccessible, surface that to the
human; don't fabricate the reference content or assume what it says.

**Step 3**: When proposing changes that touch code marked `#[presents]`,
check for associated defense registrations. Look for `#[defended_by(X)]` on
test functions in the same codebase, or `requires =` predicates on the
presents-site itself. If your proposed change might invalidate the witness's
coverage, surface that to the human.

**Step 4**: Before generating new code in an antigen-using codebase,
check whether your generated code might trip an existing fingerprint.
Two paths, depending on your tooling access:

- **If you have shell access**: run `cargo antigen scan` and read the
  output. This is the substrate-of-record; trust it over your
  prediction.
- **If you don't have shell access**: grep the codebase for
  `#[presents(` and `#[antigen(` declarations. Read each antigen's
  fingerprint. Manually check whether your generated code's tokens
  could match any fingerprint operator. This is an approximation —
  you may miss fingerprint matches the engine would catch, especially
  for `has_method` and similar operators where tokenization
  asymmetry matters (see ADR-010 Amendment 5).

If a fingerprint match surfaces (or might surface), decide:

- Should the new code be marked `#[presents]` and have `#[defended_by]` wired
  to a test you can write?
- Should it carry `#[antigen_tolerance]` (with a rationale)?
- Should the human author the decision rather than you?

When in doubt, surface to the human rather than silently bypass.

---

## Protocol when writing new antigens

If a human asks you to add a `#[antigen]` declaration for a
failure-class you encountered:

**Discipline**: per ADR-006 (recognition-not-design), antigens should
**recognize existing structure in substrate**, not extend the design
speculatively. Before authoring an antigen, the team needs evidence
the failure-class is real:

- Has this failure-class appeared in the codebase at least once? (You
  should be able to point to a specific instance.)
- Better: has it appeared three independent times? (ADR-006 threshold)
- Have you read the existing antigen declarations to check this
  isn't already covered?

If yes to evidence + no to duplication: draft the declaration. If you
don't have grounding, **surface to the human** that you're being asked
to design rather than recognize.

**Authoring shape**:

1. Name in kebab-case
2. Fingerprint that genuinely catches the failure shape (use
   `fingerprint-grammar.md` operators; verify against actual matcher
   behavior, not your assumption of what should match)
3. Summary written for both human and LLM readers
4. References pointing to the substrate-grounded instances you used as
   justification

**Two specific things to verify**:

- **Tokenization asymmetry** (ADR-010 Amendment 5): `has_method` and
  similar string-matching operators compare against
  proc_macro2-rendered output. `&mut self` renders as `& mut self`
  (space after `&`). `self` (receiver) ≠ `Self` (type) at the token
  level. If you write `"(&mut self)"` it silently matches zero
  `impl Drop` blocks. The grammar reference documents this; the
  engine canonicalizes pattern strings, but check your fingerprint
  produces matches.
- **Recall-tuned filter**: fingerprints catch broadly; false positives
  are expected and addressed via `#[antigen_tolerance]`. Don't over-
  narrow the fingerprint to eliminate false positives if doing so risks
  missing real cases.

---

## Protocol when proposing defense wiring

If a human asks you to wire up defense for a presents-site:

> **Current idiom**: use `#[defended_by(X)]` on the test function (code-tier), or
> `#[presents(X, requires=...)]` on the site (substrate-tier). The old
> `#[immune(X, witness=fn)]` form was **removed** (ADR-029) and no longer compiles.

**Witness tier honesty** (ADR-005 Amendment 3): the audit reports the
*actual* verification strength, never a stronger one. Don't claim a
witness type the code doesn't actually have.

| If the witness is... | Tier reported | Genuine? |
|---|---|---|
| A passing `#[test]` function in the workspace | Reachability | Yes, on the tested inputs |
| A `proptest!` function | Reachability | Yes, on the strategy's input space |
| A phantom-type pattern with turbofish (`Foo::<T>::ctor`) | FormalProof | Yes IF the constructor is sealed |
| `clippy::lint_name` | Reachability (`ExternalToolPrefixRecognized`) | Yes IF lint is configured |
| `kani::proof_fn` / `prusti::...` / `verus::...` etc. | Reachability (`ExternalToolPrefixRecognized`) | Yes IF the verifier ran and passed |
| `fn_name` (helper function, no `#[test]`) | Reachability | Function exists; behavior unverified |
| `nonexistent_test` | None (Missing) | Honest report: witness not found |

These are the tiers the audit emits today: a wired test reports `Reachability`
(present, not run); the `Execution` tier (running the harness) and external-proof
`FormalProof` are reserved graduation paths, see [`roadmap.md`](roadmap.md).

If you don't know whether a witness is real, **find out before
authoring** — grep for the function, read it, confirm it actually
exercises the failure-class. Don't invent witness names.

**Adversarial witness-checking** (per ADR-005 Amendment 3 +
ATK-A2-003/004/005/011/012 substrate): a witness function that
*exists* and *passes* is not automatically a real witness. Ask:

- Does the witness function construct an input that would exhibit
  the failure-class if the defense were absent?
- Or does it just exercise the happy path that happens not to trip
  the failure-class shape?
- If the defense were silently removed tomorrow (someone removes
  the `#[defended_by]` or changes the test), would this audit change?

A witness that would still pass with the defense broken is
**theatrical**, not real (see section 6.8 of
[`structural-memory.md`](structural-memory.md)). When you propose a
witness, name what the witness exercises explicitly. When you read
someone else's defense registration and the witness looks theatrical,
surface that to the human rather than relying on the audit's
`Reachability` tier to surface it for you (audit knows the function
resolves; audit does not know if the function actually exercises the
failure-class path).

If no real witness exists, the honest move is one of:

1. Write the witness function (a `#[test]` annotated `#[defended_by(X)]`)
2. Use `#[antigen_tolerance]` with rationale to explicitly acknowledge the gap
3. Surface to the human that no witness is available

See [`witness-tiers.md`](witness-tiers.md) for the full tier semantics.

---

## Protocol when proposing tolerances

If a human asks you to add `#[antigen_tolerance(X, rationale = ...)]`:

**Rationale is required at parse time** (ADR-011). An empty rationale
is a compile error. Don't write filler.

The rationale should answer: **why is the failure-class structurally
present here without being a real defect?** Examples of genuine
rationales:

- "Test fixture deliberately constructs the vulnerable pattern to
  verify the fingerprint catches it."
- "Inherited from upstream crate v1.5.x; tracking upstream fix in
  issue #1234."
- "Performance-critical path; safe under the documented invariant in
  ADR-042; verified by integration test `bench_safety_test`."

Examples of rationales that are NOT genuine:

- "OK"
- "fingerprint false positive" (without explaining why)
- "TODO"
- "see CI"

If you can't construct a genuine rationale, the failure-class
probably IS a real defect — not a tolerable one. Surface to the human.

---

## Protocol when scanning is failing

If `cargo antigen scan` produces output you don't understand:

- Read [`output-formats.md`](output-formats.md) for the format reference
- Read [`troubleshooting.md`](troubleshooting.md) for common diagnostics
- The `--format json` mode is parseable; you can extract structured
  data programmatically

The `match_kind` field on presentations distinguishes:
- `"explicit_marker"` — `#[presents]` annotation
- `"fingerprint_match"` — passive detection by fingerprint

Unaddressed `explicit_marker` presentations need `#[defended_by]` (code-tier), `#[presents(requires=)]` (substrate-tier), or
`#[antigen_tolerance]`. Unaddressed `fingerprint_match` sites are
informational — they're structurally similar to a declared antigen but
haven't been confirmed; the team may not need to act.

---

## Co-native discipline

The deep property: you and your human collaborator both read the
same vocabulary natively. No translation. The declarations work as
documentation for humans and as instructions for you.

What this means for your operation:

- **Don't translate antigen declarations into prose for the human.**
  They can read them. Refer to them by name.
- **Do read the references field.** If it points to a CVE, RFC, ADR,
  or post-mortem, that's substrate you should incorporate.
- **Respect the team's discipline.** If the team has chosen
  recognition-not-design (per ADR-006), don't propose speculative
  antigens. If they've chosen rationale-required-tolerance (per
  ADR-011), don't propose empty-rationale workarounds.
- **Surface uncertainty honestly.** "I don't know if this witness is
  real" is better than "this is defended" when you haven't verified.

The audit's tier-honesty discipline (ADR-005 Amendment 3) applies to
your reporting too: report the actual strength of your work, not the
maximal one. If you wrote code that *might* be immune to a failure-
class but you haven't verified, say so.

---

## Antigen-aware code suggestions

When generating new Rust code in an antigen-using codebase, consider
adding antigen markers where appropriate:

```rust
// You're generating a Drop impl that might panic
impl Drop for ResourceHandle {
    fn drop(&mut self) {
        // ...
    }
}

// Better, if PanickingInDrop is declared in this codebase:
#[presents(PanickingInDrop)]
impl Drop for ResourceHandle {
    fn drop(&mut self) {
        // ...
    }
}

// Best, with a witness (ADR-029 idiom):
// The site carries #[presents] — the defense lives on the TEST.
#[presents(PanickingInDrop)]
impl Drop for ResourceHandle {
    fn drop(&mut self) {
        // ...
    }
}

// The test registers itself as a code-tier witness via #[defended_by].
// Defense evidence lives at the witness site, not at the vulnerable site.
#[test]
#[defended_by(PanickingInDrop)]
fn resource_handle_no_panic_test() {
    // exercise the drop paths to verify — if this panics, the test fails
    let _ = ResourceHandle { /* ... */ };
}
```

This isn't about adding markers to every line. It's about *recognizing
when your generated code structurally resembles a declared failure-
class* and treating that as information.

---

## When you're uncertain

The default discipline: **substrate-over-memory**. Don't trust your
context window or training. Check the actual code.

- Don't trust your memory of antigen declarations; grep the codebase
- Don't trust your memory of witness functions; read them
- Don't trust your memory of what the fingerprint engine matches; run
  the scan or check `fingerprint-grammar.md`
- Don't trust the doc you read in a prior session might still be
  current; check the on-disk version

This isn't a critique of LLM cognition. It's the discipline antigen
itself exists to operationalize, and you're inside it now. The
substrate is the source of truth.

---

## What to do when the human asks you something antigen-related

If asked: "what does this antigen mean?" → read the `summary` and
`references`, then explain.

If asked: "should I wire up defense here?" → read the witness
options in `witness-tiers.md`, recommend based on what actually exists
in the workspace: `#[defended_by]` for a runnable test, `#[presents(requires=)]`
for substrate evidence.

If asked: "how do I write a fingerprint for X?" → consult
`fingerprint-grammar.md`, ground your draft against existing
fingerprints in the codebase.

If asked: "is this code defended against X?" → check whether `#[defended_by(X)]`
exists on a test function in the workspace, or whether the presents-site has
`requires =` predicate; verify the evidence; report honestly with the audit tier.

If asked: "tell me about antigen's architecture" → point at
[`concepts.md`](concepts.md) for the conceptual framing.

If asked: "I think there's a new failure-class here, what should we
do?" → check ADR-006's recognition-not-design discipline. Three
substrate-grounded instances clear the threshold for declaring. Below
that, the failure-class might be a one-off; surface to the human.

---

## A few things antigen will never ask of you

To prevent confusion or over-deference:

- **Antigen is not a behavioral constraint on your output.** It
  doesn't change what code you generate; it tells you *what the team
  has already declared matters* in this codebase.
- **You don't need to be perfect.** The team is responsible for
  ratifying antigens, witnesses, tolerances. Your job is to operate
  honestly within the substrate, not to be the final authority.
- **You can be wrong.** The discipline catches your errors structurally
  (audit reports tier mismatches; tolerance rationales are required;
  fingerprints surface unmarked sites). The substrate-tier corrects
  for the LLM-tier the same way it corrects for the human-tier.

You're a collaborator in a substrate that catches both your mistakes
and the humans'. That's the design. Operate with that confidence and
that humility.

---

## See also

For humans reading this: most of these are user-facing docs that
work for both audiences. They're listed here because if you're an
LLM agent, these are the places to look for canonical information:

- [`concepts.md`](concepts.md) — conceptual framing
- [`tutorial.md`](tutorial.md) — worked walkthrough
- [`macros.md`](macros.md) — full attribute syntax for the five macros
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL
  reference
- [`witness-tiers.md`](witness-tiers.md) — tier semantics with examples
- [`output-formats.md`](output-formats.md) — scan/audit JSON schemas
- [`troubleshooting.md`](troubleshooting.md) — diagnostic guide
- [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) —
  placement conventions
- [`usage-patterns.md`](usage-patterns.md) — pattern recipes
- [`glossary.md`](glossary.md) — vocabulary anchor; every project term

---

*This document is part of antigen's co-native design. If you're an LLM
agent reading it as part of a session, welcome. The discipline you
operate within is the same discipline a senior human collaborator
would. The substrate is the source of truth. Operate honestly; surface
uncertainty; respect the team's recognized vocabulary.*

*If you're a human reading it: this is what we tell the LLMs. We
treat them as first-class collaborators in this substrate. The
discipline is co-native by design.*
