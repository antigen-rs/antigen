//! The efferent bridge — the stroma's first ACT: emit a `Finding` from a sensed structural risk.
//!
//! This is the first place the afferent wing (the stroma's sensing) connects to the efferent wing
//! (antigen's `Finding` pipeline — render / audit / route). The whole efferent pipeline consumes
//! `Vec<Finding>` and is source-agnostic (a `Finding` carries `severity` + `file:line` + a
//! `FindingBody`; it does not know whether it came from the syntactic scanner or here), so the
//! pipeline **lifts for free** — the only missing piece was that the stroma never learned to *speak*
//! `Finding`. This module is that bridge.
//!
//! # What it senses — "risky-to-change" (a `presents(risk)`, not a defect)
//!
//! A node is risky-to-change when structural signals converge on it:
//! - **high construct-in-degree** — many sites build this shape, so changing its *shape* is high-blast;
//! - **high call-blast** — it is on a path that reaches much, so changing its *behavior* propagates.
//!
//! A node high on **both** is a *depended-on shape on a hot path* — you can neither freeze it nor
//! move it safely. The marker's magnitude is set by how many independent signals converge
//! (corroboration: one → `Aura`, both → `Dread`) and it carries its *reason* in the `trigger` (a
//! principled-undefined: liftable because it names WHY). It is a **suggested** marker, surfaced —
//! never written into source (mirror `propose`: observe, don't mutate).
//!
//! Held, not gated: these are `MarkedUnknown` findings; nothing here gates a build. The efferent
//! router (the cytokine-signaling organ) decides consequence from severity × the user's setpoint;
//! this bridge only produces the finding.

use antigen::finding::{
    ExistenceCertainty, Finding, FindingBody, Magnitude, OriginStage, Presentation, Provenance,
    Severity, cluster_key_of,
};

/// One node's risk signals + its source location — the input the bridge grades into a `Finding`.
#[derive(Debug, Clone)]
pub struct NodeSignal {
    /// The node's symbol path (for the trigger text).
    pub path: String,
    /// How many distinct sites CONSTRUCT this node (Construct-edge in-degree).
    pub construct_in: usize,
    /// The node's downstream call-blast (forward reachability size over Call edges).
    pub call_blast: usize,
    /// The node's definition file (SCIP definition occurrence) — the marker's site.
    pub file: String,
    /// The node's definition line.
    pub line: usize,
}

/// The class name every risky-to-change marker carries.
const CLASS: &str = "risky-to-change";

