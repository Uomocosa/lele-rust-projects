use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct DomainImport;

impl DomainImport {
    pub const NAME: &'static str = "domain_import";
    pub const CODE: &'static str = "E011";
    pub const DOC: RuleDoc = RuleDoc {
        category: "imports",
        summary: "Import the domain, not the item: `use crate::stock;` then `stock::Item`.",
        why: "The domain prefix at every use site says where a name comes from, so names never need to be unique crate-wide.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod shop;
pub mod stock;
",
            },
            ExampleFile {
                path: "src/shop/mod.rs",
                source: r"mod total;

pub use total::total;
",
            },
            ExampleFile {
                path: "src/shop/total.rs",
                source: r#"use crate::stock::Item;

pub fn total(items: &[Item]) -> u32 {
    items.iter().map(|item| item.price).sum()
}

#[cfg(test)]
mod tests {
    use super::total;
    use crate::stock::Item;

    #[test]
    fn test_usage() {
        let items = vec![Item {
            name: "apple".to_string(),
            price: 3,
        }];
        assert_eq!(total(&items), 3);
    }
}
"#,
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
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod shop;
pub mod stock;
",
            },
            ExampleFile {
                path: "src/shop/mod.rs",
                source: r"mod total;

pub use total::total;
",
            },
            ExampleFile {
                path: "src/shop/total.rs",
                source: r#"use crate::stock;

pub fn total(items: &[stock::Item]) -> u32 {
    items.iter().map(|item| item.price).sum()
}

#[cfg(test)]
mod tests {
    use super::total;
    use crate::stock::Item;

    #[test]
    fn test_usage() {
        let items = vec![Item {
            name: "apple".to_string(),
            price: 3,
        }];
        assert_eq!(total(&items), 3);
    }
}
"#,
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
impl Checker for DomainImport {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(DomainImport)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl DomainImport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(DomainImport));
    }
}

// no test_usage necessary
