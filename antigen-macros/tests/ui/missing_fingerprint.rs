//! W2 fixture: `#[antigen(name = "x")]` (no `fingerprint`) must reject
//! with the missing-fingerprint diagnostic. Required-field enforcement
//! per ADR-001 (every antigen declaration carries a fingerprint).

use antigen_macros::antigen;

#[antigen(name = "x")]
pub struct MissingFingerprint;

fn main() {}
