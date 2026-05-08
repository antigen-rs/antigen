//! Two-pointer glob matcher for `name = matches("...")` operator.
//!
//! Per scout's ADR-006 (recognition-not-design) analysis: a 20-line bespoke
//! matcher covers the v1 operator set (`*` any, `?` one, exact). When A4+
//! adds character classes or multi-pattern matching, swap to `globset`. Not
//! before.

/// Match a glob pattern against an identifier name.
///
/// Metachars: `*` matches any (possibly empty) substring; `?` matches
/// exactly one character. All other characters match literally.
/// Case-sensitive. The match is whole-name (anchored at both ends).
///
/// # Examples
///
/// ```
/// use antigen_fingerprint::glob_match_ident;
///
/// assert!(glob_match_ident("*", "anything"));
/// assert!(glob_match_ident("Foo", "Foo"));
/// assert!(glob_match_ident("*Foo*", "BarFooBaz"));
/// assert!(glob_match_ident("F?o", "Foo"));
/// assert!(!glob_match_ident("F?o", "Fo"));
/// assert!(!glob_match_ident("Foo", "Bar"));
/// ```
#[must_use]
pub fn glob_match_ident(pattern: &str, name: &str) -> bool {
    // Two-pointer with backtracking: standard wildcard match.
    // O(p * n) worst case; fine for identifier-length inputs.
    let p_bytes = pattern.as_bytes();
    let n_bytes = name.as_bytes();
    let mut p = 0usize; // index into pattern
    let mut n = 0usize; // index into name
    let mut star_p: Option<usize> = None; // last position of `*` in pattern
    let mut star_n: usize = 0; // matching position in name when `*` was seen

    while n < n_bytes.len() {
        if p < p_bytes.len() && (p_bytes[p] == b'?' || p_bytes[p] == n_bytes[n]) {
            p += 1;
            n += 1;
        } else if p < p_bytes.len() && p_bytes[p] == b'*' {
            star_p = Some(p);
            star_n = n;
            p += 1;
        } else if let Some(sp) = star_p {
            // Backtrack: advance the substring matched by `*` by one.
            p = sp + 1;
            star_n += 1;
            n = star_n;
        } else {
            return false;
        }
    }
    // Trailing `*`s in the pattern are fine.
    while p < p_bytes.len() && p_bytes[p] == b'*' {
        p += 1;
    }
    p == p_bytes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn star_matches_anything() {
        assert!(glob_match_ident("*", ""));
        assert!(glob_match_ident("*", "x"));
        assert!(glob_match_ident("*", "anything_at_all_42"));
    }

    #[test]
    fn empty_pattern_matches_only_empty() {
        assert!(glob_match_ident("", ""));
        assert!(!glob_match_ident("", "x"));
    }

    #[test]
    fn exact_match() {
        assert!(glob_match_ident("Foo", "Foo"));
        assert!(!glob_match_ident("Foo", "Bar"));
        assert!(!glob_match_ident("Foo", "Foox"));
        assert!(!glob_match_ident("Foo", "xFoo"));
    }

    #[test]
    fn question_matches_one() {
        assert!(glob_match_ident("F?o", "Foo"));
        assert!(glob_match_ident("F?o", "Fxo"));
        assert!(!glob_match_ident("F?o", "Fo"));
        assert!(!glob_match_ident("F?o", "Foox"));
    }

    #[test]
    fn star_prefix() {
        assert!(glob_match_ident("*Class", "FooClass"));
        assert!(glob_match_ident("*Class", "Class"));
        assert!(!glob_match_ident("*Class", "ClassFoo"));
    }

    #[test]
    fn star_suffix() {
        assert!(glob_match_ident("Test*", "TestFoo"));
        assert!(glob_match_ident("Test*", "Test"));
        assert!(!glob_match_ident("Test*", "MyTest"));
    }

    #[test]
    fn star_middle() {
        assert!(glob_match_ident("F*o", "Foo"));
        assert!(glob_match_ident("F*o", "Fxxxo"));
        assert!(glob_match_ident("F*o", "Fo"));
        assert!(!glob_match_ident("F*o", "Bar"));
    }

    #[test]
    fn star_both() {
        assert!(glob_match_ident("*Foo*", "PrefixFooSuffix"));
        assert!(glob_match_ident("*Foo*", "Foo"));
        assert!(glob_match_ident("*Foo*", "FooSuffix"));
        assert!(glob_match_ident("*Foo*", "PrefixFoo"));
        assert!(!glob_match_ident("*Foo*", "Far"));
    }

    #[test]
    fn multiple_stars() {
        assert!(glob_match_ident("**", "anything"));
        assert!(glob_match_ident("a*b*c", "abc"));
        assert!(glob_match_ident("a*b*c", "axxbxxc"));
        assert!(!glob_match_ident("a*b*c", "abxx"));
    }

    #[test]
    fn case_sensitive() {
        assert!(!glob_match_ident("Foo", "foo"));
        assert!(!glob_match_ident("foo", "Foo"));
    }

    #[test]
    fn star_with_question() {
        assert!(glob_match_ident("*?Bar", "FooBar"));
        assert!(!glob_match_ident("*?Bar", "Bar"));
    }
}

#[cfg(test)]
mod props {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn star_alone_matches_any_non_empty(name in "[A-Za-z][A-Za-z0-9_]{0,16}") {
            prop_assert!(glob_match_ident("*", &name));
        }

        #[test]
        fn exact_self_match(name in "[A-Za-z][A-Za-z0-9_]{0,16}") {
            prop_assert!(glob_match_ident(&name, &name));
        }

        #[test]
        fn empty_pattern_rejects_non_empty(name in "[A-Za-z][A-Za-z0-9_]{0,16}") {
            prop_assert!(!glob_match_ident("", &name));
        }

        #[test]
        fn star_prefix_matches_suffix(suffix in "[A-Za-z][A-Za-z0-9_]{0,8}", prefix in "[A-Za-z0-9_]{0,8}") {
            // Pattern `*<suffix>` matches `<prefix><suffix>` for any prefix.
            let pat = format!("*{suffix}");
            let name = format!("{prefix}{suffix}");
            prop_assert!(glob_match_ident(&pat, &name));
        }

        #[test]
        fn star_both_matches_contains(needle in "[A-Za-z][A-Za-z0-9_]{0,6}", pre in "[A-Za-z0-9_]{0,6}", suf in "[A-Za-z0-9_]{0,6}") {
            let pat = format!("*{needle}*");
            let name = format!("{pre}{needle}{suf}");
            prop_assert!(glob_match_ident(&pat, &name));
        }
    }
}
