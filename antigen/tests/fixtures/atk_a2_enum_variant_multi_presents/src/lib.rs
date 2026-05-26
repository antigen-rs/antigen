// Fixture: #[presents] on TWO different enum variants.
// Both variants share the same structural fingerprint (the enclosing enum's).
// When two presentations have the same structural_fingerprint, a consumer
// using fingerprint as a unique key would silently deduplicate or conflate them.
// This test probes whether both presentations appear with distinct identity.

use antigen::presents;

pub struct BoundaryViolation;
pub struct CapabilityEscape;

pub enum RequestKind {
    #[presents(BoundaryViolation)]
    External { payload: Vec<u8> },
    #[presents(CapabilityEscape)]
    Privileged { token: String },
    Internal { payload: Vec<u8> },
}
