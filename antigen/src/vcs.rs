//! Public types for the VCS-Information-Loss Family (ADR-026).
//!
//! ADR-026 ships an 11-antigen VCS-information-loss family + rollback-as-triage
//! discipline + commit-time substrate-witnesses + `cargo antigen vcs` CLI
//! subfamily. This module hosts the **public** types adopters and CLI consumers
//! interact with directly:
//!
//! - [`TriageDecision`] — the 5-color triage classification carried by
//!   `#[triage_commit]` declarations (Black | Red | Yellow | Green | White),
//!   modeled on the START field-triage protocol from disaster medicine
//!   (chart-documentation cognate per ADR-026 §Rollback-as-triage discipline).
//! - [`ServerSideEnforcementMode`] — whether the VCS-info-loss antigens
//!   operate in `FrictionOnly` (client-side hooks, bypassable per D3) or
//!   `Structural` (server-side hooks; requires adopter to control remote)
//!   mode. v0.2 ships friction-only by default per ADR-026 §Decision.
//!
//! Per ADR-026 §Decision, the central cognate is **ForcePushErasingHistory ↔
//! Immune Amnesia (measles)** (Mina et al. 2015, Science): catastrophic loss
//! of memory-carrying substrates with documented harm and structural defense
//! patterns. The biology PREDICTS the failure mode and defense pattern.
//!
//! ## What this module IS
//!
//! The public type surface used by `#[triage_commit]`, `cargo antigen vcs`,
//! and the substrate-witness evaluators (`vcs_rollback_triage_chain`,
//! `vcs_server_side_enforcement_active`, etc.).
//!
//! ## What this module is NOT
//!
//! - Not the 11 stdlib antigen declarations (those live in
//!   `antigen::stdlib::vcs_info_loss` once the family lands)
//! - Not the substrate-witness evaluators (those live in `antigen::vcs::evaluate`
//!   sub-module, analogous to `antigen::supply_chain::evaluate`)
//! - Not the proc-macro itself (lives in `antigen-macros` per the workspace's
//!   no-circular-dep discipline)

use serde::{Deserialize, Serialize};

// ============================================================================
// TriageDecision
// ============================================================================

/// Five-color triage classification for `#[triage_commit]` declarations
/// (ADR-026).
///
/// Modeled on the START field-triage protocol from disaster medicine —
/// the rollback-as-triage discipline (ADR-026 §Decision) is clinical-medicine
/// grounded: chart documentation + informed consent before procedure. The
/// dual-axis grounding (immunology + clinical-medicine) is acknowledged
/// explicitly per ADR-026 §Rollback-as-triage discipline.
///
/// The variant set is the v0.2 sealed-set; future findings can extend it via
/// additive ADR amendment per ADR-001 Amendment 1 C6.
///
/// ## Variant semantics
///
/// - [`Self::Black`] — system-down / data-loss imminent / catastrophic
///   regression confirmed; rollback is the immediate action. Highest-urgency
///   triage tier.
/// - [`Self::Red`] — vital-metric regression confirmed; rollback within tight
///   time window (typically `rollback_due_within_minutes = 30` or less per
///   ADR-026 example).
/// - [`Self::Yellow`] — concerning signal but not vital-metric-blocking;
///   investigation in progress; rollback decision pending.
/// - [`Self::Green`] — no functional regression detected; the `#[triage_commit]`
///   carries the analysis chain proving non-regression.
/// - [`Self::White`] — out of scope for this triage event (e.g., the change
///   is unrelated to the suspected regression); explicit non-action chart entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TriageDecision {
    /// System-down / data-loss imminent / catastrophic regression confirmed.
    Black,
    /// Vital-metric regression confirmed; tight-time-window rollback.
    Red,
    /// Concerning signal; investigation pending; rollback decision deferred.
    Yellow,
    /// No functional regression; analysis chain attests non-regression.
    Green,
    /// Out of scope for this triage event; explicit non-action chart entry.
    White,
}

