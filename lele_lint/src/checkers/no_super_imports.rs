use crate::Category;
use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct NoSuperImports;

impl NoSuperImports {
    pub const NAME: &'static str = "no_super_imports";
    pub const CODE: &'static str = "E033";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Imports,
        summary: "`super::` is allowed only inside `#[cfg(test)]`; production code imports the domain (`use crate::stock;`).",
        why: "`super::` paths break when files move; domain paths read the same from every file.",
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
mod price_of;
pub use price_of::price_of;
"#,
            },
            ExampleFile {
                path: "src/stock/price_of.rs",
                source: r"use super::Item;

pub fn price_of(item: &Item) -> u32 {
    item.price
}
",
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
mod price_of;
pub use price_of::price_of;
"#,
            },
            ExampleFile {
                path: "src/stock/price_of.rs",
                source: r#"use crate::stock;

pub fn price_of(item: &stock::Item) -> u32 {
    item.price
}

#[cfg(test)]
mod tests {
    use super::price_of;
    use crate::stock::Item;

    #[test]
    fn test_usage() {
        let item = Item {
            name: "apple".to_string(),
            price: 3,
        };
        assert_eq!(price_of(&item), 3);
    }
}
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
impl Checker for NoSuperImports {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(NoSuperImports)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoSuperImports {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoSuperImports));
    }
}

// no test_usage necessary
