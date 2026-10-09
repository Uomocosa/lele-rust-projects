use atomic_delegate_macros::atomic_delegate;
use lele_lint::Category;
use lele_lint::Checker;
use lele_lint::Diagnostic;
use lele_lint::ExampleFile;
use lele_lint::Project;
use lele_lint::RuleDoc;

pub struct BevyUi;

impl BevyUi {
    pub const NAME: &'static str = "bevy_ui";
    pub const CODE: &'static str = "E029";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Tests,
        summary: "A file whose visual spawn is reachable from production must ship an ignored `*_ui_png_preview` (and, when it drives a recorder, a `*_ui_mp4_preview`) test ending with `assert!(exists)` plus `println!(\"PREVIEW_ARTIFACT=...\")`.",
        why: "UI that is never rendered cannot be reviewed; the named preview test is the contract that forces an artifact for every production visual.",
        bad: &[ExampleFile {
            path: "src/discovery/ui/spawn_root.rs",
            source: r"pub struct Sprite;
pub struct Spawner;

pub fn setup(spawner: &mut Spawner) {
    spawner.spawn((Sprite,));
}
",
        }],
        good: &[ExampleFile {
            path: "src/discovery/ui/spawn_root.rs",
            source: r#"pub struct Sprite;
pub struct Spawner;

pub fn setup(spawner: &mut Spawner) {
    spawner.spawn((Sprite,));
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "headed preview"]
    fn spawn_root_ui_png_preview() {
        let shot = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spawn_root.png");
        assert!(shot.exists());
        println!("PREVIEW_ARTIFACT={}", shot.display());
    }
}
"#,
        }],
    };
}

#[rustfmt::skip]
impl Checker for BevyUi {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(BevyUi)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl BevyUi {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(Self));
    }
}

// no test_usage necessary
