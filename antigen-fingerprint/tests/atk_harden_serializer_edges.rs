//! ATK — HARDEN re-attack on the Fingerprint→DSL serializer (ADR-063), fresh eyes
//! that never built it. Default-to-refuted: each test below is an attempt to make
//! `parse(serialize(fp)) == fp` (or the `to_antigen_attr` wrapper) NOT hold.
//!
//! The existing `serialize_roundtrip_proptest.rs` searches the well-formed-fp space
//! with an ASCII-leading generator. These tests aim at the corners that generator's
//! `[A-Za-z]`-leading, ASCII-only, no-`to_antigen_attr` strategy never reaches:
//!   - the `to_antigen_attr` raw-string envelope vs a payload containing `"#`
//!     (the briefing's named "raw-string boundary" hazard);
//!   - Unicode / emoji in string payloads and globs;
//!   - deeply-nested combinator round-trips at ADR depth budget;
//!   - whitespace-leading / control-char payloads that the parser accepts;
//!   - the `has_method` raw-signature path with adversarial spacing AND `"#`.
//!
//! ID-chain: ATK-HARDEN-SER-NN.

use antigen_fingerprint::{Constraint, Fingerprint, to_antigen_attr};

/// Re-parse helper that fails loud with the divergence.
fn assert_round_trips_ast(fp: &Fingerprint, label: &str) {
    let serialized = fp.to_string();
    let reparsed = Fingerprint::parse(&serialized)
        .unwrap_or_else(|e| panic!("[{label}] serialized form must re-parse: {serialized:?}: {e}"));
    assert_eq!(
        fp, &reparsed,
        "[{label}] round-trip diverged:\n  serialized: {serialized}",
    );
}

// ── ATK-HARDEN-SER-01: the to_antigen_attr raw-string envelope vs `"#` ──────────
//
// to_antigen_attr wraps the inner DSL in `r#"...DSL..."#`. A doc_contains payload
// that, once serialized, contains the raw-string CLOSING delimiter `"#` will
// PREMATURELY CLOSE that raw string — the produced attribute text is then NOT a
// single well-formed `#[antigen(fingerprint = r#"..."#)]` token the consumer can
// paste-and-compile. The serializer's own module doc claims (line 173) the raw
// string means inner `"` "need no further escaping" — but `"#` is the one inner
// sequence that DOES break the r#"…"# envelope.
#[test]
fn atk_harden_ser_01_to_antigen_attr_survives_hash_quote_in_payload() {
    // A payload that serializes to text containing `"#`. doc_contains is a free-form
    // leaf (non-empty, non-whitespace) so this fp IS parser-producible.
    let fp = Fingerprint {
        constraints: vec![Constraint::DocContains("ends with \"#".to_string())],
    };
    // First: the inner DSL round-trips (this is expected to hold).
    assert_round_trips_ast(&fp, "SER-01-inner");

    // Now the consumer-facing wrapper. The wrapper's whole job (per its doc) is to
    // produce text that "parses back to the SAME fingerprint the tool computed —
    // paste-and-compile, not paste-and-rewrite". So the wrapped attribute must
    // itself be parseable as a Rust attribute AND its fingerprint literal must
    // re-parse to `fp`. Extract the fingerprint literal back out the way a consumer
    // (the macro) would and re-parse it.
    let attr = to_antigen_attr(&fp);
    // The attribute must tokenize as Rust (the macro path runs it through the
    // compiler's tokenizer). If `"#` closed the raw string early, this is a
    // malformed token stream the consumer cannot paste-and-compile.
    assert_attr_parses(&attr, "ATK-HARDEN-SER-01");
}

/// Parse `#[antigen(fingerprint = r#"..."#)]` the way a consumer's compiler would:
/// as an OUTER attribute. A `"#` that closed the raw string early yields a token
/// stream that does not parse as a single well-formed attribute.
fn assert_attr_parses(attr: &str, label: &str) {
    use syn::parse::Parser;
    let parser = syn::Attribute::parse_outer;
    let result = parser.parse_str(attr);
    let attrs = result.unwrap_or_else(|e| {
        panic!(
            "[{label}]: to_antigen_attr produced text that is not a valid Rust \
             attribute — the `\"#` in the payload broke the r#\"…\"# envelope.\n  \
             attr: {attr}\n  err: {e}"
        )
    });
    assert_eq!(
        attrs.len(),
        1,
        "[{label}]: expected exactly ONE attribute, got {} — the raw-string envelope \
         was broken so the text tokenized as something else.\n  attr: {attr}",
        attrs.len(),
    );
}

// ── ATK-HARDEN-SER-02: has_method signature containing `"#` through the wrapper ──
#[test]
fn atk_harden_ser_02_to_antigen_attr_survives_hash_quote_in_signature() {
    // has_method signature is a free string that must merely tokenize. A signature
    // containing a string literal whose closing produces `"#` stresses the wrapper.
    // Build a signature that tokenizes (a doc-ish default arg) yet carries `"#`.
    // Simpler: doc payload already covered SER-01; here use attr_present which is a
    // bare free string with NO tokenization requirement.
    let fp = Fingerprint {
        constraints: vec![Constraint::AttrPresent("a\"#b".to_string())],
    };
    assert_round_trips_ast(&fp, "SER-02-inner");
    let attr = to_antigen_attr(&fp);
    assert_attr_parses(&attr, "ATK-HARDEN-SER-02");
}

