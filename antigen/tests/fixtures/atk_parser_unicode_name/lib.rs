// ATK-PARSER-1: Unicode characters in the antigen name.
// The scan parser must not panic on Unicode names.
// The macro parser WOULD reject this at compile time (non-kebab-case).
// The scan parser is permissive: it stores whatever it finds.

#[antigen(
    name = "café-au-lait",
    fingerprint = "some structural pattern"
)]
pub struct CafeAuLait;
