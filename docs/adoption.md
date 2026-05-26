# Adopting antigen: gotchas a first-time binary-crate user hits

> Audience: a Rust developer adopting antigen into a real crate for the first
> time — especially a *binary* crate, especially via the lightweight `attest
> sign` path rather than writing `requires =` predicates from scratch. This
> doc collects the friction points the antigen team has caught by running its
> own discipline on adopter codebases, with honest fix-status as of v0.2.

The good news: antigen adoption works. You can declare your failure-classes,
mark vulnerable sites with `#[presents]`, claim immunity with `#[immune]`,
scaffold and sign `.attest/` sidecars, and run `cargo antigen scan` / `audit`
end-to-end on day one.

The honest news: there are about eight specific points where a first-time
adopter trips. Most are now fixed in the macro layer or documented
prominently in the canonical example. Two remain open as v0.2 silent-failure
modes — listed here so you can route around them.

This page is the antigen team's audit of its own developer experience.

---

## What's fixed (no workaround needed)

### Marker structs no longer trip `dead_code` in binary crates

**The trap (older versions):** `#[antigen(...)] pub struct MyAntigen;` in a
binary crate would trip `warning: struct MyAntigen is never constructed`. Under
a `-D warnings` clippy gate that's a hard error. Library crates didn't see this
because `pub` exempts items from `dead_code` (they're public API); binary
crates have no external API surface, so `pub` doesn't save you.

**The fix:** the `#[antigen]` macro now emits a zero-cost use-token next to
the marker — an anonymous `const _: fn() = || { let _: MyAntigen; };` — so the
type counts as "used" from the compiler's view. Binary adopters need no
`#![allow(dead_code)]`.

**Verify on your end:** in a fresh binary crate with `#![warn(dead_code)]`,
declare a `#[antigen(...)] pub struct Foo;`, run `cargo build`. Clean.

### `AntigenCategory` is taken as a token — don't import it

**The trap:** writing `category = AntigenCategory::SubstrateAlignment` looks
like a normal qualified path, so the natural reflex is to add
`use antigen::AntigenCategory;`. Under `-D warnings` that triggers `unused
import: AntigenCategory` — hard error. Counterintuitive: the argument *looks*
like it needs the type in scope, but the macro consumes it as a token path,
not a real type reference.

**The fix:** the `#[antigen]` macro doc now explicitly says **do not import
`AntigenCategory`** — write the path inline. If you've imported it and your
build fails, the import is what to remove.

**Verify on your end:** `#[antigen(name = "x", fingerprint = "item = struct",
category = AntigenCategory::SubstrateAlignment)] pub struct X;` with no
`use antigen::AntigenCategory;` line — compiles clean.

### Self-matches no longer pollute scan output

**The trap (older versions):** an antigen declaration with a broad
`doc_contains(...)` fingerprint would match its *own* declaration struct
(`MyAntigen` matches the doc-comment of the `MyAntigen` struct itself), adding
trivial noise to `cargo antigen scan` output.

**The fix:** scan now suppresses the case where a declaration's own struct
matches its own fingerprint. The match carries no signal and is no longer
reported. Legitimate matches in other files still surface.

**Verify on your end:** `cargo antigen scan` on a crate where the antigen
declaration struct also matches its own `doc_contains` fingerprint — the
self-match no longer appears in the output.

### `scan --format json` emits per-item structural fingerprint

**The trap (older versions):** the documented `against = "current"` workflow
told adopters to "use the fingerprint from `cargo antigen scan --format json`"
— but the JSON didn't carry a fingerprint field on immunity or presentation
entries. The fingerprint was un-fetchable, so `attest scaffold`/`sign` got
written with an empty `--fingerprint`, and every `against = "current"` /
`fresh_within_days` predicate failed silently in audit.

**The fix:** scan JSON now carries `structural_fingerprint` on every
`Immunity`, `Toleration`, and presentation entry. The documented workflow
is reachable.

**Verify on your end:** `cargo antigen scan --format json | jq
'.report.immunities[0].structural_fingerprint'` — returns an `fnv1a64:...`
string per item.

> **Open subgap:** see "What's still open" below. The fingerprint *is* in the
> JSON, but a per-item *selector* (e.g. by `item_path`) isn't yet ergonomic —
> the scan JSON doesn't carry an item-name field on the immunity entry. For
> the common case (`attest scaffold` already knows `--source-file` and
> `--item-path`), see the auto-fill discussion below.

### `attest check` per-leaf diagnostics: failed predicates explain themselves

**The trap (older versions):** a failing compound predicate (`all_of([
signers(required = ["alice"]), ratified_doc(...), fresh_within_days(90) ])`)
reported only a tree-level `audit_hint: DisciplinePredicateFailed`. Which leaf
failed? Why? The tooling gave no signal. Debugging required reading the
evaluator source.

**The fix:** `attest check` and `audit` now render a per-leaf block:

```
Per-leaf:
  signers(required=["alice"]): FAIL — no signer named "alice" (found names: ["bob"])
  ratified_doc(path=docs/discipline.md): PASS — found at v1.2
  fresh_within_days(90): PASS — most-recent signer 12 days ago
```

Each leaf reports its own pass/fail with expected-vs-found text. The
20-minute source-dive becomes a 5-second read.

**Verify on your end:** intentionally write a predicate that won't pass
(e.g. require a signer name that doesn't exist), run
`cargo antigen attest check --sidecar ... --predicate ...` and read the
`Per-leaf:` block.

### `signers(required = [...])` matches NAMES — and the canonical example is now explicit

**The semantic:** `signers(required = ["alice"])` matches against the
signer's *name*, not their role. Roles are a separate, optional constraint:
`signers(required = ["alice"], roles = { alice = "math-researcher" })` means
"alice must have signed, AND alice's role must be `math-researcher`."

**The trap (older versions):** the canonical example (`examples/substrate_witness.rs`)
declared `signers(required = ["math-researcher"])` and instructed signing with
`--signer alice --role math-researcher` — which by the impl would NOT match
(no signer *named* `math-researcher`). The example couldn't satisfy its own
predicate.

**The fix:** the example now reads `signers(required = ["alice"], roles =
{ alice = "math-researcher" })` and the sign command uses `--signer alice
--role math-researcher`. The example satisfies its own predicate.

**Verify on your end:** copy the four-step substrate-witness workflow from
`examples/substrate_witness.rs` verbatim — it runs clean and the audit
reports Execution tier.

---

## What's still open (known silent-failure modes — route around them)

These two haven't landed yet. They aren't fatal — adoption works around them
— but knowing the failure mode saves the debugging time.

### 1. Signed sidecar on a `witness =` site is silently uncredited

**The shape:** if you scaffold + sign an `.attest/` sidecar for an antigen
whose `#[immune]` site uses `witness = <test>` (the code-witness form),
`audit` will **silently ignore** the sidecar. `attest list` shows the sidecar
+ signatures correctly, but the immunity gets reported at Reachability tier
as if the sidecar weren't there. Nothing warns you that the sidecar can
never be credited because the immune syntax is `witness =` not `requires =`.

**Why:** substrate-witness sidecars are credited only by `requires = <predicate>`
immunities (the predicate is what reads the sidecar). A `witness = <test>`
immunity is a code-witness — it's checked by running the test, not by reading
a sidecar. The two channels are mutually exclusive, but the silent disconnect
isn't called out at scaffold/sign time.

**The choosing rule** (the simple version): can a *test* execute the thing
you're defending? Yes → `witness = <test>`. No (it's about substrate state a
test can't verify — a stale doc, an unpinned dependency, an un-reviewed
discipline) → `requires = <predicate>` and sign a sidecar. See
[`tutorial.md`](tutorial.md) and the `#[immune]` macro doc for the full
contrast.

**Route around it:** if you've scaffolded a sidecar, the immune site at that
path **must** use `requires = ...`. If you wrote `witness = ...` and signed
a sidecar, change one or the other — they can also coexist as two separate
`#[immune]` attributes on the same item (one with each form).

### 2. Empty placeholder fingerprint signs without warning

**The shape:** `attest scaffold` without `--fingerprint` writes
`current_fingerprint: ""` (an empty placeholder) and prints a note saying
"update before signing." But `attest sign` happily signs against the empty
string, and then `audit` fails the `against = "current"` predicate with no
indication that the empty fingerprint is the cause.

**Why:** the guard ("refuse to sign when the sidecar's `current_fingerprint`
is empty and the predicate is `against = "current"`") isn't in place yet.
The empty placeholder is a footgun.

**Route around it:** always pass `--fingerprint` to `attest scaffold` (and
keep it consistent on `attest sign`). Get the value from
`cargo antigen scan --format json | jq '.report.immunities[] |
.structural_fingerprint'` for the relevant item. (See the open subgap on F6
above: the JSON has the fingerprint but selecting one specific item by
`item_path` isn't ergonomic yet — for now, scan the file in isolation or
pick the entry by line number, or wait for the `attest scaffold` auto-fill
that's in flight.)

---

## The bigger story (why this exists)

Antigen's first binary-crate + lightweight-sign adopter walked the full
discipline — declared real failure-classes, marked sites `#[immune]`, signed
`.attest/` sidecars, ran scan + audit end-to-end. The adoption worked, but
it surfaced these eight points of friction. None of them would have appeared
from antigen-on-antigen dogfooding, because antigen-the-lib doesn't walk the
adopter paths (binary crate, "I just want to sign it," "I followed the
example and it failed").

The team treated each friction point as a real DX finding, fixed six of them
at the source, and is honest about the two that haven't landed yet. That's
what this page is — not marketing, just an audit. If you hit one of these
and the doc doesn't help, please [open an issue](https://github.com/antigen-rs/antigen/issues);
the next adopter will benefit.

---

## See also

- [`tutorial.md`](tutorial.md) — your first 15 minutes, including the
  substrate-witness flow that surfaced most of the findings above
- [`quickstart.md`](quickstart.md) — the 5-minute path
- [`witness-tiers.md`](witness-tiers.md) — `WitnessTier` semantics and the
  honest-tier-naming discipline
- [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) —
  placement rules (markers go type-level only; variant/field markers fail
  at compile, which is also a worth-knowing trap)
- [`meta-finding-pattern.md`](meta-finding-pattern.md) — the recognition
  loop for hardcore adopters: how to turn your own recurring drifts into
  typed antigens
