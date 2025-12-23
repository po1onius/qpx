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

fn spawn_rect_pass(
    cmd: &mut Commands,
    rect: &Vec4,
    index: u32,
    lv_idx_entity_paires: &mut ResMut<IdxEntityPair>,
) {
    let rect_pass = MapItemBundle::rect_pass(rect);
    let id = cmd.spawn(rect_pass).id();

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

// 随着镜头移动创建和销毁地图资源
pub fn dynamic_map_item(
    mut cmd: Commands,
    level_data: Res<LevelData>,
    mut lv_idx_entity_paires: ResMut<IdxEntityPair>,
    _asset_server: Res<AssetServer>,
    camera_transform: Single<&mut Transform, (With<Camera>, Without<RoleSpeed>, Without<MapItem>)>,
) {
    let screen_half_x = (WINDOW_RESOLUTION_X / 2) as f32;

    type SpawnRectArgs<'a, 'b, 'c, 'd> =
        (&'a mut Commands<'b, 'c>, &'a mut ResMut<'d, IdxEntityPair>);

    for (i, lv_data) in level_data.data.iter().enumerate() {
        let i = i as u32;
        let (spawn_f, left, right): (Box<dyn Fn(SpawnRectArgs)>, _, _) = match lv_data {
            MapItemData::RectFlyBegin(rect) => (
                Box::new(|args: SpawnRectArgs| {
                    spawn_rect_fly(args.0, rect, i, args.1, true);
                }),
                rect.x - rect.z,
                rect.x + rect.z,
            ),
            MapItemData::RectFlyEnd(rect) => (
                Box::new(|args: SpawnRectArgs| {
                    spawn_rect_fly(args.0, rect, i, args.1, false);
                }),
                rect.x - rect.z,
                rect.x + rect.z,
            ),
            MapItemData::RectObstacle(rect) => (
                Box::new(|args: SpawnRectArgs| {
                    spawn_rect_obstacle(args.0, rect, i, args.1);
                }),
                rect.x - rect.z,
                rect.x + rect.z,
            ),
            MapItemData::RectPass(rect) => (
                Box::new(|args: SpawnRectArgs| {
                    spawn_rect_pass(args.0, rect, i, args.1);
                }),
                rect.x - rect.z,
                rect.x + rect.z,
            ),
            MapItemData::Floor(rect) => (
                Box::new(|args: SpawnRectArgs| {
                    spawn_floor(args.0, rect, i, args.1);
                }),
                rect.x - rect.z,
                rect.x + rect.z,
            ),
            MapItemData::TriObstacle(tri) => (
                Box::new(|args: SpawnRectArgs| {
                    spawn_tri_obstacle(args.0, tri, i, args.1);
                }),
                tri.vertices[0].x,
                tri.vertices[2].x,
            ),

            MapItemData::DoubleJumpCircle(pos, radius) => (
                Box::new(|args: SpawnRectArgs| {
                    spawn_circle(args.0, pos, *radius, i, args.1);
                }),
                pos.x - radius,
                pos.x + radius,
            ),
        };

        if let Some(entity_idx) = lv_idx_entity_paires.pairs.get(&i) {
            if camera_transform.translation.x - right > screen_half_x {
                cmd.entity(entity_idx.0).despawn();
                info!("destroy entity {}", entity_idx.0);
                if let Some(attach_entity) = entity_idx.1 {
                    cmd.entity(attach_entity).despawn();
                    info!("destroy entity {}", attach_entity);
                }
                lv_idx_entity_paires.pairs.remove(&i);
            }
        } else {
            let coming_distance = left - camera_transform.translation.x;
            if coming_distance > 0.0 && coming_distance < screen_half_x {
                spawn_f((&mut cmd, &mut lv_idx_entity_paires));
            }
        }
    }
}

pub fn game_init(
    mut cmd: Commands,
    mut camera_transform: Single<
        &mut Transform,
        (With<Camera>, Without<RoleSpeed>, Without<MapItem>),
    >,
) {
    info!("game init");
    //let block_texture = asset_server.load("block.png");
    camera_transform.translation.x = 0.0;
    camera_transform.translation.y = 0.0;

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
    mut collision_events: MessageReader<CollisionEvent>,
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
                            MapItem::Pass => {
                                info!("collide pass");
                                nxt_state.set(GameState::Paused);
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
                                | MapItem::Pass
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
