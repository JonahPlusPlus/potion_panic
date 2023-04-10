use bevy::{app::AppExit, prelude::*};

#[cfg(feature = "native")]
use bevy::{window::PrimaryWindow, winit::WinitWindows};
use bevy::utils::Duration;
use bevy_ecs_ldtk::LevelSelection;
use enemies::DamageGiven;
use player::{MainCamera, PlayerHealth};
use world::{StandardFont, CursiveFont};

mod animator;
#[cfg(debug_assertions)]
mod debug;
mod enemies;
mod player;
mod sound;
mod world;

const GAME_TIME: u64 = 180;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Potion Panic!".into(),
                    canvas: Some("#game".to_owned()),
                    fit_canvas_to_parent: true,
                    resize_constraints: WindowResizeConstraints {
                        min_width: 480.,
                        min_height: 320.,
                        max_width: 2400.,
                        max_height: 1600.,
                    },
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugin(world::WorldPlugin)
    .add_plugin(animator::AnimatorPlugin)
    .add_plugin(sound::SoundPlugin)
    .add_plugin(player::PlayerPlugin)
    .add_plugin(enemies::EnemyPlugin);

    #[cfg(debug_assertions)]
    app.add_plugin(debug::DebugPlugin);

    app.insert_resource(GameState::StartMenu);
    app.insert_resource(GameTimer(Timer::new(
        Duration::from_secs(GAME_TIME),
        TimerMode::Once,
    )));
    app.add_startup_system(spawn_start_menu);
    app.add_system(start_menu);
    app.add_system(despawn_start_menu);

    app.add_system(spawn_game_over);
    app.add_system(game_over);
    app.add_system(despawn_game_over);

    app.add_system(spawn_win_screen);
    app.add_system(win_screen);
    app.add_system(despawn_win_screen);

    #[cfg(feature = "native")]
    app.add_startup_system(set_window_icon);

    app.run();
}

#[derive(Resource, Eq, PartialEq)]
pub enum GameState {
    StartMenu,
    Gameplay,
    GameOver,
    WinScreen,
}

#[derive(Resource)]
pub struct GameTimer(pub Timer);

#[derive(Component)]
struct StartMenu;

fn spawn_start_menu(mut commands: Commands, game_state: Res<GameState>, font: Res<StandardFont>) {
    if *game_state != GameState::StartMenu {
        return;
    }

    commands
        .spawn(StartMenu)
        .insert(SpatialBundle::default())
        .with_children(|parent| {
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    "Potion Panic!",
                    TextStyle {
                        font: font.0.clone(),
                        font_size: 75.0,
                        color: Color::WHITE,
                    },
                )
                .with_alignment(TextAlignment::Center),
                ..default()
            });

            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    "[Press Space to Start]",
                    TextStyle {
                        font: font.0.clone(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                )
                .with_alignment(TextAlignment::Center),
                transform: Transform::from_xyz(0., -64.0, 0.),
                ..default()
            });
        });
}

fn start_menu(mut game_state: ResMut<GameState>, keys: Res<Input<KeyCode>>) {
    if *game_state != GameState::StartMenu {
        return;
    }

    if keys.just_pressed(KeyCode::Space) {
        *game_state = GameState::Gameplay;
    }
}

fn despawn_start_menu(
    mut commands: Commands,
    game_state: Res<GameState>,
    start_menu: Query<Entity, With<StartMenu>>,
) {
    if game_state.is_changed() && *game_state != GameState::StartMenu {
        let Ok(start_menu) = start_menu.get_single() else { return };
        commands.entity(start_menu).despawn_recursive();
    }
}

#[derive(Component)]
struct GameOver;

fn spawn_game_over(
    mut commands: Commands,
    game_state: Res<GameState>,
    font: Res<StandardFont>,
    camera: Query<Entity, With<MainCamera>>,
) {
    if game_state.is_changed() && *game_state == GameState::GameOver {
        let Ok(camera) = camera.get_single() else { return };

        commands.entity(camera).with_children(|parent| {
            parent
                .spawn(GameOver)
                .insert(SpatialBundle::default())
                .with_children(|parent| {
                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            "Game Over",
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 75.0,
                                color: Color::RED,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                        ..default()
                    });

                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            "[Press Space to Restart]",
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 20.0,
                                color: Color::RED,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                        transform: Transform::from_xyz(0., -64.0, 0.),
                        ..default()
                    });

                    #[cfg(feature = "native")]
                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            "[Press Q to Quit]",
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 20.0,
                                color: Color::RED,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                        transform: Transform::from_xyz(0., -96.0, 0.),
                        ..default()
                    });
                });
        });
    }
}

