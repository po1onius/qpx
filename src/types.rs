use bevy::math::prelude::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::path::Path;

pub const FLOOR_H: f32 = 20.0;
pub const JUMP_SPEED: f32 = 600.0;
pub const ROLE_SPEED: f32 = 300.0;
pub const GRAVITY: f32 = 1300.0;
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const BALL_SIZE: f32 = 30.0;
pub const LV_DATA_PATH: &str = "level_data";
pub const WINDOW_RESOLUTION_X: f32 = 1280.0;
pub const WINDOW_RESOLUTION_Y: f32 = 720.0;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Main,
    InitLevel,
    Playing,
    Paused,
}

#[derive(Component)]
pub struct RoleSpeed(pub f32, pub f32);

#[derive(Component)]
pub enum RoleState {
    Air(u32),
    Normal,
}

#[derive(Bundle)]
pub struct MapItemBundle {
    rigid: RigidBody,
    collider: Collider,
    position: Transform,
    map_item: MapItem,
}

#[derive(Component)]
pub enum MapItem {
    Obstacle,
    Normal,
    DoubleJump,
    FlyBegin,
    FlyEnd,
}

#[derive(Component)]
pub struct StartGameButton;

#[derive(Component)]
pub struct LeftSelectButton;

#[derive(Component)]
pub struct RightSelectButton;

#[derive(Component)]
pub struct ReturnMainMenuButton;

#[derive(Component)]
pub struct MainUIEntity;

#[derive(Component)]
pub struct PauseUIEntity;

#[derive(Component)]
pub struct CurLvLabel;

#[derive(Deserialize)]
pub struct LevelDataOrigin {
    data: Vec<(u32, Vec<f32>)>,
}

pub enum MapItemData {
    Floor(Vec4),
    TriObstacle(Triangle2d),
    RectObstacle(Vec4),
    DoubleJumpCircle(Vec2, f32),
    RectFlyBegin(Vec4),
    RectFlyEnd(Vec4),
}

#[derive(Resource, Default)]
pub struct LevelData {
    pub data: Vec<MapItemData>,
}

#[derive(Resource, Default)]
pub struct IdxEntityPair {
    pub pairs: HashMap<u32, (Entity, Option<Entity>)>,
}

#[derive(Resource)]
pub struct CurLevel {
    pub lvs: Vec<String>,
    pub cur_idx: usize,
}

impl Default for CurLevel {
    fn default() -> Self {
        let dirs = read_dir(LV_DATA_PATH).unwrap();
        let lvs = dirs
            .map(|e| e.unwrap().path().to_str().to_owned().unwrap().to_string())
            .collect::<Vec<String>>();
        Self { lvs, cur_idx: 0 }
    }
}

impl LevelData {
    pub fn from_file(path: impl AsRef<Path>) -> Self {
        let file_data = read_to_string(path).unwrap();
        let level_data_origin: LevelDataOrigin = toml::from_str(&file_data).unwrap();
        let mut data = Vec::new();
        for (typ, v) in level_data_origin.data {
            if v.len() == 4 {
                let rect = Vec4::new(v[0], v[1], v[2], v[3]);
                if typ == 0 {
                    data.push(MapItemData::Floor(rect));
                } else if typ == 2 {
                    data.push(MapItemData::RectObstacle(rect));
                } else if typ == 4 {
                    data.push(MapItemData::RectFlyBegin(rect));
                } else {
                    data.push(MapItemData::RectFlyEnd(rect));
                }
            } else if v.len() == 6 {
                data.push(MapItemData::TriObstacle(Triangle2d::new(
                    Vec2::new(v[0], v[1]),
                    Vec2::new(v[2], v[3]),
                    Vec2::new(v[4], v[5]),
                )));
            } else if v.len() == 3 {
                data.push(MapItemData::DoubleJumpCircle(Vec2::new(v[0], v[1]), v[2]));
            } else {
                panic!();
            }
        }
        Self { data }
    }
}

impl MapItemBundle {
    pub fn rect_item(rect: &Vec4, obstacle: bool) -> Self {
        Self {
            rigid: RigidBody::Fixed,
            collider: Collider::cuboid(rect.z, rect.w),
            position: Transform::from_xyz(rect.x, rect.y, 0.0),
            map_item: if obstacle {
                MapItem::Obstacle
            } else {
                MapItem::Normal
            },
        }
    }

    pub fn rect_fly(rect: &Vec4, begin: bool) -> Self {
        Self {
            rigid: RigidBody::Fixed,
            collider: Collider::cuboid(rect.z, rect.w),
            position: Transform::from_xyz(rect.x, rect.y, 0.0),
            map_item: if begin {
                MapItem::FlyBegin
            } else {
                MapItem::FlyEnd
            },
        }
    }

    pub fn tri_obstacle(tri: &Triangle2d) -> Self {
        info!("spawn tri: {} {}", tri.vertices[0].x, tri.vertices[0].y);
        Self {
            rigid: RigidBody::Fixed,
            //collider: Collider::triangle(tri.vertices[0], tri.vertices[1], tri.vertices[2]),
            collider: Collider::triangle(
                Vec2 { x: 0.0, y: 0.0 },
                tri.vertices[1] - tri.vertices[0],
                tri.vertices[2] - tri.vertices[0],
            ),
            position: Transform::from_xyz(tri.vertices[0].x, tri.vertices[0].y, 0.0),
            map_item: MapItem::Obstacle,
        }
    }

    pub fn circle_double_jump(pos: &Vec2, radius: f32) -> Self {
        info!("spawn circle");
        Self {
            rigid: RigidBody::Fixed,
            collider: Collider::ball(radius),
            position: Transform::from_xyz(pos.x, pos.y, 0.0),
            map_item: MapItem::DoubleJump,
        }
    }
}
