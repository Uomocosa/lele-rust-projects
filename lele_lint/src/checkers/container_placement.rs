use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::container_placement_check;
use super::container_placement_register;

pub struct ContainerPlacement;

impl ContainerPlacement {
    pub const NAME: &'static str = "container_placement";
    pub const CODE: &'static str = "E029";
}

#[rustfmt::skip]
impl Checker for ContainerPlacement {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { container_placement_check::check(self, project) }
}

#[rustfmt::skip]
impl ContainerPlacement {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        container_placement_register::register(checkers, config)
    }
}

// no test_usage necessary
