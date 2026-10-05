use crate::preview;
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::camera::RenderTarget;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::time::TimeUpdateStrategy;
use bevy::ui::IsDefaultUiCamera;
use bevy::window::PrimaryWindow;

use crate::preview::basic::constants::{
    BACKGROUND_A, BACKGROUND_B, BACKGROUND_G, BACKGROUND_R, CLEAR_B, CLEAR_G, CLEAR_R,
};

pub fn install(app: &mut App, config: &preview::Config) -> Handle<Image> {
    add_plugins(app);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.insert_resource(ClearColor(Color::srgb(CLEAR_R, CLEAR_G, CLEAR_B)));
    app.init_resource::<bevy::input_focus::InputFocus>();
    app.init_resource::<bevy::input_focus::InputFocusVisible>();
    let target = spawn_viewport(app, config);
    app.finish();
    app.cleanup();
    target
}

// needed helper: winit panics off-main-thread and needs a display, so the windowing stack is off
fn add_plugins(app: &mut App) {
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::window::WindowPlugin>()
            .disable::<bevy::a11y::AccessibilityPlugin>()
            .disable::<bevy::input_focus::InputFocusPlugin>()
            .disable::<bevy::input_focus::InputDispatchPlugin>(),
    );
    register_window_messages(app);
}

// needed helper: WindowPlugin is disabled, so every message it registered must be re-registered
fn register_window_messages(app: &mut App) {
    app.add_message::<bevy::window::WindowEvent>()
        .add_message::<bevy::window::WindowResized>()
        .add_message::<bevy::window::WindowCreated>()
        .add_message::<bevy::window::WindowClosing>()
        .add_message::<bevy::window::WindowClosed>()
        .add_message::<bevy::window::WindowCloseRequested>()
        .add_message::<bevy::window::WindowDestroyed>()
        .add_message::<bevy::window::WindowFocused>()
        .add_message::<bevy::window::WindowOccluded>()
        .add_message::<bevy::window::WindowScaleFactorChanged>()
        .add_message::<bevy::window::WindowBackendScaleFactorChanged>()
        .add_message::<bevy::window::WindowMoved>()
        .add_message::<bevy::window::WindowThemeChanged>()
        .add_message::<bevy::window::Ime>();
}

// needed helper: IsDefaultUiCamera is what makes bevy_ui draw at all for an offscreen camera
fn spawn_viewport(app: &mut App, config: &preview::Config) -> Handle<Image> {
    let target = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(offscreen_image(config.width, config.height));
    app.world_mut().spawn((
        Camera2d,
        IsDefaultUiCamera,
        RenderTarget::Image(target.clone().into()),
    ));
    app.world_mut().spawn((
        PrimaryWindow,
        Window {
            resolution: (config.width, config.height).into(),
            ..default()
        },
    ));
    target
}

fn offscreen_image(width: u32, height: u32) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[BACKGROUND_R, BACKGROUND_G, BACKGROUND_B, BACKGROUND_A],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage |=
        TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::TEXTURE_BINDING;
    image
}
// no test_usage necessary
