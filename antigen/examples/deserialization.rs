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
    fingerprint = r#"any_of([body_calls("from_reader"), body_calls("from_slice")])"#,
    family = "deserialization-trust-boundary",
    summary = "A byte/reader-source deserialization (from_reader / from_slice) with no size/depth/recursion limit — a DoS surface.",
    references = ["RUSTSEC-2024-0012"],
)]
pub struct UnboundedDeserialization;

/// Toy stand-in for a deserialization entrypoint — keeps the `from_reader` /
/// `from_slice` call-shape the fingerprint anchors on without a `serde`
/// dependency.
mod toy_de {
    /// Stand-in for `serde_json::from_reader` — unbounded by construction.
    pub fn from_reader<R: std::io::Read>(mut r: R) -> Vec<u8> {
        let mut buf = Vec::new();
        // No `.take(limit)` — reads the whole source (the unbounded shape).
        let _ = std::io::Read::read_to_end(&mut r, &mut buf);
        buf
    }

    /// Stand-in for a depth/size-guarded read (the safe step).
    pub fn from_reader_bounded<R: std::io::Read>(r: R, limit: u64) -> Vec<u8> {
        let mut buf = Vec::new();
        // Bounded: `.take(limit)` caps the byte source.
        let _ = std::io::Read::read_to_end(&mut r.take(limit), &mut buf);
        buf
    }
}

/// BAD (the bind): deserializes from a reader with no bound — an attacker's
/// deeply-nested / huge input blows the stack or allocates unboundedly.
///
/// `body_calls("from_reader")` matches → the `any_of` **binds**.
#[presents(UnboundedDeserialization)]
fn load_unbounded<R: std::io::Read>(r: R) -> Vec<u8> {
    toy_de::from_reader(r)
}

/// GOOD (the spare): the bounded sibling caps the byte source with `.take(limit)`
/// — it calls neither `from_reader` nor `from_slice` directly, so the call-tell
/// does not anchor.
///
/// Neither `body_calls("from_reader")` nor `body_calls("from_slice")` matches →
/// the `any_of` is **spared**. (The build-now member detects the unbounded
/// entrypoint *call*; the bounded-guard-presence refinement is the next
/// increment — here the safe path simply routes through a different, guarded
/// entrypoint.)
fn load_bounded<R: std::io::Read>(r: R) -> Vec<u8> {
    toy_de::from_reader_bounded(r, 1 << 20)
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
