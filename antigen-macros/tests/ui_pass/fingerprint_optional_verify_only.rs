//! ADR-009 Amendment 1 fixture: `#[antigen(name = "x")]` (no `fingerprint`)
//! must COMPILE successfully — absent fingerprint is a valid verify-only
//! antigen declaration (detection model is external-substrate, no syn-scan
//! surface). This replaces the old `tests/ui/missing_fingerprint.rs` which
//! asserted compile failure (stale after AMD1 made fingerprint optional).

use antigen_macros::antigen;

#[antigen(name = "x")]
pub struct VerifyOnlyAntigen;

fn main() {}
