use super::*;

use crate::animator::*;

use bevy_rapier2d::rapier::prelude::CollisionEventFlags;

#[derive(Component)]
pub struct PurplePotion;

impl Ability for PurplePotion {
    fn splash_image(
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Handle<TextureAtlas> {
        let texture = asset_server.load("images/abilities/purple_splash.png");
        let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(32., 32.), 3, 3, None, None);
        texture_atlases.add(texture_atlas)
    }

    fn ui_image(asset_server: &AssetServer) -> Handle<Image> {
        asset_server.load("images/abilities/purple.png")
    }

    fn ui_position() -> f32 {
        -80.
    }

    fn activate(
        mut commands: Commands,
        position: Vec3,
        velocity: Velocity,
        right: bool,
        asset_server: &AssetServer,
    ) {
        let new_velocity =
            Vec2::new(if right { 400. } else { -400. }, 200.) + velocity.linvel * 0.5;

        commands.spawn((
            PotionBundle::default(),
            PurplePotion,
            SpriteBundle {
                texture: asset_server.load("images/abilities/purple_small.png"),
                transform: Transform::from_translation(position),
                ..default()
            },
            Velocity {
                linvel: new_velocity,
                angvel: 10.0,
            },
        ));
    }
}

pub fn checks(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    potions: Query<(Entity, &Transform), With<PurplePotion>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    for collision_event in collision_events.iter() {
        let CollisionEvent::Started(a, b, flags) = collision_event else { continue };

        if *flags & CollisionEventFlags::SENSOR == CollisionEventFlags::SENSOR {
            continue;
        }

        let (entity, transform, other) = if let Ok((entity, transform)) = potions.get(*a) {
            (entity, transform, *b)
        } else if let Ok((entity, transform)) = potions.get(*b) {
            (entity, transform, *a)
        } else {
            continue;
        };

        commands
            .entity(other)
            .insert(HealthEffect { amount: -2 })
            .insert(DamageEffect { multiplier: 3.0 })
            .insert(DamageFlash::default());
        commands.entity(entity).despawn();
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: PurplePotion::splash_image(&asset_server, &mut texture_atlases),
                transform: *transform,
                ..default()
            },
            AnimationIndices { first: 0, last: 6 },
            AnimationTimer(Timer::from_seconds(1. / 12., TimerMode::Repeating)),
            Destruct,
        ));
    }
}
