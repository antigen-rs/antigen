//! STEP 2 — node identity + the salsa storage shape.
//!
//! The deepest cut of the frame (aristotle A2/A4/A8): **separate the stable LOCATOR (salsa key) from
//! the IDENTITY (a value on the input).** Same structure as git — path is the stable locator, blob-
//! SHA is the content-identity. An edited item keeps its PATH (locator) and gets a new digest.
//! Conflating them (keying salsa on the digest) is the bug; separating them is the structure git
//! already validates.
//!
//! - [`locator::Locator`] — `#[salsa::interned]`, value-stable, survives body edits. THE KEY.
//! - [`node::Node`] — `#[salsa::input]`, carries the changing digests. Held + mutated-in-place.
//! - [`id::StromaNodeId`] — the SEMANTIC identity (hashing/equality/overlay-anchor/cross-snapshot),
//!   NOT the salsa storage handle.

pub mod cfg;
pub mod digest;
pub mod id;
pub mod locator;
pub mod node;
pub mod path;

pub use cfg::{CfgAtom, CfgSet};
pub use digest::{IdentityDigest, ShapeDigest};
pub use id::StromaNodeId;
pub use locator::Locator;
pub use node::Node;
