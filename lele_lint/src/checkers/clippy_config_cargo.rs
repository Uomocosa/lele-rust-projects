use crate::checkers;
use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

pub struct ClippyConfigCargo;

impl ClippyConfigCargo {
    pub const NAME: &'static str = "clippy_config_cargo";
    pub const CODE: &'static str = "E021";
}

#[rustfmt::skip]
impl Checker for ClippyConfigCargo {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::clippy_config_cargo_check::check(self, project) }
}

#[rustfmt::skip]
impl ClippyConfigCargo {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        checkers::clippy_config_cargo_register::register(checkers, config)
    }
}

// no test_usage necessary
