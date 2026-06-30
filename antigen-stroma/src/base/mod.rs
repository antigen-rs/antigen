//! STEP 3 — the relational base: facts as salsa inputs, ONE input per relation.

pub mod facts;

pub use facts::{ContractFacts, EdgeFact, EdgeFacts, EdgeKind, NodeFacts};
