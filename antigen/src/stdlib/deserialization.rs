//! # Deserialization-Trust-Boundary Family — stdlib antigens (beta.2 voyage)
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
    fingerprint = r#"all_of([derives("Deserialize"), not(serde_arg("deny_unknown_fields"))])"#,
    family = "deserialization-trust-boundary",
    summary = "A #[derive(Deserialize)] type without #[serde(deny_unknown_fields)] silently drops unknown input fields at the trust boundary, masking API drift / smuggled fields. The absence of the safe argument is the tell.",
    references = [
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
/// **Tell:** a call to a byte/reader-source deserialization entrypoint —
/// `from_reader` / `from_slice` — without a bounded reader / depth-limited
/// deserializer (`.take(limit)`, `serde_stacker`). `any_of([body_calls("from_reader"),
/// body_calls("from_slice")])`.
///
/// **Scope (honest defect-slice):** `from_str` is deliberately **excluded** —
/// `body_calls` matches by last path segment with no path resolution, so
/// `from_str` would fire on every `i32::from_str` / `bool::from_str`
/// (`FromStr`, not deserialization). `from_reader` / `from_slice` have no such
/// stdlib collision. Disambiguating `serde_json::from_str` from `FromStr::from_str`
/// needs path resolution (a charter / next-increment concern); shipping
/// `from_str` here would be dishonestly noisy at the named tier.
///
/// **Tier:** **named/confident** for the recursion-DoS call-presence form
/// (RUSTSEC-backed); the bounded-guard-absence relational refinement (does a
/// `.take(limit)` guard the call?) is the next-increment tightening.
///
/// **Witness:** a bounded reader (`.take(n)`) / depth guard (`serde_stacker`) on
/// the deserialization path.
///
/// **Category:** `FunctionalCorrectness` — the deserialization verb produces a
/// wrong *effect* (a `DoS` / stack-blow) on adversarial input.
#[antigen(
    name = "unbounded-deserialization",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"any_of([body_calls("from_reader"), body_calls("from_slice")])"#,
    family = "deserialization-trust-boundary",
    summary = "A byte/reader-source deserialization (from_reader / from_slice) with no size/depth/recursion limit — a DoS surface (stack exhaustion on deeply-nested input). Named tier; from_str excluded (FromStr collision needs path resolution).",
    references = [
        "RUSTSEC-2024-0012",
        "RUSTSEC-2022-0004",
        "RUSTSEC-2026-0009",
        "ADR-040",
    ]
)]
pub struct UnboundedDeserialization;
