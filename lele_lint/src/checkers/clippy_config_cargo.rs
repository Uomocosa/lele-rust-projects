use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct ClippyConfigCargo;

impl ClippyConfigCargo {
    pub const NAME: &'static str = "clippy_config_cargo";
    pub const CODE: &'static str = "E021";
}

#[rustfmt::skip]
impl Checker for ClippyConfigCargo {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(ClippyConfigCargo)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl ClippyConfigCargo {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(ClippyConfigCargo));
    }
}

// no test_usage necessary
