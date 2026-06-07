// diff-native DETECT fixture — the BEFORE (HEAD~1) version. Parsed-as-text.
//
// `validate` has a bounds guard. The matching AFTER version removes the guard.
// The (name, structural_digest) set-diff must surface that `validate`'s structure
// changed. A `helper` fn is present unchanged in all three versions to prove the
// diff does NOT surface items whose structure is identical.

pub fn validate(input: &[u8], max: usize) -> Result<(), ()> {
    if input.len() > max {
        return Err(());
    }
    process(input);
    Ok(())
}

pub fn helper(x: u32) -> u32 {
    x.wrapping_add(1)
}

fn process(_input: &[u8]) {}
