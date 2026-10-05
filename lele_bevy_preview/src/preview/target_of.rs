use bevy::image::Image;
use bevy::prelude::*;
use bevy::ui::IsDefaultUiCamera;

pub fn target_of(app: &mut App) -> Handle<Image> {
    let world = app.world_mut();
    let mut cameras =
        world.query_filtered::<&bevy::camera::RenderTarget, With<IsDefaultUiCamera>>();
    cameras
        .iter(world)
        .find_map(|target| match target {
            bevy::camera::RenderTarget::Image(image) => Some(image.handle.clone()),
            _ => None,
        })
        .unwrap_or_default()
}
// no test_usage necessary
