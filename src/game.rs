use bevy::math::prelude::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::types::*;

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

fn spawn_rect_obstacle(
    cmd: &mut Commands,
    rect: &Vec4,
    index: u32,
    lv_idx_entity_paires: &mut ResMut<IdxEntityPair>,
) {
    let rect_obstacle = MapItemBundle::rect_item(rect, true);
    let id = cmd.spawn(rect_obstacle).id();

    lv_idx_entity_paires.pairs.insert(index, (id, None));
    info!("spawn: entity {}", id);
}

fn spawn_rect_fly(
    cmd: &mut Commands,
    rect: &Vec4,
    index: u32,
    lv_idx_entity_paires: &mut ResMut<IdxEntityPair>,
    begin: bool,
) {
    let rect_fly = MapItemBundle::rect_fly(rect, begin);
    let id = cmd.spawn(rect_fly).insert(Sensor).id();

    lv_idx_entity_paires.pairs.insert(index, (id, None));
    info!("spawn: entity {}", id);
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

fn spawn_circle(
    cmd: &mut Commands,
    pos: &Vec2,
    radius: f32,
    index: u32,
    lv_idx_entity_paires: &mut ResMut<IdxEntityPair>,
) {
    let id = cmd
        .spawn(MapItemBundle::circle_double_jump(pos, radius))
        .insert(Sensor)
        .id();
    lv_idx_entity_paires.pairs.insert(index, (id, None));
    info!("sapwn: entity {}", id);
}
pub fn game_init(
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

    let screen_half_x = WINDOW_RESOLUTION_X / 2.0;

    for (i, l) in level_data.data.iter().enumerate() {
        match l {
            MapItemData::Floor(rect) => {
                let lpx = rect.x - rect.z;
                if lpx > -screen_half_x && lpx < screen_half_x {
                    spawn_floor(&mut cmd, rect, i as u32, &mut lv_idx_entity_paires);
                }
            }
            MapItemData::RectObstacle(rect) => {
                let lpx = rect.x - rect.z;
                if lpx > -screen_half_x && lpx < screen_half_x {
                    spawn_rect_obstacle(&mut cmd, rect, i as u32, &mut lv_idx_entity_paires);
                }
            }
            MapItemData::RectFlyBegin(rect) | MapItemData::RectFlyEnd(rect) => {
                let lpx = rect.x - rect.z;
                let mut begin = false;
                if lpx > -screen_half_x && lpx < screen_half_x {
                    if let MapItemData::RectFlyBegin(_) = l {
                        begin = true;
                    }
                    spawn_rect_fly(&mut cmd, rect, i as u32, &mut lv_idx_entity_paires, begin);
                }
            }
            MapItemData::TriObstacle(tri) => {
                let lpx = tri.vertices[0].x;
                if lpx > -screen_half_x && lpx < screen_half_x {
                    spawn_tri_obstacle(&mut cmd, tri, i as u32, &mut lv_idx_entity_paires);
                }
            }
            MapItemData::DoubleJumpCircle(pos, radius) => {
                let lpx = pos.x - radius;
                if lpx > -screen_half_x && lpx < screen_half_x {
                    spawn_circle(&mut cmd, pos, *radius, i as u32, &mut lv_idx_entity_paires);
                }
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
        RoleState::Air(999),
        RoleSpeed(ROLE_SPEED, 0.0),
        Transform::from_xyz(-100.0, 200.0, 0.0),
    ));
}

pub fn gravity(role_sv: Single<(&mut RoleSpeed, &RoleState)>, time: Res<Time>) {
    let (mut role_speed, role_state) = role_sv.into_inner();
    if let RoleState::Air(_) = *role_state {
        role_speed.1 -= GRAVITY * time.delta_secs();
    }
}

/* A system that displays the events. */
pub fn collide_events(
    mut cmd: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    role_sv: Single<(&mut RoleSpeed, &mut RoleState)>,
    role_entity: Single<Entity, With<RoleState>>,
    map_item_entities: Query<(Entity, &MapItem)>,
    mut nxt_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    mut lv_idx_entity_paires: ResMut<IdxEntityPair>,
) {
    let (mut role_speed, mut role_state) = role_sv.into_inner();
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            info!("collide: {}, {}", entity1, entity2);
            if *entity2 == *role_entity || *entity1 == *role_entity {
                let mut other_entity = entity1;
                if *entity1 == *role_entity {
                    other_entity = entity2;
                }
                for (entity, map_item) in map_item_entities.iter() {
                    if entity == *other_entity {
                        match map_item {
                            MapItem::Obstacle => {
                                //nxt_state.set(GameState::Paused);
                                info!("boom!");
                                for (dee, _) in map_item_entities.iter() {
                                    cmd.entity(dee).despawn();
                                }
                                lv_idx_entity_paires.pairs.clear();
                                cmd.entity(*role_entity).despawn();
                                nxt_state.set(GameState::InitLevel);
                                return;
                            }
                            MapItem::DoubleJump => {
                                if let RoleState::Air(_) = *role_state {
                                    *role_state = RoleState::Air(1);
                                }
                            }
                            MapItem::Normal => {
                                info!("collide floor");
                                *role_state = RoleState::Normal;
                                role_speed.1 = 0.0;
                            }
                            MapItem::FlyBegin => {
                                info!("collide fly begin");
                                *role_state = RoleState::Air(999);
                            }
                            MapItem::FlyEnd => {
                                info!("collide fly end");
                                *role_state = RoleState::Air(0);
                            }
                        }
                    }
                }
            }
        }
        if let CollisionEvent::Stopped(entity1, entity2, _) = collision_event {
            if let GameState::Playing = state.get() {
                info!("collide: {}, {}", entity1, entity2);
                if *entity2 == *role_entity || *entity1 == *role_entity {
                    let mut other_entity = entity1;
                    if *entity1 == *role_entity {
                        other_entity = entity2;
                    }
                    for (entity, map_item) in map_item_entities.iter() {
                        if entity == *other_entity {
                            match map_item {
                                MapItem::Obstacle
                                | MapItem::Normal
                                | MapItem::DoubleJump
                                | MapItem::FlyEnd => {
                                    *role_state = RoleState::Air(0);
                                }
                                MapItem::FlyBegin => {
                                    info!("collide fly begin");
                                    //*role_state = RoleState::Air(999);
                                }
                            }
                        }
                    }
                }
            } else {
                info!("swallow after boom");
            }
        }
    }
}

