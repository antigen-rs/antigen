//! # `dead_code` — the first F-axis producer: rustc's dead-code lint as witnessed absence.
//!
//! Parses `cargo check --message-format=json` output into [`AbsenceFact`]s carrying
//! [`AbsenceWitness::DeadCode`]: an item the compiler reports *declared but never used* is a witnessed
//! absence-of-use — clean F (unlike the noisy syn-scip bare-name conflict the phantom census ruled out).
//!
//! **Pure string→data (ADR-002 library purity).** This parses the JSON the CLI/example captures from the
//! `cargo` subprocess; the library never shells out itself (the same boundary as `learn::szz`, which
//! takes already-read commit data). The subject is the diagnostic's primary-span `file:line` — the
//! instrument's own identity, held verbatim (the [`AbsenceFact`] contract); joining it to a graph node is
//! an organ's job.
//!
//! Note: a disciplined, warnings-denied codebase (antigen itself) emits *zero* `dead_code` diagnostics,
//! so this producer's yield is a corpus signal, not a self-dogfood one.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::base::facts::{AbsenceFact, AbsenceWitness};

/// Parse `cargo check --message-format=json` output (NDJSON — one JSON object per line) into the
/// dead-code absences it reports.
///
/// Non-compiler-message lines, non-`dead_code` diagnostics, and unparseable lines are skipped; a
/// `dead_code` message with no locatable primary span is skipped (not panicked).
#[must_use]
pub fn dead_code_absences(cargo_json: &str) -> Vec<AbsenceFact> {
    let mut out = Vec::new();
    for line in cargo_json.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if v.get("reason").and_then(Value::as_str) != Some("compiler-message") {
            continue;
        }
        let message = &v["message"];
        let is_dead_code = message
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(Value::as_str)
            == Some("dead_code");
        if !is_dead_code {
            continue;
        }
        if let Some(subject) = primary_span_subject(message) {
            out.push(AbsenceFact {
                subject,
                witnesses: BTreeSet::from([AbsenceWitness::DeadCode]),
            });
        }
    }
    out
}

/// `file:line` of a diagnostic's primary span — the located dead item.
fn primary_span_subject(message: &Value) -> Option<String> {
    let spans = message.get("spans")?.as_array()?;
    let primary = spans
        .iter()
        .find(|s| s.get("is_primary").and_then(Value::as_bool) == Some(true))?;
    let file = primary.get("file_name")?.as_str()?;
    let line = primary.get("line_start")?.as_u64()?;
    Some(format!("{file}:{line}"))
}

#[cfg(test)]
mod tests {
    use super::dead_code_absences;
    use crate::base::facts::AbsenceWitness;

    /// A real-shaped rustc `dead_code` diagnostic line (the fields this parser reads).
    const DEAD_CODE_LINE: &str = r#"{"reason":"compiler-message","message":{"code":{"code":"dead_code","explanation":null},"level":"warning","message":"function `foo` is never used","spans":[{"file_name":"src/scan.rs","is_primary":true,"line_start":42,"line_end":42,"column_start":4,"column_end":7}]}}"#;

    #[test]
    fn parses_a_dead_code_diagnostic_into_a_deadcode_absence() {
        let out = dead_code_absences(DEAD_CODE_LINE);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].subject, "src/scan.rs:42");
        assert!(out[0].witnesses.contains(&AbsenceWitness::DeadCode));
    }

    #[test]
    fn skips_non_dead_code_messages_artifacts_and_junk() {
        let mixed = "{\"reason\":\"compiler-artifact\",\"package_id\":\"x\"}\n\
             {\"reason\":\"compiler-message\",\"message\":{\"code\":{\"code\":\"unused_variables\"},\"level\":\"warning\",\"message\":\"unused\",\"spans\":[{\"file_name\":\"a.rs\",\"is_primary\":true,\"line_start\":1}]}}\n\
             not json at all";
        assert!(dead_code_absences(mixed).is_empty());
    }

    #[test]
    fn a_dead_code_message_with_no_primary_span_is_skipped_not_panicked() {
        let no_primary = r#"{"reason":"compiler-message","message":{"code":{"code":"dead_code"},"level":"warning","message":"x","spans":[]}}"#;
        assert!(dead_code_absences(no_primary).is_empty());
    }

    #[test]
    fn several_diagnostics_across_lines_all_land() {
        let two = format!("{DEAD_CODE_LINE}\n{DEAD_CODE_LINE}");
        assert_eq!(dead_code_absences(&two).len(), 2);
    }
}
