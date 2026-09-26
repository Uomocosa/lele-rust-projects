use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct ClippyConfigClippy;

impl ClippyConfigClippy {
    pub const NAME: &'static str = "clippy_config_clippy";
    pub const CODE: &'static str = "E022";
}

#[rustfmt::skip]
impl Checker for ClippyConfigClippy {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(ClippyConfigClippy)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl ClippyConfigClippy {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(ClippyConfigClippy));
    }
}

// no test_usage necessary
