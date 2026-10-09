use atomic_delegate_macros::atomic_delegate;
use lele_lint::Category;
use lele_lint::Checker;
use lele_lint::Diagnostic;
use lele_lint::ExampleFile;
use lele_lint::Project;
use lele_lint::RuleDoc;

pub struct BevyFolder;

impl BevyFolder {
    pub const NAME: &'static str = "bevy_folder";
    pub const CODE: &'static str = "E008";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Layout,
        summary: "A `pub fn` registered with `app.add_systems()` whose parameters read Bevy system types must live in the domain's `bevy_systems/` folder.",
        why: "Systems are the domain's Bevy edge; keeping them in one folder makes the plugin surface readable.",
        bad: &[ExampleFile {
            path: "src/enemy/tick_enemies.rs",
            source: r"pub struct Query;

pub fn tick_enemies(_q: Query) {}

pub fn register_enemies() {
    let app = App;
    app.add_systems(Update, tick_enemies);
}
",
        }],
        good: &[
            ExampleFile {
                path: "src/enemy/mod.rs",
                source: r"pub mod bevy_systems;
",
            },
            ExampleFile {
                path: "src/enemy/bevy_systems/mod.rs",
                source: r"pub mod tick_enemies;
pub use tick_enemies::tick_enemies;
",
            },
            ExampleFile {
                path: "src/enemy/bevy_systems/tick_enemies.rs",
                source: r"pub struct Query;

pub fn tick_enemies(_q: Query) {}
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for BevyFolder {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(BevyFolder)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl BevyFolder {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(Self));
    }
}

// no test_usage necessary
