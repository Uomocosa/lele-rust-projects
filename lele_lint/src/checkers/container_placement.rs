use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct ContainerPlacement;

impl ContainerPlacement {
    pub const NAME: &'static str = "container_placement";
    pub const CODE: &'static str = "E029";
}

#[rustfmt::skip]
impl Checker for ContainerPlacement {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::container_placement_check::check(self, project) }
}

#[atomic_delegates]
impl ContainerPlacement {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
