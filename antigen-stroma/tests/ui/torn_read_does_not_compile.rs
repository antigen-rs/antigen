// ATK-FRAME-TORN-READ (the gem, ADR-070 §3.5) — a torn read MUST be a COMPILE error.
//
// The atomic-publish keystone falls out of salsa + Rust's borrow rules FOR FREE: a read session holds
// `&db` across a multi-query detection walk; publishing (advancing the base) needs `&mut db`. While
// any `&db` borrow is live, the borrow checker EXCLUDES `&mut db` — so observing a half-published base
// is unconstructible, not a runtime lock (T6 becomes a compile-time impossibility).
//
// THIS FIXTURE: open a read session (hold `&db`), then attempt to publish (`&mut db`) WHILE the
// session is still live, then keep reading through the session. The publish-in-the-middle is the torn
// read. It MUST fail to compile (cannot borrow `db` mutably while an immutable borrow is held).
//
// BORN-RED: until `StromaDb` + a `&db`-holding `SnapshotHandle` + a `&mut db` publish path exist, this
// either fails to compile for the WRONG reason (missing items) or compiles. The blessed `.stderr` is
// the E0502 borrow-conflict. When it shows E0502 (cannot borrow `*db` as mutable ... also borrowed as
// immutable), the torn-read guarantee is proven.

use antigen_stroma::db::StromaDb;
use antigen_stroma::read::answer::SnapshotHandle;

// A stand-in publish path: advancing the base requires `&mut db`. (The real publish is the
// maintenance `set_*` path; any `&mut db` consumer reproduces the borrow conflict.)
fn publish(_db: &mut StromaDb) {}

// A stand-in read over a held session (borrows `&db` for the session's lifetime).
fn read_through<'db>(_snap: &SnapshotHandle<'db>) -> u32 {
    0
}

fn main() {
    let mut db = StromaDb::default();

    // Open a read session — holds `&db` alive across the detection walk.
    let snap = SnapshotHandle { db: &db };

    let _first = read_through(&snap); // detection: observes revision R

    // TORN READ: publish (advance the base) WHILE the session is still live. `&mut db` cannot coexist
    // with the outstanding `&db` borrow held by `snap`. This line MUST NOT compile.
    publish(&mut db);

    let _second = read_through(&snap); // would observe a DIFFERENT revision — the torn read
}
