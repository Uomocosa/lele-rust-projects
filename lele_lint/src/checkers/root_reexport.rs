use crate::checkers;
use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

pub struct RootReexport;

impl RootReexport {
    pub const NAME: &'static str = "root_reexport";
    pub const CODE: &'static str = "E024";
}

#[rustfmt::skip]
impl Checker for RootReexport {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::root_reexport_check::check(self, project) }
}

#[rustfmt::skip]
impl RootReexport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        checkers::root_reexport_register::register(checkers, config)
    }
}

// no test_usage necessary
