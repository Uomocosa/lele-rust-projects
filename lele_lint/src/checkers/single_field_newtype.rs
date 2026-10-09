use crate::Category;
use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct SingleFieldNewtype;

impl SingleFieldNewtype {
    pub const NAME: &'static str = "single_field_newtype";
    pub const CODE: &'static str = "E018";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Types,
        summary: "A struct with one field is a tuple newtype deriving `Deref`; two or more fields are named.",
        why: "Field count decides the shape, so every single-value wrapper reads the same: `*meters`, never `meters.value`.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod track;
",
            },
            ExampleFile {
                path: "src/track/mod.rs",
                source: r#"#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::structs::Meters;
"#,
            },
            ExampleFile {
                path: "src/track/__basic__/mod.rs",
                source: r"pub mod structs;
",
            },
            ExampleFile {
                path: "src/track/__basic__/structs.rs",
                source: r"pub struct Meters {
    pub value: u32,
}
",
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod track;
",
            },
            ExampleFile {
                path: "src/track/mod.rs",
                source: r#"#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::newtypes::Meters;
"#,
            },
            ExampleFile {
                path: "src/track/__basic__/mod.rs",
                source: r"pub mod newtypes;
",
            },
            ExampleFile {
                path: "src/track/__basic__/newtypes.rs",
                source: r"use derive_more::Deref;

#[derive(Deref)]
pub struct Meters(pub u32);
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for SingleFieldNewtype {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(SingleFieldNewtype)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl SingleFieldNewtype {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(SingleFieldNewtype));
    }
}

// no test_usage necessary
