use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct ClippyConfigClippy;

impl ClippyConfigClippy {
    pub const NAME: &'static str = "clippy_config_clippy";
    pub const CODE: &'static str = "E022";
    pub const DOC: RuleDoc = RuleDoc {
        category: "config",
        summary: "`clippy.toml` sets `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests` and `allow-indexing-slicing-in-tests` to `true`.",
        why: "Tests may unwrap and index freely; production code may not.",
        bad: &[
            ExampleFile {
                path: "clippy.toml",
                source: r"allow-unwrap-in-tests = true
",
            },
        ],
        good: &[
            ExampleFile {
                path: "clippy.toml",
                source: r"allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for ClippyConfigClippy {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
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
