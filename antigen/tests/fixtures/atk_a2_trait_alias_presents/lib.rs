#![feature(trait_alias)]
use antigen::presents;
pub struct AliasCapabilityLeak;

#[presents(AliasCapabilityLeak)]
pub trait Transformer = Clone + Send;