/// Grade each node's converging structural risk-signals into a `MarkedUnknown` `Finding`.
///
/// A node clearing **both** thresholds is `Dread`/`High` (a depended-on shape on a hot path); a node
/// clearing **one** is `Aura`/`Medium` (a single faint signal); a node clearing **neither** yields
/// no finding. The magnitude escalation on convergence is the corroboration law applied to markers
/// (independent signals raise the grade). `existence_certainty` is `Unsure` — a forward heuristic,
/// not a named present defect.
#[must_use]
pub fn risky_to_change_findings(
    nodes: &[NodeSignal],
    construct_hi: usize,
    blast_hi: usize,
) -> Vec<Finding> {
    let mut out = Vec::new();
    for n in nodes {
        let hi_construct = n.construct_in >= construct_hi && construct_hi > 0;
        let hi_blast = n.call_blast >= blast_hi && blast_hi > 0;
        let signals = usize::from(hi_construct) + usize::from(hi_blast);
        if signals == 0 {
            continue;
        }

        // Magnitude/severity escalate with convergence (corroboration): both → Dread, one → Aura.
        let (magnitude, severity) = if signals >= 2 {
            (Magnitude::Dread, Severity::High)
        } else {
            (Magnitude::Aura, Severity::Medium)
        };

        // The trigger NAMES the reason (principled — liftable). "risky IF changed", the honest
        // conditional: this is a hazard on the change-action, not a present defect.
        let trigger = match (hi_construct, hi_blast) {
            (true, true) => format!(
                "high construct-in-degree ({}) AND high call-blast ({}): a depended-on shape on a hot path — risky to change (shape OR behavior is wide-blast)",
                n.construct_in, n.call_blast
            ),
            (true, false) => format!(
                "high construct-in-degree ({}): a widely-constructed shape — risky to change its SHAPE",
                n.construct_in
            ),
            (false, true) => format!(
                "high call-blast ({}): on a hot path — risky to change its BEHAVIOR",
                n.call_blast
            ),
            (false, false) => unreachable!("signals > 0 guarantees at least one is high"),
        };

        out.push(Finding {
            schema_version: antigen::finding::FINDING_SCHEMA_VERSION,
            file: n.file.clone(),
            line: n.line,
            structural_digest: String::new(),
            shape_digest: String::new(),
            cluster_key: cluster_key_of("", CLASS),
            severity,
            // Honest provenance: a heuristic tooling emission, the lowest class tier. And the source
            // string carries the precise origin (OriginStage has only Scan/Audit today — a future
            // stroma-organ variant is the honest home; Scan is the least-wrong coarse label).
            source: "stroma:risky-to-change".to_string(),
            class_provenance: Provenance::Imagined,
            presentation: Presentation::Passive,
            timestamp: 0, // the emit-time stamp is the pipeline's job; the bridge produces content
            origin_stage: OriginStage::Scan,
            body: FindingBody::MarkedUnknown {
                magnitude,
                existence_certainty: ExistenceCertainty::Unsure,
                trigger,
            },
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(path: &str, construct_in: usize, call_blast: usize) -> NodeSignal {
        NodeSignal {
            path: path.to_string(),
            construct_in,
            call_blast,
            file: "src/lib.rs".to_string(),
            line: 1,
        }
    }

    /// A node high on BOTH signals is the fight-quadrant → Dread/High, and the finding is a
    /// `MarkedUnknown` that names both reasons (it flows through the existing `Finding` pipeline).
    #[test]
    fn both_signals_high_is_a_dread_marker_naming_both() {
        let f = risky_to_change_findings(&[node("Foo", 400, 700)], 100, 100);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, Severity::High);
        let FindingBody::MarkedUnknown {
            magnitude, trigger, ..
        } = &f[0].body
        else {
            panic!("expected a MarkedUnknown");
        };
        assert_eq!(*magnitude, Magnitude::Dread);
        assert!(trigger.contains("construct-in-degree") && trigger.contains("call-blast"));
    }

    /// A single high signal is a faint Aura/Medium (corroboration: one signal is low magnitude), and
    /// it names only the signal that fired.
    #[test]
    fn one_signal_high_is_an_aura_marker() {
        let construct_only = risky_to_change_findings(&[node("Bar", 400, 3)], 100, 100);
        assert_eq!(construct_only.len(), 1);
        assert_eq!(construct_only[0].severity, Severity::Medium);
        let FindingBody::MarkedUnknown {
            magnitude, trigger, ..
        } = &construct_only[0].body
        else {
            panic!("expected a MarkedUnknown");
        };
        assert_eq!(*magnitude, Magnitude::Aura);
        assert!(trigger.contains("SHAPE") && !trigger.contains("call-blast"));
    }

    /// A node high on neither yields NO finding — the marker fires only on a real risk-signal.
    #[test]
    fn neither_signal_high_yields_nothing() {
        assert!(risky_to_change_findings(&[node("Baz", 3, 3)], 100, 100).is_empty());
    }

    /// The emitted finding serializes into the pipeline's `Finding` JSON contract (the proof the
    /// stroma now speaks Finding — the efferent lift works end-to-end).
    #[test]
    fn the_finding_serializes_into_the_pipeline_contract() {
        let f = risky_to_change_findings(&[node("Foo", 400, 700)], 100, 100);
        let json = serde_json::to_string(&f[0]).expect("Finding serializes");
        assert!(json.contains("\"kind\":\"marked-unknown\""));
        assert!(json.contains("\"severity\":\"high\""));
        let back: Finding = serde_json::from_str(&json).expect("round-trips");
        assert_eq!(back, f[0]);
    }
}
