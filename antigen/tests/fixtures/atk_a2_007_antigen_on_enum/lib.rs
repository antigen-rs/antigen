// Fixture: antigen attribute applied to an enum (not a unit struct).
// The scan visitor visit_item_enum currently silently discards this.
// Expected behavior: either record a diagnostic or record the declaration
// so the developer gets feedback.

#[antigen(name = "frame-translation", fingerprint = "class enum + meet")]
pub enum DeterminismClass {
    Additive,
    Multiplicative,
}
