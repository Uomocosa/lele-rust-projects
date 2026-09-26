use crate::checkers;
use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

pub struct ConstantsPlacement;

impl ConstantsPlacement {
    pub const NAME: &'static str = "constants_placement";
    pub const CODE: &'static str = "E026";
}

#[rustfmt::skip]
impl Checker for ConstantsPlacement {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::constants_placement_check::check(self, project) }
}

#[rustfmt::skip]
impl ConstantsPlacement {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        checkers::constants_placement_register::register(checkers, config)
    }
}

// no test_usage necessary
