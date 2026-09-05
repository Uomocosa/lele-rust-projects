use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::no_stuttered_type_check;
use super::no_stuttered_type_register;

pub struct NoStutteredType;

impl NoStutteredType {
    pub const NAME: &'static str = "no_stuttered_type";
    pub const CODE: &'static str = "E027";
}

#[rustfmt::skip]
impl Checker for NoStutteredType {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { no_stuttered_type_check::check(self, project) }
}

#[rustfmt::skip]
impl NoStutteredType {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_stuttered_type_register::register(checkers, config)
    }
}

// no test_usage necessary
