use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_single::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, role_move)
        .run()
}

#[derive(Component)]
struct RoleSpeed(f32);

#[derive(Component)]
struct Role;

#[derive(Component)]
struct TargetTransform(Transform);

fn setup(
    mut cmd: Commands,
    mut meshs: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    cmd.spawn(Camera2dBundle::default());
    let mesh_handle = Mesh2dHandle(meshs.add(Circle::new(30.0)));
    cmd.spawn((
        mesh_handle,
        materials.add(Color::srgba(0.2, 0.2, 0.2, 1.0)),
        Transform::from_xyz(30.0, 30.0, 30.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Role,
        TargetTransform(Transform::from_xyz(30.0, 30.0, 30.0)),
        RoleSpeed(0.8),
    ));
}

fn role_move(
    window: Single<&Window>,
    input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut comps: Query<(&mut Transform, &mut TargetTransform, &RoleSpeed), With<Role>>,
) {
    if input.just_pressed(MouseButton::Left) {
        let (camera, camera_transform) = *camera;
        if let Some(pos) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            for (_, mut t, _) in comps.iter_mut() {
                t.0.translation.x = pos.x;
                t.0.translation.y = pos.y;
                break;
            }
        }
    }

    for (mut r, t, spd) in comps.iter_mut() {
        if *r != t.0 {
            let x = t.0.translation.x - r.translation.x;
            let y = t.0.translation.y - r.translation.y;
            let v = (x * x + y * y).sqrt();
            r.translation.x += spd.0 / v * x;
            r.translation.y += spd.0 / v * y;
        }
        break;
    }
}

fn move_ins(
    mut target_transform: Query<&mut TargetTransform, With<Role>>,
    mouse_events: EventReader<CursorMoved>,
) {
    let Ok(mut tt) = target_transform.get_single_mut() else {
        return;
    };
}
