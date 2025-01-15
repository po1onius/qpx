use bevy::input::common_conditions::input_just_pressed;
use bevy::math::prelude::*;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier2d::prelude::*;

use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::path::Path;

fn main() -> AppExit {
    App::new()
        .insert_resource(CurLevel::default())
        .insert_resource(LevelData::default())
        .insert_resource(IdxEntityPair::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(1280.0, 720.0).with_scale_factor_override(1.0),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::InitLevel), game_init)
        .add_systems(
            Update,
            (
                gravity,
                jump.run_if(input_just_pressed(KeyCode::Space)),
                role_move,
                loop_block,
                collide_events,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                game_pause_play.run_if(input_just_pressed(KeyCode::Escape)),
                start_playing.run_if(input_just_pressed(KeyCode::Enter)),
                start_button_action.run_if(in_state(GameState::Main)),
                select_lv_left_button_action.run_if(in_state(GameState::Main)),
                select_lv_right_button_action.run_if(in_state(GameState::Main)),
            ),
        )
        .run()
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Main,
    InitLevel,
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

#[derive(Component)]
struct StartGameButton;

#[derive(Component)]
struct LeftSelectButton;

#[derive(Component)]
struct RightSelectButton;

#[derive(Component)]
struct ContinueButton;

#[derive(Component)]
struct ReturnMainMenuButton;

#[derive(Component)]
struct MainUIEntity;

#[derive(Component)]
struct CurLvLabel;

#[derive(Deserialize)]
struct LevelDataOrigin {
    data: Vec<Vec<f32>>,
}

enum MapItemData {
    Tri(Triangle2d),
    Rect(Vec4),
}

#[derive(Resource, Default)]
struct LevelData {
    data: Vec<MapItemData>,
}

#[derive(Resource, Default)]
struct IdxEntityPair {
    pairs: HashMap<u32, (Entity, Option<Entity>)>,
}

#[derive(Resource)]
struct CurLevel {
    lvs: Vec<String>,
    cur_idx: usize,
}

const FLOOR_H: f32 = 20.0;
const JUMP_SPEED: f32 = 600.0;
const GRAVITY: f32 = 1300.0;
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const BALL_SIZE: f32 = 30.0;
const LV_DATA_PATH: &str = "level_data";

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
            collider: Collider::cuboid(rect.z, rect.w),
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
    let hw = FLOOR_H;
    let hy = rect.y + rect.w - hw / 2.0;
    let floor_high = MapItemBundle::rect_item(&Vec4::new(rect.x, hy, rect.z, hw / 2.0), false);
    let floor_low = MapItemBundle::rect_item(
        &Vec4::new(rect.x, rect.y - hw / 2.0, rect.z, rect.w - hw / 2.0),
        true,
    );
    let id1 = cmd.spawn(floor_high).id();
    let id2 = cmd.spawn(floor_low).id();

    lv_idx_entity_paires.pairs.insert(index, (id1, Some(id2)));
    info!("spawn: entity {} {}", id1, id2);
}

fn spawn_tri_obstacle(
    cmd: &mut Commands,
    tri: &Triangle2d,
    index: u32,
    lv_idx_entity_paires: &mut ResMut<IdxEntityPair>,
) {
    let id = cmd.spawn(MapItemBundle::tri_obstacle(tri)).id();
    lv_idx_entity_paires.pairs.insert(index, (id, None));
    info!("sapwn: entity {}", id);
}

