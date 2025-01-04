use bevy::input::common_conditions::input_just_pressed;
use bevy::math::prelude::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> AppExit {
    App::new()
        .insert_resource(LevelData::from_file("level_data/1.toml"))
        .insert_resource(IdxEntityPair::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                gravity,
                jump.run_if(input_just_pressed(KeyCode::Space)),
                game_pause.run_if(input_just_pressed(KeyCode::Escape)),
                role_move,
                loop_block,
                collide_events,
            ),
        )
        .run()
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Playing,
    Paused,
}

#[derive(Component)]
struct RoleSpeed(f32, f32);

#[derive(Component)]
enum RoleState {
    Air,
    Normal,
}

#[derive(Bundle)]
struct MapItemBundle {
    rigid: RigidBody,
    collider: Collider,
    position: Transform,
    map_item: MapItem,
}

#[derive(Component)]
enum MapItem {
    Obstacle,
    Normal,
}

#[derive(Deserialize)]
struct LevelDataOrigin {
    data: Vec<Vec<f32>>,
}

enum MapItemData {
    Tri(Triangle2d),
    Rect(Vec4),
}

#[derive(Resource)]
struct LevelData {
    data: Vec<MapItemData>,
}

#[derive(Resource, Default)]
struct IdxEntityPair {
    pairs: HashMap<u32, (u32, Option<u32>)>,
}

const FLOOR_H: f32 = 5.0;
const JUMP_SPEED: f32 = 600.0;
const GRAVITY: f32 = 1300.0;

impl LevelData {
    fn from_file(path: impl AsRef<Path>) -> Self {
        let file_data = read_to_string(path).unwrap();
        let level_data_origin: LevelDataOrigin = toml::from_str(&file_data).unwrap();
        let mut data = Vec::new();
        for v in level_data_origin.data {
            if v.len() == 4 {
                data.push(MapItemData::Rect(Vec4::new(v[0], v[1], v[2], v[3])));
            } else if v.len() == 6 {
                data.push(MapItemData::Tri(Triangle2d::new(
                    Vec2::new(v[0], v[1]),
                    Vec2::new(v[2], v[3]),
                    Vec2::new(v[4], v[5]),
                )));
            } else {
                panic!();
            }
        }
        Self { data }
    }
}

impl MapItemBundle {
    fn rect_item(rect: &Vec4, obstacle: bool) -> Self {
        Self {
            rigid: RigidBody::Fixed,
            collider: Collider::cuboid(rect.z, rect.w - FLOOR_H),
            position: Transform::from_xyz(rect.x, rect.y, 0.0),
            map_item: if obstacle {
                MapItem::Obstacle
            } else {
                MapItem::Normal
            },
        }
    }

    fn tri_obstacle(tri: &Triangle2d) -> Self {
        Self {
            rigid: RigidBody::Fixed,
            collider: Collider::triangle(tri.vertices[0], tri.vertices[1], tri.vertices[2]),
            position: Transform::from_xyz(tri.vertices[0].x, tri.vertices[1].y, 0.0),
            map_item: MapItem::Obstacle,
        }
    }
}

fn spawn_floor(
    cmd: &mut Commands,
    rect: &Vec4,
    index: u32,
    lv_idx_entity_paires: &mut ResMut<IdxEntityPair>,
) {
    let hw = 10.0;
    let hy = rect.y + rect.w - hw;
    let floor_high = MapItemBundle::rect_item(&Vec4::new(rect.x, hy, rect.z, hw), false);
    let floor_low = MapItemBundle::rect_item(&Vec4::new(rect.x, rect.y, rect.z, rect.w - hw), true);
    let id1 = cmd.spawn(floor_high).id().index();
    let id2 = cmd.spawn(floor_low).id().index();

    lv_idx_entity_paires.pairs.insert(index, (id1, Some(id2)));
    info!("spawn: entity {} {}", id1, id2);
}

fn spawn_tri_obstacle(
    cmd: &mut Commands,
    tri: &Triangle2d,
    index: u32,
    lv_idx_entity_paires: &mut ResMut<IdxEntityPair>,
) {
    let id = cmd.spawn(MapItemBundle::tri_obstacle(tri)).id().index();
    lv_idx_entity_paires.pairs.insert(index, (id, None));
    info!("sapwn: entity {}", id);
}

fn setup(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    level_data: Res<LevelData>,
    mut lv_idx_entity_paires: ResMut<IdxEntityPair>,
) {
    cmd.spawn(Camera2d::default());
    //let block_texture = asset_server.load("block.png");

    for (i, l) in level_data.data.iter().enumerate() {
        if i > 1 {
            break;
        }
        match l {
            MapItemData::Rect(rect) => {
                spawn_floor(&mut cmd, rect, i as u32, &mut lv_idx_entity_paires);
            }
            MapItemData::Tri(tri) => {
                spawn_tri_obstacle(&mut cmd, tri, i as u32, &mut lv_idx_entity_paires);
            }
        }
    }

    cmd.spawn((
        RigidBody::Dynamic,
        Collider::ball(50.0),
        GravityScale(0.0),
        Restitution::coefficient(0.0),
        Friction::coefficient(0.0),
        ActiveEvents::COLLISION_EVENTS,
        Sprite::from_image(asset_server.load("block.png")),
        RoleState::Air,
        RoleSpeed(400.0, 0.0),
        Transform::from_xyz(-100.0, 200.0, 0.0),
    ));
}

