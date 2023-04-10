#[cfg(feature = "native")]
use std::time::Instant;

use bevy::{render::camera::Viewport, utils::Duration};

#[cfg(feature = "browser")]
use stdweb::web::Date;

use bevy::{prelude::*, time::Stopwatch};
use bevy_ecs_ldtk::prelude::*;
use bevy_pixel_camera::PixelCameraBundle;
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};

use crate::{
    animator::{AnimationIndices, AnimationTimer, DamageFlash},
    enemies::EnemyDamageActivator,
    world::{StandardFont, WorldCollider},
    GameState, GameTimer,
};

use self::abilities::DamageEffect;

pub mod abilities;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_pixel_camera::PixelCameraPlugin)
            .register_ldtk_entity::<PlayerBundle>("Player")
            .add_startup_system(spawn_camera)
            .insert_resource(PlayerHealth::default())
            .add_systems((
                on_player_spawn,
                player_physics_checks,
                player_movement.after(player_physics_checks),
                camera_controller,
                update_viewport,
                update_player_health_ui,
                game_over,
                switch_levels,
                update_timer,
                spawn_player_ui,
                despawn_player_ui,
            ));

        app.add_plugin(abilities::AbilityPlugin);

        let asset_server = app.world.resource::<AssetServer>();
        app.insert_resource(HeartImages {
            full: asset_server.load("images/heart/full.png"),
            half: asset_server.load("images/heart/half.png"),
            empty: asset_server.load("images/heart/empty.png"),
            full_flash: asset_server.load("images/heart/full_flash.png"),
            half_flash: asset_server.load("images/heart/half_flash.png"),
        });
    }
}

#[derive(Resource)]
pub struct PlayerHealth(pub i32);

impl Default for PlayerHealth {
    fn default() -> Self {
        Self(6)
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Default)]
pub struct PlayerPhysics {
    pub total_ground_collisions: i32,
    pub grounded: bool,
    pub slamming: bool,
    #[cfg(feature = "native")]
    pub early_jump: Option<Instant>,
    #[cfg(feature = "browser")]
    pub early_jump: Option<f64>,
    #[cfg(feature = "native")]
    pub coyote_time: Option<Instant>,
    #[cfg(feature = "browser")]
    pub coyote_time: Option<f64>,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub player_physics: PlayerPhysics,
    pub rigidbody: RigidBody,
    pub velocity: Velocity,
    pub damping: Damping,
    pub mass: ColliderMassProperties,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub locked_axes: LockedAxes,
    pub animation_indices: AnimationIndices,
    pub animation_timer: AnimationTimer,
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
}

impl LdtkEntity for PlayerBundle {
    fn bundle_entity(
        _: &EntityInstance,
        _: &LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&TilesetDefinition>,
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Self {
        let texture = asset_server.load("images/cloak_spritesheet.png");
        let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(32., 32.), 2, 2, None, None);
        let texture_atlas = texture_atlases.add(texture_atlas);

        Self {
            player: Player,
            player_physics: PlayerPhysics::default(),
            rigidbody: RigidBody::Dynamic,
            velocity: Velocity::zero(),
            damping: Damping {
                linear_damping: 8.,
                angular_damping: 0.,
            },
            mass: ColliderMassProperties::Density(2.0),
            collider: Collider::capsule_y(5., 11.),
            collision_groups: CollisionGroups::new(Group::GROUP_2, Group::GROUP_1 | Group::GROUP_4 | Group::GROUP_6),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            animation_indices: AnimationIndices { first: 0, last: 3 },
            animation_timer: AnimationTimer(Timer::from_seconds(1. / 12., TimerMode::Repeating)),
            sprite: TextureAtlasSprite::default(),
            texture_atlas,
        }
    }
}

#[derive(Component)]
pub struct PlayerGroundSensor;

#[derive(Component)]
pub struct PlayerDamageSensor;

fn on_player_spawn(mut commands: Commands, player: Query<Entity, Added<Player>>) {
    let Ok(player) = player.get_single() else { return };
    commands.entity(player).with_children(|parent| {
        parent.spawn((
            PlayerGroundSensor,
            Sensor,
            Collider::cuboid(8., 8.),
            ActiveEvents::COLLISION_EVENTS,
            ActiveHooks::MODIFY_SOLVER_CONTACTS,
            CollisionGroups::new(Group::GROUP_3, Group::GROUP_1 | Group::GROUP_4),
            TransformBundle {
                local: Transform::from_xyz(0., -11., 0.),
                ..default()
            },
        ));

        parent.spawn((
            PlayerDamageSensor,
            Sensor,
            Collider::capsule_y(5., 12.),
            ActiveEvents::COLLISION_EVENTS,
            CollisionGroups::new(Group::GROUP_5, Group::GROUP_5),
            TransformBundle::default(),
        ));
    });
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource)]
struct HeartImages {
    full: Handle<Image>,
    half: Handle<Image>,
    empty: Handle<Image>,
    full_flash: Handle<Image>,
    half_flash: Handle<Image>,
}

