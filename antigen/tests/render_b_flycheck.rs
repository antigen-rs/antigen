//! Render B — editor-flycheck (`--message-format=json`). Adversarial gates over
//! the SHIPPED `findings_to_cargo_jsonl` / `finding_to_cargo_message`.
//!
//! The flycheck unit tests already cover the core (primary span, never-error,
//! code-is-class, claim-scope note, non-match skipped, newline-delimited,
//! rustc-schema round-trip). THESE gates target what a unit test over a single
//! hand-built finding cannot: an EDITOR consumes this stream, so EVERY line must
//! be independently valid rustc JSON, and a pathological input (a path/class with
//! JSON-special characters) must NOT corrupt the stream — one bad finding silently
//! breaking the whole flycheck parse is the editor-side silent failure.

use antigen::finding::{
    DialTier, FINDING_SCHEMA_VERSION, Finding, FindingBody, OriginStage, Presentation, Provenance,
    Severity, cluster_key_of,
};
use antigen::render::flycheck::{finding_to_cargo_message, findings_to_cargo_jsonl};

fn fp_finding(class: &str, file: &str, line: usize) -> Finding {
    Finding {
        schema_version: FINDING_SCHEMA_VERSION,
        file: file.to_string(),
        line,
        structural_digest: "d".to_string(),
        shape_digest: "d".to_string(),
        cluster_key: cluster_key_of("d", class),
        severity: Severity::High,
        source: "scan:test".to_string(),
        class_provenance: Provenance::Constructable,
        presentation: Presentation::Passive,
        timestamp: 0,
        origin_stage: OriginStage::Scan,
        body: FindingBody::FingerprintMatch {
            class: class.to_string(),
            tier: DialTier::Suspected,
        },
    }
}

fn marked_unknown_finding() -> Finding {
    Finding {
        schema_version: FINDING_SCHEMA_VERSION,
        file: "m.rs".to_string(),
        line: 1,
        structural_digest: String::new(),
        shape_digest: "s".to_string(),
        cluster_key: cluster_key_of("s", "dread"),
        severity: Severity::Medium,
        source: "scan:marked-unknown:dread".to_string(),
        class_provenance: Provenance::Encountered,
        presentation: Presentation::Active,
        timestamp: 0,
        origin_stage: OriginStage::Scan,
        body: FindingBody::MarkedUnknown {
            magnitude: antigen::finding::Magnitude::Dread,
            existence_certainty: antigen::finding::ExistenceCertainty::Unsure,
            trigger: "x".to_string(),
        },
    }
}

// ===========================================================================
// (1) EVERY LINE IS INDEPENDENTLY VALID rustc JSON. An editor parses the stream
//     line-by-line; one un-parseable line silently breaks flycheck. Parse each
//     non-empty line as JSON and assert the rustc envelope shape.
// ===========================================================================

#[test]
fn every_jsonl_line_is_valid_rustc_compiler_message_json() {
    let findings = vec![
        fp_finding("get-unchecked-without-proof", "src/a.rs", 12),
        fp_finding("panic-in-drop", "src/b.rs", 40),
        fp_finding("unbounded-deser", "src/c.rs", 7),
    ];
    let jsonl = findings_to_cargo_jsonl(&findings).expect("serializes");

    let lines: Vec<&str> = jsonl.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 3, "one line per match; got {}", lines.len());

    for line in &lines {
        let v: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line is not valid JSON ({e}): {line}"));
        assert_eq!(
            v["reason"], "compiler-message",
            "each line must carry the cargo `compiler-message` reason: {line}"
        );
        assert_eq!(
            v["message"]["level"], "warning",
            "antigen flycheck diagnostics are warnings, never errors: {line}"
        );
        // a primary span pointing at the finding's file
        let spans = v["message"]["spans"]
            .as_array()
            .expect("spans array present");
        assert!(
            spans.iter().any(|s| s["is_primary"] == true),
            "at least one primary span: {line}"
        );
    }
}

// ===========================================================================
// (2) JSON-INJECTION SAFETY — a path / class with JSON-special characters
//     (quote, backslash, newline) must serialize to VALID JSON. If serde didn't
//     escape it, the line would break the editor's parse and silently drop ALL
//     antigen findings. Prove the line still parses and the path round-trips.
// ===========================================================================

#[test]
fn pathological_path_and_class_serialize_to_valid_json() {
    // A path with a quote, a backslash, and an embedded newline — adversarial but
    // not impossible on odd filesystems / generated sources.
    let nasty_file = "src/we\"ird\\path\nbroken.rs";
    let nasty_class = "class-with-\"quote\"-and-\\slash";
    let findings = vec![fp_finding(nasty_class, nasty_file, 3)];

    let jsonl = findings_to_cargo_jsonl(&findings).expect("serializes even pathological input");
    // The whole output must still be ONE logical JSON line (serde escapes the
    // embedded newline as \n inside the string, so `.lines()` sees one record).
    let lines: Vec<&str> = jsonl.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        lines.len(),
        1,
        "the embedded newline must be ESCAPED inside the JSON string, not emitted \
         as a raw line break that splits the record into two un-parseable halves. \
         got {} lines: {lines:?}",
        lines.len()
    );

    let v: serde_json::Value = serde_json::from_str(lines[0]).unwrap_or_else(|e| {
        panic!(
            "pathological input produced INVALID JSON ({e}): {}",
            lines[0]
        )
    });
    // The file path round-trips exactly through the JSON (serde un-escaped it).
    let span_file = v["message"]["spans"][0]["file_name"]
        .as_str()
        .expect("file_name is a JSON string");
    assert_eq!(
        span_file, nasty_file,
        "the pathological path must round-trip exactly through the JSON escaping"
    );
}

// ===========================================================================
// (3) NON-MATCH FINDINGS EMIT NOTHING — a marked-unknown (or dial-verdict) in the
//     population produces NO flycheck line (the render is the match surface only;
//     a marked-unknown leaking out as a compiler diagnostic would be scope creep).
// ===========================================================================

#[test]
fn marked_unknown_findings_produce_no_flycheck_line() {
    assert!(
        finding_to_cargo_message(&marked_unknown_finding()).is_none(),
        "a marked-unknown finding must NOT become a flycheck compiler-message"
    );

    let mixed = vec![
        fp_finding("real-match", "a.rs", 1),
        marked_unknown_finding(),
        fp_finding("real-match-2", "b.rs", 2),
    ];
    let jsonl = findings_to_cargo_jsonl(&mixed).expect("serializes");
    let lines: Vec<&str> = jsonl.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        lines.len(),
        2,
        "only the two FingerprintMatch findings emit lines; the marked-unknown is \
         skipped. got {} lines",
        lines.len()
    );
}

// ===========================================================================
// (4) EMPTY POPULATION — zero matches produces EMPTY output (the editor parses an
//     empty stream cleanly), never whitespace/garbage.
// ===========================================================================

#[test]
fn empty_population_produces_empty_output() {
    let jsonl = findings_to_cargo_jsonl(&[]).expect("serializes");
    assert!(
        jsonl.is_empty(),
        "an empty finding population must produce EMPTY flycheck output (no blank \
         lines / whitespace the editor would choke on). got {jsonl:?}"
    );

    // A population of ONLY non-matches is also empty output.
    let only_mu = vec![marked_unknown_finding()];
    let jsonl2 = findings_to_cargo_jsonl(&only_mu).expect("serializes");
    assert!(
        jsonl2.is_empty(),
        "a population with no FingerprintMatch findings produces empty output. got {jsonl2:?}"
    );
}
