use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::GameState;

use super::{MainCamera, Player};

mod green;
mod purple;

use green::GreenPotion;
use purple::PurplePotion;

#[derive(Component)]
pub struct Potion;

#[derive(Bundle)]
pub struct PotionBundle {
    pub potion: Potion,
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub active_events: ActiveEvents,
    pub collision_groups: CollisionGroups,
    pub dominance: Dominance,
}

impl Default for PotionBundle {
    fn default() -> Self {
        Self {
            potion: Potion,
            rigidbody: RigidBody::Dynamic,
            collider: Collider::ball(8.),
            active_events: ActiveEvents::COLLISION_EVENTS,
            collision_groups: CollisionGroups {
                memberships: Group::GROUP_5,
                filters: Group::GROUP_4 | Group::GROUP_1,
            },
            dominance: Dominance { groups: -1 },
        }
    }
}

pub trait Ability {
    fn splash_image(
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Handle<TextureAtlas>;

    fn ui_image(asset_server: &AssetServer) -> Handle<Image>;

    fn ui_position() -> f32;

    fn activate(
        commands: Commands,
        position: Vec3,
        velocity: Velocity,
        right: bool,
        asset_server: &AssetServer,
    );
}

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveAbility::Green)
            .insert_resource(AbilityCooldown::default())
            .add_system(spawn_ability_ui)
            .add_system(update_active_ability)
            .add_system(despawn_ability_ui)
            .add_system(update_ability_ui)
            .add_system(use_ability)
            .add_system(update_cooldowns)
            .add_system(update_potion_gravity);

        // Green
        app.add_system(green::checks);

        // Purple
        app.add_system(purple::checks);

        let asset_server = app.world.resource::<AssetServer>();
        let texture = asset_server.load("images/cooldown.png");

        let mut assets = app.world.resource_mut::<Assets<TextureAtlas>>();
        
        let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(32., 32.), 4, 5, None, None);
        let texture_atlas = assets.add(texture_atlas);

        app.insert_resource(CooldownSpritesheet(texture_atlas));
    }
}

#[derive(Resource, PartialEq, Eq)]
pub enum ActiveAbility {
    Green,
    Purple,
}

impl ActiveAbility {
    pub fn add(&mut self) {
        *self = match self {
            Self::Green => Self::Purple,
            Self::Purple => Self::Green,
        };
    }

    pub fn subtract(&mut self) {
        *self = match self {
            Self::Green => Self::Purple,
            Self::Purple => Self::Green,
        };
    }

    pub fn ui_position(&self) -> f32 {
        match self {
            Self::Green => GreenPotion::ui_position(),
            Self::Purple => PurplePotion::ui_position(),
        }
    }

    pub fn activate(
        &self,
        mut commands: Commands,
        camera: Entity,
        cooldown: &mut AbilityCooldown,
        cooldown_sheet: &CooldownSpritesheet,
        position: Vec3,
        velocity: Velocity,
        right: bool,
        asset_server: &AssetServer,
    ) {
        match self {
            Self::Green => {
                if cooldown.green.is_none() {
                    let timer = Timer::from_seconds(0.75, TimerMode::Once);
                    commands.entity(camera).with_children(|parent| {
                        parent.spawn((
                            Cooldown(timer.clone()),
                            SpriteSheetBundle {
                                texture_atlas: cooldown_sheet.0.clone(),
                                transform: Transform::from_xyz(164., GreenPotion::ui_position(), -1.),
                                ..default()
                            },
                        ));
                    });
                    GreenPotion::activate(commands, position, velocity, right, asset_server);
                    cooldown.green = Some(timer);
                }
            },
            Self::Purple => {
                if cooldown.purple.is_none() {
                    let timer = Timer::from_seconds(1.5, TimerMode::Once);
                    commands.entity(camera).with_children(|parent| {
                        parent.spawn((
                            Cooldown(timer.clone()),
                            SpriteSheetBundle {
                                texture_atlas: cooldown_sheet.0.clone(),
                                transform: Transform::from_xyz(164., PurplePotion::ui_position(), -1.),
                                ..default()
                            },
                        ));
                    });
                    PurplePotion::activate(commands, position, velocity, right, asset_server);
                    cooldown.purple = Some(timer);
                }
            }
        }
    }
}

#[derive(Component)]
pub struct AbilityUi;

#[derive(Component)]
pub struct ActiveAbilityUi;