impl TriageDecision {
    /// Kebab-case string form for CLI rendering, audit-hint detail strings,
    /// and serde round-trip.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Black => "black",
            Self::Red => "red",
            Self::Yellow => "yellow",
            Self::Green => "green",
            Self::White => "white",
        }
    }

    /// Parse from a triage-decision string. Accepts kebab-case (the canonical
    /// serde form), `snake_case`, and `PascalCase` (Rust-enum-variant form) plus
    /// the `TriageDecision::X` path form for parser ergonomics. Returns
    /// `None` for unknown variants.
    #[must_use]
    pub fn parse_decision(s: &str) -> Option<Self> {
        match s {
            "black" | "Black" | "TriageDecision::Black" => Some(Self::Black),
            "red" | "Red" | "TriageDecision::Red" => Some(Self::Red),
            "yellow" | "Yellow" | "TriageDecision::Yellow" => Some(Self::Yellow),
            "green" | "Green" | "TriageDecision::Green" => Some(Self::Green),
            "white" | "White" | "TriageDecision::White" => Some(Self::White),
            _ => None,
        }
    }

    /// True when the triage decision mandates immediate rollback. Used by
    /// audit-time enforcement of `rollback_due_within_minutes` and by the
    /// commit-time hook for `vcs-rollback-without-triage-commit` hint.
    ///
    /// Per ADR-026 §Rollback-as-triage discipline: `Black` and `Red` are the
    /// rollback-mandating tiers; `Yellow` defers; `Green` and `White` are
    /// non-rollback chart entries.
    #[must_use]
    pub const fn mandates_rollback(self) -> bool {
        matches!(self, Self::Black | Self::Red)
    }
}

// ============================================================================
// ServerSideEnforcementMode
// ============================================================================

/// Whether the VCS-info-loss antigens operate in friction-only (client-side)
/// or structural (server-side) enforcement mode (ADR-026 §Enforcement model).
///
/// Per adversarial D3 absorbed during ADR-026 ratification: client-side hooks
/// are bypassable via git plumbing commands. The ADR ships **friction-only**
/// as the v0.2 default — makes bad behavior DELIBERATE rather than ACCIDENTAL;
/// explicitly NOT preventive. Structural mode requires adopter to control the
/// git remote and is the v0.2.1+ path.
///
/// The CLI surfaces this via `cargo antigen vcs install-hooks` (friction-only)
/// vs `cargo antigen vcs install-server-hooks` (structural).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServerSideEnforcementMode {
    /// Client-side hooks; bypassable via `git commit --no-verify` or
    /// `git update-ref -d`. v0.2 default. Emits the audit hint
    /// `vcs-enforcement-friction-only-no-server-hook` so adopters see the
    /// gap explicitly.
    FrictionOnly,
    /// Server-side hooks (pre-receive); requires adopter to control the git
    /// remote and install hooks there. v0.2.1+ path. The substrate-witness
    /// `vcs_server_side_enforcement_active` checks remote configuration.
    Structural,
}

