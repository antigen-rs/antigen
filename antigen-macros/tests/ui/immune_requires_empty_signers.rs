//! P3b compile-fail: `signers(required = [])` must reject at compile time
//! (NFA-7: empty required list is a semantic no-op that vacuously passes).

use antigen_macros::immune;

pub struct DummyAntigen;

#[immune(DummyAntigen, requires = signers(required = []))]
pub struct ImmuneEmptySigners;

fn main() {}