pub fn jump(role_sv: Single<(&mut RoleSpeed, &mut RoleState)>) {
    let (mut role_speed, mut role_state) = role_sv.into_inner();
    if let RoleState::Air(jn) = *role_state {
        info!("jump times {}", jn);
        if jn == 0 {
            return;
        } else {
            *role_state = RoleState::Air(jn - 1);
        }
    } else {
        *role_state = RoleState::Air(0);
    }
    role_speed.1 = JUMP_SPEED;
}

pub fn role_move(
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

pub fn loop_block(
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
            MapItemData::Floor(rect)
            | MapItemData::RectObstacle(rect)
            | MapItemData::RectFlyBegin(rect)
            | MapItemData::RectFlyEnd(rect) => {
                if camera_transform.translation.x - (rect.x + rect.z) > 500.0 {
                    despawn_by_lv_idx(&mut cmd, &mut lv_idx_entity_paires, i as u32);
                }
                let ng = rect.x - rect.z - camera_transform.translation.x;
                if ng < 500.0 && ng > 300.0 && !lv_idx_entity_paires.pairs.contains_key(&(i as u32))
                {
                    if let MapItemData::RectObstacle(_) = item_data {
                        spawn_rect_obstacle(&mut cmd, rect, i as u32, &mut lv_idx_entity_paires);
                    } else if let MapItemData::Floor(_) = item_data {
                        spawn_floor(&mut cmd, rect, i as u32, &mut lv_idx_entity_paires);
                    } else if let MapItemData::RectFlyBegin(_) = item_data {
                        spawn_rect_fly(&mut cmd, rect, i as u32, &mut lv_idx_entity_paires, true);
                    } else {
                        spawn_rect_fly(&mut cmd, rect, i as u32, &mut lv_idx_entity_paires, false);
                    }
                }
            }
            MapItemData::TriObstacle(tri) => {
                if camera_transform.translation.x - tri.vertices[2].x > 500.0 {
                    despawn_by_lv_idx(&mut cmd, &mut lv_idx_entity_paires, i as u32);
                }
                let ng = tri.vertices[0].x - camera_transform.translation.x;
                if ng < 500.0 && ng > 300.0 && !lv_idx_entity_paires.pairs.contains_key(&(i as u32))
                {
                    spawn_tri_obstacle(&mut cmd, tri, i as u32, &mut lv_idx_entity_paires);
                }
            }
            MapItemData::DoubleJumpCircle(pos, radius) => {
                if camera_transform.translation.x - pos.x > 500.0 {
                    despawn_by_lv_idx(&mut cmd, &mut lv_idx_entity_paires, i as u32);
                }
                let ng = pos.x - radius - camera_transform.translation.x;
                if ng < 500.0 && ng > 300.0 && !lv_idx_entity_paires.pairs.contains_key(&(i as u32))
                {
                    spawn_circle(&mut cmd, pos, *radius, i as u32, &mut lv_idx_entity_paires);
                }
            }
        }
    }
}

pub fn game_pause_play(state: Res<State<GameState>>, mut nxt_state: ResMut<NextState<GameState>>) {
    match state.get() {
        GameState::Playing => {
            info!("game pause");
            nxt_state.set(GameState::Paused);
        }
        GameState::Paused => {
            info!("game continue");
            nxt_state.set(GameState::Playing);
        }
        _ => (),
    }
}

pub fn setup(mut cmd: Commands) {
    cmd.spawn(Camera2d::default());
    //spawn_main_menu(cmd, lvs);
}
