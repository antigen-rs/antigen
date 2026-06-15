// The CLEAN CORPUS for the route-to-human case.
//
// Known-good sibling code the operator labels clean. The draft anti-unified from
// the swallow-error twins must SPARE every item here. This corpus is deliberately
// UNRELATED to the defect family: pure arithmetic / formatting, no directory walk,
// no swallowed error. Nothing here is a "near-miss" (one discriminating constraint
// away from binding the draft) — so the gate cannot witness that the draft's
// generalization is corpus-exercised, and it routes the candidate to a human
// rather than promote it. That refusal is the gate being HONEST, not failing.
//
// The clean corpus is OPERATOR-SUPPLIED — antigen never auto-labels unmarked code
// as clean. You point `--clean-root` at the siblings you vouch for.

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn greet(name: &str) -> String {
    format!("hi {name}")
}
