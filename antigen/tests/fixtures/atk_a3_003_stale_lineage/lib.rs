// ATK-A3-003 fixture: a #[descended_from] reference whose parent antigen
// is not declared in the scanned workspace. Biology cognate: B-cell lineage
// whose progenitor no longer exists.
//
// `Child` declares descent from `MissingParent`, but no #[antigen]
// declaration of `MissingParent` exists. scan_workspace must:
//   - record the lineage edge (the declaration is structurally well-formed)
//   - NOT emit a parse_failure (parse_failures is for structural errors)
//   - surface it via ScanReport::orphaned_lineage_edges() as a semantic
//     warning, parallel to orphaned_tolerances()

#[antigen(name = "child", fingerprint = "item: struct")]
#[descended_from(MissingParent)]
pub struct Child;
