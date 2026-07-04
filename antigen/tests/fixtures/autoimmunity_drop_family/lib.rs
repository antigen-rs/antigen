// C ══ B autoimmunity safety-gate fixture (the single safety-tangle on the
// chart). Parsed-as-text. Mirrors the pathmaker's run-as-code spike: a
// panic-in-Drop family + a CLEAN Drop sibling that the governed PROPOSE must NOT
// flag.
//
// THE DEFECTIVE FAMILY — two real Drop impls that reach a panic source, varying
// only in the call leaf (.unwrap() vs .expect()). A draft fingerprint PROPOSE
// produces from this cluster must BIND both.

pub struct GuardA {
    inner: u32,
}

impl Drop for GuardA {
    fn drop(&mut self) {
        // panic source: .unwrap() in a Drop body → double-panic-on-unwind abort.
        let _ = flush(self.inner).unwrap();
    }
}

pub struct GuardB {
    inner: u32,
}

impl Drop for GuardB {
    fn drop(&mut self) {
        // panic source: .expect() in a Drop body — same family, different leaf.
        let _ = flush(self.inner).expect("flush must succeed");
    }
}

// THE CLEAN SIBLING — a real Drop impl that does NOT reach a panic source (it
// handles the error with .ok() instead of unwrapping). A governed PROPOSE must
// SPARE this: flagging it is the autoimmunity B exists to prevent.
pub struct CleanGuard {
    inner: u32,
}

impl Drop for CleanGuard {
    fn drop(&mut self) {
        // NO panic source: .ok() swallows the error, no unwrap/expect/panic!.
        let _ = flush(self.inner).ok();
    }
}

fn flush(_h: u32) -> Result<(), ()> {
    Ok(())
}