#[derive(Component)]
struct Heart<const ID: u8>;

#[derive(Component)]
struct GameTimerUi;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        PixelCameraBundle::from_resolution(480, 320),
        VisibilityBundle::default(),
    ));
}

#[derive(Component)]
struct PlayerUi;

fn spawn_player_ui(
    mut commands: Commands,
    camera: Query<Entity, With<MainCamera>>,
    game_state: Res<GameState>,
    heart_images: Res<HeartImages>,
) {
    if game_state.is_changed() && *game_state == GameState::Gameplay {
        let Ok(camera) = camera.get_single() else { return };
        commands.entity(camera).with_children(|parent| {
            parent
                .spawn(PlayerUi)
                .insert(SpatialBundle::default())
                .with_children(|parent| {
                    parent
                        .spawn(SpriteBundle {
                            texture: heart_images.full.clone(),
                            transform: Transform::from_xyz(-208., -128., -1.),
                            ..default()
                        })
                        .insert(Heart::<0>);

                    parent
                        .spawn(SpriteBundle {
                            texture: heart_images.full.clone(),
                            transform: Transform::from_xyz(-172., -128., -1.),
                            ..default()
                        })
                        .insert(Heart::<1>);

                    parent
                        .spawn(SpriteBundle {
                            texture: heart_images.full.clone(),
                            transform: Transform::from_xyz(-136., -128., -1.),
                            ..default()
                        })
                        .insert(Heart::<2>);

                    parent
                        .spawn(Text2dBundle {
                            transform: Transform::from_xyz(0., 150., -1.),
                            ..default()
                        })
                        .insert(GameTimerUi);
                });
        });
    }
}

fn despawn_player_ui(
    mut commands: Commands,
    ui: Query<Entity, With<PlayerUi>>,
    game_state: Res<GameState>,
) {
    if game_state.is_changed() && *game_state != GameState::Gameplay {
        let Ok(ui) = ui.get_single() else { return };
        commands.entity(ui).despawn_recursive();
    }
}

fn camera_controller(
    player_transform: Query<&Transform, With<Player>>,
    mut camera_transform: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    if let Ok(mut camera_transform) = camera_transform.get_single_mut() {
        if let Ok(player_transform) = player_transform.get_single() {
            let player_pos = player_transform.translation;
            camera_transform.translation = Vec3::new(player_pos.x, player_pos.y + 75.0, 10f32);
        }
    }
}

fn update_viewport(mut cameras: Query<&mut Camera, With<MainCamera>>, windows: Query<&Window>) {
    let Ok(mut camera) = cameras.get_single_mut() else { return };
    let Ok(window) = windows.get_single() else { return };

    let res = window.resolution.clone();
    let (width, height) = (res.physical_width(), res.physical_height());

    let w_scale = width / 480;
    let h_scale = height / 320;

    let scale = if w_scale < h_scale { w_scale } else { h_scale };

    let i_width = 480 * scale;
    let i_height = 320 * scale;

    let x = (width - i_width) / 2;
    let y = (height - i_height) / 2;

    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(x, y),
        physical_size: UVec2::new(i_width, i_height),
        depth: 0f32..1f32,
    });
}

const GROUND_FORCE: f32 = 5000f32;
const AIR_FORCE: f32 = 2500f32;
const JUMP_IMPULSE: f32 = 1000f32;
const SLAM_FORCE: f32 = 5000f32;

const MAX_GROUND_SPEED: f32 = 1500f32;
const MAX_AIR_SPEED: f32 = 1000f32;

#[cfg(feature = "native")]
const EARLY_JUMP_TIME: Duration = Duration::from_millis(40);
#[cfg(feature = "browser")]
const EARLY_JUMP_TIME: f64 = 40.0;
#[cfg(feature = "native")]
const COYOTE_TIME: Duration = Duration::from_millis(100);
#[cfg(feature = "browser")]
const COYOTE_TIME: f64 = 40.0;

const EASY_UP_GRAVITY: f32 = 9.81 * 25f32;
const UP_GRAVITY: f32 = 9.81 * 100f32;
const EASY_DOWN_GRAVITY: f32 = 9.81 * 200f32;
const DOWN_GRAVITY: f32 = 9.81 * 275f32;

