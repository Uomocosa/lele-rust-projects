use bevy::prelude::*;

use crate::clicker::component::CounterText::CounterText;
use crate::clicker::component::IncrementButton::IncrementButton;

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
            parent.spawn((
                Text::new("Freenet Clicker"),
                TextFont {
                    font_size: FontSize::Px(48.0),
                    ..Default::default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("0"),
                TextFont {
                    font_size: FontSize::Px(96.0),
                    ..Default::default()
                },
                TextColor(Color::WHITE),
                CounterText,
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
                    IncrementButton,
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
                Text::new("Connect to a Freenet node to participate"),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}
