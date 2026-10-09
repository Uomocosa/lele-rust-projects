use crate::Category;
use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct NoDunderTests;

impl NoDunderTests {
    pub const NAME: &'static str = "no_dunder_tests";
    pub const CODE: &'static str = "E034";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Tests,
        summary: "No `#[cfg(test)]` inside `__basic__/` containers.",
        why: "Containers hold declarations only; test the function or method that uses the types instead.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod stock;
",
            },
            ExampleFile {
                path: "src/stock/mod.rs",
                source: r#"#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::structs::Item;
"#,
            },
            ExampleFile {
                path: "src/stock/__basic__/mod.rs",
                source: r"pub mod structs;
",
            },
            ExampleFile {
                path: "src/stock/__basic__/structs.rs",
                source: r#"pub struct Item {
    pub name: String,
    pub price: u32,
}

#[cfg(test)]
mod tests {
    use super::Item;

    #[test]
    fn test_usage() {
        let item = Item {
            name: "apple".to_string(),
            price: 3,
        };
        assert_eq!(item.price, 3);
    }
}
"#,
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod stock;
",
            },
            ExampleFile {
                path: "src/stock/mod.rs",
                source: r#"#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::structs::Item;
"#,
            },
            ExampleFile {
                path: "src/stock/__basic__/mod.rs",
                source: r"pub mod structs;
",
            },
            ExampleFile {
                path: "src/stock/__basic__/structs.rs",
                source: r"pub struct Item {
    pub name: String,
    pub price: u32,
}
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for NoDunderTests {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(NoDunderTests)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoDunderTests {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoDunderTests));
    }
}

// no test_usage necessary
