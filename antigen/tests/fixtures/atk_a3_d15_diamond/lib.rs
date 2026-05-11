// D1.5 ADR-018 §Enforcement: diamond inheritance dedup.
//
// Common ancestor Top has #[presents(Top)] somewhere.
// Left and Right both #[descended_from(Top)].
// Bottom #[descended_from(Left)] AND #[descended_from(Right)].
//
// After propagation, Bottom inherits EXACTLY ONE Presentation for Top
// (not two). Per ADR-018 §"The synthesis algorithm", ProvenanceEntry's
// antigen_type is the ancestor whose PRESENTATIONS are being propagated
// — so the chain is [{antigen_type: "Top", ...}], not [{Left}, {Right}].
// Left and Right contribute no presentations of their own and appear
// only as DFS intermediates; they don't surface in inherited_from.
//
// The diamond dedup key (antigen, item_target, canonical_path) +
// per-DFS-source `visited` HashSet together guarantee one Presentation
// record per descendant per ancestor-with-presentations, regardless
// of how many paths reach that ancestor.

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
