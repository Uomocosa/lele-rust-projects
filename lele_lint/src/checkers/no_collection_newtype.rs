use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct NoCollectionNewtype;

impl NoCollectionNewtype {
    pub const NAME: &'static str = "no_collection_newtype";
    pub const CODE: &'static str = "E028";
    pub const DOC: RuleDoc = RuleDoc {
        category: "types",
        summary: "No newtype around a collection (`Rooms(Vec<String>)`); wrap the element (`Room(String)`) and use `Vec<Room>`.",
        why: "The element is the concept; a collection wrapper hides the standard collection API behind a new name.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod lobby;
",
            },
            ExampleFile {
                path: "src/lobby/mod.rs",
                source: r#"#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::newtypes::Rooms;
"#,
            },
            ExampleFile {
                path: "src/lobby/__basic__/mod.rs",
                source: r"pub mod newtypes;
",
            },
            ExampleFile {
                path: "src/lobby/__basic__/newtypes.rs",
                source: r"use derive_more::Deref;

#[derive(Deref)]
pub struct Rooms(pub Vec<String>);
",
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod lobby;
",
            },
            ExampleFile {
                path: "src/lobby/mod.rs",
                source: r#"#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::newtypes::Room;
"#,
            },
            ExampleFile {
                path: "src/lobby/__basic__/mod.rs",
                source: r"pub mod newtypes;
",
            },
            ExampleFile {
                path: "src/lobby/__basic__/newtypes.rs",
                source: r"use derive_more::Deref;

#[derive(Deref)]
pub struct Room(pub String);
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for NoCollectionNewtype {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(NoCollectionNewtype)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoCollectionNewtype {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoCollectionNewtype));
    }
}

// no test_usage necessary
