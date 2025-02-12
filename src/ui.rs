use bevy::prelude::*;

use crate::types::*;
use std::path::Path;

pub fn main_ui(mut cmd: Commands, lvs: Res<CurLevel>) {
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
        for i in [
            "<",
            Path::new(lvs.lvs[lvs.cur_idx].as_str())
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap(),
            ">",
            "start",
        ] {
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

pub fn pause_ui(mut cmd: Commands) {
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
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|parent| {
        parent
            .spawn((btn_bundle.clone(), ReturnMainMenuButton))
            .with_children(|parent| {
                parent.spawn((text_bundle.clone(), Text::new("return")));
            });
        parent.spawn((text_bundle.clone(), Text::new("press ECS to continue")));
    })
    .insert(PauseUIEntity);
}

pub fn leave_pause(mut cmd: Commands, pause_ui: Single<Entity, With<PauseUIEntity>>) {
    cmd.entity(*pause_ui).despawn_recursive();
}

pub fn leave_main(mut cmd: Commands, main_ui: Single<Entity, With<MainUIEntity>>) {
    cmd.entity(*main_ui).despawn_recursive();
}

pub fn start_button_action(
    start_button: Query<&Interaction, (Changed<Interaction>, With<StartGameButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    lvs: Res<CurLevel>,
    mut lvd: ResMut<LevelData>,
) {
    let Ok(interaction) = start_button.get_single() else {
        return;
    };
    if let Interaction::Pressed = interaction {
        info!("start game");
        *lvd = LevelData::from_file(&lvs.lvs[lvs.cur_idx]);
        next_state.set(GameState::InitLevel);
    }
}

pub fn select_lv_left_button_action(
    left_select_btn: Query<&Interaction, (Changed<Interaction>, With<LeftSelectButton>)>,
    mut lvs: ResMut<CurLevel>,
    cur_lv_text: Single<&mut Text, With<CurLvLabel>>,
) {
    let Ok(interaction) = left_select_btn.get_single() else {
        return;
    };
    if let Interaction::Pressed = interaction {
        if lvs.cur_idx == 0 {
            lvs.cur_idx = lvs.lvs.len() - 1;
        } else {
            lvs.cur_idx -= 1;
        }
        let mut text = cur_lv_text.into_inner();
        **text = Path::new(lvs.lvs[lvs.cur_idx].as_str())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
    }
}

pub fn select_lv_right_button_action(
    right_select_btn: Query<&Interaction, (Changed<Interaction>, With<RightSelectButton>)>,
    mut lvs: ResMut<CurLevel>,
    cur_lv_text: Single<&mut Text, With<CurLvLabel>>,
) {
    let Ok(interaction) = right_select_btn.get_single() else {
        return;
    };
    if let Interaction::Pressed = interaction {
        lvs.cur_idx = (lvs.cur_idx + 1) % lvs.lvs.len();
        let mut text = cur_lv_text.into_inner();
        **text = Path::new(lvs.lvs[lvs.cur_idx].as_str())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
    }
}

pub fn return_main_ui(
    mut cmd: Commands,
    return_btn: Query<&Interaction, (Changed<Interaction>, With<ReturnMainMenuButton>)>,
    map_item: Query<Entity, With<MapItem>>,
    role: Single<Entity, With<RoleSpeed>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut lv_idx_entity_paires: ResMut<IdxEntityPair>,
) {
    let Ok(interaction) = return_btn.get_single() else {
        return;
    };
    if let Interaction::Pressed = interaction {
        for entity in &map_item {
            cmd.entity(entity).despawn_recursive();
        }
        cmd.entity(*role).despawn_recursive();
        lv_idx_entity_paires.pairs.clear();
        next_state.set(GameState::Main);
    }
}

pub fn start_playing(state: Res<State<GameState>>, mut nxt_state: ResMut<NextState<GameState>>) {
    match state.get() {
        GameState::InitLevel => {
            info!("game start");
            nxt_state.set(GameState::Playing);
        }
        _ => (),
    }
}
