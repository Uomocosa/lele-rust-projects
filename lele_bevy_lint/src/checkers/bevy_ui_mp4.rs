use atomic_delegate_macros::atomic_delegate;
use lele_lint::Category;
use lele_lint::Checker;
use lele_lint::Diagnostic;
use lele_lint::ExampleFile;
use lele_lint::Project;
use lele_lint::RuleDoc;

pub struct BevyUiMp4;

impl BevyUiMp4 {
    pub const NAME: &'static str = "bevy_ui_mp4";
    pub const CODE: &'static str = "E037";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Tests,
        summary: "A file that spawns UI reachable from production and drives it over time or input must ship an ignored `*_ui_mp4_preview` test.",
        why: "Static frames cannot show motion, hover or press states; a time/input-driven UI needs a recorded clip to be reviewable.",
        bad: &[ExampleFile {
            path: "src/discovery/ui/sync_room_list.rs",
            source: r"pub fn setup(s: &mut S) { s.spawn((Node,)); }

pub fn tick(time: Res<Time>, q: Query<&Interaction>) {
    let _ = time.delta_secs();
    let _ = q;
}
",
        }],
        good: &[ExampleFile {
            path: "src/discovery/ui/sync_room_list.rs",
            source: r#"pub fn setup(s: &mut S) { s.spawn((Node,)); }

pub fn tick(time: Res<Time>) { let _ = time.delta_secs(); }

#[cfg(test)]
mod tests {
    use lele_bevy_preview::{run, scene::Scene};

    #[test]
    #[ignore = "headed recording"]
    fn sync_room_list_ui_mp4_preview() {
        let scene = Scene { name: String::from("sync_room_list") };
        let _ = run(&scene, &Config::default(), "x");
    }
}
"#,
        }],
    };
}

#[rustfmt::skip]
impl Checker for BevyUiMp4 {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(BevyUiMp4)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl BevyUiMp4 {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(Self));
    }
}

// no test_usage necessary
