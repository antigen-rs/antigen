//! P3b compile-fail: `ratified_doc(min_version = "")` must reject at compile time
//! (NFA-15: empty min_version vacuously passes the version floor for any versioned doc).

use antigen_macros::immune;

pub struct DummyAntigen;

#[immune(DummyAntigen, requires = ratified_doc(min_version = ""))]
pub struct ImmuneEmptyMinVersion;

fn main() {}
