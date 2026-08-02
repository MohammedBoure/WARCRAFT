mod audio;
mod config;
mod gameplay;
mod interaction;
mod menu;
mod player;
mod state;
mod ui;
mod world;

use std::env;

use bevy::prelude::*;

use crate::audio::*;
use crate::gameplay::*;
use crate::interaction::*;
use crate::menu::*;
use crate::player::*;
use crate::state::*;
use crate::ui::*;
use crate::world::*;

fn main() {
    let options = ViewerOptions::parse(env::args().skip(1)).unwrap_or_else(|error| {
        eprintln!("نقطة الانهيار: {error}");
        std::process::exit(2);
    });
    if options.help {
        println!("{}", ViewerOptions::help_text());
        return;
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "نقطة الانهيار — Critical Point".to_string(),
                resolution: (1280, 720).into(),
                resizable: true,
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.055, 0.075)))
        .insert_resource(VoxelViewerWorld {
            settings: options.generation_settings(),
            load_radius: options.load_radius,
        })
        .insert_resource(VoxelViewerCamera {
            center: Vec2::ZERO,
            yaw: -0.72,
            height: CAMERA_DEFAULT_HEIGHT,
            shake: 0.0,
        })
        .init_resource::<ArabicFont>()
        .init_resource::<GameSession>()
        .init_resource::<BalanceConfig>()
        .init_resource::<GamePreferences>()
        .init_resource::<PlayerState>()
        .init_resource::<MiddleClickResetTimer>()
        .init_resource::<VoxelViewerWeatherState>()
        .init_resource::<VoxelWorldEdits>()
        .init_resource::<MiningState>()
        .init_resource::<LoadedVoxelChunks>()
        .init_resource::<VoxelViewerRenderAssets>()
        .init_resource::<RunLifecycle>()
        .init_resource::<CollapseDirector>()
        .init_resource::<AudioRiskState>()
        .init_resource::<LoadingTimer>()
        .add_message::<CrystalCollected>()
        .add_message::<CriticalityChanged>()
        .add_message::<CollapseTriggered>()
        .add_message::<ChoiceCommitted>()
        .add_message::<RunFinished>()
        .add_message::<VoxelActionSound>()
        .add_systems(Startup, load_shared_assets)
        .add_systems(
            PostStartup,
            (
                setup_viewer_scene,
                spawn_player_character,
                setup_target_highlight,
                setup_gameplay_assets,
                setup_audio,
                setup_hud,
            ),
        )
        .add_systems(OnEnter(AppState::Loading), setup_loading_screen)
        .add_systems(Update, finish_loading.run_if(in_state(AppState::Loading)))
        .add_systems(OnExit(AppState::Loading), cleanup_screen)
        .add_systems(
            OnEnter(AppState::MainMenu),
            (enter_main_menu, setup_main_menu).chain(),
        )
        .add_systems(
            Update,
            handle_menu_buttons.run_if(in_state(AppState::MainMenu)),
        )
        .add_systems(OnExit(AppState::MainMenu), cleanup_screen)
        .add_systems(
            OnEnter(AppState::Playing),
            (reset_player_for_run, prepare_new_run).chain(),
        )
        .add_systems(
            Update,
            (
                update_player_movement,
                update_target_block_highlight,
                handle_voxel_digging_and_building,
                process_crystal_events,
                update_run_clock,
                handle_final_interaction,
                drive_collapses,
                resolve_collapse_warnings,
            )
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(OnEnter(AppState::Paused), setup_pause_overlay)
        .add_systems(OnExit(AppState::Paused), cleanup_screen)
        .add_systems(OnEnter(AppState::Decision), setup_decision_overlay)
        .add_systems(OnExit(AppState::Decision), cleanup_screen)
        .add_systems(OnEnter(AppState::Ending), setup_ending_overlay)
        .add_systems(OnExit(AppState::Ending), cleanup_screen)
        .add_systems(
            Update,
            (
                sync_visible_chunks,
                control_viewer_camera,
                update_world_mood,
                update_landmarks,
                animate_player,
                sync_player_visibility,
                update_hud,
                toggle_pause,
                pause_when_unfocused,
                handle_overlay_buttons,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                play_ui_clicks,
                play_voxel_actions,
                play_crystal_audio,
                play_choice_audio,
                play_collapse_audio,
                play_criticality_audio,
                play_finish_audio,
                update_ambient_audio,
            ),
        )
        .run();
}
