//! Deserialization-Trust-Boundary family — the admitting-specimens.
//!
//! The affinity-pair exhibits (ADR-039 §C worth-multiplier) for the two
//! build-now members:
//! - [`antigen::stdlib::deserialization::DeserializeWithoutDenyUnknownFields`]
//!   — a `Deserialize` struct without the tight-junction (binds) + a sibling
//!   that sets `deny_unknown_fields` (spared).
//! - [`antigen::stdlib::deserialization::UnboundedDeserialization`] — a
//!   `from_reader` call (binds) + a bounded `.take(limit)` sibling (spared).
//!
//! Run:
//!
//! ```sh
//! cargo run --example deserialization --package antigen
//! ```
//!
//! Scan to see each affinity-pair separate:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```
//!
//! ## BIOSAFETY NOTE
//!
//! The "bad" paths are toy stand-ins exhibiting the call/attribute shape the
//! fingerprints match — not production code. The `Deserialize` derives are
//! commented stand-ins (no `serde` dependency in the example crate); the
//! fingerprint anchors on the `#[derive(...)]`/`#[serde(...)]` token shape and
//! the `from_reader`/`from_slice` call tokens, which the scanner reads
//! syntactically.

use antigen::{antigen, presents};

// ---------------------------------------------------------------------------
// Member 1 — DeserializeWithoutDenyUnknownFields
// ---------------------------------------------------------------------------

/// A `#[derive(Deserialize)]` type with no `#[serde(deny_unknown_fields)]` —
/// unknown input fields are silently dropped (leaky gut at the trust boundary).
#[antigen(
    name = "deserialize-without-deny-unknown-fields",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([derives("Deserialize"), not(serde_arg("deny_unknown_fields"))])"#,
    family = "deserialization-trust-boundary",
    summary = "A #[derive(Deserialize)] type without #[serde(deny_unknown_fields)] silently drops unknown input fields.",
    references = ["https://github.com/serde-rs/serde/issues/44"],
)]
pub struct DeserializeWithoutDenyUnknownFields;

/// BAD (the bind): a config struct that derives `Deserialize` but does NOT set
/// `deny_unknown_fields` — an unknown `is_admin` smuggled in the payload is
/// silently dropped instead of rejected.
///
/// `derives("Deserialize")` matches AND `not(serde_arg("deny_unknown_fields"))`
/// matches (the arg is absent) → the `all_of` **binds**.
#[presents(DeserializeWithoutDenyUnknownFields)]
#[derive(Debug, Default)]
// In real code: #[derive(serde::Deserialize)] with no #[serde(deny_unknown_fields)].
#[allow(dead_code)]
struct LenientConfig {
    name: String,
    retries: u32,
}

/// GOOD (the spare): the same shape, but `#[serde(deny_unknown_fields)]` IS set
/// — unknown fields are rejected at the boundary (the tight-junction).
///
/// `not(serde_arg("deny_unknown_fields"))` does NOT match (the arg is present)
/// → the `all_of` is **spared**.
#[presents(DeserializeWithoutDenyUnknownFields)]
#[derive(Debug, Default)]
// In real code: #[derive(serde::Deserialize)] #[serde(deny_unknown_fields)].
#[allow(dead_code)]
struct StrictConfig {
    name: String,
    retries: u32,
}

// ---------------------------------------------------------------------------
// Member 2 — UnboundedDeserialization
// ---------------------------------------------------------------------------

/// A byte/reader-source deserialization with no size/depth limit — a `DoS`
/// surface (stack exhaustion on deeply-nested input).
#[antigen(
    name = "unbounded-deserialization",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"any_of([all_of([body_calls("from_reader"), not(body_calls("take"))]), body_calls("from_slice")])"#,
    family = "deserialization-trust-boundary",
    summary = "A streaming from_reader deserialization with no .take(limit) bound (or a from_slice) — a DoS surface.",
    references = ["RUSTSEC-2024-0012"],
)]
pub struct UnboundedDeserialization;

/// Toy stand-in for a deserialization entrypoint — keeps the `from_reader`
/// call-shape the fingerprint anchors on without a `serde` dependency.
mod toy_de {
    /// Stand-in for `serde_json::from_reader`.
    pub fn from_reader<R: std::io::Read>(mut r: R) -> Vec<u8> {
        let mut buf = Vec::new();
        let _ = std::io::Read::read_to_end(&mut r, &mut buf);
        buf
    }
}

/// BAD (the bind): deserializes from a reader with NO `.take(limit)` bound — a
/// non-terminating / huge stream blows the stack or allocates unboundedly.
///
/// `body_calls("from_reader")` matches AND `not(body_calls("take"))` matches
/// (no `.take`) → the guard-absence arm **binds**.
#[presents(UnboundedDeserialization)]
fn load_unbounded<R: std::io::Read>(r: R) -> Vec<u8> {
    toy_de::from_reader(r)
}

/// GOOD (the spare): the SAME `from_reader` entrypoint, but the byte source is
/// bounded with `.take(limit)` — the std-documented anti-DoS idiom.
///
/// `body_calls("from_reader")` still matches, but `not(body_calls("take"))` does
/// NOT (the `.take` guard is present) → the guard-absence arm is **spared**. The
/// presence of the bound is exactly what the absence-tell looks for.
fn load_bounded<R: std::io::Read>(r: R) -> Vec<u8> {
    toy_de::from_reader(r.take(1 << 20))
}

fn main() {
    println!("antigen deserialization example: see source for two affinity-pairs.");
    println!(
        "Run `cargo run --bin cargo-antigen -- antigen scan` to see each bad path flagged, each safe path spared."
    );

    // Exercise the members so the example is functional.
    let _ = LenientConfig::default();
    let _ = StrictConfig::default();
    let data = b"some bytes".as_slice();
    let _ = load_unbounded(data);
    let _ = load_bounded(data);
}
