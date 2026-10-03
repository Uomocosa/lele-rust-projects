//! Replacement lobby UI used by the discovery e2e test (`--ui custom`).
//!
//! It shows that the plugin's default UI can be swapped out entirely: this
//! plugin only reads `discovery::Snapshot` and writes `discovery::Command`.
#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;
use bevy::text::EditableText;
use derive_more::Deref;
use freenet_libp2p_bevy_plugin::discovery;

const PANEL_COLOR: Color = Color::srgb(0.30, 0.12, 0.34);
const FIELD_COLOR: Color = Color::srgb(0.10, 0.10, 0.12);
const ROOM_COLOR: Color = Color::srgb(0.12, 0.28, 0.38);

pub struct LobbyUi;

impl Plugin for LobbyUi {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (rebuild_list, on_create, on_join));
    }
}

#[derive(Component)]
pub struct NameField;

#[derive(Component)]
pub struct NewRoomButton;

#[derive(Component)]
struct RoomColumn;

#[derive(Component, Deref)]
pub struct JoinButton(pub discovery::RoomName);

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
                NameField,
                EditableText::new(""),
                Node {
                    width: Val::Px(200.0),
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(FIELD_COLOR),
            ),
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
    snapshot: Res<discovery::Snapshot>,
    column: Single<Entity, With<RoomColumn>>,
) {
    if !snapshot.is_changed() {
        return;
    }
    tracing::info!("lobby ui rooms={}", snapshot.directory.len());
    commands
        .entity(*column)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for (name, record) in &snapshot.directory {
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
    field: Single<&EditableText, With<NameField>>,
    mut commands: MessageWriter<discovery::Command>,
) {
    for interaction in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let typed = field.value().to_string();
        let name = typed.trim();
        if name.is_empty() {
            continue;
        }
        tracing::info!("lobby ui click create room={name}");
        commands.write(discovery::Command::Create(discovery::RoomName(name.to_string())));
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
