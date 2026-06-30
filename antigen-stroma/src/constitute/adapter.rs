//! STEP 4 — the `ScanReport -> NodeFacts/EdgeFacts` lowering (the integration seam).
//!
//! The existing scan (`antigen::scan::scan_workspace -> ScanReport`) is the input feeder. This
//! adapter lowers `ScanReport`'s per-attribute Vecs (presentations, immunities, ...) into the keyed
//! relational base. The SCIP ingestion (resolved edges) is a SECOND feeder into `EdgeFacts`,
//! tier-stamped resolved.
//!
//! CONVERGE-WAVE FLAG (do not let this go silent): does `ScanReport` stay the wire format with the
//! stroma INDUCED from it, or does the stroma become primary and `ScanReport` a projection? The
//! implementer's lean: stroma primary, `ScanReport` a backward-compatible projection (the genome's
//! "`ScanReport` is an induced view" reading). This is a converge-wave call — named here so the
//! builder doesn't decide it implicitly.

use crate::base::facts::{EdgeFacts, NodeFacts};

/// Lower a scan report into the base fact tables. **STUB — fill (frame epoch).**
///
/// Each per-attribute record (which today carries its own `item_target: ItemTarget` +
/// `structural_fingerprint: String`) becomes an EDGE/ATTRIBUTE on a `StromaNodeId`-keyed base node.
/// The migration is additive — the frame is the new base; `ScanReport` projects onto it.
pub fn lower_scan_report(_report: &antigen::scan::ScanReport) -> (NodeFacts, EdgeFacts) {
    todo!("frame epoch: ScanReport per-attribute Vecs -> StromaNodeId-keyed NodeFacts/EdgeFacts")
}
