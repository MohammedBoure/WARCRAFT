mod config;
mod controls;
mod state;
mod ui;
mod world;

use std::env;
use bevy::prelude::*;
use crate::controls::*;
use crate::state::*;
use crate::ui::*;
use crate::world::*;

fn main() {
    let options = ViewerOptions::parse(env::args().skip(1)).unwrap_or_else(|error| {
        eprintln!("voxel_world_viewer: {error}");
        eprintln!("{}", ViewerOptions::help_text());
        std::process::exit(2);
    });

    if options.help {
        println!("{}", ViewerOptions::help_text());
        return;
    }

    let initial_composition = options.composition;
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.34, 0.43, 0.55)))
        .insert_resource(VoxelViewerWorld {
            settings: options.generation_settings(),
            load_radius: options.load_radius,
        })
        .insert_resource(VoxelViewerCamera {
            center: Vec2::new(options.start_x as f32, options.start_z as f32),
            yaw: -0.72,
            height: CAMERA_DEFAULT_HEIGHT,
        })
        .insert_resource(VoxelViewerLiveControls::from_composition(
            initial_composition,
        ))
        .init_resource::<VoxelGenerationDialogState>()
        .init_resource::<VoxelViewerWeatherState>()
        .init_resource::<LoadedVoxelChunks>()
        .init_resource::<VoxelViewerRenderAssets>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Astra Voxel World Viewer".to_string(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_viewer_scene)
        .add_systems(
            Update,
            (
                handle_generation_dialog_buttons,
                handle_generation_dialog_keyboard_input,
                control_world_generation,
                control_viewer_camera,
                update_viewer_weather_scene,
                sync_visible_chunks,
                update_generation_dialog_ui,
                update_generation_hud,
            )
                .chain(),
        )
        .run();
}
