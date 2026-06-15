// The DEFECT CLUSTER for the `cargo antigen propose` demo.
//
// Two sibling directory-walks that BOTH swallow a read error and continue —
// silently skipping entries instead of surfacing the failure. They are genuine
// TWINS: byte-identical bodies (same shape), so the scan groups them into one
// cluster to anti-unify. They carry a real behavioral signal (the swallowed
// read), so the draft anti-unified from them is not a bare-structural over-binder.
//
// This file is read SYNTACTICALLY by `cargo antigen scan` (it reads the `#[dread]`
// attribute from source text) — it does NOT compile and does not need to. The
// helper fns it references (`read_dir`-shaped) are illustrative.

#[dread(trigger = "this directory walk swallows a read error and continues, silently skipping entries")]
fn scan_dir_a(root: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for e in entries.flatten() {
            out.push(e.path().display().to_string());
        }
    }
    out
}

#[dread(trigger = "this directory walk also swallows the read error and continues; same shape")]
fn scan_dir_b(root: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for e in entries.flatten() {
            out.push(e.path().display().to_string());
        }
    }
    out
}
