//! Render B — **editor-flycheck** rustc-JSON serializer (v0.4 ADR-043 §E).
//!
//! Emits each [`Finding`] in the **cargo/rustc `--message-format=json` shape** so
//! an editor's flycheck consumes antigen findings as if they were compiler
//! diagnostics — with **no custom LSP server**. The integration is a one-liner in
//! the editor: point rust-analyzer's `check.overrideCommand` at
//! `cargo antigen scan --message-format json`, and antigen's matches surface as
//! inline squiggles next to rustc's own.
//!
//! This is deliberately **NOT** the existing `OutputFormat::Json` (which emits
//! antigen's own report envelope). rust-analyzer parses the *cargo* line-protocol
//! — newline-delimited JSON objects, each a
//! `{"reason":"compiler-message","message":{…rustc Diagnostic…}}` — so this
//! render matches that wire shape exactly. We model the schema with serde structs
//! rather than depend on the `cargo_metadata` crate: the format is stable and we
//! only *produce* it, so a heavy dependency would buy nothing (compose, don't
//! compete — ADR-002).
//!
//! # Claim-scope (ADR-044)
//!
//! A flycheck diagnostic is a re-presentation of a fingerprint match — a
//! scan-fact. It is emitted at `level: "warning"` (never `"error"`): antigen
//! does not fail the build, and a fingerprint match is a candidate to inspect,
//! never an audited verdict. The message text states it is a fingerprint match
//! and names the provenance tier, so the editor surface never reads as "antigen
//! proved this is a defect" or "this is audited / defended".

use serde::{Deserialize, Serialize};

use crate::finding::{Finding, FindingBody, Provenance};

/// One line of the cargo `--message-format=json` protocol: a
/// `compiler-message`-reason envelope wrapping a rustc [`Diagnostic`].
///
/// rust-analyzer's flycheck reads a stream of these (newline-delimited) from the
/// `check.overrideCommand`. Only the `compiler-message` reason carries a
/// diagnostic; other cargo reasons (`compiler-artifact`, `build-finished`, …) are
/// ignored by the flycheck for diagnostics, so we emit only this one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoMessage {
    /// Always `"compiler-message"` for a diagnostic line.
    pub reason: String,
    /// The rustc diagnostic payload.
    pub message: Diagnostic,
}

/// A rustc diagnostic (the subset rust-analyzer's flycheck reads). Field names
/// match the rustc JSON schema exactly so the editor parses it unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// The human-readable message line.
    pub message: String,
    /// Diagnostic level — `"warning"` for antigen (never `"error"`: antigen does
    /// not fail the build, and a match is a candidate, not an audited verdict).
    pub level: String,
    /// An optional diagnostic code (e.g. the antigen class name) — rendered by
    /// the editor as the lint code. `None` serializes as JSON `null`, matching
    /// rustc's schema.
    pub code: Option<DiagnosticCode>,
    /// The source spans this diagnostic points at (at least one, `is_primary`).
    pub spans: Vec<DiagnosticSpan>,
    /// Sub-diagnostics (notes/helps). Antigen attaches one note naming the
    /// provenance tier so the claim-scope is visible inline.
    pub children: Vec<Self>,
    /// The fully-rendered diagnostic text (rust-analyzer prefers this when
    /// present for the hover/problems-panel rendering).
    pub rendered: Option<String>,
}

/// A rustc diagnostic code (`{ "code": "...", "explanation": null }`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticCode {
    /// The lint/diagnostic code — here, the antigen class name.
    pub code: String,
    /// rustc carries an optional long explanation; antigen has none.
    pub explanation: Option<String>,
}

/// A rustc diagnostic span. The flycheck keys off `file_name` + the 1-based
/// line/column range; `is_primary` marks the main span.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticSpan {
    /// The source file path (as the scan recorded it).
    pub file_name: String,
    /// 1-based start line.
    pub line_start: usize,
    /// 1-based end line.
    pub line_end: usize,
    /// 1-based start column.
    pub column_start: usize,
    /// 1-based end column.
    pub column_end: usize,
    /// Whether this is the primary span of the diagnostic.
    pub is_primary: bool,
}

/// Serialize one [`Finding`] into a cargo `compiler-message` envelope.
///
/// Returns `None` for a finding that is not a fingerprint match (the flycheck
/// render is the match surface only; marked-unknown / dial-verdict findings are
/// out of scope here).
#[must_use]
pub fn finding_to_cargo_message(finding: &Finding) -> Option<CargoMessage> {
    let class = match &finding.body {
        FindingBody::FingerprintMatch { class, .. } => class.clone(),
        FindingBody::DialVerdict { .. } | FindingBody::MarkedUnknown { .. } => return None,
    };

    // The whole-line span: the scan records file + line but not a column range,
    // so we point at the line start (column 1). The editor highlights the line;
    // a precise column range is a future enhancement when the scan carries it.
    let span = DiagnosticSpan {
        file_name: finding.file.clone(),
        line_start: finding.line,
        line_end: finding.line,
        column_start: 1,
        column_end: 1,
        is_primary: true,
    };

    let provenance = provenance_label(finding.class_provenance);
    let message = format!(
        "antigen: structure matches the `{class}` failure-class fingerprint \
         (provenance: {provenance}). This is a fingerprint match to inspect, not an \
         audited verdict."
    );

    // A child "note" makes the claim-scope explicit in the editor's hover.
    let note = Diagnostic {
        message: format!(
            "fingerprint match only — antigen has not audited a defense for this \
             site. Mark it with #[presents({class})] + #[defended_by(...)] to \
             record the defense, or #[antigen_tolerance({class}, rationale=...)] to \
             accept it."
        ),
        level: "note".to_string(),
        code: None,
        spans: Vec::new(),
        children: Vec::new(),
        rendered: None,
    };

    let rendered = format!("warning: {message}");

    let diagnostic = Diagnostic {
        message,
        level: "warning".to_string(),
        code: Some(DiagnosticCode {
            code: format!("antigen::{class}"),
            explanation: None,
        }),
        spans: vec![span],
        children: vec![note],
        rendered: Some(rendered),
    };

    Some(CargoMessage {
        reason: "compiler-message".to_string(),
        message: diagnostic,
    })
}

