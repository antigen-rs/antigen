// D1.5 ADR-018 §Enforcement: state 6 anti-case.
//
// "An item with #[presents(A)] and an inherited Presentation for B (different
// antigen) is state 1 for A and state 7 for B. #[presents(A)] does NOT
// re-attest an inherited Presentation for a different antigen."
//
// Setup: ParentA has #[presents(ParentA)] somewhere. ParentB has
// #[presents(ParentB)] somewhere. Child #[descended_from(ParentA)] AND
// #[descended_from(ParentB)] AND #[presents(ParentA)] (only).
//
// After propagation:
//   - Child has Presentation for ParentA, ExplicitMarker, inherited_from = Some([ParentA])
//   - Child has Presentation for ParentB, inherited from B, inherited_from = Some([ParentB])
// The explicit #[presents(ParentA)] does NOT cover the ParentB inheritance.

#[antigen(name = "parent-a", fingerprint = "item: struct")]
pub struct ParentA;

#[antigen(name = "parent-b", fingerprint = "item: struct")]
pub struct ParentB;

#[antigen(name = "child", fingerprint = "item: struct")]
#[descended_from(ParentA)]
#[descended_from(ParentB)]
#[presents(ParentA)]
pub struct Child;

// Sources for the propagation: one per parent.
pub struct VulnerableA;
pub struct VulnerableB;

impl VulnerableA {
    #[presents(ParentA)]
    pub fn dangerous_a() {}
}

impl VulnerableB {
    #[presents(ParentB)]
    pub fn dangerous_b() {}
}