// ── ATK-HARDEN-SER-03: Unicode / emoji in a free-form payload ───────────────────
#[test]
fn atk_harden_ser_03_unicode_emoji_payload_round_trips() {
    for payload in [
        "café",
        "naïve→meet",
        "日本語のドキュメント",
        "emoji 🦀 ferris 🔥",
        "zero\u{200b}width",    // zero-width space
        "combining e\u{0301}",  // combining acute
        "rtl \u{202e}override", // RTL override (a known render-vs-bytes trap)
    ] {
        let fp = Fingerprint {
            constraints: vec![Constraint::DocContains(payload.to_string())],
        };
        assert_round_trips_ast(&fp, "SER-03");
    }
}

// ── ATK-HARDEN-SER-04: leading/trailing/embedded control chars (parser-producible) ─
#[test]
fn atk_harden_ser_04_control_and_whitespace_in_free_payload() {
    // doc_contains rejects all-whitespace, but accepts a payload with INTERNAL control
    // chars as long as it isn't whitespace-only after trim. These exercise proc_macro2
    // Literal::string ↔ syn::LitStr on the control range.
    for payload in [
        "x\ty",         // embedded tab
        "x\r\ny",       // CRLF
        "x\u{0000}y",   // NUL
        "x\u{0007}y",   // bell
        "x\u{001b}[0m", // ESC + ansi
        " leading-space-but-has-content",
        "trailing-space-but-has-content ",
    ] {
        let fp = Fingerprint {
            constraints: vec![Constraint::DocContains(payload.to_string())],
        };
        // It must EITHER round-trip cleanly OR be rejected by parse — never silently
        // re-parse to a DIFFERENT payload. We assert the round-trip equality directly;
        // a divergence (not a rejection) is the bug we hunt.
        let serialized = fp.to_string();
        match Fingerprint::parse(&serialized) {
            Ok(reparsed) => assert_eq!(
                fp, reparsed,
                "ATK-HARDEN-SER-04: control-char payload re-parsed to a DIFFERENT fp \
                 (silent corruption).\n  payload: {payload:?}\n  serialized: {serialized:?}",
            ),
            Err(e) => panic!(
                "ATK-HARDEN-SER-04: a parser-producible doc_contains payload failed to \
                 re-parse after serialize (round-trip broken, not a scoped-domain reject).\n  \
                 payload: {payload:?}\n  serialized: {serialized:?}\n  err: {e}"
            ),
        }
    }
}

// ── ATK-HARDEN-SER-05: deep nesting at the depth budget round-trips ─────────────
#[test]
fn atk_harden_ser_05_deep_nesting_round_trips() {
    // Build all_of([item=struct, all_of([item=struct, all_of([... not(...) ...])])])
    // up to depth 9 (under MAX_DEPTH=10), each level carrying a positive sibling so
    // `not` placement stays legal, plus a `not` to exercise the negation arm deep down.
    let mut inner = Constraint::AllOf(vec![
        Constraint::Item(antigen_fingerprint::ItemKind::Struct),
        Constraint::Not(Box::new(Constraint::Derives("Eq".to_string()))),
    ]);
    for _ in 0..7 {
        inner = Constraint::AllOf(vec![
            Constraint::Item(antigen_fingerprint::ItemKind::Struct),
            inner,
        ]);
    }
    let fp = Fingerprint {
        constraints: vec![inner],
    };
    // Confirm it actually parses first (within budget), then round-trips.
    assert_round_trips_ast(&fp, "SER-05");
}

// ── ATK-HARDEN-SER-06: has_method adversarial spacing raw round-trip ────────────
#[test]
fn atk_harden_ser_06_has_method_weird_spacing_raw_round_trips() {
    use antigen_fingerprint::MethodPattern;
    // The parser stores the RAW signature; PartialEq compares raw. The serializer must
    // emit the raw bytes so re-parse yields the same raw string. Stress with tabs,
    // newlines, and runs of spaces in the signature.
    for raw in [
        "(&\tmut\tself,\tT)\t->\tU",
        "(  &  mut  self  ,  T  )  ->  U",
        "(\n  &mut self,\n  T,\n) -> U",
        "()", // degenerate empty-arg
    ] {
        let fp = Fingerprint {
            constraints: vec![Constraint::HasMethod(MethodPattern {
                name: "f".to_string(),
                signature: raw.to_string(),
                normalized_signature: None,
            })],
        };
        let serialized = fp.to_string();
        let reparsed = Fingerprint::parse(&serialized).unwrap_or_else(|e| {
            panic!("ATK-HARDEN-SER-06: raw-sig {raw:?} failed to re-parse: {serialized:?}: {e}")
        });
        assert_eq!(
            fp, reparsed,
            "ATK-HARDEN-SER-06: has_method raw signature diverged on round-trip.\n  \
             raw: {raw:?}\n  serialized: {serialized:?}",
        );
    }
}

// ── ATK-HARDEN-SER-07: glob with quote/backslash/unicode metachars round-trips ──
#[test]
fn atk_harden_ser_07_glob_special_chars_round_trip() {
    use antigen_fingerprint::GlobPattern;
    for g in [
        "a\"b", // quote in glob
        "a\\b", // backslash in glob
        "*🦀*", // emoji between wildcards
        "?\t?", // tab between single-char wildcards
        "ends-hash\"#",
    ] {
        let fp = Fingerprint {
            constraints: vec![Constraint::NameMatches(GlobPattern(g.to_string()))],
        };
        let serialized = fp.to_string();
        match Fingerprint::parse(&serialized) {
            Ok(reparsed) => assert_eq!(
                fp, reparsed,
                "ATK-HARDEN-SER-07: glob {g:?} re-parsed to a DIFFERENT fp.\n  serialized: {serialized:?}",
            ),
            Err(e) => panic!(
                "ATK-HARDEN-SER-07: a non-empty glob {g:?} failed round-trip re-parse.\n  \
                 serialized: {serialized:?}\n  err: {e}"
            ),
        }
    }
}
