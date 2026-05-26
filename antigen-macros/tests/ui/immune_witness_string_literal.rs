//! W2 fixture: `#[immune(X, witness = "test_name")]` with the witness as a
//! STRING LITERAL must reject. A string-literal witness silently never resolves
//! at audit time (the resolver matches fn names against the token, and the
//! literal carries its quotes), so the macro rejects it loudly — the witness
//! must be a bare identifier/path to the verifying item.

use antigen_macros::immune;

struct SomeAntigen;

#[immune(SomeAntigen, witness = "my_test")]
fn defended() {}

fn main() {}
