// Scan fixture for the P0a degenerate / non-canonical-item-position mark (TEST 4).
// Parsed-as-text.
//
// Per the scout's refinement: a #[dread] on an ENUM VARIANT shares the enclosing
// ENUM's structural_digest (a PRINCIPLED stand-in: a variant has no independent
// structural digest of its own — parse.rs:1611-1613 design comment). The required
// behavior the test pins: the variant mark's digest is NON-EMPTY (the enclosing
// enum's digest), and it does NOT silently re-collapse into the empty "dread@"
// bucket.
//
// Two marked variants of ONE enum: under the enum-granular stand-in they share the
// enclosing enum's digest. The test asserts the digest is non-empty AND principled
// (not the empty-string sentinel that the over-merge bug produced).

enum SignalKind {
    #[dread(trigger = "this variant is reachable on the cache-miss path with no rate limit; smells unbounded")]
    Unbounded,

    #[dread(trigger = "this variant carries a raw pointer across an await; the Send bound is hand-asserted")]
    RawAcrossAwait,

    Plain,
}
