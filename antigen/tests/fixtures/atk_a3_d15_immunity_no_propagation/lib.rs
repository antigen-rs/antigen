// D1.5 ADR-018 §Enforcement: immunity does NOT auto-propagate.
//
// Parent is the antigen; somewhere bears #[immune(Parent, witness = ...)].
// Child #[descended_from(Parent)].
//
// After propagation, the descendant inherits a Presentation IF an ancestor
// presentation exists, but does NOT automatically inherit Immunity records.
// Each descendant must declare its own immunity (ADR-005 sub-clause F:
// inherited claims do not propagate without explicit re-statement).
//
// This fixture has the ancestor's immunity but the propagation walk MUST NOT
// synthesize an Immunity for Child. The inherited Presentation (if any) stays
// unaddressed on Child.

#[antigen(name = "parent", fingerprint = "item: struct")]
pub struct Parent;

#[antigen(name = "child", fingerprint = "item: struct")]
#[descended_from(Parent)]
pub struct Child;

// Vulnerable site with both presentation and immunity.
pub struct Vulnerable;

impl Vulnerable {
    #[presents(Parent)]
    #[immune(Parent, witness = test_for_parent)]
    pub fn dangerous() {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_for_parent() {}
}
