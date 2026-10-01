//! Replacement lobby UI used by the discovery e2e test (`--ui custom`).
//!
//! It shows that the plugin's default UI can be swapped out entirely: this
//! plugin only reads `discovery::Multiplayer` and writes `discovery::Command`.
#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;
use derive_more::Deref;
use freenet_libp2p_bevy_plugin::discovery;

const PANEL_COLOR: Color = Color::srgb(0.30, 0.12, 0.34);
const ROOM_COLOR: Color = Color::srgb(0.12, 0.28, 0.38);

pub struct LobbyUi;

impl Plugin for LobbyUi {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (rebuild_list, on_create, on_join));
    }
}

#[derive(Component)]
pub struct NewRoomButton;

#[derive(Component)]
struct RoomColumn;

#[derive(Component, Deref)]
pub struct JoinButton(pub discovery::id::RoomName);

fn spawn(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            right: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(PANEL_COLOR),
        children![
            (
                NewRoomButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    ..default()
                },
                children![Text::new("+ New lobby")],
            ),
            (
                RoomColumn,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
            ),
        ],
    ));
}

fn rebuild_list(
    mut commands: Commands,
    multiplayer: Res<discovery::Multiplayer>,
    column: Single<Entity, With<RoomColumn>>,
) {
    if !multiplayer.is_changed() {
        return;
    }
    tracing::info!("lobby ui rooms={}", multiplayer.catalogue.len());
    commands
        .entity(*column)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for (name, record) in &multiplayer.catalogue {
                parent.spawn((
                    JoinButton(name.clone()),
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(ROOM_COLOR),
                    children![Text::new(format!(
                        "> {} [{} online]",
                        name.as_str(),
                        record.members.len()
                    ))],
                ));
            }
        });
}

fn on_create(
    buttons: Query<&Interaction, (Changed<Interaction>, With<NewRoomButton>)>,
    mut commands: MessageWriter<discovery::Command>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            let name = format!("lobby-{}", *discovery::id::now_epoch());
            tracing::info!("lobby ui click create room={name}");
            commands.write(discovery::Command::Create(discovery::id::RoomName(name)));
        }
    }
}

fn on_join(
    buttons: Query<(&Interaction, &JoinButton), Changed<Interaction>>,
    mut commands: MessageWriter<discovery::Command>,
) {
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed {
            tracing::info!("lobby ui click join room={}", button.as_str());
            commands.write(discovery::Command::Join((**button).clone()));
        }
    }
}
