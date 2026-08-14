#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod audio;
mod combat;
mod config;
mod gameplay;
mod interaction;
mod menu;
mod player;
mod qa;
mod save_load;
mod state;
mod ui;
mod world;

use std::env;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;

use crate::audio::*;
use crate::combat::*;
use crate::gameplay::*;
use crate::interaction::*;
use crate::menu::*;
use crate::player::*;
use crate::qa::*;
use crate::save_load::*;
use crate::state::*;
use crate::ui::*;
use crate::world::*;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameUpdateSet {
    World,
    Input,
    Aim,
    Actions,
    Combat,
    Mission,
    Presentation,
    Ui,
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    let options = ViewerOptions::default();
    #[cfg(not(target_arch = "wasm32"))]
    let options = ViewerOptions::parse(env::args().skip(1)).unwrap_or_else(|error| {
        eprintln!("نقطة العبور: {error}");
        std::process::exit(2);
    });
    if options.help {
        println!("{}", ViewerOptions::help_text());
        return;
    }

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "نقطة العبور — Critical Point".to_string(),
                        resolution: (1280, 720).into(),
                        resizable: true,
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_root(),
                    ..default()
                }),
        )
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
        .init_resource::<VoxelWorldEdits>()
        .init_resource::<MiningState>()
        .init_resource::<AimSolution>()
        .init_resource::<LoadedVoxelChunks>()
        .init_resource::<VoxelViewerRenderAssets>()
        .init_resource::<RunLifecycle>()
        .init_resource::<EnemyDirector>()
        .init_resource::<CombatRuntime>()
        .init_resource::<CraftState>()
        .init_resource::<AudioDirector>()
        .init_resource::<LoadingTimer>()
        .init_resource::<PendingRoute>()
        .init_resource::<QaScreenshot>()
        .init_resource::<SaveLoadState>()
        .add_message::<DamageEvent>()
        .add_message::<ResourceCollected>()
        .add_message::<BlockCollected>()
        .add_message::<EnemyKilled>()
        .add_message::<WeaponCrafted>()
        .add_message::<RouteCommitted>()
        .add_message::<FinalChoiceCommitted>()
        .add_message::<RelayDestroyed>()
        .add_message::<RunFinished>()
        .add_message::<WaveStarted>()
        .add_message::<PlayerRespawned>()
        .add_message::<GameSound>()
        .add_systems(
            PostStartup,
            (
                setup_viewer_scene,
                spawn_player_character,
                setup_target_highlight,
                setup_gameplay_assets,
                setup_combat_assets,
                setup_audio,
                setup_hud,
                setup_save_status_ui,
                check_save_on_startup,
            ),
        )
        .add_systems(OnEnter(AppState::Loading), menu::setup_loading_screen)
        .add_systems(Update, finish_loading.run_if(in_state(AppState::Loading)))
        .add_systems(OnExit(AppState::Loading), menu::cleanup_screen)
        .add_systems(
            OnEnter(AppState::MainMenu),
            (enter_main_menu, setup_main_menu).chain(),
        )
        .add_systems(
            Update,
            handle_menu_buttons.run_if(in_state(AppState::MainMenu)),
        )
        .add_systems(OnExit(AppState::MainMenu), menu::cleanup_screen)
        .add_systems(OnEnter(AppState::RouteChoice), setup_route_choice)
        .add_systems(
            Update,
            handle_route_buttons.run_if(in_state(AppState::RouteChoice)),
        )
        .add_systems(OnExit(AppState::RouteChoice), menu::cleanup_screen)
        .add_systems(
            OnEnter(AppState::Playing),
            (prepare_new_run, reset_player_for_run).chain(),
        )
        .add_systems(OnEnter(AppState::Paused), setup_pause_overlay)
        .add_systems(OnExit(AppState::Paused), menu::cleanup_screen)
        .add_systems(
            OnEnter(AppState::FinalDecision),
            setup_final_decision_overlay,
        )
        .add_systems(OnExit(AppState::FinalDecision), menu::cleanup_screen)
        .add_systems(OnEnter(AppState::Ending), setup_ending_overlay)
        .add_systems(OnExit(AppState::Ending), menu::cleanup_screen)
        .configure_sets(
            Update,
            (
                GameUpdateSet::World,
                GameUpdateSet::Input,
                GameUpdateSet::Aim,
                GameUpdateSet::Actions,
                GameUpdateSet::Combat,
                GameUpdateSet::Mission,
                GameUpdateSet::Presentation,
                GameUpdateSet::Ui,
            )
                .chain(),
        )
        .add_systems(Update, sync_visible_chunks.in_set(GameUpdateSet::World))
        .add_systems(
            Update,
            (
                update_player_movement.run_if(in_state(AppState::Playing)),
                control_viewer_camera,
                handle_tool_selection.run_if(in_state(AppState::Playing)),
            )
                .chain()
                .in_set(GameUpdateSet::Input),
        )
        .add_systems(
            Update,
            (compute_aim_solution, update_target_highlights)
                .chain()
                .in_set(GameUpdateSet::Aim),
        )
        .add_systems(
            Update,
            (
                handle_voxel_actions.run_if(in_state(AppState::Playing)),
                handle_weapon_crafting.run_if(in_state(AppState::Playing)),
                handle_weapon_fire.run_if(in_state(AppState::Playing)),
            )
                .chain()
                .in_set(GameUpdateSet::Actions),
        )
        .add_systems(
            Update,
            (
                drive_enemy_spawns.run_if(in_state(AppState::Playing)),
                process_damage.run_if(in_state(AppState::Playing)),
                damage_player_blocks.run_if(in_state(AppState::Playing)),
            )
                .chain()
                .in_set(GameUpdateSet::Combat),
        )
        .add_systems(
            Update,
            (
                update_mission.run_if(in_state(AppState::Playing)),
                handle_relay_destroyed.run_if(in_state(AppState::Playing)),
            )
                .chain()
                .in_set(GameUpdateSet::Mission),
        )
        .add_systems(
            Update,
            (
                animate_landmarks,
                update_enemy_visuals,
                apply_outline_materials,
                setup_warden_animation,
                update_warden_animation,
                animate_player,
                sync_player_weapon_visual,
                update_weapon_recoil,
                update_combat_effects,
                sync_player_visibility,
                update_world_mood,
                update_hud,
                capture_qa_screenshot,
            )
                .chain()
                .in_set(GameUpdateSet::Presentation),
        )
        .add_systems(
            Update,
            (
                handle_save_load_input,
                process_save_load,
                toggle_pause,
                pause_when_unfocused,
                handle_overlay_buttons,
                play_ui_clicks,
                update_combat_audio,
                play_game_sounds,
                update_perk_modal,
                update_save_status_ui,
            )
                .chain()
                .in_set(GameUpdateSet::Ui),
        )
        .add_systems(
            FixedUpdate,
            (update_enemy_ai, update_projectiles)
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        .run();
}

fn asset_root() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        "assets".to_string()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if std::path::Path::new("assets").is_dir() {
            return "assets".to_string();
        }
        let beside_executable = env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|parent| parent.join("assets")));
        beside_executable
            .filter(|candidate| candidate.is_dir())
            .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"))
            .to_string_lossy()
            .into_owned()
    }
}
