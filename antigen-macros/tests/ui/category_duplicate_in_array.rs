//! Adversarial fixture: `category = [SA, SA]` must be rejected.
//! Duplicate category entries produce a misleading 2-element vec that
//! looks like hybrid but contains only one distinct variant.

use antigen_macros::antigen;

#[antigen(
    name = "test-antigen",
    fingerprint = "item = struct",
    category = [AntigenCategory::SubstrateAlignment, AntigenCategory::SubstrateAlignment],
)]
pub struct DuplicateCategory;

fn main() {}
