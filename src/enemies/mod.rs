use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::LdtkEntityAppExt;
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};

mod skeleton;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<skeleton::SkeletonBundle>("Skeleton")
            .insert_resource(DamageGiven(false))
            .add_system(enemy_physics_checks)
            .add_system(enemy_gravity)
            .add_system(enemy_direction);

        app.add_systems((
            skeleton::on_skeleton_spawn,
            skeleton::checks,
            skeleton::ai,
            skeleton::health_effects,
            skeleton::health,
        ));
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub physics: EnemyPhysics,
    pub rigidbody: RigidBody,
    pub velocity: Velocity,
    pub damping: Damping,
    pub collision_groups: CollisionGroups,
    pub locked_axes: LockedAxes,
    pub sprite: TextureAtlasSprite,
}

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            enemy: Enemy,
            physics: EnemyPhysics::default(),
            rigidbody: RigidBody::Dynamic,
            velocity: Velocity::zero(),
            damping: Damping {
                linear_damping: 8.,
                angular_damping: 0.,
            },
            collision_groups: CollisionGroups::new(
                Group::GROUP_4,
                Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_3 | Group::GROUP_4 | Group::GROUP_5,
            ),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            sprite: TextureAtlasSprite::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct EnemyPhysics {
    pub total_ground_collisions: i32,
    pub grounded: bool,
}

#[derive(Component)]
pub struct EnemyGroundSensor;

fn enemy_physics_checks(
    mut collision_events: EventReader<CollisionEvent>,
    mut data: Query<&mut EnemyPhysics>,
    sensors: Query<&Parent, With<EnemyGroundSensor>>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(a, b, flags) => {
                if *flags & CollisionEventFlags::SENSOR != CollisionEventFlags::SENSOR {
                    continue;
                };

                let parent = if let Ok(parent) = sensors.get(*a) {
                    parent
                } else if let Ok(parent) = sensors.get(*b) {
                    parent
                } else {
                    continue;
                };

                let Ok(mut physics) = data.get_mut(**parent) else { continue };

                physics.total_ground_collisions += 1;
                if physics.total_ground_collisions > 0 {
                    physics.grounded = true;
                }
            }
            CollisionEvent::Stopped(a, b, flags) => {
                if *flags & CollisionEventFlags::SENSOR != CollisionEventFlags::SENSOR {
                    continue;
                };

                let parent = if let Ok(parent) = sensors.get(*a) {
                    parent
                } else if let Ok(parent) = sensors.get(*b) {
                    parent
                } else {
                    continue;
                };

                let Ok(mut physics) = data.get_mut(**parent) else { continue };

                physics.total_ground_collisions -= 1;
                if physics.total_ground_collisions < 1 {
                    physics.grounded = false;
                }
            }
        }
    }
}

const ENEMY_GRAVITY: f32 = 9.81 * 275f32;

fn enemy_gravity(mut enemies: Query<(&mut Velocity, &EnemyPhysics)>, time: Res<Time>) {
    for (mut velocity, physics) in enemies.iter_mut() {
        if !physics.grounded {
            velocity.linvel.y -= ENEMY_GRAVITY * time.delta_seconds();
        }
    }
}

fn enemy_direction(mut enemies: Query<(&mut TextureAtlasSprite, &Velocity), With<Enemy>>) {
    for (mut sprite, velocity) in enemies.iter_mut() {
        if velocity.linvel.x > 0.1 {
            sprite.flip_x = false;
        } else if velocity.linvel.x < -0.1 {
            sprite.flip_x = true;
        }
    }
}

#[derive(Component)]
pub struct EnemyDamageActivator(pub i32);

#[derive(Resource)]
pub struct DamageGiven(pub bool);