/// Render a [`Finding`] population as the newline-delimited cargo JSON protocol.
///
/// One `compiler-message` line per fingerprint-match finding — exactly what
/// `cargo antigen scan --message-format json` writes to stdout for
/// rust-analyzer's `check.overrideCommand` to consume.
///
/// # Errors
///
/// Returns a [`serde_json::Error`] if a message fails to serialize (not expected
/// for these plain structs).
pub fn findings_to_cargo_jsonl(findings: &[Finding]) -> Result<String, serde_json::Error> {
    let mut out = String::new();
    for finding in findings {
        if let Some(msg) = finding_to_cargo_message(finding) {
            out.push_str(&serde_json::to_string(&msg)?);
            out.push('\n');
        }
    }
    Ok(out)
}

/// A short human label for a provenance tier, used in the diagnostic message.
const fn provenance_label(p: Provenance) -> &'static str {
    match p {
        Provenance::Encountered => "encountered (seen in real code)",
        Provenance::Constructable => "constructable (a verified minimal case exists)",
        Provenance::Heuristic => "heuristic (correlational)",
        Provenance::Imagined => "imagined (reasoned, no demo yet)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{DialTier, OriginStage, Presentation, Severity, cluster_key_of};

    fn match_finding(class: &str, file: &str, line: usize) -> Finding {
        let digest = format!("d-{class}");
        Finding {
            schema_version: crate::finding::FINDING_SCHEMA_VERSION,
            file: file.to_string(),
            line,
            structural_digest: digest.clone(),
            shape_digest: String::new(),
            cluster_key: cluster_key_of(&digest, class),
            severity: Severity::High,
            source: "scan:catalog-match".to_string(),
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

    #[test]
    fn emits_a_compiler_message_with_a_primary_span() {
        let msg = finding_to_cargo_message(&match_finding("panic-in-drop", "src/a.rs", 12))
            .expect("a fingerprint match yields a message");
        assert_eq!(msg.reason, "compiler-message");
        assert_eq!(msg.message.spans.len(), 1);
        let span = &msg.message.spans[0];
        assert_eq!(span.file_name, "src/a.rs");
        assert_eq!(span.line_start, 12);
        assert!(span.is_primary);
    }

    #[test]
    fn antigen_never_emits_error_level_only_warning() {
        // Claim-scope: antigen does not fail the build; a match is a candidate.
        let msg = finding_to_cargo_message(&match_finding("c", "a.rs", 1)).unwrap();
        assert_eq!(msg.message.level, "warning");
        assert_ne!(msg.message.level, "error");
    }

    #[test]
    fn code_is_the_antigen_class() {
        let msg = finding_to_cargo_message(&match_finding("unbounded-deser", "a.rs", 1)).unwrap();
        let code = msg.message.code.expect("a code");
        assert_eq!(code.code, "antigen::unbounded-deser");
    }

    #[test]
    fn message_names_it_a_fingerprint_match_not_an_audited_verdict() {
        // The editor surface must not read as "audited / proved a defect".
        let msg = finding_to_cargo_message(&match_finding("c", "a.rs", 1)).unwrap();
        let text = &msg.message.message;
        assert!(text.contains("fingerprint match"));
        assert!(text.contains("not an audited verdict"));
    }

    #[test]
    fn non_fingerprint_match_findings_are_skipped() {
        let mut f = match_finding("c", "a.rs", 1);
        f.body = FindingBody::DialVerdict {
            class: "c".to_string(),
            tier: DialTier::Named,
        };
        assert!(finding_to_cargo_message(&f).is_none());
    }

    #[test]
    fn jsonl_is_newline_delimited_one_line_per_match() {
        let findings = vec![match_finding("a", "x.rs", 1), match_finding("b", "y.rs", 2)];
        let jsonl = findings_to_cargo_jsonl(&findings).expect("serializes");
        let lines: Vec<&str> = jsonl.lines().collect();
        assert_eq!(lines.len(), 2);
        // Each line is a standalone valid JSON object (the cargo line-protocol).
        for line in lines {
            let _: CargoMessage = serde_json::from_str(line).expect("each line is valid JSON");
        }
    }

    #[test]
    fn round_trips_through_the_rustc_schema() {
        let msg = finding_to_cargo_message(&match_finding("c", "a.rs", 5)).unwrap();
        let json = serde_json::to_string(&msg).unwrap();
        // The shape rust-analyzer keys off.
        assert!(json.contains("\"reason\":\"compiler-message\""));
        assert!(json.contains("\"is_primary\":true"));
        assert!(json.contains("\"level\":\"warning\""));
        let back: CargoMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
    }
}
