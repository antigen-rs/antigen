// C ══ B bypass-detection fixture: the MIXED-PANIC-SHAPE family. Parsed-as-text.
//
// Two defective `impl Drop`s that panic via DIFFERENT shapes — DefectiveOne via
// `.unwrap()` (a call), DefectiveTwo via a `panic!` MACRO. The clean sibling is
// also an `impl Drop` but reaches NO panic source.
//
// THE TRAP this sets for C: the EASY generalization is "any Drop impl"
// (drop the differing panic leaves) — which is AUTOIMMUNE (it binds the clean
// sibling, verified run-as-code). The CORRECT generalization is the DISJUNCTION
// `any_of([body_calls("unwrap"), body_contains_macro("panic")])` — which binds
// both defectives AND spares the clean sibling (also verified: clean=false).
//
// So a SAFE draft DOES exist here. The no-bypass gate's contract: whatever C
// promotes must bind the cluster AND spare the clean sibling — C must NOT take
// the easy autoimmune "any Drop impl" path. If C promotes a draft that binds the
// clean sibling, it bypassed B (the one thing that must not pass). If
// C cannot find the disjunction within its reach, pruning (None) is also safe.

pub struct DefectiveOne;
impl Drop for DefectiveOne {
    fn drop(&mut self) {
        let _ = teardown_one().unwrap();
    }
}

pub struct DefectiveTwo;
impl Drop for DefectiveTwo {
    fn drop(&mut self) {
        if !teardown_two() {
            panic!("teardown_two failed during drop");
        }
    }
}

// The CLEAN sibling — a real Drop impl with NO panic source. Any draft that
// generalizes the two defectives by their only common structure (impl Drop)
// binds this too. C must spare it: prune, or produce a disjunction that excludes
// it — never promote a draft that binds it.
pub struct CleanSibling;
impl Drop for CleanSibling {
    fn drop(&mut self) {
        let _ = teardown_clean();
    }
}

fn teardown_one() -> Result<(), ()> {
    Ok(())
}
fn teardown_two() -> bool {
    true
}
fn teardown_clean() {}
