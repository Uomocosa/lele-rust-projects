use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct MethodVisibility;

impl MethodVisibility {
    pub const NAME: &'static str = "method_visibility";
    pub const CODE: &'static str = "E003";
}

#[rustfmt::skip]
impl Checker for MethodVisibility {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(MethodVisibility)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl MethodVisibility {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(MethodVisibility));
    }
}

// no test_usage necessary
