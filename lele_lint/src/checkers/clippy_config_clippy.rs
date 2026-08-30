use crate::checker;
use crate::config;
use crate::diagnostic;
use crate::project;

use super::clippy_config_clippy_check;
use super::clippy_config_clippy_register;

pub struct ClippyConfigClippy;

impl ClippyConfigClippy {
    pub const NAME: &'static str = "clippy_config_clippy";
    pub const CODE: &'static str = "E022";
}

#[rustfmt::skip]
impl checker::Checker for ClippyConfigClippy {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &project::Project) -> Vec<diagnostic::Diagnostic> { clippy_config_clippy_check::check(self, project) }
}

#[rustfmt::skip]
impl ClippyConfigClippy {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        clippy_config_clippy_register::register(checkers, config)
    }
}

// no test_usage necessary