fn spawn_ability_ui(
    mut commands: Commands,
    main_camera: Query<Entity, With<MainCamera>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_state: Res<GameState>,
) {
    let Ok(main_camera) = main_camera.get_single() else { return; };
    if game_state.is_changed() && *game_state == GameState::Gameplay {
        commands.entity(main_camera).with_children(|parent| {
            parent
                .spawn(AbilityUi)
                .insert(SpatialBundle::default())
                .with_children(|parent| {
                    parent
                        .spawn(ColorMesh2dBundle {
                            mesh: meshes
                                .add(shape::Quad::new(Vec2::new(64., 40.)).into())
                                .into(),
                            material: materials
                                .add(ColorMaterial::from(Color::rgba(0.5, 0.5, 0.5, 0.5))),
                            transform: Transform::from_xyz(216., GreenPotion::ui_position(), -2.),
                            ..default()
                        })
                        .insert(ActiveAbilityUi);

                    parent.spawn(SpriteBundle {
                        texture: GreenPotion::ui_image(&asset_server),
                        transform: Transform::from_xyz(208., GreenPotion::ui_position(), -1.),
                        ..default()
                    });

                    parent.spawn(SpriteBundle {
                        texture: PurplePotion::ui_image(&asset_server),
                        transform: Transform::from_xyz(208., PurplePotion::ui_position(), -1.),
                        ..default()
                    });
                });
        });
    }
}

fn despawn_ability_ui(
    mut commands: Commands,
    ui: Query<Entity, With<AbilityUi>>,
    game_state: Res<GameState>,
) {
    if game_state.is_changed() && *game_state != GameState::Gameplay {
        let Ok(ui) = ui.get_single() else { return };
        commands.entity(ui).despawn_recursive();
    }
}

fn update_active_ability(
    mut active: ResMut<ActiveAbility>,
    mut scroll_evr: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
) {
    let mut delta = 0.;
    for ev in scroll_evr.iter() {
        delta += ev.y;
    }

    if keys.just_pressed(KeyCode::W) {
        delta += 1.;
    }

    if keys.just_pressed(KeyCode::S) {
        delta -= 1.;
    }

    if delta > 0. {
        active.add();
    } else if delta < 0. {
        active.subtract();
    }
}

#[derive(Resource, Default)]
pub struct AbilityCooldown {
    green: Option<Timer>,
    purple: Option<Timer>,
}

#[derive(Resource)]
pub struct CooldownSpritesheet(Handle<TextureAtlas>);

#[derive(Component)]
pub struct Cooldown(pub Timer);

fn use_ability(
    commands: Commands,
    camera: Query<Entity, With<MainCamera>>,
    mut cooldown: ResMut<AbilityCooldown>,
    cooldown_sheet: Res<CooldownSpritesheet>,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    asset_server: Res<AssetServer>,
    player: Query<(&Transform, &Velocity, &TextureAtlasSprite), With<Player>>,
    active_ability: Res<ActiveAbility>,
    game_state: Res<GameState>,
) {
    if *game_state != GameState::Gameplay {
        return;
    };

    let Ok(camera) = camera.get_single() else { return };

    if keys.just_pressed(KeyCode::E) || buttons.just_pressed(MouseButton::Left) {
        let Ok((transform, velocity, sprite)) = player.get_single() else { return };

        let right = !sprite.flip_x;

        let position = if right {
            transform.translation + Vec3::X * 12.
        } else {
            transform.translation - Vec3::X * 12.
        };

        active_ability.activate(commands, camera, &mut *cooldown, &cooldown_sheet, position, *velocity, right, &*asset_server);
    }
}

fn update_ability_ui(
    mut ui: Query<&mut Transform, With<ActiveAbilityUi>>,
    active: Res<ActiveAbility>,
) {
    let Ok(mut ui) = ui.get_single_mut() else { return };

    ui.translation.y = active.ui_position();
}

const POTION_GRAVITY: f32 = 9.81 * 175f32;

fn update_potion_gravity(mut potions: Query<&mut Velocity, With<Potion>>, time: Res<Time>) {
    for mut velocity in potions.iter_mut() {
        velocity.linvel.y -= POTION_GRAVITY * time.delta_seconds();
    }
}

fn update_cooldowns(mut cooldown: ResMut<AbilityCooldown>, time: Res<Time>) {
    if let Some(green) = &mut cooldown.green {
        green.tick(time.delta());
        if green.finished() {
            cooldown.green = None;
        }
    }

    if let Some(purple) = &mut cooldown.purple {
        purple.tick(time.delta());
        if purple.finished() {
            cooldown.purple = None;
        }
    }
}

#[derive(Component)]
pub struct HealthEffect {
    pub amount: i32,
}

#[derive(Component)]
pub struct SpeedEffect {
    pub multiplier: f32,
}

#[derive(Component)]
pub struct DamageEffect {
    pub multiplier: f32,
}
