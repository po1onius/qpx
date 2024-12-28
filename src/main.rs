use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                jump.run_if(input_just_pressed(KeyCode::Space)),
                role_move,
                loop_block,
                collision,
                gravity,
            )
                .chain(),
        )
        .run()
}

#[derive(Component)]
struct RoleSpeed(f32, f32);

#[derive(Component)]
enum RoleState {
    Air,
    Floor,
}

#[derive(Component)]
struct MapItem;

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(Camera2d::default());

    let block_texture = asset_server.load("block.png");

    for i in 0..10 {
        cmd.spawn((
            Sprite::from_image(block_texture.clone()),
            Transform::from_xyz(-200.0 + i as f32 * 112.0, -160.0, 0.0),
            MapItem,
        ));
    }

    cmd.spawn((
        Sprite::from_image(asset_server.load("branding/bevy_bird_dark.png")),
        RoleState::Air,
        RoleSpeed(80.0, 0.0),
        Transform::from_xyz(-100.0, 0.0, 0.0),
    ));
}

fn gravity(time: Res<Time>, rss: Single<(&mut RoleSpeed, &RoleState)>) {
    let delta = time.delta_secs();
    let (mut speed, role_state) = rss.into_inner();
    match role_state {
        RoleState::Air => {
            speed.1 -= GRAVITY * delta;
        }
        RoleState::Floor => {
            if speed.1 != 0.0 {
                speed.1 = 0.0;
            }
        }
    }
}

fn jump(role_sv: Single<(&mut RoleSpeed, &mut RoleState)>) {
    let (mut role_speed, mut role_state) = role_sv.into_inner();
    if let RoleState::Floor = *role_state {
        role_speed.1 += 100.0;
        *role_state = RoleState::Air;
    }
}

fn collision(
    map_items: Query<&Transform, With<MapItem>>,
    role: Single<(&Transform, &mut RoleState)>,
) {
    let (role_transform, mut role_state) = role.into_inner();
    if let RoleState::Air = *role_state {
        for map_item_transform in map_items.iter() {
            let yg = (map_item_transform.translation.y - role_transform.translation.y).abs();
            let xg = (map_item_transform.translation.x - role_transform.translation.x).abs();
            if xg < 20.0 && yg < 20.0 {
                info!("collision!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                *role_state = RoleState::Floor;
                return;
            }
        }
    }
}

const GRAVITY: f32 = 9.821 * 10.0;

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
            cmd.entity(item_entity).despawn();
        }
    }
}
