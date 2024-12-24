use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (role_move, gravity))
        .run()
}

#[derive(Component)]
struct RoleSpeed(f32, f32);

#[derive(Component)]
struct Role;

#[derive(Component)]
enum RoleState {
    Air,
    Floor,
}

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(Camera2d::default());
    cmd.spawn((
        Sprite::from_image(asset_server.load("branding/bevy_bird_dark.png")),
        RoleState::Floor,
        RoleSpeed(10.0, 0.0),
    ));
}

fn gravity(time: Res<Time>, rss: Single<(&mut RoleSpeed, &RoleState), With<RoleState>>) {
    let delta = time.delta_secs();
    let (mut speed, role_state) = rss.into_inner();
    match role_state {
        RoleState::Air => {
            speed.1 -= GRAVITY * delta;
        }
        RoleState::Floor => {
            speed.1 = 0.0;
        }
    }
}

fn collisions() {}

const GRAVITY: f32 = 9.821 * 10.0;

fn role_move(
    input: Res<ButtonInput<KeyCode>>,
    mut role: Single<(&mut Transform, &mut RoleSpeed), With<RoleSpeed>>,
    time: Res<Time>,
    mut camera_transform: Single<&mut Transform, (With<Camera>, Without<RoleSpeed>)>,
) {
    if input.just_pressed(KeyCode::Space) {}
    let (mut role_transform, mut speed) = role.into_inner();
    role_transform.translation.x += speed.0 * time.delta_secs();
    role_transform.translation.y += speed.1 * time.delta_secs();
    camera_transform.translation.x += speed.0 * time.delta_secs();
}
