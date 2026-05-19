//! P3b compile-fail: `ratified_doc(anchor = "")` must reject at compile time
//! (NFA-14: empty anchor vacuously passes str::contains("") for any doc content).

use antigen_macros::immune;

pub struct DummyAntigen;

#[immune(DummyAntigen, requires = ratified_doc(anchor = ""))]
pub struct ImmuneEmptyAnchor;

fn main() {}
