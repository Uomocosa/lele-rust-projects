use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct MethodsLayout;

impl MethodsLayout {
    pub const NAME: &'static str = "methods_layout";
    pub const CODE: &'static str = "E030";
}

#[rustfmt::skip]
impl Checker for MethodsLayout {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::methods_layout_check::check(self, project) }
}

#[atomic_delegates]
impl MethodsLayout {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
