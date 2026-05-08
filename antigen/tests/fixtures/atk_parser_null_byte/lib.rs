// ATK-PARSER-2: Null byte (\u{0000}) embedded in a string literal.
// \u{0000} is a valid Rust string escape producing a null character.
// The scan parser must handle this without panicking.

#[antigen(
    name = "null-byte-test",
    fingerprint = "body\u{0000}contains-null"
)]
pub struct NullByteTest;