fn gravity(role_sv: Single<(&mut RoleSpeed, &RoleState)>, time: Res<Time>) {
    let (mut role_speed, role_state) = role_sv.into_inner();
    if let RoleState::Air = *role_state {
        role_speed.1 -= GRAVITY * time.delta_secs();
    }
}

/* A system that displays the events. */
fn collide_events(
    mut collision_events: EventReader<CollisionEvent>,
    role_sv: Single<(&mut RoleSpeed, &mut RoleState)>,
    role_entity: Single<Entity, With<RoleState>>,
    floor_entities: Query<(Entity, &MapItem)>,
) {
    let (mut role_speed, mut role_state) = role_sv.into_inner();
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            if *entity2 == *role_entity {
                for (entity, map_item) in floor_entities.iter() {
                    if let MapItem::Obstacle = map_item {
                        if entity == *entity1 {
                            info!("boom!");
                            break;
                        }
                    }
                }
            }
            info!("collide floor");
            *role_state = RoleState::Normal;
            role_speed.1 = 0.0;
        }
        if let CollisionEvent::Stopped(..) = collision_event {
            *role_state = RoleState::Air;
        }
    }
}

fn jump(role_sv: Single<(&mut RoleSpeed, &mut RoleState)>) {
    let (mut role_speed, mut role_state) = role_sv.into_inner();
    if let RoleState::Normal = *role_state {
        *role_state = RoleState::Air;
        role_speed.1 += JUMP_SPEED;
    }
}

fn role_move(
    role: Single<(&mut Transform, &RoleSpeed)>,
    time: Res<Time>,
    mut camera_transform: Single<
        &mut Transform,
        (With<Camera>, Without<RoleSpeed>, Without<MapItem>),
    >,
) {
    let (mut role_transform, speed) = role.into_inner();
    role_transform.translation.x += speed.0 * time.delta_secs();
    role_transform.translation.y += speed.1 * time.delta_secs();
    camera_transform.translation.x += speed.0 * time.delta_secs();
}

fn loop_block(
    mut cmd: Commands,
    camera_transform: Single<&Transform, (With<Camera>, Without<RoleSpeed>, Without<MapItem>)>,
    level_data: Res<LevelData>,
    mut lv_idx_entity_paires: ResMut<IdxEntityPair>,
    map_item_entities: Query<(Entity, &MapItem)>,
) {
    fn despawn_by_lv_idx(
        cmd: &mut Commands,
        lv_idx_entity_paires: &mut ResMut<IdxEntityPair>,
        idx: u32,
        map_item_entities: &Query<(Entity, &MapItem)>,
    ) {
        if let Some((entity_id1, entity_op_id2)) = lv_idx_entity_paires.pairs.get(&idx) {
            map_item_entities
                .iter()
                .find(|(entity, _)| entity.index() == *entity_id1)
                .map(|(entity, _)| {
                    info!("despawn: {}", entity_id1);
                    cmd.entity(entity).despawn();
                });

            if let Some(entity_id2) = entity_op_id2 {
                map_item_entities
                    .iter()
                    .find(|(entity, _)| entity.index() == *entity_id2)
                    .map(|(entity, _)| {
                        info!("despawn: {}", entity_id2);
                        cmd.entity(entity).despawn();
                    });
            }
        }
        lv_idx_entity_paires.pairs.remove(&idx);
    }
    for (i, item_data) in level_data.data.iter().enumerate() {
        match item_data {
            MapItemData::Rect(rect) => {
                if camera_transform.translation.x - (rect.x + rect.z) > 500.0 {
                    despawn_by_lv_idx(
                        &mut cmd,
                        &mut lv_idx_entity_paires,
                        i as u32,
                        &map_item_entities,
                    );
                }
                let ng = rect.x - camera_transform.translation.x;
                if ng < 500.0 && ng > 300.0 && !lv_idx_entity_paires.pairs.contains_key(&(i as u32))
                {
                    spawn_floor(&mut cmd, rect, i as u32, &mut lv_idx_entity_paires);
                }
            }
            MapItemData::Tri(tri) => {
                if camera_transform.translation.x - tri.vertices[2][0] > 500.0 {
                    despawn_by_lv_idx(
                        &mut cmd,
                        &mut lv_idx_entity_paires,
                        i as u32,
                        &map_item_entities,
                    );
                }
                let ng = tri.vertices[0][0] - camera_transform.translation.x;
                if ng < 500.0 && ng > 300.0 && !lv_idx_entity_paires.pairs.contains_key(&(i as u32))
                {
                    spawn_tri_obstacle(&mut cmd, tri, i as u32, &mut lv_idx_entity_paires);
                }
            }
        }
    }
}

fn game_pause(role_sc: Single<(&mut RoleSpeed, &mut RigidBody)>) {
    info!("game pause");
    let (mut role_speed, mut rigid_body) = role_sc.into_inner();
    role_speed.0 = 0.0;
    role_speed.1 = 0.0;
}
