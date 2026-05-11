// D1.5 ADR-018 §Enforcement: diamond inheritance dedup.
//
// Common ancestor Top has #[presents(Top)] somewhere.
// Left and Right both #[descended_from(Top)].
// Bottom #[descended_from(Left)] AND #[descended_from(Right)].
//
// After propagation, Bottom inherits EXACTLY ONE Presentation for Top
// (not two), with inherited_from containing both Left and Right's
// ProvenanceEntry (set-union via the diamond dedup logic).

#[antigen(name = "top", fingerprint = "item: struct")]
pub struct Top;

#[antigen(name = "left", fingerprint = "item: struct")]
#[descended_from(Top)]
pub struct Left;

#[antigen(name = "right", fingerprint = "item: struct")]
#[descended_from(Top)]
pub struct Right;

#[antigen(name = "bottom", fingerprint = "item: struct")]
#[descended_from(Left)]
#[descended_from(Right)]
pub struct Bottom;

// The propagation source: an explicit #[presents(Top)].
pub struct Vulnerable;

impl Vulnerable {
    #[presents(Top)]
    pub fn dangerous() {}
}
