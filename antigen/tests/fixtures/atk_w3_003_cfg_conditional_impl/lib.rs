// ATK-W3-003 fixture: cfg-conditional impl blocks.
//
// The structural question: if #[presents] is on a cfg(not(test)) impl and
// #[immune] is on a cfg(test) impl-method (or vice versa), does item-identity
// matching correctly handle the cfg-conditional split?
//
// This is a real pattern: production code has the vulnerability; tests have
// the immunity proof. The AST visitor sees both in the same file but they're
// in separate cfg branches.
//
// W3 must either:
// (a) match them regardless of cfg gates (the cfg is irrelevant to item identity), or
// (b) surface a diagnostic: "presents and immune are in different cfg branches"
//
// Option (a) is correct for most cases; option (b) is wrong — it would force
// users to put immune inside cfg(not(test)) which defeats the purpose.

struct CfgSensitiveType {
    data: Vec<u8>,
}

#[cfg(not(test))]
#[presents(PanickingInDrop)]
impl Drop for CfgSensitiveType {
    fn drop(&mut self) {
        // might panic in some conditions
        let _ = self.data.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[immune(PanickingInDrop, witness = cfg_sensitive_drop_test)]
    impl Drop for CfgSensitiveType {
        fn drop(&mut self) {
            // test-mode drop — guaranteed safe
        }
    }

    #[test]
    fn cfg_sensitive_drop_test() {
        let t = CfgSensitiveType { data: vec![1, 2, 3] };
        drop(t);
    }
}
