//! W2 fixture: `#[antigen(name = "x", fingerprint = "y", bogus = "z")]`
//! must reject with an "unknown field" diagnostic that names `bogus` and
//! suggests the valid alternatives. Macro-side strictness on unknown
//! fields is the documented asymmetry from the scan side (which tolerates
//! them for forward-compat).

use antigen_macros::antigen;

#[antigen(name = "x", fingerprint = "y", bogus = "z")]
pub struct UnknownField;

fn main() {}
