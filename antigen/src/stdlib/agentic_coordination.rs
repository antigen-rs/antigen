//! # Agentic-Coordination Failure-Class Family
//!
//! Antigens for failure-classes that emerge specifically in multi-session,
//! multi-agent, or human-LLM-collab development workflows. These classes are
//! rare-to-nonexistent in single-developer single-session work; they emerge
//! when representation (agent context state, coordination substrate, stub
//! API surface) diverges from actual state across session or agent boundaries.
//!
//! Per ADR-007 (structurally-guaranteed-need): the 2026+ development landscape
//! commits to agentic workflows. These failure-classes are structurally
//! guaranteed to recur. Naming them now prevents re-discovery.
//!
//! ## Antigen-category (ADR-028)
//!
//! All antigens in this family are `SubstrateAlignment`: the agent's
//! representation (context state, API surface claim) diverges from actual
//! substrate state. The gap is invisible at runtime — it does not crash,
//! it produces wrong decisions.
//!
//! ## Biology grounding
//!
//! These antigens are grounded in the **generation-inspection asymmetry**
//! framing (project memory entry `project_antigen_compensates_agent_limitations`):
//! agents generate faster than they inspect, and context state is a snapshot
//! that evaporates. The structural memory these antigens encode compensates
//! for the gap between what an agent *thinks* is true (context) and what
//! *is* true (substrate). Biology cognate: immunological memory loss during
//! session gap — the immune system's learned state evaporates; the next
//! encounter requires re-learning rather than recognizing.

use crate::antigen;

// ============================================================================
// 1. AgentWakeWithoutSubstrateDeltaInjection
// ============================================================================

/// An agent resumes (or a new session starts) in a multi-session workflow
/// without reading the substrate delta that accumulated while the agent was
/// idle — producing decisions based on stale context state.
///
/// **The lived failure pattern**:
/// An agent with a compaction summary describing "current state" routes work
/// based on that summary. Meanwhile, other agents committed 3 new families,
/// recorded completed work units, and shipped ATK test bodies. The agent's context says
/// "task X pending" while the substrate says "X shipped at commit abc123."
/// The agent re-does work, routes stale claims, or blocks on gates already
/// cleared.
///
/// **Why it is silent**: the agent does not error. It acts confidently on
/// its summary. The only signal is downstream wrong behavior — re-doing
/// completed work, failing to find "pending" issues because they were already
/// resolved, missing changes that affect current decisions.
///
/// **Category**: `SubstrateAlignment` — the agent's context representation
/// diverges from actual substrate state (git log, coordination state, file system)
/// across the session boundary.
///
/// **Defense at v0.2** (substrate-witness, not enforcement):
/// `git log --oneline -N` + a coordination-state read at session start, BEFORE any
/// routing or task-claiming. The discipline: treat context-held state as
/// hypothesis; substrate-grep confirms or rejects.
///
/// **ADR-002 composition**: this antigen composes with `cargo antigen scan`
/// scanning for inline `// TODO:` and `#[ignore]` markers — sites where
/// stale state commonly anchors wrong decisions.
#[antigen(
    name = "agent-wake-without-substrate-delta-injection",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("substrate-delta")"#,
    family = "agentic-coordination",
    summary = "Agent resumes without reading substrate delta accumulated during idle gap; context state diverges silently from actual substrate, producing stale routing and re-done work.",
    references = ["ADR-007", "ADR-028"]
)]
pub struct AgentWakeWithoutSubstrateDeltaInjection;

// ============================================================================
// 2. DelegateCrossCrateResolutionGap
// ============================================================================

/// A delegate-target resolution mechanism resolves handler references using
/// an intra-crate name index, silently false-positiving on cross-crate
/// delegates as "missing."
///
/// **The structural gap (from mucosal sign-pass)**:
/// `audit_mucosal` builds `handler_kinds: HashMap<&str, HashSet<MucosalKind>>`
/// by indexing `#[mucosal]` declarations from the current `ScanReport`. The
/// index key is the function's local identifier (e.g., `sanitize_user_input`).
/// A delegate that says `handled_by = crate_b::http::sanitize_request` —
/// where `sanitize_request` lives in a different crate — will NOT appear in
/// the index. The audit emits `MucosalDisciplineDelegateTargetMissing`. No
/// error in the declaration; the handler exists; the audit lies.
///
/// **Why it is silent**: the delegate declaration is syntactically correct.
/// The handler is real. The delegate kind-match would succeed if both were
/// in the same crate. The false-positive only appears when adopters split
/// sanitization into a shared library crate — exactly the pattern ADR-027's
/// `#[mucosal_delegate]` is designed to ENCOURAGE (centralized sanitizer).
/// The antigen for centralizing defense produces false-positives at the
/// boundaries it incentivizes.
///
/// **Category**: `SubstrateAlignment` — the resolution index (intra-crate
/// only) diverges from the actual delegate-target graph (cross-crate).
///
/// **Residual risk at v0.2**: cross-crate resolution requires a multi-crate
/// scan pass and is v0.3+ scope. At v0.2, adopters with cross-crate delegates
/// should add a workspace-level `#[mucosal]` re-export stub in the calling
/// crate that delegates to the library handler, satisfying the intra-crate
/// index check. Not ideal; documented here so the failure is explicit.
#[antigen(
    name = "delegate-cross-crate-resolution-gap",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("cross-crate")"#,
    family = "agentic-coordination",
    summary = "Delegate-target resolution uses intra-crate name index; cross-crate handlers produce false MucosalDisciplineDelegateTargetMissing. Incentivizes exactly the centralized-sanitizer pattern it then rejects.",
    references = ["ADR-027", "ADR-027#Amendment-1"]
)]
pub struct DelegateCrossCrateResolutionGap;
