//! # Deserialization-Trust-Boundary Family — stdlib antigens
//!
//! The un-shipped DEEP tier of the shipped Mucosal-Boundary family (ADR-027):
//! deserialization is THE canonical place untrusted bytes cross into typed-Rust
//! land — the gut mucosa, the largest/busiest trust surface. serde's own issue
//! tracker is the prior-art goldmine (serde #1087 "is Serde safe deserializing
//! untrusted input?", #44 the silent-drop origin, #2634 `deny_unknown_fields`),
//! and the recursion-DoS class has recorded harm across ≥3 RUSTSEC advisories
//! spanning 2022→2026 (RUSTSEC-2024-0012 serde-json-wasm, RUSTSEC-2022-0004
//! rustc-serialize, RUSTSEC-2026-0009 time) — survivor-bias-exempt evidence.
//!
//! Biology cognate: `deny_unknown_fields` = the tight-junction that decides
//! what molecules cross the gut wall; its absence = leaky gut (uncontrolled
//! admission). This is mucosal-boundary's deepest, most-trafficked tier — the
//! metaphor said it MUST exist, the wild confirms it recurs.
//!
//! ## Antigen-category (ADR-028)
//!
//! Members are `FunctionalCorrectness`: the deserialization verb produces a
//! wrong *effect* — a `DoS` / stack-blow on malformed input
//! (`UnboundedDeserialization`), or a silently-dropped unknown field that masks
//! API drift / smuggled data (`DeserializeWithoutDenyUnknownFields`).
//!
//! ## How these antigens are evaluated
//!
//! Both members carry a **syntactic fingerprint** matched by the AST-walking
//! scanner — the call-tell (`body_calls` on a deser entrypoint) and the
//! attribute-presence-AND-absence tell (`derives` + `not(serde_arg(...))`).

use crate::antigen;

// ============================================================================
// 1. DeserializeWithoutDenyUnknownFields
// ============================================================================

/// A `#[derive(Deserialize)]` type that does not set `#[serde(deny_unknown_fields)]`
/// — unknown input fields are silently dropped at the trust boundary.
///
/// **Where in the wild:** serde's DEFAULT silently discards unknown fields
/// (serde #44 is the origin issue) — "deserializes even if data has fields that
/// don't match." For config / auth / payment payloads this masks API drift and
/// smuggled fields. "Users employ `deny_unknown_fields` because they want
/// notified as soon as the format changes" (serde #2634).
///
/// **Tell:** `#[derive(Deserialize)]` **present** AND `#[serde(deny_unknown_fields)]`
/// **absent** — `all_of([derives("Deserialize"), not(serde_arg("deny_unknown_fields"))])`.
/// The cleanest attribute-presence-AND-ABSENCE driver in the family: the
/// presence of the safe argument (`deny_unknown_fields`) spares the sibling.
///
/// **Tier:** **suspected** — not every `Deserialize` is at a trust boundary; the
/// member graduates to named when paired with a trust-boundary marker.
///
/// **Witness:** `deny_unknown_fields` present, OR a documented "lenient-by-design"
/// tolerance, OR a validating wrapper.
///
/// **Known caveat (serde #2283 / #1600):** `#[serde(flatten)]` bypasses the
/// `deny_unknown_fields` check — a flattened struct re-opens the boundary. The
/// syntactic tell cannot see this; the confidence dial carries the honest gap.
///
/// **Category:** `FunctionalCorrectness` — the deserializer silently accepts
/// input it should reject, diverging from the intended contract.
#[antigen(
    name = "deserialize-without-deny-unknown-fields",
    category = AntigenCategory::FunctionalCorrectness,
    provenance = Provenance::Constructable,
    presentation = Presentation::Passive,
    fingerprint = r#"all_of([derives("Deserialize"), not(serde_arg("deny_unknown_fields"))])"#,
    family = "deserialization-trust-boundary",
    summary = "A #[derive(Deserialize)] type without #[serde(deny_unknown_fields)] silently drops unknown input fields at the trust boundary, masking API drift / smuggled fields. The absence of the safe argument is the tell.",
    references = [
        // Differential-reference oracle (silent class, ADR-039 §C): the serde
        // deny_unknown_fields container-attribute doc — correct = reject-unknown
        // vs actual = silently-accept (the divergence the dropped-field hides).
        "https://serde.rs/container-attrs.html#deny_unknown_fields",
        "https://github.com/serde-rs/serde/issues/44",
        "https://github.com/serde-rs/serde/issues/2634",
        "ADR-027",
        "ADR-040",
    ]
)]
pub struct DeserializeWithoutDenyUnknownFields;

// ============================================================================
// 2. UnboundedDeserialization
// ============================================================================

