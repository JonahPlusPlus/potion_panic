use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};

use crate::{GameState, animator::{AnimationIndices, AnimationTimer}};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb_u8(10, 12, 12)))
            .insert_resource(LdtkSettings {
                level_background: LevelBackground::Nonexistent,
                ..default()
            })
            .add_plugin(LdtkPlugin)
            .insert_resource(RapierConfiguration {
                gravity: Vec2::ZERO,
                ..default()
            })
            .add_plugin(RapierPhysicsPlugin::<GamePhysicsHooks>::pixels_per_meter(
                32.0,
            ))
            .configure_set(LdtkSystemSet::ProcessApi.before(PhysicsSet::SyncBackend))
            .insert_resource(LevelSelection::Index(0))
            .register_ldtk_int_cell::<WallBundle>(1)
            .register_ldtk_entity::<GoldHeartBundle>("GoldHeart")
            .add_system(setup_world)
            .add_system(spawn_wall_collision)
            .add_system(heart_checks)
            .add_system(despawn_world);

        let asset_server = app.world.resource::<AssetServer>();

        let font =
            asset_server.load("fonts/NotoSerifSinhala/NotoSerifSinhala-VariableFont_wdth,wght.ttf");

        let cursive_font =
            asset_server.load("fonts/GreatVibes/GreatVibes-Regular.ttf");

        app.insert_resource(StandardFont(font));

        app.insert_resource(CursiveFont(cursive_font));
    }
}

#[derive(Resource)]
pub struct StandardFont(pub Handle<Font>);

#[derive(Resource)]
pub struct CursiveFont(pub Handle<Font>);

#[derive(Component)]
pub struct World;

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>, game_state: Res<GameState>) {
    if game_state.is_changed() && *game_state == GameState::Gameplay {
        commands
            .spawn(LdtkWorldBundle {
                ldtk_handle: asset_server.load("map.ldtk"),
                ..Default::default()
            })
            .insert(World);
    }
}

fn despawn_world(
    mut commands: Commands,
    world: Query<Entity, With<World>>,
    game_state: Res<GameState>,
) {
    if game_state.is_changed() && *game_state != GameState::Gameplay {
        let Ok(world) = world.get_single() else { return };
        commands.entity(world).despawn_recursive();
    }
}

#[derive(Component)]
pub struct WorldCollider;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}

pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a wall of any size
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations
                .entry(grandparent.get())
                .or_default()
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level
                    .level
                    .layer_instances
                    .clone()
                    .expect("Level asset should have layers")[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1)
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        level
                            .spawn(WorldCollider)
                            .insert(Collider::cuboid(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                            ))
                            .insert(CollisionGroups::new(
                                Group::GROUP_1,
                                Group::all() & !Group::GROUP_1,
                            ))
                            .insert(RigidBody::Fixed)
                            .insert(Friction::new(0.5))
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}

#[derive(Component)]
pub struct GoldHeart;

#[derive(Bundle)]
pub struct GoldHeartBundle {
    pub gold_heart: GoldHeart,
    pub sensor: Sensor,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub active_events: ActiveEvents,
    pub animation_indices: AnimationIndices,
    pub animation_timer: AnimationTimer,
    pub sprite: TextureAtlasSprite,
    pub texture_atlas: Handle<TextureAtlas>,
}

impl LdtkEntity for GoldHeartBundle {
    fn bundle_entity(
            _: &EntityInstance,
            _: &LayerInstance,
            _: Option<&Handle<Image>>,
            _: Option<&TilesetDefinition>,
            asset_server: &AssetServer,
            texture_atlases: &mut Assets<TextureAtlas>,
        ) -> Self {
            let texture = asset_server.load("images/heart/gold.png");
            let texture_atlas = TextureAtlas::from_grid(texture, Vec2::new(64., 64.), 2, 2, None, None);
            let texture_atlas = texture_atlases.add(texture_atlas);

            Self {
                gold_heart: GoldHeart,
                sensor: Sensor,
                collider: Collider::ball(16.0),
                collision_groups: CollisionGroups { memberships: Group::GROUP_6, filters: Group::GROUP_2 },
                active_events: ActiveEvents::COLLISION_EVENTS,
                animation_indices: AnimationIndices { first: 0, last: 3 },
                animation_timer: AnimationTimer(Timer::from_seconds(1.0 / 4.0, TimerMode::Repeating)),
                sprite: TextureAtlasSprite::default(),
                texture_atlas,
            }
    }
}

fn heart_checks(
    mut collision_events: EventReader<CollisionEvent>,
    heart: Query<Entity, With<GoldHeart>>,
    mut game_state: ResMut<GameState>,
) {
    let Ok(heart) = heart.get_single() else { return };
    for collision_event in collision_events.iter() {
        if let CollisionEvent::Started(a, b, flags) = collision_event {
            if *flags & CollisionEventFlags::SENSOR != CollisionEventFlags::SENSOR { continue };

            if *a == heart || *b == heart {
                *game_state = GameState::WinScreen;
            }
        }
    }
}

#[derive(SystemParam)]
struct GamePhysicsHooks<'w, 's> {
    world_colliders: Query<'w, 's, &'static WorldCollider>,
}

impl BevyPhysicsHooks for GamePhysicsHooks<'_, '_> {
    fn modify_solver_contacts(&self, context: ContactModificationContextView) {
        if !self.world_colliders.contains(context.collider1())
            && !self.world_colliders.contains(context.collider2())
        {
            return;
        }
        let friction_scale = Vec2::new(context.raw.normal.x, context.raw.normal.y)
            .dot(Vec2::Y)
            .abs();

        for solver_contact in &mut *context.raw.solver_contacts {
            solver_contact.friction *= friction_scale;
        }
    }
}
