use crate::Category;
use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct TestInline;

impl TestInline {
    pub const NAME: &'static str = "test_inline";
    pub const CODE: &'static str = "E007";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Tests,
        summary: "Unit tests live in the same file as the code (no `tests/` folders under `src/`).",
        why: "The test sits next to what it tests, so reading one means reading the other.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"mod greet;
pub use greet::greet;
",
            },
            ExampleFile {
                path: "src/greet.rs",
                source: r#"pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
    use super::greet;

    #[test]
    fn test_usage() {
        assert_eq!(greet("Ada"), "Hello, Ada!");
    }
}
"#,
            },
            ExampleFile {
                path: "src/tests/greet_more.rs",
                source: r#"#[test]
fn greets_empty_name() {
    assert_eq!(crate::greet(""), "Hello, !");
}
"#,
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"mod greet;
pub use greet::greet;
",
            },
            ExampleFile {
                path: "src/greet.rs",
                source: r#"pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
    use super::greet;

    #[test]
    fn test_usage() {
        assert_eq!(greet("Ada"), "Hello, Ada!");
    }
}
"#,
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for TestInline {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(TestInline)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl TestInline {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(TestInline));
    }
}

// no test_usage necessary
