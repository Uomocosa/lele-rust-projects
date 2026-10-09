use crate::Category;
use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct ModRsPurity;

impl ModRsPurity {
    pub const NAME: &'static str = "mod_rs_purity";
    pub const CODE: &'static str = "E019";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Layout,
        summary: "`mod.rs` holds only `mod`/`pub mod` declarations and `pub use` re-exports.",
        why: "`mod.rs` is the folder's table of contents; logic hidden there is logic nobody looks for.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod greeting;
",
            },
            ExampleFile {
                path: "src/greeting/mod.rs",
                source: r#"pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}
"#,
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod greeting;
",
            },
            ExampleFile {
                path: "src/greeting/mod.rs",
                source: r"mod greet;

pub use greet::greet;
",
            },
            ExampleFile {
                path: "src/greeting/greet.rs",
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
impl Checker for ModRsPurity {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(ModRsPurity)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl ModRsPurity {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(ModRsPurity));
    }
}

// no test_usage necessary
