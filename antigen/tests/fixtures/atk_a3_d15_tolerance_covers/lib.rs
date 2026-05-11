// D1.5 ADR-018 §Enforcement: tolerance covers inherited presentations.
//
// State 4 (Tolerated) absorbs the inherited + tolerated case — when an
// ancestor has #[presents(Parent)] AND the descendant declares
// #[antigen_tolerance(Parent, ...)], the inherited Presentation is
// tolerated (state 4), NOT unaddressed (state 7).

#[antigen(name = "parent", fingerprint = "item: struct")]
pub struct Parent;

#[antigen(name = "child", fingerprint = "item: struct")]
#[descended_from(Parent)]
#[antigen_tolerance(
    Parent,
    rationale = "Test fixture: descendant explicitly tolerates inherited Parent presentation."
)]
pub struct Child;

// Propagation source.
pub struct Vulnerable;

impl Vulnerable {
    #[presents(Parent)]
    pub fn dangerous() {}
}
