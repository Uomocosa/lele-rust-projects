use crate::Category;
use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct AtomicFile;

impl AtomicFile {
    pub const NAME: &'static str = "atomic_file";
    pub const CODE: &'static str = "E001";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Layout,
        summary: "One public item per file, and the file is named after it (`greet.rs` holds `pub fn greet`).",
        why: "A file name tells you exactly what is inside; finding code means finding a file, and every unit gets its own test.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"mod greeting;
pub use greeting::{farewell, greet};
",
            },
            ExampleFile {
                path: "src/greeting.rs",
                source: r#"pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

pub fn farewell(name: &str) -> String {
    format!("Bye, {name}!")
}
"#,
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"mod farewell;
mod greet;
pub use farewell::farewell;
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
                path: "src/farewell.rs",
                source: r#"pub fn farewell(name: &str) -> String {
    format!("Bye, {name}!")
}

#[cfg(test)]
mod tests {
    use super::farewell;

    #[test]
    fn test_usage() {
        assert_eq!(farewell("Ada"), "Bye, Ada!");
    }
}
"#,
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for AtomicFile {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(AtomicFile)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl AtomicFile {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(AtomicFile));
    }
}

// no test_usage necessary
