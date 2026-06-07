// diff-native DETECT fixture — the AFTER (HEAD) version: `validate`'s bounds
// guard is REMOVED. Parsed-as-text.
//
// vs diff_native_guard_before: `validate`'s body lost the `if input.len() > max
// { return Err(()) }` guard. Its structural_digest must therefore DIFFER from the
// before version → the set-diff surfaces "validate changed structure" (the
// guard-removal-on-a-PR blind-spot snapshot scanning cannot see).
//
// `helper` is BYTE-IDENTICAL to the before version → it must NOT appear in the
// diff (no phantom churn for an unchanged item).

pub fn validate(input: &[u8], max: usize) -> Result<(), ()> {
    let _ = max;
    process(input);
    Ok(())
}

pub fn helper(x: u32) -> u32 {
    x.wrapping_add(1)
}

fn process(_input: &[u8]) {}
