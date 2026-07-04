//! Prescriptive work-orchestration — **the board is the code** (ADR-033).
//!
//! Run the example itself:
//!
//! ```sh
//! cargo run --example prescriptive_board --package antigen
//! ```
//!
//! But the point of this family is what `cargo antigen audit` does with it:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
//! ```
//!
//! The audit prints a `── Work board (ADR-033)` section: every `#[panel]` /
//! `#[rx]` / `#[refer]` / `#[biopsy]` / `#[ddx]` / `#[triage]` / `#[culture]` /
//! `#[quarantine]` in scope becomes a board row, with `OVERDUE` rows sorted
//! loud to the top. **The board is a live projection of the current code**
//! (ADR-034) — recomputed every run, never stored, so it cannot drift the way a
//! `// TODO` comment or an external tracker does. That is the v0.3 thesis:
//! *code IS the Asana board*.
//!
//! # The four verdicts
//!
//! `cargo antigen audit` projects every work-need onto a four-valued
//! `WorkVerdict` (ADR-033 §Decision 3). The verdict lattice is the defense
//! tri-state with the unsatisfied cell *temporally split* by the frame:
//!
//! | Verdict | Meaning | Loud? |
//! |---|---|---|
//! | `Fulfilled` | Satisfaction met at the current fingerprint | no |
//! | `Pending` | Declared and evaluable; not yet satisfied. The *expected* state | no |
//! | `Overdue` | Past the `due` frame and unsatisfied (and evaluable) | **yes** |
//! | `OutOfFrame` | Un-evaluable in the current substrate — an unknown who-ref, a missing source, an unparseable date | advisory |
//!
//! The load-bearing distinction is `Overdue` (late, but the audit *can* grade
//! it) versus `OutOfFrame` (the audit cannot even tell whether it is late). The
//! two are NEVER collapsed (ATK-PRES-8 — the three-valued-logic gem). An
//! `OutOfFrame` row carries a typed sub-cause + a per-cause remedy, so an
//! un-evaluable need routes a *different* fix than a late one.
//!
//! # What this file demonstrates
//!
//! A tiny config-parser module carries real work-needs at real code sites.
//! Each macro below is chosen so the audit verdict is **deterministic and
//! stable across calendar time** — no row silently flips verdict as the clock
//! advances (see "A note on time-stable verdicts" at the bottom). When you run
//! the audit you will see, in board order (overdue first):
//!
//! - **OVERDUE** — `#[quarantine]` whose `until` date is in the past with no
//!   release attestation (a frame elapsed without positive closure).
//! - **pending** — `#[culture]` with no frame (an open observation window) and
//!   `#[triage]` over resolvable code sites awaiting a `triaged_by` attestation.
//! - **out-of-frame** — three rows, each a *different* typed sub-cause:
//!   - `#[refer]` to a who-ref with no signed sidecar → `unknown-who-ref`
//!   - `#[panel]` with no who-step at all → `missing-work-step`
//!   - `#[quarantine]` with an unparseable `until` string → `unparseable-frame`
//!
//! `Fulfilled` is intentionally *not* shipped as a live row — reaching it
//! requires a signed `.attest/<item>.json` sidecar that is fingerprint-pinned
//! (NFA-21), and a checked-in sidecar would go stale the moment this file's
//! bytes change. See "Reaching Fulfilled" below for how a real adopter closes a
//! work-need.

#![allow(dead_code, unused_variables)]

use antigen::{culture, panel, quarantine, refer, triage};

// ── The mini-codebase the work-needs are about ───────────────────────────────
//
// A small, plausible config-parser. The prescriptive macros below sit on these
// real items; the work-need's identity IS its (file, item-path) — move the
// item and the need moves with it; delete the item and the need is moot
// (ADR-033 T1, the locality test). That locality is exactly what distinguishes
// an antigen-prescriptive work-need from a camp campsite.

/// A parsed configuration value.
#[derive(Debug)]
pub enum ConfigValue {
    /// A string scalar.
    Text(String),
    /// An integer scalar.
    Number(i64),
}

