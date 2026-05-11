// D1.5 ADR-018 §Enforcement: explicit + inherited co-existence.
//
// Parent has #[presents(Parent)] somewhere.
// Child #[descended_from(Parent)] AND ALSO bears #[presents(Parent)] explicitly.
//
// After propagation: ONE Presentation for `Parent` on Child's site (NOT two)
// with match_kind = ExplicitMarker AND inherited_from = Some([Parent]).
// The explicit marker dominates the match_kind; inheritance still records
// provenance.

#[antigen(name = "parent", fingerprint = "item: struct")]
pub struct Parent;

#[antigen(name = "child", fingerprint = "item: struct")]
#[descended_from(Parent)]
#[presents(Parent)]
pub struct Child;

// Propagation source from a separate Vulnerable site.
pub struct Vulnerable;

impl Vulnerable {
    #[presents(Parent)]
    pub fn dangerous() {}
}
