use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct MethodsLayout;

impl MethodsLayout {
    pub const NAME: &'static str = "methods_layout";
    pub const CODE: &'static str = "E030";
}

#[rustfmt::skip]
impl Checker for MethodsLayout {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(MethodsLayout)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl MethodsLayout {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(MethodsLayout));
    }
}

// no test_usage necessary
