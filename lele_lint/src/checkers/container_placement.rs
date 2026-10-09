use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct ContainerPlacement;

impl ContainerPlacement {
    pub const NAME: &'static str = "container_placement";
    pub const CODE: &'static str = "E040";
    pub const DOC: RuleDoc = RuleDoc {
        category: "layout",
        summary: "Behavior-free types (plain structs, enums, newtypes, aliases, ECS markers) go in the domain's `__basic__/<role>.rs`.",
        why: "Small data types don't need a file and a test each; grouping them keeps atomic files for code that does something.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod stock;
",
            },
            ExampleFile {
                path: "src/stock/mod.rs",
                source: r"mod item;

pub use item::Item;
",
            },
            ExampleFile {
                path: "src/stock/item.rs",
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
impl Checker for ContainerPlacement {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(ContainerPlacement)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl ContainerPlacement {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(ContainerPlacement));
    }
}

// no test_usage necessary
