use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct NoStutteredType;

impl NoStutteredType {
    pub const NAME: &'static str = "no_stuttered_type";
    pub const CODE: &'static str = "E027";
}

#[rustfmt::skip]
impl Checker for NoStutteredType {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(NoStutteredType)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoStutteredType {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoStutteredType));
    }
}

// no test_usage necessary