fn game_init(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    level_data: Res<LevelData>,
    mut lv_idx_entity_paires: ResMut<IdxEntityPair>,
    mut camera_transform: Single<
        &mut Transform,
        (With<Camera>, Without<RoleSpeed>, Without<MapItem>),
    >,
) {
    info!("game init");
    //let block_texture = asset_server.load("block.png");
    camera_transform.translation.x = 0.0;
    camera_transform.translation.y = 0.0;

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
        Ccd::enabled(),
        Collider::ball(BALL_SIZE),
        GravityScale(0.0),
        Restitution::coefficient(0.0),
        Friction::coefficient(0.0),
        ActiveEvents::COLLISION_EVENTS,
        //Sprite::from_image(asset_server.load("block.png")),
        RoleState::Air,
        RoleSpeed(300.0, 0.0),
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
    mut cmd: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    role_sv: Single<(&mut RoleSpeed, &mut RoleState)>,
    role_entity: Single<Entity, With<RoleState>>,
    floor_entities: Query<(Entity, &MapItem)>,
    mut nxt_state: ResMut<NextState<GameState>>,
    mut lv_idx_entity_paires: ResMut<IdxEntityPair>,
) {
    let (mut role_speed, mut role_state) = role_sv.into_inner();
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            info!("collide: {}, {}", entity1, entity2);
            if *entity2 == *role_entity || *entity1 == *role_entity {
                let mut obstacle_entity = entity1;
                if *entity1 == *role_entity {
                    obstacle_entity = entity2;
                }
                for (entity, map_item) in floor_entities.iter() {
                    if let MapItem::Obstacle = map_item {
                        if entity == *obstacle_entity {
                            //nxt_state.set(GameState::Paused);
                            info!("boom!");
                            for (dee, _) in floor_entities.iter() {
                                cmd.entity(dee).despawn();
                            }
                            lv_idx_entity_paires.pairs.clear();
                            cmd.entity(*role_entity).despawn();
                            nxt_state.set(GameState::InitLevel);
                            return;
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
) {
    fn despawn_by_lv_idx(
        cmd: &mut Commands,
        lv_idx_entity_paires: &mut ResMut<IdxEntityPair>,
        idx: u32,
    ) {
        if let Some((entity, entity_op)) = lv_idx_entity_paires.pairs.get(&idx) {
            info!("despawn: {}", entity);
            cmd.entity(*entity).despawn();

            if let Some(entity) = entity_op {
                info!("despawn: {}", entity);
                cmd.entity(*entity).despawn();
            }
        }
        lv_idx_entity_paires.pairs.remove(&idx);
    }
    for (i, item_data) in level_data.data.iter().enumerate() {
        match item_data {
            MapItemData::Rect(rect) => {
                if camera_transform.translation.x - (rect.x + rect.z) > 500.0 {
                    despawn_by_lv_idx(&mut cmd, &mut lv_idx_entity_paires, i as u32);
                }
                let ng = rect.x - camera_transform.translation.x;
                if ng < 500.0 && ng > 300.0 && !lv_idx_entity_paires.pairs.contains_key(&(i as u32))
                {
                    spawn_floor(&mut cmd, rect, i as u32, &mut lv_idx_entity_paires);
                }
            }
            MapItemData::Tri(tri) => {
                if camera_transform.translation.x - tri.vertices[2][0] > 500.0 {
                    despawn_by_lv_idx(&mut cmd, &mut lv_idx_entity_paires, i as u32);
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

fn game_pause_play(
    mut cmd: Commands,
    state: Res<State<GameState>>,
    mut nxt_state: ResMut<NextState<GameState>>,
) {
    match state.get() {
        GameState::Playing => {
            info!("game pause");
            spawn_pause_ui(&mut cmd);
            nxt_state.set(GameState::Paused);
        }
        GameState::Paused => {
            info!("game continue");
            nxt_state.set(GameState::Playing);
        }
        _ => (),
    }
}

fn setup(mut cmd: Commands, lvs: Res<CurLevel>) {
    spawn_main_menu(&mut cmd, &lvs);
}

fn spawn_main_menu(cmd: &mut Commands, lvs: &Res<CurLevel>) {
    cmd.spawn(Camera2d::default());
    let btn_bundle = (
        Button,
        Node {
            width: Val::Px(150.),
            height: Val::Px(65.),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
    );
    let text_bundle = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
    );

    cmd.spawn(Node {
        // center button
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|parent| {
        for i in ["<", lvs.lvs[lvs.cur_idx].as_str(), ">", "start"] {
            match i {
                "<" => {
                    parent
                        .spawn((btn_bundle.clone(), LeftSelectButton))
                        .with_children(|parent| {
                            parent.spawn((text_bundle.clone(), Text::new(i)));
                        });
                }
                ">" => {
                    parent
                        .spawn((btn_bundle.clone(), RightSelectButton))
                        .with_children(|parent| {
                            parent.spawn((text_bundle.clone(), Text::new(i)));
                        });
                }
                "start" => {
                    parent
                        .spawn((btn_bundle.clone(), StartGameButton))
                        .with_children(|parent| {
                            parent.spawn((text_bundle.clone(), Text::new(i)));
                        });
                }
                s => {
                    parent.spawn((Text::new(s), CurLvLabel, text_bundle.clone()));
                }
            }
        }
    })
    .insert(MainUIEntity);
}

fn spawn_pause_ui(cmd: &mut Commands) {
    let btn_bundle = (
        Button,
        Node {
            width: Val::Px(150.),
            height: Val::Px(65.),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
    );
    let btn_text_bundle = (
        TextFont {
            font_size: 33.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
    );

    cmd.spawn(Node {
        // center button
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|parent| {
        for i in ["continue", "return"] {
            match i {
                "continue" => {
                    parent
                        .spawn((btn_bundle.clone(), ContinueButton))
                        .with_children(|parent| {
                            parent.spawn((btn_text_bundle.clone(), Text::new(i)));
                        });
                }
                "return" => {
                    parent
                        .spawn((btn_bundle.clone(), ReturnMainMenuButton))
                        .with_children(|parent| {
                            parent.spawn((btn_text_bundle.clone(), Text::new(i)));
                        });
                }
                _ => (),
            }
        }
    });
}

fn start_button_action(
    mut cmd: Commands,
    start_button: Query<&Interaction, (Changed<Interaction>, With<StartGameButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    main_ui: Single<Entity, With<MainUIEntity>>,
    lvs: Res<CurLevel>,
    mut lvd: ResMut<LevelData>,
) {
    let Ok(interaction) = start_button.get_single() else {
        return;
    };
    if let Interaction::Pressed = interaction {
        info!("start game");
        cmd.entity(*main_ui).despawn_recursive();
        *lvd = LevelData::from_file(&lvs.lvs[lvs.cur_idx]);
        next_state.set(GameState::InitLevel);
    }
}

fn select_lv_left_button_action(
    start_button: Query<&Interaction, (Changed<Interaction>, With<LeftSelectButton>)>,
    mut lvs: ResMut<CurLevel>,
    cur_lv_text: Single<&mut Text, With<CurLvLabel>>,
) {
    let Ok(interaction) = start_button.get_single() else {
        return;
    };
    if let Interaction::Pressed = interaction {
        if lvs.cur_idx == 0 {
            lvs.cur_idx = lvs.lvs.len() - 1;
        } else {
            lvs.cur_idx -= 1;
        }
        let mut text = cur_lv_text.into_inner();
        **text = lvs.lvs[lvs.cur_idx].to_string();
    }
}

fn select_lv_right_button_action(
    start_button: Query<&Interaction, (Changed<Interaction>, With<RightSelectButton>)>,
    mut lvs: ResMut<CurLevel>,
    cur_lv_text: Single<&mut Text, With<CurLvLabel>>,
) {
    let Ok(interaction) = start_button.get_single() else {
        return;
    };
    if let Interaction::Pressed = interaction {
        lvs.cur_idx = (lvs.cur_idx + 1) % lvs.lvs.len();
        let mut text = cur_lv_text.into_inner();
        **text = lvs.lvs[lvs.cur_idx].to_string();
    }
}

fn start_playing(state: Res<State<GameState>>, mut nxt_state: ResMut<NextState<GameState>>) {
    match state.get() {
        GameState::InitLevel => {
            info!("game start");
            nxt_state.set(GameState::Playing);
        }
        _ => (),
    }
}
