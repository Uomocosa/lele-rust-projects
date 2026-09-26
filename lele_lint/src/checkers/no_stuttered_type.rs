use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct NoStutteredType;

impl NoStutteredType {
    pub const NAME: &'static str = "no_stuttered_type";
    pub const CODE: &'static str = "E027";
}

#[rustfmt::skip]
impl Checker for NoStutteredType {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::no_stuttered_type_check::check(self, project) }
}

#[atomic_delegates]
impl NoStutteredType {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