fn game_over(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    keys: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if *game_state != GameState::GameOver {
        return;
    }

    if keys.just_pressed(KeyCode::Space) {
        *game_state = GameState::Gameplay;
        commands.insert_resource(GameTimer(Timer::new(
            Duration::from_secs(GAME_TIME),
            TimerMode::Once,
        )));
        commands.insert_resource(PlayerHealth::default());
        commands.insert_resource(LevelSelection::Index(0));
        commands.insert_resource(DamageGiven(false));
    }

    if keys.just_pressed(KeyCode::Q) {
        exit.send(AppExit);
    }
}

fn despawn_game_over(
    mut commands: Commands,
    game_over: Query<Entity, With<GameOver>>,
    game_state: Res<GameState>,
) {
    if game_state.is_changed() && *game_state != GameState::GameOver {
        for game_over in game_over.iter() {
            commands.entity(game_over).despawn_recursive();
        }
    }
}

#[derive(Component)]
struct WinScreen;

fn spawn_win_screen(
    mut commands: Commands,
    game_state: Res<GameState>,
    font: Res<StandardFont>,
    cursive_font: Res<CursiveFont>,
    camera: Query<Entity, With<MainCamera>>,
    damage_given: Res<DamageGiven>,
    player_health: Res<PlayerHealth>,
) {
    if game_state.is_changed() && *game_state == GameState::WinScreen {
        let Ok(camera) = camera.get_single() else { return };

        commands.entity(camera).with_children(|parent| {
            parent
                .spawn(WinScreen)
                .insert(SpatialBundle::default())
                .with_children(|parent| {
                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            "You Win!",
                            TextStyle {
                                font: cursive_font.0.clone(),
                                font_size: 75.0,
                                color: Color::GOLD,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                        ..default()
                    });

                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            "[Press Space to Play Again]",
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 20.0,
                                color: Color::GOLD,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                        transform: Transform::from_xyz(0., -64.0, 0.),
                        ..default()
                    });

                    #[cfg(feature = "native")]
                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            "[Press Q to Quit]",
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 20.0,
                                color: Color::GOLD,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                        transform: Transform::from_xyz(0., -96.0, 0.),
                        ..default()
                    });

                    let damage_taken_color = if player_health.0 == 6 {
                        Color::GREEN
                    } else {
                        Color::RED
                    };

                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            "Don't take damage.",
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 20.0,
                                color: damage_taken_color,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                        transform: Transform::from_xyz(-128., -128.0, 0.),
                        ..default()
                    });

                    let damage_given_color = if !damage_given.0 {
                        Color::GREEN
                    } else {
                        Color::RED
                    };

                    parent.spawn(Text2dBundle {
                        text: Text::from_section(
                            "Don't hurt enemies.",
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 20.0,
                                color: damage_given_color,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                        transform: Transform::from_xyz(128., -128.0, 0.),
                        ..default()
                    });
                });
        });
    }
}

fn win_screen(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    keys: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if *game_state != GameState::WinScreen {
        return;
    }

    if keys.just_pressed(KeyCode::Space) {
        *game_state = GameState::Gameplay;
        commands.insert_resource(GameTimer(Timer::new(
            Duration::from_secs(GAME_TIME),
            TimerMode::Once,
        )));
        commands.insert_resource(PlayerHealth::default());
        commands.insert_resource(LevelSelection::Index(0));
        commands.insert_resource(DamageGiven(false));
    }

    if keys.just_pressed(KeyCode::Q) {
        exit.send(AppExit);
    }
}

fn despawn_win_screen(
    mut commands: Commands,
    win_screen: Query<Entity, With<WinScreen>>,
    game_state: Res<GameState>,
) {
    if game_state.is_changed() && *game_state != GameState::WinScreen {
        for win_screen in win_screen.iter() {
            commands.entity(win_screen).despawn_recursive();
        }
    }
}

#[cfg(feature = "native")]
fn set_window_icon(
    primary: Query<Entity, With<PrimaryWindow>>,
    winit_windows: NonSend<WinitWindows>,
) {
    let Ok(primary) = primary.get_single() else { return; };

    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/images/logo.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = winit::window::Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    let Some(window_id) = winit_windows.entity_to_winit.get(&primary) else { return };

    let Some(window) = winit_windows.windows.get(window_id) else { return };

    window.set_window_icon(Some(icon));
}
