extern crate antigen;
use antigen::presents;
pub struct ExternalDependencyRisk;

#[presents(ExternalDependencyRisk)]
extern crate std;
