use bevy::prelude::*;
use lele_bevy_preview::preview::Config;
use lele_bevy_preview::scene::Scene;
use lele_bevy_preview::scene::basic::structs::State;

#[derive(Component)]
struct Panel;

fn build_real_ui(app: &mut App) {
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((
            Panel,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(24.0),
                left: Val::Px(24.0),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.35, 0.65)),
            children![Text::new("HELLO PREVIEW")],
        ));
    });
}

fn paint_red(world: &mut World) {
    let mut query = world.query_filtered::<&mut BackgroundColor, With<Panel>>();
    for mut color in query.iter_mut(world) {
        *color = BackgroundColor(Color::srgb(0.85, 0.2, 0.15));
    }
}

fn scene() -> Scene {
    Scene {
        name: String::from("panel"),
        build: build_real_ui,
        states: vec![
            State {
                label: String::from("blue"),
                apply: noop,
            },
            State {
                label: String::from("red"),
                apply: paint_red,
            },
        ],
        timeline: None,
    }
}

fn noop(_world: &mut World) {}

#[test]
fn writes_a_viewable_png_of_a_real_ui_node_with_text() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("panel.png");
    let config = Config {
        out_dir: dir.path().to_path_buf(),
        warmup_frames: 60,
        max_capture_frames: 300,
        ..Config::default()
    };
    let rendered = lele_bevy_preview::preview::render_states(&scene(), &config).unwrap();
    let (path, label, pixels) = &rendered[0];
    assert_eq!(label, "blue");
    assert_eq!(pixels.len(), 16);
    let keep = std::env::var("LELE_KEEP_PREVIEW").unwrap_or_default();
    if keep.is_empty() {
        std::fs::copy(path, &out).unwrap();
    } else {
        let _ = keep;
    }
    let meta = std::fs::metadata(&out).unwrap();
    assert!(
        meta.len() > 4096,
        "a 960x540 PNG with a coloured panel and text should not be {}-bytes thin",
        meta.len()
    );
}