fn player_movement(
    mut player: Query<(&mut Velocity, &mut TextureAtlasSprite, &mut PlayerPhysics), With<Player>>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    state: Res<GameState>,
) {
    if *state != GameState::Gameplay {
        return;
    };
    let Ok((mut velocity, mut sprite, mut physics)) = player.get_single_mut() else { return };
    #[cfg(feature = "native")]
    let now = Instant::now();
    #[cfg(feature = "browser")]
    let now = Date::now();
    let prev_velocity = velocity.linvel.clone();
    let mut new_velocity = Vec2::ZERO;
    let mut new_impulse = Vec2::ZERO;
    let mut x_input = 0f32;
    let mut just_jumped = false;
    let mut jump = false;
    let mut crouch = false;

    if keys.pressed(KeyCode::D) {
        x_input += 1.;
    }
    if keys.pressed(KeyCode::A) {
        x_input -= 1.;
    }
    if keys.just_pressed(KeyCode::Space) {
        just_jumped = true;
    }
    if keys.pressed(KeyCode::Space) {
        jump = true;
    }
    if keys.just_pressed(KeyCode::LControl) {
        crouch = true;
    }

    if x_input != 0. {
        sprite.flip_x = x_input.is_sign_negative();
    }

    let mut max_speed = MAX_GROUND_SPEED;

    let mut is_early_jump = false;
    if let Some(early_jump) = physics.early_jump {
        #[cfg(feature = "native")]
        let val = Instant::now() - early_jump < EARLY_JUMP_TIME;
        #[cfg(feature = "browser")]
        let val = Date::now() - early_jump < EARLY_JUMP_TIME;
        if val {
            is_early_jump = true;
        } else {
            physics.early_jump = None;
        }
    }

    let mut is_coyote_time = false;
    if let Some(coyote_time) = physics.coyote_time {
        #[cfg(feature = "native")]
        let val = Instant::now() - coyote_time < COYOTE_TIME;
        #[cfg(feature = "browser")]
        let val = Date::now() - coyote_time < COYOTE_TIME;
        if val {
            is_coyote_time = true;
        } else {
            physics.coyote_time = None;
        }
    }

    if physics.grounded || is_coyote_time {
        if just_jumped || is_early_jump {
            new_impulse.y += JUMP_IMPULSE;
            physics.coyote_time = None;
        } else if physics.grounded {
            physics.coyote_time = Some(now);
        }
        new_velocity.x += x_input * GROUND_FORCE;
        physics.slamming = false;
    } else {
        if crouch || physics.slamming {
            new_velocity.y -= SLAM_FORCE;
            if crouch {
                physics.slamming = true;
            }
        } else if just_jumped {
            physics.early_jump = Some(now);
        }

        new_velocity.x += x_input * AIR_FORCE;
        max_speed = MAX_AIR_SPEED;

        if prev_velocity.y >= 0. {
            if jump {
                new_velocity.y -= EASY_UP_GRAVITY;
            } else {
                new_velocity.y -= UP_GRAVITY;
            }
        } else {
            if jump {
                new_velocity.y -= EASY_DOWN_GRAVITY;
            } else {
                new_velocity.y -= DOWN_GRAVITY;
            }
        }
    }

    let max_speed = max_speed.max(prev_velocity.length());

    let clamped_velocity = Vec2::new(new_velocity.x.clamp(-max_speed, max_speed), new_velocity.y);

    velocity.linvel = clamped_velocity * time.delta_seconds() + prev_velocity + new_impulse;
}

fn player_physics_checks(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut player: Query<(Entity, &mut PlayerPhysics)>,
    mut health: ResMut<PlayerHealth>,
    ground_sensor: Query<Entity, With<PlayerGroundSensor>>,
    damage_sensor: Query<Entity, With<PlayerDamageSensor>>,
    damage_activator: Query<(&Parent, &EnemyDamageActivator)>,
    damage_effect: Query<&DamageEffect>,
) {
    let Ok((entity, mut physics)) = player.get_single_mut() else { return };
    let Ok(ground_sensor) = ground_sensor.get_single() else { return };
    let Ok(damage_sensor) = damage_sensor.get_single() else { return };

    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(a, b, flags) => {
                if *flags & CollisionEventFlags::SENSOR != CollisionEventFlags::SENSOR {
                    continue;
                };

                if *a == ground_sensor || *b == ground_sensor {
                    physics.total_ground_collisions += 1;
                    if physics.total_ground_collisions > 0 {
                        physics.grounded = true;
                    }
                    continue;
                }

                let activator = if *a == damage_sensor {
                    b
                } else if *b == damage_sensor {
                    a
                } else {
                    continue;
                };

                let Ok((parent, activator)) = damage_activator.get(*activator) else { continue };

                let effect = damage_effect.get(**parent);

                let multiplier = match effect {
                    Ok(effect) => effect.multiplier,
                    Err(_) => 1.0,
                };

                health.0 += (activator.0 as f32 * multiplier) as i32;
                commands.entity(entity).insert(DamageFlash::default());
            }
            CollisionEvent::Stopped(a, b, flags) => {
                if *flags & CollisionEventFlags::SENSOR != CollisionEventFlags::SENSOR {
                    continue;
                };

                if *a == ground_sensor || *b == ground_sensor {
                    physics.total_ground_collisions -= 1;
                    if physics.total_ground_collisions < 1 {
                        physics.grounded = false;
                    }
                }
            }
        }
    }
}

