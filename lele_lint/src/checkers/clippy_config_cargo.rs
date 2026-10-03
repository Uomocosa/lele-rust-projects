use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct ClippyConfigCargo;

impl ClippyConfigCargo {
    pub const NAME: &'static str = "clippy_config_cargo";
    pub const CODE: &'static str = "E021";
    pub const DOC: RuleDoc = RuleDoc {
        category: "config",
        summary: "`Cargo.toml` has `[lints.clippy]` with `pedantic`/`nursery` denied at priority -1 and the 13 required deny lints (or inherits them from the workspace).",
        why: "Every crate gets the same strict baseline, so no crate silently allows unwraps, panics or lossy casts.",
        bad: &[
            ExampleFile {
                path: "Cargo.toml",
                source: r#"[package]
name = "example"
version = "0.1.0"
edition = "2021"
"#,
            },
        ],
        good: &[
            ExampleFile {
                path: "Cargo.toml",
                source: r#"[package]
name = "example"
version = "0.1.0"
edition = "2021"

[lints.clippy]
pedantic = { level = "deny", priority = -1 }
nursery = { level = "deny", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
indexing_slicing = "deny"
arithmetic_side_effects = "deny"
unreachable = "deny"
unimplemented = "deny"
unchecked_time_subtraction = "deny"
todo = "deny"
string_slice = "deny"
panic_in_result_fn = "deny"
panic = "deny"
exit = "deny"
as_conversions = "deny"
"#,
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for ClippyConfigCargo {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
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
