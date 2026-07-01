//! The salsa database. **Everything attaches here.**
//!
//! The whole salsa world (inputs, interned, tracked-fns) lives on `StromaDb`.

/// The stroma's salsa database — the revision/memoization clock (ADR-068).
///
/// Reads take `&StromaDb`; advancing the base takes `&mut StromaDb`. **The atomic-publish keystone
/// (ADR-068 B6) falls out of this split for free:** while any reader holds `&db`, no maintenance
/// writer can take `&mut db` — a torn read is a *compile error*, not a runtime lock.
#[salsa::db]
#[derive(Default, Clone)]
pub struct StromaDb {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for StromaDb {}

// A read SESSION holds `&db` across detection→field→provenance so all three observe ONE published
// revision. Live-edit concurrency (multi-reader + background-writer) is salsa's snapshot/fork
// facility (an ADR-068 cadence choice); the serialized `&`/`&mut` model is sufficient for the
// batch cadence this uses.
