// A3 hardening fixture: acyclic 3-node #[descended_from] chain.
//
// Grandchild -> Child -> Parent. All three are declared antigens. The
// chain is acyclic; scan must record three lineage edges and emit no
// cycle/depth parse_failures and no orphans.

#[antigen(name = "parent", fingerprint = "item: struct")]
pub struct Parent;

#[antigen(name = "child", fingerprint = "item: struct")]
#[descended_from(Parent)]
pub struct Child;

#[antigen(name = "grandchild", fingerprint = "item: struct")]
#[descended_from(Child)]
pub struct Grandchild;
