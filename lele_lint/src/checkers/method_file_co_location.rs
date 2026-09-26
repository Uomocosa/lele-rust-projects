use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct MethodFileCoLocation;

impl MethodFileCoLocation {
    pub const NAME: &'static str = "method_file_co_location";
    pub const CODE: &'static str = "E017";
}

#[rustfmt::skip]
impl Checker for MethodFileCoLocation {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(MethodFileCoLocation)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl MethodFileCoLocation {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(MethodFileCoLocation));
    }
}

// no test_usage necessary
