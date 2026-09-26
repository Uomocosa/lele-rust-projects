use crate::checkers;
use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

pub struct NoCratePaths;

impl NoCratePaths {
    pub const NAME: &'static str = "no_crate_paths";
    pub const CODE: &'static str = "E020";
}

#[rustfmt::skip]
impl Checker for NoCratePaths {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::no_crate_paths_check::check(self, project) }
}

#[rustfmt::skip]
impl NoCratePaths {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        checkers::no_crate_paths_register::register(checkers, config)
    }
}

// no test_usage necessary
