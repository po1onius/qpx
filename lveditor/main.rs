use bevy::color::palettes::basic::*;
use bevy::input::common_conditions::{input_just_pressed, input_just_released};
use bevy::math::prelude::*;
use bevy::prelude::*;
use bevy::window::{
    CursorGrabMode, EnabledButtons, PresentMode, PrimaryWindow, WindowLevel, WindowMode,
    WindowResolution, WindowTheme,
};
use bevy_rapier2d::prelude::*;

use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // 窗口标题
                title: "Window Properties".into(),
                // 窗口的应用程序 ID（Wayland)、WM_CLASS（X11) 或 窗口类名称（Windows）
                name: Some("bevy.window".into()),
                // 控制窗口模式
                mode: WindowMode::Windowed,
                // 窗口位置
                position: WindowPosition::Automatic,
                // 窗口透明
                transparent: true,
                // 窗口分辨率
                resolution: WindowResolution::new(1280.0, 720.0).with_scale_factor_override(1.0),
                resizable: false,
                // 呈现模式
                present_mode: PresentMode::AutoVsync,
                // 窗口装饰
                decorations: true,
                // 窗口按钮
                enabled_buttons: EnabledButtons {
                    // 最大化
                    minimize: true,
                    // 最小化
                    maximize: true,
                    // 关闭
                    close: true,
                },
                // 焦点
                focused: true,
                // 窗口等级
                window_level: WindowLevel::Normal,
                // 窗口主题
                window_theme: Some(WindowTheme::Dark),
                // 窗口可见性
                visible: true,
                // 跳过任务栏
                skip_taskbar: false,
                // 最大排队帧数
                desired_maximum_frame_latency: None,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(CursorWorldPos(None))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                button_system,
                get_cursor_world_pos,
                (
                    start_drag.run_if(input_just_pressed(MouseButton::Left)),
                    end_drag.run_if(input_just_released(MouseButton::Left)),
                    drag.run_if(resource_exists::<DragOperation>),
                )
                    .chain(),
            ),
        )
        .run()
}

#[derive(Resource)]
struct CursorWorldPos(Option<Vec2>);

/// The current drag operation including the offset with which we grabbed the Bevy logo.
#[derive(Resource)]
struct DragOperation(Vec2, Entity);

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const FONT_PATH: &'static str =
    "/home/srus/.local/share/fonts/JetBrainsMono/JetBrainsMonoNerdFont-Medium.ttf";

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
    commands.spawn(Camera2d);
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        left: Val::Px(1280.0 / 2.0 - 150.0 / 2.0),
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                ))
                .with_child((
                    Text::new("Button"),
                    TextFont {
                        font: asset_server.load(FONT_PATH),
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
        });
}

fn get_cursor_world_pos(
    mut cursor_world_pos: ResMut<CursorWorldPos>,
    primary_window: Single<&Window, With<PrimaryWindow>>,
    q_camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (main_camera, main_camera_transform) = *q_camera;
    // Get the cursor position in the world
    cursor_world_pos.0 = primary_window.cursor_position().and_then(|cursor_pos| {
        main_camera
            .viewport_to_world_2d(main_camera_transform, cursor_pos)
            .ok()
    });
}

/// Update whether the window is clickable or not
fn start_drag(
    mut commands: Commands,
    cursor_world_pos: Res<CursorWorldPos>,
    items: Query<(Entity, &Transform, &Mesh2d, &MapItem)>,
) {
    // If the cursor is not within the primary window skip this system
    let Some(cursor_world_pos) = cursor_world_pos.0 else {
        return;
    };

    for (e, transform, _, _) in items.iter() {
        let drag_offset = transform.translation.truncate() - cursor_world_pos;
        if drag_offset.length() < 30.0 {
            commands.insert_resource(DragOperation(drag_offset, e));
            break;
        }
    }
    // Get the offset from the cursor to the Bevy logo sprite
    //let drag_offset = bevy_logo_transform.translation.truncate() - cursor_world_pos;

    // If the cursor is within the Bevy logo radius start the drag operation and remember the offset of the cursor from the origin
}

/// Stop the current drag operation
fn end_drag(mut commands: Commands) {
    commands.remove_resource::<DragOperation>();
}

/// Drag the Bevy logo
fn drag(
    drag_offset: Res<DragOperation>,
    cursor_world_pos: Res<CursorWorldPos>,
    mut items: Query<(Entity, &mut Transform, &Mesh2d, &MapItem)>,
) {
    // If the cursor is not within the primary window skip this system
    let Some(cursor_world_pos) = cursor_world_pos.0 else {
        return;
    };

    // Calculate the new translation of the Bevy logo based on cursor and drag offset
    let new_translation = cursor_world_pos + drag_offset.0;
    for (e, mut t, _, _) in items.iter_mut() {
        if e == drag_offset.1 {
            // Update the translation of Bevy logo transform to new translation
            t.translation = new_translation.extend(t.translation.z);
            break;
        }
    }
    // Calculate how fast we are dragging the Bevy logo (unit/second)

    // Add the cursor drag velocity in the opposite direction to each pupil.
    // Remember pupils are using local coordinates to move. So when the Bevy logo moves right they need to move left to
    // simulate inertia, otherwise they will move fixed to the parent.
}

#[derive(Component)]
struct MapItem;

fn spawn_rect(
    cmd: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    cmd.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(Color::from(PURPLE))),
        Transform::default().with_scale(Vec3::splat(128.)),
        MapItem,
    ));
}

fn button_system(
    mut cmd: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                spawn_rect(&mut cmd, &mut meshes, &mut materials);
            }
            Interaction::Hovered => {
                **text = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                **text = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}