/// Parse one `key = value` line into a `(key, ConfigValue)` pair.
///
/// `#[culture]` with **no** `runs_until` frame: an open observation window.
/// There is no closure attestation declared, so the audit can tell the window
/// is un-closed but not yet expired → **`Pending`** (the expected, quiet
/// state). It stays `Pending` regardless of today's date because there is no
/// frame to elapse. Biology: a culture incubating — read the result later.
#[culture(test_kind = "fuzz parser against malformed UTF-8 and oversized keys")]
pub fn parse_line(line: &str) -> Option<(String, ConfigValue)> {
    let (key, raw) = line.split_once('=')?;
    let key = key.trim().to_string();
    let raw = raw.trim();
    let value = raw
        .parse::<i64>()
        .map_or_else(|_| ConfigValue::Text(raw.to_string()), ConfigValue::Number);
    Some((key, value))
}

/// Resolve `${VAR}` interpolations against the process environment.
///
/// `#[refer]` hands the work to an external owner — here a who-ref with no
/// signed sidecar in `antigen/examples/.attest/`. The audit cannot read the
/// referral's response, so the row is **`OutOfFrame`** with the typed sub-cause
/// `unknown-who-ref` (remedy: scaffold + sign the sidecar for the named
/// who-ref). This is *not* `Overdue` — the audit cannot even grade lateness,
/// which is a different intervention. Biology: a specialist referral awaiting a
/// response.
#[refer(to = "security-team", response_due = "2026-08-15")]
pub fn interpolate_env(raw: &str) -> String {
    // The security team needs to review the injection surface of `${...}`
    // expansion before this ships — the referral is anchored right here.
    raw.to_string()
}

/// Merge a child config over a parent, child keys winning.
///
/// `#[panel]` declares a review battery (`needs`) but names **no** who-step —
/// no `filled_by`, `reviewed_by`, or `ordered_by`. With nothing to attest the
/// audit is structurally un-evaluable: **`OutOfFrame`** with the sub-cause
/// `missing-work-step` (remedy: declare the missing who-step). A panel that
/// names what must be checked but nobody to check it is a vacuous board row —
/// the audit says so out loud rather than silently treating it as done.
#[panel(needs = [
    "verify child-wins precedence for nested tables",
    "confirm no key is silently dropped on type mismatch",
])]
pub fn merge_configs(parent: &str, child: &str) -> String {
    if child.is_empty() {
        parent.to_string()
    } else {
        format!("{parent}\n{child}")
    }
}

/// Strip comments (`# ...`) from a raw config blob.
///
/// `#[quarantine]` with an `until` date in the **past** and no release
/// attestation. A frame elapsed without positive closure is **`Overdue`** — the
/// one loud verdict (sorted to the top of the board). Frame-expiry alone never
/// fulfills a quarantine (the positive-closure guard, ATK-PRES-13): the hold is
/// released by a release event, not by the calendar. The `reason` is required
/// (ADR-005 Amendment 2 — every suppression-shaped primitive must say why).
#[quarantine(
    scope = "comment-stripping skips `#` inside quoted strings",
    until = "2025-12-01",
    reason = "Known bug: a `#` inside a double-quoted value is wrongly treated \
              as a comment start. Held until the quote-aware lexer lands."
)]
pub fn strip_comments(raw: &str) -> String {
    raw.lines()
        .map(|l| l.split('#').next().unwrap_or("").trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Validate that required keys are present.
///
/// `#[quarantine]` whose `until` is an **unparseable** date string. The audit
/// cannot read the deadline, so it cannot say whether the hold has expired:
/// **`OutOfFrame`** with the sub-cause `unparseable-frame` (remedy: fix the date
/// to ISO-8601 `YYYY-MM-DD`). A frame the audit cannot parse blocks every other
/// reading — note it takes precedence even though a real `scope`/`reason` is
/// present. This is the third distinct `OutOfFrame` sub-cause; routing the right
/// remedy per cause is the `SubCauseCollapse` fix (ADR-033 §typed `OutOfFrameCause`).
#[quarantine(
    scope = "schema validation rejects valid optional-with-default keys",
    until = "when-the-config-rfc-lands",
    reason = "The optional-with-default semantics are unspecified pending the \
              config RFC; isolate validation of these keys until then."
)]
pub fn validate_required(config: &str, required: &[&str]) -> bool {
    required.iter().all(|k| config.contains(k))
}

