use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::LdtkEntity;
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};

use crate::{
    animator::*,
    player::abilities::{HealthEffect, SpeedEffect},
};

use super::{EnemyBundle, EnemyDamageActivator, EnemyGroundSensor, DamageGiven};

#[derive(Component)]
pub struct Skeleton {
    pub going_right: bool,
    pub left_sensor: i32,
    pub right_sensor: i32,
    pub hp: i32,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self {
            going_right: false,
            left_sensor: 0,
            right_sensor: 0,
            hp: 3,
        }
    }
}

#[derive(Bundle)]
pub struct SkeletonBundle {
    pub skeleton: Skeleton,
    pub enemy: EnemyBundle,
    pub animation_indices: AnimationIndices,
    pub animation_timer: AnimationTimer,
    pub texture_atlas: Handle<TextureAtlas>,
    pub collider: Collider,
    pub mass: ColliderMassProperties,
}

impl LdtkEntity for SkeletonBundle {
    fn bundle_entity(
        _: &bevy_ecs_ldtk::EntityInstance,
        _: &bevy_ecs_ldtk::prelude::LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&bevy_ecs_ldtk::prelude::TilesetDefinition>,
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Self {
        let texture = asset_server.load("images/enemies/skeleton_spritesheet.png");
        let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(32., 64.), 3, 2, None, None);
        let texture_atlas = texture_atlases.add(texture_atlas);

        Self {
            skeleton: Skeleton::default(),
            enemy: EnemyBundle::default(),
            animation_indices: AnimationIndices { first: 0, last: 4 },
            animation_timer: AnimationTimer(Timer::from_seconds(1. / 12., TimerMode::Repeating)),
            texture_atlas,
            collider: Collider::capsule_y(20., 11.),
            mass: ColliderMassProperties::Density(0.1),
        }
    }
}

#[derive(Component)]
pub struct SkeletonSensorRight;

#[derive(Component)]
pub struct SkeletonSensorLeft;

#[derive(Component)]
pub struct SkeletonDamageSensor;

pub fn on_skeleton_spawn(mut commands: Commands, skeletons: Query<Entity, Added<Skeleton>>) {
    for skeleton in skeletons.iter() {
        commands.entity(skeleton).with_children(|parent| {
            parent.spawn((
                SkeletonSensorRight,
                Sensor,
                Collider::cuboid(4., 16.),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(Group::GROUP_3, Group::GROUP_1 | Group::GROUP_3),
                TransformBundle {
                    local: Transform::from_xyz(10., 0., 0.),
                    ..default()
                },
            ));

            parent.spawn((
                SkeletonSensorLeft,
                Sensor,
                Collider::cuboid(4., 16.),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(Group::GROUP_3, Group::GROUP_1 | Group::GROUP_3),
                TransformBundle {
                    local: Transform::from_xyz(-10., 0., 0.),
                    ..default()
                },
            ));

            parent.spawn((
                EnemyGroundSensor,
                Sensor,
                Collider::cuboid(8., 8.),
                ActiveEvents::COLLISION_EVENTS,
                ActiveHooks::MODIFY_SOLVER_CONTACTS,
                CollisionGroups::new(
                    Group::GROUP_3,
                    Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_4,
                ),
                TransformBundle {
                    local: Transform::from_xyz(0., -26., 0.),
                    ..default()
                },
            ));

            parent.spawn((
                EnemyDamageActivator(-1),
                Sensor,
                Collider::capsule_y(12., 12.),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(Group::GROUP_5, Group::GROUP_5),
                TransformBundle::default(),
            ));
        });
    }
}

pub fn ai(
    mut skeletons: Query<(&mut Velocity, &mut Skeleton, Option<&SpeedEffect>)>,
    time: Res<Time>,
) {
    for (mut velocity, mut skeleton, speed_effect) in skeletons.iter_mut() {
        if skeleton.going_right && skeleton.right_sensor > 0 && skeleton.left_sensor < 1 {
            skeleton.going_right = false;
        } else if !skeleton.going_right && skeleton.right_sensor < 1 && skeleton.left_sensor > 0 {
            skeleton.going_right = true;
        }

        let mut speed = 1000f32;

        if let Some(multiplier) = speed_effect {
            speed *= multiplier.multiplier;
        }

        if skeleton.going_right {
            velocity.linvel.x += speed * time.delta_seconds();
        } else {
            velocity.linvel.x -= speed * time.delta_seconds();
        }
    }
}

pub fn checks(
    mut collision_events: EventReader<CollisionEvent>,
    mut skeletons: Query<&mut Skeleton>,
    left_sensors: Query<&Parent, With<SkeletonSensorLeft>>,
    right_sensors: Query<&Parent, With<SkeletonSensorRight>>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(a, b, flags) => {
                if *flags & CollisionEventFlags::SENSOR != CollisionEventFlags::SENSOR {
                    continue;
                };
                if let Ok(parent) = left_sensors.get(*a) {
                    let Ok(mut skeleton) = skeletons.get_mut(**parent) else { continue };
                    skeleton.left_sensor += 1;
                } else if let Ok(parent) = left_sensors.get(*b) {
                    let Ok(mut skeleton) = skeletons.get_mut(**parent) else { continue };
                    skeleton.left_sensor += 1;
                }

                if let Ok(parent) = right_sensors.get(*a) {
                    let Ok(mut skeleton) = skeletons.get_mut(**parent) else { continue };
                    skeleton.right_sensor += 1;
                } else if let Ok(parent) = right_sensors.get(*b) {
                    let Ok(mut skeleton) = skeletons.get_mut(**parent) else { continue };
                    skeleton.right_sensor += 1;
                }
            }
            CollisionEvent::Stopped(a, b, flags) => {
                if *flags & CollisionEventFlags::SENSOR != CollisionEventFlags::SENSOR {
                    continue;
                };

                if let Ok(parent) = left_sensors.get(*a) {
                    let Ok(mut skeleton) = skeletons.get_mut(**parent) else { continue };
                    skeleton.left_sensor -= 1;
                } else if let Ok(parent) = left_sensors.get(*b) {
                    let Ok(mut skeleton) = skeletons.get_mut(**parent) else { continue };
                    skeleton.left_sensor -= 1;
                }

                if let Ok(parent) = right_sensors.get(*a) {
                    let Ok(mut skeleton) = skeletons.get_mut(**parent) else { continue };
                    skeleton.right_sensor -= 1;
                } else if let Ok(parent) = right_sensors.get(*b) {
                    let Ok(mut skeleton) = skeletons.get_mut(**parent) else { continue };
                    skeleton.right_sensor -= 1;
                }
            }
        }
    }
}

pub fn health_effects(
    mut commands: Commands,
    mut skeletons: Query<(Entity, &mut Skeleton, &HealthEffect)>,
    mut damage_given: ResMut<DamageGiven>,
) {
    for (entity, mut skeleton, effect) in skeletons.iter_mut() {
        skeleton.hp += effect.amount;
        commands.entity(entity).remove::<HealthEffect>();
        damage_given.0 = true;
    }
}

pub fn health(mut commands: Commands, skeletons: Query<(Entity, &Skeleton)>) {
    for (entity, skeleton) in skeletons.iter() {
        if skeleton.hp < 1 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
