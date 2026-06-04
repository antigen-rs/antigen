// Scan fixture for the marked-unknown markers (ADR-041). The scanner reads the
// `#[aura]` / `#[dread]` / `#[red_flag]` attributes directly (source-walk), so
// this file is parsed-as-text — it does not compile or need the macro crate.

#[dread(trigger = "the teardown drops the guard before the flush; the ordering feels wrong")]
fn risky_teardown() {}

#[aura(trigger = "this retry loop has no jitter; under load it might thundering-herd")]
fn retry_request() {}

#[red_flag(trigger = "this auth check is reachable with an empty token on the cache-hit path")]
fn authorize() {}
