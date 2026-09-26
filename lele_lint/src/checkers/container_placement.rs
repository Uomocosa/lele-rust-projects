use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct ContainerPlacement;

impl ContainerPlacement {
    pub const NAME: &'static str = "container_placement";
    pub const CODE: &'static str = "E029";
}

#[rustfmt::skip]
impl Checker for ContainerPlacement {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(ContainerPlacement)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl ContainerPlacement {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(ContainerPlacement));
    }
}

// no test_usage necessary
