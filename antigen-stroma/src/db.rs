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
    // SIMPLIFIED (Fable, read-engine): the point-wise queries receive only a `SnapshotHandle { db }`,
    // so the edge relation must be retrievable from `&StromaDb` alone. `base::facts::EdgeFacts` is a
    // plain `#[salsa::input]` (not a salsa singleton — `base/facts.rs` is frozen), so there is no
    // `EdgeFacts::get(db)`. We hold the created input's HANDLE here so the closure can read it back via
    // the salsa accessor (`ef.edges(db)`). Faithful to R9 (SCIP still populates the real salsa input).
    edge_facts: Option<crate::base::facts::EdgeFacts>,
}

#[salsa::db]
impl salsa::Database for StromaDb {}

impl StromaDb {
    /// Publish the edge relation into the base: store the created [`crate::base::facts::EdgeFacts`]
    /// input handle so a read session can retrieve it from `&db` alone (the read-engine population
    /// door — R9). Takes `&mut self` (a base advance), mirroring the atomic-publish split.
    pub const fn set_edge_facts(&mut self, edge_facts: crate::base::facts::EdgeFacts) {
        self.edge_facts = Some(edge_facts);
    }

    /// The published edge relation handle, if any has been populated (`None` on a fresh base).
    #[must_use]
    pub const fn edge_facts(&self) -> Option<crate::base::facts::EdgeFacts> {
        self.edge_facts
    }
}

// A read SESSION holds `&db` across detection→field→provenance so all three observe ONE published
// revision. Live-edit concurrency (multi-reader + background-writer) is salsa's snapshot/fork
// facility (an ADR-068 cadence choice); the serialized `&`/`&mut` model is sufficient for the
// batch cadence this uses.