/// A standing priority ordering over the parser's remediation backlog.
///
/// `#[triage]` is a re-validatable ORDERING over **code-site references** (not
/// camp campsites — anchor #3: the audit never reads camp). Each entry in
/// `priority_order` is resolved against the scanned workspace (ADR-017
/// Amendment 1). Here every entry names a function declared *in this file*, so
/// all refs resolve → the ordering is evaluable. With no `triaged_by`
/// attestation yet and no elapsed `re_triage_due`, the verdict is **`Pending`**:
/// the ordering is declared and well-posed, just not yet attested. (Had any
/// entry pointed at a function that does not exist, the whole triage would be
/// `OutOfFrame` / `unresolvable-ref` — never silently satisfied.)
#[triage(
    priority_order = [
        "strip_comments",
        "validate_required",
        "interpolate_env",
    ],
    re_triage_due = "2026-12-31",
)]
pub const fn parser_remediation_order() {}

fn main() {
    println!("prescriptive_board example — the work-needs live on the parser items above.");
    println!("The board is rendered by the audit, not by this binary:");
    println!("  cargo run --bin cargo-antigen -- antigen audit --root antigen/examples");
    println!();

    // Exercise the parser so the example is a real, working program.
    let parsed = parse_line("retries = 3");
    println!("parse_line: {parsed:?}");

    let merged = merge_configs("timeout = 30", "retries = 5");
    println!("merge_configs:\n{merged}");

    let stripped = strip_comments("retries = 5  # inline comment\nname = prod");
    println!("strip_comments:\n{stripped}");

    let ok = validate_required("retries = 5\nname = prod", &["retries", "name"]);
    println!("validate_required: {ok}");

    let expanded = interpolate_env("home = ${HOME}");
    println!("interpolate_env: {expanded}");

    parser_remediation_order();
}

// ── Reaching Fulfilled (what an adopter does, not shipped here) ───────────────
//
// A work-need is `Fulfilled` when its closing who-steps are attested at the
// CURRENT fingerprint. The attestation lives in a sidecar co-located with the
// annotated item: `.attest/<item>.json`. For a panel like `merge_configs`:
//
//   1. Add the who-steps that close the battery:
//        #[panel(
//            needs = ["verify child-wins precedence", "no key silently dropped"],
//            filled_by = ["alice"],
//            reviewed_by = ["bob"],
//        )]
//   2. Scaffold + sign the sidecar (the same pipeline defense attestation uses):
//        cargo antigen attest scaffold --root antigen/examples <item>
//        cargo antigen attest sign --root antigen/examples <item> --signer alice
//        cargo antigen attest sign --root antigen/examples <item> --signer bob
//   3. Re-run the audit — the row flips to `fulfilled`.
//
// Satisfaction is fingerprint-pinned (NFA-21): if `merge_configs`'s body later
// changes, the stored signature no longer matches the current fingerprint and
// the row drops back to `pending` (the review is stale — re-attest). That
// staleness is the whole point: a signed-but-stale review is the silent-wrong
// bug this family exists to prevent. A checked-in sidecar in this example
// directory would therefore go stale on the next edit to this file — which is
// exactly why `Fulfilled` is documented here rather than shipped as a frozen,
// soon-to-rot row. The other six rows are driven by declared dates + who-refs
// alone, so their verdicts are deterministic and stable across calendar time.

// ── A note on time-stable verdicts ───────────────────────────────────────────
//
// The audit grades frames against `Local::now()` (audit.rs `FrameState::classify`),
// so a verdict that depends on a *future* date is not stable — a `due` in 2027
// reads `Pending` today but flips `Overdue` once 2027 passes, silently making
// this doc-example wrong. Every row above is therefore chosen to be
// time-invariant:
//
//   - `Overdue`  — a date clearly in the PAST (`2025-12-01`); it stays past forever.
//   - `Pending`  — NO frame at all (the `#[culture]`), or a resolvable ordering
//                  not yet attested (the `#[triage]`); unsatisfied-with-no-elapsed-
//                  frame is `Pending` regardless of the clock.
//   - `OutOfFrame` — date-independent by construction: an unknown who-ref, a
//                  missing who-step, or an unparseable date string. None of these
//                  three sub-causes depends on what day it is.
//
// (`re_triage_due = "2026-12-31"` on the triage is a *staleness* frame, not a
// deadline; the triage is `Pending` because it is un-attested, and would only
// read `Overdue` if it were both attested AND past that date — so the future
// date here does not flip the present verdict.)
