// Scan fixture for the P0a marked-unknown digest seam (keystone PROPOSE-slice
// input precondition; ADR-045 Amd-1). Parsed-as-text — does NOT compile, does
// not need the macro crate.
//
// Three #[dread]-marked structs:
//   - Alpha (body shape A)
//   - Beta  (body shape B != A)
//   - Gamma (body STRUCTURALLY IDENTICAL to Alpha)
//
// The PROPOSE-slice clusters marks by the ENCLOSING ITEM's structural_digest.
// For that to produce a useful least-general-generalization (not a hole that
// matches everything), digest(Alpha) must == digest(Gamma) and != digest(Beta).
//
// As-shipped (the bug these tests define done against): every marked-unknown's
// structural_digest is "" and cluster_key is "dread@", so all three collapse
// into ONE bucket regardless of structure — defeating the engine.

#[dread(trigger = "this teardown drops the lock guard before the buffer flush; ordering feels wrong")]
struct Alpha {
    guard: LockGuard,
    buffer: Vec<u8>,
}

#[dread(trigger = "this retry path re-enters the connection pool while a borrow is live; aliasing smell")]
struct Beta {
    pool: ConnectionPool,
    deadline: Instant,
    attempts: u32,
}

// Gamma is structurally identical to Alpha (same field names, same field types,
// same order). Its enclosing-item structural_digest MUST equal Alpha's.
#[dread(trigger = "this teardown also drops the lock guard before the buffer flush; same shape as Alpha")]
struct Gamma {
    guard: LockGuard,
    buffer: Vec<u8>,
}
