use crate::checker;
use crate::config;
use crate::diagnostic;
use crate::project;

use super::clippy_config_cargo_check;
use super::clippy_config_cargo_register;

pub struct ClippyConfigCargo;

impl ClippyConfigCargo {
    pub const NAME: &'static str = "clippy_config_cargo";
    pub const CODE: &'static str = "E021";
}

#[rustfmt::skip]
impl checker::Checker for ClippyConfigCargo {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &project::Project) -> Vec<diagnostic::Diagnostic> { clippy_config_cargo_check::check(self, project) }
}

#[rustfmt::skip]
impl ClippyConfigCargo {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        clippy_config_cargo_register::register(checkers, config)
    }
}

// no test_usage necessary
