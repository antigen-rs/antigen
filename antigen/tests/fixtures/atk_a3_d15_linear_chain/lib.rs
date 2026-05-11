// D1.5 ADR-018 §Enforcement: linear chain inheritance.
//
// Parent has a #[presents(Parent)] on something; Child #[descended_from(Parent)].
// After propagation, Child inherits a Presentation for `Parent` with
// inherited_from = Some([{antigen_type: "Parent", canonical_path: None}]).
//
// Setup: both Parent and Child are declared antigens (no fingerprints, so
// fingerprint synthesis does not generate FingerprintMatch presentations).
// An explicit #[presents(Parent)] on an unrelated impl produces the
// ancestor Presentation that should propagate.

#[antigen(name = "parent", fingerprint = "item: struct")]
pub struct Parent;

#[antigen(name = "child", fingerprint = "item: struct")]
#[descended_from(Parent)]
pub struct Child;

// Some unrelated impl carrying an explicit #[presents(Parent)] —
// the propagation source.
pub struct Vulnerable;

impl Vulnerable {
    #[presents(Parent)]
    pub fn dangerous() {}
}
