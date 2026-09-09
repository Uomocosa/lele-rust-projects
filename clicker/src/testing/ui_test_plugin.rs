use bevy::prelude::*;
use bevy::winit::WinitPlugin;

pub struct UiTestPlugin {
    pub title: String,
    pub visible: bool,
}

impl Plugin for UiTestPlugin {
    fn build(&self, app: &mut App) {
        let mut winit = WinitPlugin::default();
        winit.run_on_any_thread = true;
        app.add_plugins(DefaultPlugins.build().set(winit).set(WindowPlugin {
            primary_window: Some(Window {
                title: self.title.clone(),
                resolution: (800, 450).into(),
                visible: self.visible,
                ..default()
            }),
            ..default()
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::UiTestPlugin;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let plugin = UiTestPlugin {
            title: "test".to_owned(),
            visible: false,
        };
        assert_eq!(plugin.title, "test");
        assert!(!plugin.visible);
        let mut app = App::new();
        app.add_plugins(plugin);
    }
}
