//! # `verdict` — join the (T, I, F) axes into a per-subject verdict.
//!
//! Organs project; the join IS a projection (nb 028), so it lives in an organ, not the base.
//!
//! The evidence axes are held apart, each its own multi-witness accumulation: **T/I** (presence) on
//! [`EdgeFacts`](crate::base::facts::EdgeFacts) via [`WitnessSource`](crate::base::facts::WitnessSource),
//! **F** (absence) on [`AbsenceFacts`](crate::base::facts::AbsenceFacts) via
//! [`AbsenceWitness`]. A unified `(T, I, F)` verdict for a subject is a
//! JOIN across the axes.
//!
//! ## The first join: the F→T bridge (def-span containment)
//!
//! The two axes key their subjects DIFFERENTLY — F by `file:line` (the dead-code producer's identity),
//! T by symbol (the graph node). [`attribute_absences_to_defs`] bridges them: it maps each absence's
//! `file:line` to the INNERMOST definition span containing it, so an absence becomes a fact ABOUT a node.
//! Since every def-span is a SCIP definition (T-present by construction), a symbol that comes out of the
//! join carrying absences is **defined (T) AND witnessed-absent (F)** — a *defined-but-dead* specimen
//! (`T ∧ F`, which the independent axes permit: strongly-exists-as-a-definition, strongly-not-used).
//!
//! Designed to fuse witness-SETS, not scalars: a subject's F is the *union* of every absence-instrument's
//! witnesses ({`DeadCode`, `NotExecuted`} = a corroborated absence), so a new absence-producer just adds
//! a witness to the set — the join never changes (the open-registry / co-native property).

use std::collections::{BTreeSet, HashMap};

use crate::base::facts::{AbsenceFact, AbsenceWitness};

/// A symbol's definition span — the T-side identity + location an absence's `file:line` joins against.
///
/// Produced by parsing SCIP definition occurrences' `enclosing_range` (an organ's own read of the base).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefSpan {
    /// The node's symbol (its T-side identity).
    pub symbol: String,
    /// The file the definition lives in (normalized to the absence producer's convention by the caller).
    pub file: String,
    /// First line of the definition's enclosing range.
    pub start_line: u32,
    /// Last line of the definition's enclosing range.
    pub end_line: u32,
}

/// Attribute each [`AbsenceFact`] to the INNERMOST [`DefSpan`] whose file + line-range contains its
/// `file:line` subject — the F→T bridge. Returns each symbol's accumulated absence-witness set (its F).
///
/// A symbol in the result is DEFINED (T, by construction — every `DefSpan` is a SCIP definition) AND
/// witnessed-absent (F): a **defined-but-dead** specimen. The witnesses ACCUMULATE across instruments, so
/// a symbol with `{DeadCode, NotExecuted}` is a corroborated F (the corroboration law, F-side). An absence
/// with no containing def (wrong file, or a line outside every span) is dropped — it cannot be attributed
/// to a node, and the base holds the raw absence regardless.
#[must_use]
pub fn attribute_absences_to_defs(
    absences: &[AbsenceFact],
    def_spans: &[DefSpan],
) -> HashMap<String, BTreeSet<AbsenceWitness>> {
    let mut out: HashMap<String, BTreeSet<AbsenceWitness>> = HashMap::new();
    for absence in absences {
        let Some((file, line)) = parse_file_line(&absence.subject) else {
            continue;
        };
        // The innermost containing definition: the smallest line-range that covers the line, same file.
        let containing = def_spans
            .iter()
            .filter(|d| d.file == file && d.start_line <= line && line <= d.end_line)
            .min_by_key(|d| d.end_line - d.start_line);
        if let Some(def) = containing {
            out.entry(def.symbol.clone())
                .or_default()
                .extend(absence.witnesses.iter().copied());
        }
    }
    out
}

/// Parse a `file:line` subject (the dead-code producer's identity) back into its parts. Splits on the
/// LAST colon so a relative path (colon-free for Rust sources) is preserved.
fn parse_file_line(subject: &str) -> Option<(&str, u32)> {
    let (file, line) = subject.rsplit_once(':')?;
    Some((file, line.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{DefSpan, attribute_absences_to_defs};
    use crate::base::facts::{AbsenceFact, AbsenceWitness};

    fn absence(subject: &str, w: AbsenceWitness) -> AbsenceFact {
        AbsenceFact {
            subject: subject.to_string(),
            witnesses: BTreeSet::from([w]),
        }
    }
    fn span(symbol: &str, file: &str, start_line: u32, end_line: u32) -> DefSpan {
        DefSpan {
            symbol: symbol.to_string(),
            file: file.to_string(),
            start_line,
            end_line,
        }
    }

    #[test]
    fn an_absence_joins_to_the_innermost_containing_def() {
        // An enclosing module (1..100) and the fn inside it (40..50); a dead line at 42 belongs to the FN.
        let spans = vec![
            span("outer", "src/lib.rs", 1, 100),
            span("foo", "src/lib.rs", 40, 50),
        ];
        let out = attribute_absences_to_defs(
            &[absence("src/lib.rs:42", AbsenceWitness::DeadCode)],
            &spans,
        );
        assert_eq!(out.len(), 1);
        assert!(out["foo"].contains(&AbsenceWitness::DeadCode)); // innermost wins
        assert!(!out.contains_key("outer"));
    }

    #[test]
    fn two_instruments_on_one_def_accumulate_a_corroborated_f() {
        let spans = vec![span("foo", "a.rs", 1, 10)];
        let out = attribute_absences_to_defs(
            &[
                absence("a.rs:5", AbsenceWitness::DeadCode),
                absence("a.rs:5", AbsenceWitness::NotExecuted),
            ],
            &spans,
        );
        assert_eq!(out["foo"].len(), 2); // {DeadCode, NotExecuted} — corroborated F on the node
    }

    #[test]
    fn an_absence_with_no_containing_def_is_dropped() {
        let spans = vec![span("foo", "a.rs", 1, 10)];
        assert!(
            attribute_absences_to_defs(&[absence("a.rs:999", AbsenceWitness::DeadCode)], &spans)
                .is_empty()
        );
    }

    #[test]
    fn an_absence_in_the_wrong_file_does_not_match() {
        let spans = vec![span("foo", "a.rs", 1, 10)];
        assert!(
            attribute_absences_to_defs(&[absence("b.rs:5", AbsenceWitness::DeadCode)], &spans)
                .is_empty()
        );
    }
}
