use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier2d::prelude::*;

mod game;
mod types;
mod ui;

use game::*;
use types::*;
use ui::*;

fn main() -> AppExit {
    App::new()
        .insert_resource(CurLevel::default())
        .insert_resource(LevelData::default())
        .insert_resource(IdxEntityPair::default())
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(WINDOW_RESOLUTION_X, WINDOW_RESOLUTION_Y)
                        .with_scale_factor_override(1.0),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Main), main_ui)
        .add_systems(OnEnter(GameState::InitLevel), game_init)
        .add_systems(OnExit(GameState::Main), leave_main)
        .add_systems(OnEnter(GameState::Paused), pause_ui)
        .add_systems(OnExit(GameState::Paused), leave_pause)
        .add_systems(
            Update,
            (
                gravity,
                jump.run_if(input_just_pressed(KeyCode::Space)),
                role_move,
                dynamic_map_item,
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
                return_main_ui.run_if(in_state(GameState::Paused)),
            ),
        )
        .add_systems(
            Update,
            collide_events.run_if(in_state(GameState::InitLevel).or(in_state(GameState::Playing))),
        )
        .run()
}
