// ATK-FRAME-TORN-READ · NEGATIVE CONTROL (must COMPILE) — a correctly-sequenced read session.
//
// Proves the type-state is not a blanket ban: a read session that holds `&db` across
// detection+field+provenance, then DROPS, then publishes, compiles cleanly and observes ONE revision.
// If this failed to compile, the &db/&mut db model would be unusable (it would forbid all publishing).

use antigen_stroma::db::StromaDb;
use antigen_stroma::read::answer::SnapshotHandle;

fn publish(_db: &mut StromaDb) {}

fn read_through<'db>(_snap: &SnapshotHandle<'db>) -> u32 {
    0
}

fn main() {
    let mut db = StromaDb::default();

    // A read session over ONE published revision — all three queries observe the same version.
    {
        let snap = SnapshotHandle { db: &db };
        let _detection = read_through(&snap);
        let _field = read_through(&snap);
        let _provenance = read_through(&snap);
        // snap drops here — the `&db` borrow ends.
    }

    // Now publishing is allowed: the session is over, `&mut db` is free. The NEXT session sees R+1.
    publish(&mut db);

    {
        let snap = SnapshotHandle { db: &db };
        let _next = read_through(&snap);
    }
}