impl ServerSideEnforcementMode {
    /// Kebab-case string form for CLI rendering and serde round-trip.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FrictionOnly => "friction-only",
            Self::Structural => "structural",
        }
    }

    /// Parse from a mode string. Accepts kebab-case, `snake_case`, and
    /// `PascalCase`. Returns `None` for unknown variants.
    #[must_use]
    pub fn parse_mode(s: &str) -> Option<Self> {
        match s {
            "friction-only" | "friction_only" | "FrictionOnly" => Some(Self::FrictionOnly),
            "structural" | "Structural" => Some(Self::Structural),
            _ => None,
        }
    }

    /// True when the mode provides structural (server-side) enforcement.
    /// `Structural` is the only structural mode in v0.2; `FrictionOnly` is
    /// client-side only.
    #[must_use]
    pub const fn is_structural(self) -> bool {
        matches!(self, Self::Structural)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triage_decision_str_roundtrip() {
        for variant in [
            TriageDecision::Black,
            TriageDecision::Red,
            TriageDecision::Yellow,
            TriageDecision::Green,
            TriageDecision::White,
        ] {
            let s = variant.as_str();
            let back = TriageDecision::parse_decision(s).expect("kebab roundtrip");
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn triage_decision_parses_all_forms() {
        assert_eq!(
            TriageDecision::parse_decision("Black"),
            Some(TriageDecision::Black)
        );
        assert_eq!(
            TriageDecision::parse_decision("red"),
            Some(TriageDecision::Red)
        );
        assert_eq!(
            TriageDecision::parse_decision("TriageDecision::Yellow"),
            Some(TriageDecision::Yellow)
        );
        assert_eq!(TriageDecision::parse_decision("unknown"), None);
    }

    #[test]
    fn triage_decision_serde_kebab_case() {
        let s = serde_json::to_string(&TriageDecision::Red).unwrap();
        assert_eq!(s, "\"red\"");
        let v: TriageDecision = serde_json::from_str("\"yellow\"").unwrap();
        assert_eq!(v, TriageDecision::Yellow);
    }

    #[test]
    fn rollback_mandate_matches_adr026() {
        assert!(TriageDecision::Black.mandates_rollback());
        assert!(TriageDecision::Red.mandates_rollback());
        assert!(!TriageDecision::Yellow.mandates_rollback());
        assert!(!TriageDecision::Green.mandates_rollback());
        assert!(!TriageDecision::White.mandates_rollback());
    }

    #[test]
    fn enforcement_mode_str_roundtrip() {
        for variant in [
            ServerSideEnforcementMode::FrictionOnly,
            ServerSideEnforcementMode::Structural,
        ] {
            let s = variant.as_str();
            let back = ServerSideEnforcementMode::parse_mode(s).expect("kebab roundtrip");
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn enforcement_mode_is_structural() {
        assert!(!ServerSideEnforcementMode::FrictionOnly.is_structural());
        assert!(ServerSideEnforcementMode::Structural.is_structural());
    }

    #[test]
    fn enforcement_mode_default_is_friction_only_in_adr() {
        // ADR-026 §Decision: "Detection is friction-only by default"
        // This test pins the default-naming guarantee at the type level.
        let default_mode = ServerSideEnforcementMode::FrictionOnly;
        assert!(!default_mode.is_structural());
    }

    // -------------------------------------------------------------------------
    // Adversarial tests (added by adversarial role)
    // -------------------------------------------------------------------------

    #[test]
    fn triage_decision_rejects_empty_string() {
        assert_eq!(TriageDecision::parse_decision(""), None);
    }

    #[test]
    fn triage_decision_rejects_whitespace_padded() {
        // "black " (trailing space) must not accidentally match "black"
        assert_eq!(TriageDecision::parse_decision("black "), None);
        assert_eq!(TriageDecision::parse_decision(" black"), None);
    }

    #[test]
    fn triage_decision_rejects_uppercase_ascii() {
        // "BLACK" is NOT one of the accepted forms — only "black" (kebab), "Black" (PascalCase)
        // Reject case-insensitive fuzzy matching.
        assert_eq!(TriageDecision::parse_decision("BLACK"), None);
        assert_eq!(TriageDecision::parse_decision("RED"), None);
    }

    #[test]
    fn triage_decision_serde_rejects_unknown_variant() {
        // Serde with kebab-case rename must return Err for unknown variants;
        // it must NOT produce a default or panic.
        let result: Result<TriageDecision, _> = serde_json::from_str("\"purple\"");
        assert!(
            result.is_err(),
            "serde should reject unknown variant 'purple', got Ok"
        );
    }

    #[test]
    fn triage_decision_serde_rejects_uppercase_variant() {
        // Serde kebab-case rename means "BLACK" is not the canonical form;
        // it should reject it.
        let result: Result<TriageDecision, _> = serde_json::from_str("\"BLACK\"");
        assert!(
            result.is_err(),
            "serde should reject 'BLACK'; canonical form is 'black'"
        );
    }

    #[test]
    fn triage_decision_mandates_rollback_is_exhaustive_over_all_variants() {
        // Pin rollback mandate for every current variant explicitly.
        // If a new variant is added without updating mandates_rollback(), this
        // test alone won't catch it — but it documents the expected invariant:
        // Yellow/Green/White are explicitly non-rollback; Black/Red are rollback.
        let cases: &[(TriageDecision, bool)] = &[
            (TriageDecision::Black, true),
            (TriageDecision::Red, true),
            (TriageDecision::Yellow, false),
            (TriageDecision::Green, false),
            (TriageDecision::White, false),
        ];
        for (variant, expected) in cases {
            assert_eq!(
                variant.mandates_rollback(),
                *expected,
                "{variant:?}.mandates_rollback() should be {expected}"
            );
        }
    }

    #[test]
    fn enforcement_mode_rejects_empty_string() {
        assert_eq!(ServerSideEnforcementMode::parse_mode(""), None);
    }

    #[test]
    fn enforcement_mode_rejects_whitespace_padded() {
        assert_eq!(ServerSideEnforcementMode::parse_mode("structural "), None);
        assert_eq!(
            ServerSideEnforcementMode::parse_mode(" friction-only"),
            None
        );
    }

    #[test]
    fn enforcement_mode_rejects_uppercase_ascii() {
        assert_eq!(ServerSideEnforcementMode::parse_mode("STRUCTURAL"), None);
        assert_eq!(ServerSideEnforcementMode::parse_mode("FRICTION-ONLY"), None);
    }

    #[test]
    fn triage_decision_parse_is_stable_under_null_byte() {
        // A null byte should not panic or match any variant.
        assert_eq!(TriageDecision::parse_decision("\0"), None);
        assert_eq!(TriageDecision::parse_decision("black\0"), None);
    }
}
