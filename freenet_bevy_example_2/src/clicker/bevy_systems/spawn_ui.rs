use bevy::prelude::*;

use crate::clicker;

pub fn spawn_ui(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(20.0),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(16.0),
                                height: Val::Px(16.0),
                                border_radius: BorderRadius::MAX,
                                ..Default::default()
                            },
                            BackgroundColor::from(Color::srgb(0.9, 0.6, 0.1)),
                            Interaction::default(),
                            clicker::StatusBubble,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: Val::Px(24.0),
                                    left: Val::Px(-8.0),
                                    max_width: Val::Px(220.0),
                                    padding: UiRect::all(Val::Px(8.0)),
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                    ..Default::default()
                                },
                                BackgroundColor::from(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                                Visibility::Hidden,
                                Text::new("connecting..."),
                                TextFont {
                                    font_size: FontSize::Px(14.0),
                                    ..Default::default()
                                },
                                TextColor(Color::WHITE),
                                clicker::StatusTooltip,
                            ));
                        });

                    parent.spawn((
                        Text::new("Freenet Clicker"),
                        TextFont {
                            font_size: FontSize::Px(48.0),
                            ..Default::default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent.spawn((
                Text::new("0"),
                TextFont {
                    font_size: FontSize::Px(96.0),
                    ..Default::default()
                },
                TextColor(Color::WHITE),
                clicker::CounterText,
            ));

            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..Default::default()
                    },
                    BorderColor::from(Color::WHITE),
                    BackgroundColor::from(Color::srgb(0.2, 0.2, 0.2)),
                    clicker::IncrementButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("+ Increment"),
                        TextFont {
                            font_size: FontSize::Px(32.0),
                            ..Default::default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(420.0),
                    row_gap: Val::Px(4.0),
                    ..Default::default()
                },
                clicker::LogContainer,
            ));
        });
}
// no test_usage necessary