/// Deserializing from a byte/reader source with no size / depth / recursion
/// limit — a `DoS` surface (stack exhaustion on deeply-nested input, unbounded
/// allocation on huge input).
///
/// **Where in the wild:** "`DoS` via malformed or deeply-nested JSON" — recursive
/// structures blow the stack; huge flat input allocates unboundedly. Recorded
/// harm across ≥3 RUSTSEC advisories 2022→2026 (serde-json-wasm stack overflow
/// fixed with a `remaining_depth` counter; rustc-serialize; time) — the
/// strongest recorded-harm evidence in the family sweep.
///
/// **Tell:** the **streaming** entrypoint `from_reader` (the real recorded-harm
/// `DoS`: std warns a `from_reader` on a non-terminating stream "will not return"):
/// `body_calls("from_reader")`.
///
/// **Why bare presence, NOT a `not(take)` guard at this named tier (the
/// surface-flag / witness-proof split, ADR-019/029).** A `.take(limit)`-capped
/// reader is the std-documented anti-`DoS` idiom — but the bounded form
/// `from_reader(reader.take(n))` should still **fire** on the fingerprint (the
/// risky *surface* — a streaming deser — is genuinely present) and be **spared by
/// the WITNESS at audit** (the `.take(limit)` is the defense, proved at the
/// audit/`#[defended_by]` stage), NOT fingerprint-spared. A `not(body_calls("take"))`
/// guard would instead *silently suppress* the finding whenever an **unrelated**
/// `Iterator::take` appears in the body — a silent false-negative that breaks the
/// **named** tier's high-confidence promise (named = "if it doesn't fire, you are
/// covered"). `take` is a **subject-slice** (a common method name, possibly
/// unrelated), and a subject-slice negation is inadmissible at named (it IS
/// admissible at heuristic, where a labeled recall hole is within-tier — cf. the
/// crypto member's `not(ct_eq)` at heuristic). The fix is surgical: drop the
/// guard, keep named — the `from_reader` core is an honest defect-slice that
/// stands alone at named.
///
/// **Why `from_slice` was DROPPED (ADR-039 §C Amendment 1, the spares-namesake
/// sub-test).** An earlier form carried a weaker `from_slice` breadth-arm
/// (`any_of([from_reader, from_slice])`). It was dropped because a slice is a
/// **bounded** source (its length is known), so `from_slice` is categorically
/// **not** an unbounded-deserialization vector — and the `from_slice` last-segment
/// fires on the *bounded-slice form that is itself the fix* for the streaming
/// `DoS`, plus ubiquitous safe constructors (`GenericArray::from_slice`,
/// `Pubkey::from_slice`). A needle that flags the recommended remediation is
/// inadmissible at **any** tier (the clean-sibling-collision rule), so `from_slice`
/// is dropped, not demoted. The in-memory **deep-nesting recursion** `DoS` (a real
/// vector that *can* arrive via a slice) is a **distinct** future
/// `#[descended_from]` depth-member keyed on the recursion structure, not on the
/// `from_slice` entrypoint — so dropping the arm leaves no silent gap.
///
/// **Scope (honest defect-slice):** `from_str` / `from_slice` are deliberately
/// **excluded** — `body_calls` matches by last path segment with no path
/// resolution, so `from_str` would fire on every `i32::from_str` (`FromStr`, not
/// deserialization); and `from_str`/`from_slice` operate on data *already fully
/// in memory* (bounded), so their unbounded risk, if any, is the caller's upstream
/// read, not the deser call — not the streaming `from_reader` defect this member
/// names.
///
/// **Tier:** **named/confident** — the `from_reader` entrypoint is rare/std-specific
/// (self-anchoring; a domain type rarely has a `from_reader` method), so the
/// effective codomain is the defect population (RUSTSEC-backed streaming `DoS`).
///
/// **Witness:** a bounded reader (`.take(n)`) / depth guard (`serde_stacker`) on
/// the deserialization path — proved at audit (the surface fires; the witness
/// spares).
///
/// **Category:** `FunctionalCorrectness` — the deserialization verb produces a
/// wrong *effect* (a `DoS` / stack-blow) on adversarial input.
#[antigen(
    name = "unbounded-deserialization",
    category = AntigenCategory::FunctionalCorrectness,
    provenance = Provenance::Constructable,
    presentation = Presentation::Passive,
    fingerprint = r#"body_calls("from_reader")"#,
    family = "deserialization-trust-boundary",
    summary = "A streaming from_reader deserialization — a DoS surface (the std-documented non-terminating-stream / stack-exhaustion harm). Named tier (from_reader is the rare/std-specific self-anchor — a domain type rarely has a from_reader method). The .take(limit)-capped form FIRES (surface present) and is spared by the WITNESS at audit, not by a not(take) guard (a subject-slice negation is a silent-FN at named). from_slice DROPPED (ADR-039 §C Amd-1, spares-namesake): a slice is BOUNDED so from_slice is not an unbounded vector + it fires on the bounded-slice FIX and on safe ctors (GenericArray/Pubkey::from_slice) — flagging the fix is inadmissible at any tier; the deep-nesting recursion DoS is a distinct future depth-member. from_str excluded (FromStr collision).",
    references = [
        "RUSTSEC-2024-0012",
        "RUSTSEC-2022-0004",
        "RUSTSEC-2026-0009",
        "ADR-040",
    ]
)]
pub struct UnboundedDeserialization;