fn game_over(
    health: Res<PlayerHealth>,
    mut game_state: ResMut<GameState>,
) {
    if health.0 > 0 {
        return;
    };
    if *game_state != GameState::GameOver {
        *game_state = GameState::GameOver;
    }
}

fn update_player_health_ui(
    health: Res<PlayerHealth>,
    mut heart_0: Query<&mut Handle<Image>, With<Heart<0>>>,
    mut heart_1: Query<&mut Handle<Image>, (With<Heart<1>>, Without<Heart<0>>)>,
    mut heart_2: Query<&mut Handle<Image>, (With<Heart<2>>, Without<Heart<1>>, Without<Heart<0>>)>,
    heart_images: Res<HeartImages>,
    time: Res<Time>,
    mut stopwatch: Local<Stopwatch>,
    mut flash: Local<bool>,
) {
    let Ok(mut heart_0) = heart_0.get_single_mut() else { return };
    let Ok(mut heart_1) = heart_1.get_single_mut() else { return };
    let Ok(mut heart_2) = heart_2.get_single_mut() else { return };

    let hp = health.0;

    stopwatch.tick(time.delta());
    let elapsed = stopwatch.elapsed();

    if *flash && elapsed > Duration::from_millis(250) {
        *flash = false;
        stopwatch.reset();
    } else if !*flash && elapsed > Duration::from_millis(300 * (hp.max(0)) as u64) {
        *flash = true;
        stopwatch.reset();
    }

    let (hp0, hp1, hp2) = (
        {
            if hp > 1 {
                2
            } else if hp == 1 {
                1
            } else {
                0
            }
        },
        {
            if hp > 3 {
                2
            } else if hp == 3 {
                1
            } else {
                0
            }
        },
        {
            if hp > 5 {
                2
            } else if hp == 5 {
                1
            } else {
                0
            }
        },
    );

    let empty = heart_images.empty.clone();
    let half = match *flash {
        false => heart_images.half.clone(),
        true => heart_images.half_flash.clone(),
    };
    let full = match *flash {
        false => heart_images.full.clone(),
        true => heart_images.full_flash.clone(),
    };

    *heart_0 = match hp0 {
        0 => empty.clone(),
        1 => half.clone(),
        _ => full.clone(),
    };

    *heart_1 = match hp1 {
        0 => empty.clone(),
        1 => half.clone(),
        _ => full.clone(),
    };

    *heart_2 = match hp2 {
        0 => empty.clone(),
        1 => half.clone(),
        _ => full.clone(),
    };
}

fn switch_levels(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut level_selection: ResMut<LevelSelection>,
    world: Query<Entity, With<WorldCollider>>,
) {
    let Ok(player) = player.get_single() else { return };

    if player.translation.y < 128.0 {
        let LevelSelection::Index(i) = &mut *level_selection else { return };
        *i += 1;
        for collider in world.iter() {
            commands.entity(collider).despawn();
        }
    }
}

fn update_timer(
    mut timer_ui: Query<&mut Text, With<GameTimerUi>>,
    mut timer: ResMut<GameTimer>,
    time: Res<Time>,
    font: Res<StandardFont>,
    mut game_state: ResMut<GameState>,
) {
    if *game_state != GameState::Gameplay {
        return;
    };

    let Ok(mut timer_ui) = timer_ui.get_single_mut() else { return };

    timer.0.tick(time.delta());

    let remaining = timer.0.remaining_secs();

    let minutes = (remaining / 60.0) as u32;
    let seconds = (remaining % 60.0) as u32;

    let color = if remaining < 30.0 {
        if seconds % 2 == 0 {
            Color::RED
        } else {
            Color::WHITE
        }
    } else {
        Color::WHITE
    };

    let style = TextStyle {
        font: font.0.clone(),
        font_size: 20.0,
        color,
    };

    *timer_ui = Text::from_section(format!("{:0>2}:{:0>2}", minutes, seconds), style)
        .with_alignment(TextAlignment::Center);

    if timer.0.finished() {
        if *game_state != GameState::GameOver {
            *game_state = GameState::GameOver;
        }
    }
}
