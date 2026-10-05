use atomic_delegate_macros::atomic_delegate;
use lele_lint::Checker;
use lele_lint::Diagnostic;
use lele_lint::ExampleFile;
use lele_lint::Project;
use lele_lint::RuleDoc;

pub struct BevyPluginScene;

impl BevyPluginScene {
    pub const NAME: &'static str = "bevy_plugin_scene";
    pub const CODE: &'static str = "E038";
    pub const DOC: RuleDoc = RuleDoc {
        category: "tests",
        summary: "A file defining a `Plugin` whose build spawns UI must ship an ignored `*_ui_scene_preview` test.",
        why: "A plugin is the assembly point for a whole screen; its scene preview proves the assembled screen renders.",
        bad: &[ExampleFile {
            path: "src/discovery/ui/default_ui_plugin.rs",
            source: r"pub fn spawn_ui(s: &mut S) { s.spawn((Node,)); }

pub struct DefaultUiPlugin;

impl Plugin for DefaultUiPlugin {
    fn build(&self, app: &mut A) { spawn_ui(app); }
}
",
        }],
        good: &[ExampleFile {
            path: "src/discovery/ui/default_ui_plugin.rs",
            source: r#"pub fn spawn_ui(s: &mut S) { s.spawn((Node,)); }

pub struct DefaultUiPlugin;

impl Plugin for DefaultUiPlugin {
    fn build(&self, app: &mut A) { spawn_ui(app); }
}

#[cfg(test)]
mod tests {
    use lele_bevy_preview::{run, scene::Scene};

    #[test]
    #[ignore = "headed scene"]
    fn default_ui_plugin_ui_scene_preview() {
        let scene = Scene { name: String::from("default_ui_plugin") };
        let _ = run(&scene, &Config::default(), "x");
    }
}
"#,
        }],
    };
}

#[rustfmt::skip]
impl Checker for BevyPluginScene {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(BevyPluginScene)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl BevyPluginScene {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(Self));
    }
}

// no test_usage necessary
