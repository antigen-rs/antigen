// E0 bundled-catalog fixture: a "consumer" crate with ZERO antigen declarations.
//
// This stands in for a fresh crate that depends on `antigen` but has not
// declared any antigens of its own. Scanned WITHOUT the bundled catalog it
// produces no fingerprint matches (the zero-hits-cliff: empty `fingerprints` →
// `synthesis_pass` never runs → false all-clear). Scanned WITH the bundled
// catalog it must surface the flagship failure-classes present in this code.
//
// Read-as-text by the scan walk (never compiled as part of the test crate).

// --- A real `panic-in-drop` flagship site (impl_of_trait("Drop") + .unwrap()). ---
pub struct UnwindBomb {
    pub handle: Option<u32>,
}

impl Drop for UnwindBomb {
    fn drop(&mut self) {
        // .unwrap() inside a Drop body — the canonical teardown footgun the
        // `panic-in-drop` flagship fingerprint binds.
        let _ = self.handle.unwrap();
    }
}

// --- A clean sibling: a Drop impl that does NOT reach a panic source. ---
// The bundled catalog's `panic-in-drop` fingerprint must NOT bind this.
pub struct CleanGuard {
    pub flushed: bool,
}

impl Drop for CleanGuard {
    fn drop(&mut self) {
        // No panic source: just a side-effect-free teardown.
        if self.flushed {
            // nothing
        }
    }
}

// --- A second flagship site: `get-unchecked-without-proof`. ---
pub fn read_at(buf: &[u8], i: usize) -> u8 {
    // SAFETY: caller-promised (the fixture deliberately omits the proof) — this
    // is exactly the `get_unchecked` flagship the bundled catalog binds.
    unsafe { *buf.get_unchecked(i) }
}
