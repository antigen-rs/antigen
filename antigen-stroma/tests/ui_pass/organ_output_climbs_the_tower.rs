// Negative control for ATK-SIGNAL-B2PRIME (ADR-071 B2′): the SOUND path MUST compile. An organ
// ingesting a raw stroma signal (tier `Base`) and climbing the afferent tower one tier per write-back
// (`Up<Base>`, then `Up<Up<Base>>`) is exactly the read-lower-write-higher DAG the wall permits. A wall
// that rejected everything would be useless; this proves the tier-wall admits the legitimate climb.

use antigen_stroma::signal::{ingest, write_back};

fn main() {
    let t0 = ingest(1u32); // OrganOutput<u32, Base>
    let t1 = write_back(&t0, 2u32); // OrganOutput<u32, Up<Base>>
    let t2 = write_back(&t1, 3u32); // OrganOutput<u32, Up<Up<Base>>>
    assert_eq!(*t2.value(), 3u32);
}
