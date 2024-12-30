use bevy::input::common_conditions::input_just_pressed;
use bevy::math::prelude::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use serde::Deserialize;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> AppExit {
    App::new()
        .insert_resource(LevelData::from_file("level_data/1.toml"))
        .insert_resource(LevelIndex(0))
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                jump.run_if(input_just_pressed(KeyCode::Space)),
                role_move,
                loop_block,
                display_events,
            ),
        )
        .run()
}

#[derive(Component)]
struct RoleSpeed(f32, f32);

#[derive(Component)]
enum RoleState {
    Jump,
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

#[derive(Resource)]
struct LevelIndex(u32);

const FLOOR_H: f32 = 5.0;
const JUMP_SPEED: f32 = 600.0;

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
    fn rect_item(rect: Vec4, obstacle: bool) -> Self {
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

    fn tri_obstacle(tri: Triangle2d) -> Self {
        Self {
            rigid: RigidBody::Fixed,
            collider: Collider::triangle(tri.vertices[0], tri.vertices[1], tri.vertices[2]),
            position: Transform::from_xyz(tri.vertices[0].x, tri.vertices[1].y, 0.0),
            map_item: MapItem::Obstacle,
        }
    }
}

fn spawn_floor(cmd: &mut Commands, rect: Vec4) {
    let hw = 10.0;
    let hy = rect.y + rect.w - hw;
    let floor_high = MapItemBundle::rect_item(Vec4::new(rect.x, hy, rect.z, hw), false);
    let floor_low = MapItemBundle::rect_item(Vec4::new(rect.x, rect.y, rect.z, rect.w - hw), true);
    cmd.spawn(floor_high);
    cmd.spawn(floor_low);
}

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>, level_data: Res<LevelData>) {
    cmd.spawn(Camera2d::default());
    //let block_texture = asset_server.load("block.png");

    match level_data.data[0] {
        MapItemData::Rect(rect) => {
            spawn_floor(&mut cmd, rect);
        }
        _ => (),
    }

    cmd.spawn((
        RigidBody::Dynamic,
        Collider::ball(50.0),
        Restitution::coefficient(0.0),
        Friction::coefficient(0.0),
        ActiveEvents::COLLISION_EVENTS,
        Sprite::from_image(asset_server.load("block.png")),
        RoleState::Normal,
        RoleSpeed(400.0, 0.0),
        Transform::from_xyz(-100.0, 200.0, 0.0),
    ));
}

/* A system that displays the events. */
fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    role_sv: Single<(&mut RoleSpeed, &mut RoleState)>,
) {
    let (mut role_speed, mut role_state) = role_sv.into_inner();
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(..) = collision_event {
            *role_state = RoleState::Normal;
            role_speed.1 = 0.0;
        }
    }
}

fn jump(role_sv: Single<(&mut RoleSpeed, &mut RoleState)>) {
    let (mut role_speed, mut role_state) = role_sv.into_inner();
    if let RoleState::Normal = *role_state {
        *role_state = RoleState::Jump;
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
    map_items: Query<(Entity, &mut Transform), With<MapItem>>,
) {
    for (item_entity, item_transform) in map_items.iter() {
        let far = camera_transform.translation.x - item_transform.translation.x;
        if far > 500.0 {
            //cmd.entity(item_entity).despawn();
            //cmd.spawn(BlockBundle::default());
        }
    }
}
