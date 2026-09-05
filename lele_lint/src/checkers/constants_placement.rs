use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::constants_placement_check;
use super::constants_placement_register;

pub struct ConstantsPlacement;

impl ConstantsPlacement {
    pub const NAME: &'static str = "constants_placement";
    pub const CODE: &'static str = "E026";
}

#[rustfmt::skip]
impl Checker for ConstantsPlacement {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { constants_placement_check::check(self, project) }
}

#[rustfmt::skip]
impl ConstantsPlacement {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        constants_placement_register::register(checkers, config)
    }
}

// no test_usage necessary
