// The CLEAN CORPUS for the PROMOTE case — a near-miss sibling.
//
// One known-good directory-walk that PROPAGATES its read error with `?` instead
// of swallowing it. This is the anti-correlated SAFE case: it shares the
// directory-walk skeleton with the defect twins but does exactly the right thing
// with the error. That makes it a NEAR-MISS — one discriminating constraint away
// from binding the draft anti-unified from the swallow-error cluster.
//
// Because this clean sibling is one-constraint-away, the gate can WITNESS that the
// draft's generalization is corpus-exercised (a real in-family discrimination: the
// draft spares code that handles the error correctly, binds code that swallows it).
// With that near-miss present, the gate PROMOTES — and `cargo antigen propose`
// renders a ratifiable SUGGESTION (a candidate fingerprint to ratify by hand).
//
// Operator-supplied, never auto-labeled: you vouch that this `?`-propagating walk
// is the clean sibling the draft must spare.

fn scan_dir_safe(root: &std::path::Path) -> std::io::Result<Vec<String>> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(root)? {
        out.push(e?.path().display().to_string());
    }
    Ok(out)
}
