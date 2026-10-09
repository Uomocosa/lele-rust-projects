use atomic_delegate_macros::atomic_delegate;
use lele_lint::Category;
use lele_lint::Checker;
use lele_lint::Diagnostic;
use lele_lint::ExampleFile;
use lele_lint::Project;
use lele_lint::RuleDoc;

pub struct BevyExport;

impl BevyExport {
    pub const NAME: &'static str = "bevy_export";
    pub const CODE: &'static str = "E005";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Layout,
        summary: "A domain declares `pub mod bevy_systems;` but never re-exports its systems at the domain root.",
        why: "Consumers reach systems as `domain::bevy_systems::system`, so the folder is always visible in the path.",
        bad: &[
            ExampleFile {
                path: "src/inventory/mod.rs",
                source: r"pub mod bevy_systems;

pub use bevy_systems::poll_inv;
",
            },
            ExampleFile {
                path: "src/inventory/bevy_systems/mod.rs",
                source: r"pub mod poll_inv;
pub use poll_inv::poll_inv;
",
            },
        ],
        good: &[
            ExampleFile {
                path: "src/inventory/mod.rs",
                source: r"pub mod bevy_systems;
",
            },
            ExampleFile {
                path: "src/inventory/bevy_systems/mod.rs",
                source: r"pub mod poll_inv;
pub use poll_inv::poll_inv;
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for BevyExport {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(BevyExport)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl BevyExport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(Self));
    }
}

// no test_usage necessary
