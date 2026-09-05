use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::clippy_config_clippy_check;
use super::clippy_config_clippy_register;

pub struct ClippyConfigClippy;

impl ClippyConfigClippy {
    pub const NAME: &'static str = "clippy_config_clippy";
    pub const CODE: &'static str = "E022";
}

#[rustfmt::skip]
impl Checker for ClippyConfigClippy {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { clippy_config_clippy_check::check(self, project) }
}

#[rustfmt::skip]
impl ClippyConfigClippy {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        clippy_config_clippy_register::register(checkers, config)
    }
}

// no test_usage necessary
