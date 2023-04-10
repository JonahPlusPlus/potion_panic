use bevy::prelude::*;

use crate::{GameState, player::abilities::Cooldown};

pub struct AnimatorPlugin;

impl Plugin for AnimatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_sprite)
            .add_system(damage_flash)
            .add_system(ability_cooldown);
    }
}

#[derive(Component, Default)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub struct Destruct;

fn animate_sprite(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<GameState>,
    mut query: Query<(
        Entity,
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        Option<&Destruct>,
    )>,
) {
    if *state == GameState::Gameplay {
        for (entity, indices, mut timer, mut sprite, destruct) in &mut query {
            timer.tick(time.delta());
            if timer.just_finished() {
                sprite.index = if sprite.index == indices.last {
                    if destruct.is_some() {
                        commands.entity(entity).despawn();
                    }
                    indices.first
                } else {
                    sprite.index + 1
                };
            }
        }
    }
}

#[derive(Component)]
pub struct DamageFlash(Timer);

impl Default for DamageFlash {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Once))
    }
}

fn damage_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DamageFlash, &mut TextureAtlasSprite)>,
) {
    for (entity, mut flash, mut sprite) in query.iter_mut() {
        flash.0.tick(time.delta());
        if flash.0.finished() {
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<DamageFlash>();
        } else {
            sprite.color = Color::RED;
        }
    }
}

fn ability_cooldown(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Cooldown, &mut TextureAtlasSprite)>,
) {
    for (entity, mut cooldown, mut sprite) in query.iter_mut() {
        cooldown.0.tick(time.delta());

        let frame = (((cooldown.0.elapsed_secs() / cooldown.0.duration().as_secs_f32()) * 17.0) as usize).min(16);

        sprite.index = frame;

        if cooldown.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}
