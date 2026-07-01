//! The relational base: facts as salsa inputs, ONE input per relation.

pub mod facts;

pub use facts::{ContractFact, ContractFacts, EdgeFact, EdgeFacts, EdgeKind, NodeFact, NodeFacts};
