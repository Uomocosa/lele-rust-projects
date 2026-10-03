use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct NoComments;

impl NoComments {
    pub const NAME: &'static str = "no_comments";
    pub const CODE: &'static str = "E031";
    pub const DOC: RuleDoc = RuleDoc {
        category: "style",
        summary: "No comments in `src/` or `methods/`, except `// needed helper: <why>` and a final `// no test_usage necessary`.",
        why: "Names, signatures and the usage test carry the meaning; comments drift from the code they describe.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"mod greet;
pub use greet::greet;
",
            },
            ExampleFile {
                path: "src/greet.rs",
                source: r#"// Builds the greeting shown on the title screen.
pub fn greet(name: &str) -> String {
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
impl Checker for NoComments {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(NoComments)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoComments {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoComments));
    }
}

// no test_usage necessary
