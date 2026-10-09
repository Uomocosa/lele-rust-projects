use atomic_delegate_macros::atomic_delegate;
use lele_lint::Category;
use lele_lint::Checker;
use lele_lint::Diagnostic;
use lele_lint::ExampleFile;
use lele_lint::Project;
use lele_lint::RuleDoc;

pub struct PreviewRouting;

impl PreviewRouting {
    pub const NAME: &'static str = "preview_routing";
    pub const CODE: &'static str = "E039";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Tests,
        summary: "Every `*_ui_png_preview`, `*_ui_mp4_preview` and `*_ui_scene_preview` test must call `lele_bevy_preview::run(...)`.",
        why: "A preview test that renders and asserts nothing can pass while producing a blank frame; routing through the harness makes empty frames fail.",
        bad: &[ExampleFile {
            path: "src/discovery/ui/spawn_root.rs",
            source: r#"pub fn setup(s: &mut S) { s.spawn((Node,)); }

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "headed"]
    fn spawn_root_ui_png_preview() {
        let shot = std::path::PathBuf::from("spawn_root.png");
        assert!(shot.exists());
    }
}
"#,
        }],
        good: &[ExampleFile {
            path: "src/discovery/ui/spawn_root.rs",
            source: r#"pub fn setup(s: &mut S) { s.spawn((Node,)); }

#[cfg(test)]
mod tests {
    use lele_bevy_preview::{run, scene::Scene};

    #[test]
    #[ignore = "headed"]
    fn spawn_root_ui_png_preview() {
        let scene = Scene { name: String::from("spawn_root") };
        let _ = run(&scene, &Config::default(), "x");
    }
}
"#,
        }],
    };
}

#[rustfmt::skip]
impl Checker for PreviewRouting {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(PreviewRouting)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl PreviewRouting {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(Self));
    }
}

// no test_usage necessary
