use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::player::{Player, PlayerPhysics};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierDebugRenderPlugin::default())
            .add_startup_system(setup_debug_info);

        let asset_server = app.world.resource::<AssetServer>();

        let font = asset_server.load("fonts/RobotoMono/RobotoMono-VariableFont_wght.ttf");
        let text_style = TextStyle {
            font: font.clone(),
            font_size: 20.0,
            color: Color::WHITE,
        };

        app.insert_resource(DebugTextStyle(text_style));

        app.add_system(debug_position);
        app.add_system(debug_velocity);
        app.add_system(debug_physics);
    }
}

#[derive(Resource)]
struct DebugTextStyle(TextStyle);

#[derive(Component)]
struct DebugPosition;

#[derive(Component)]
struct DebugVelocity;

#[derive(Component)]
struct DebugPhysics;

fn setup_debug_info(mut commands: Commands, text_style: Res<DebugTextStyle>) {
    let DebugTextStyle(ref text_style) = *text_style;

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                justify_content: JustifyContent::Start,
                flex_direction: FlexDirection::Column,
                padding: UiRect::left(Val::Px(5.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section("Debug Info", text_style.clone()).with_style(Style {
                    margin: UiRect::vertical(Val::Px(5.0)),
                    ..default()
                }),
                Label,
            ));

            parent.spawn(NodeBundle::default()).with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section("Position: ", text_style.clone()).with_style(Style {
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    }),
                    Label,
                ));

                parent.spawn((
                    TextBundle::from_section("(0.0, 0.0)", text_style.clone()).with_style(Style {
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    }),
                    Label,
                    DebugPosition,
                ));
            });

            parent.spawn(NodeBundle::default()).with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section("Velocity: ", text_style.clone()).with_style(Style {
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    }),
                    Label,
                ));

                parent.spawn((
                    TextBundle::from_section("(0.0, 0.0)", text_style.clone()).with_style(Style {
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    }),
                    Label,
                    DebugVelocity,
                ));
            });

            parent.spawn(NodeBundle::default()).with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section("Physics: ", text_style.clone()).with_style(Style {
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    }),
                    Label,
                ));

                parent.spawn((
                    TextBundle::from_section("{}", text_style.clone()).with_style(Style {
                        margin: UiRect::vertical(Val::Px(5.0)),
                        ..default()
                    }),
                    Label,
                    DebugPhysics,
                ));
            });
        });
}

fn debug_position(
    text_style: Res<DebugTextStyle>,
    mut debug: Query<&mut Text, With<DebugPosition>>,
    transform: Query<&Transform, With<Player>>,
) {
    let Ok(mut debug) = debug.get_single_mut() else { return };
    let Ok(transform) = transform.get_single() else { return };

    let DebugTextStyle(ref text_style) = *text_style;

    *debug = Text::from_section(
        format!(
            "({:.2}, {:.2})",
            transform.translation.x, transform.translation.y
        ),
        text_style.clone(),
    );
}

fn debug_velocity(
    text_style: Res<DebugTextStyle>,
    mut debug: Query<&mut Text, With<DebugVelocity>>,
    velocity: Query<&Velocity, With<Player>>,
) {
    let Ok(mut debug) = debug.get_single_mut() else { return };
    let Ok(velocity) = velocity.get_single() else { return };

    let DebugTextStyle(ref text_style) = *text_style;

    *debug = Text::from_section(
        format!("({:.2}, {:.2})", velocity.linvel.x, velocity.linvel.y),
        text_style.clone(),
    );
}

fn debug_physics(
    text_style: Res<DebugTextStyle>,
    mut debug: Query<&mut Text, With<DebugPhysics>>,
    physics: Query<&PlayerPhysics>,
) {
    let Ok(mut debug) = debug.get_single_mut() else { return };
    let Ok(physics) = physics.get_single() else { return };

    let DebugTextStyle(ref text_style) = *text_style;

    *debug = Text::from_section(format!("{:?}", physics), text_style.clone());
}
