use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::no_stuttered_path_check;
use super::no_stuttered_path_register;

pub struct NoStutteredPath;

impl NoStutteredPath {
    pub const NAME: &'static str = "no_stuttered_path";
    pub const CODE: &'static str = "E025";
}

#[rustfmt::skip]
impl Checker for NoStutteredPath {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { no_stuttered_path_check::check(self, project) }
}

#[rustfmt::skip]
impl NoStutteredPath {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_stuttered_path_register::register(checkers, config)
    }
}

// no test_usage necessary
