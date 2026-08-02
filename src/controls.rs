use astra_voxel_world::prelude::*;
use bevy::prelude::*;
use crate::state::*;
use crate::world::*;

pub fn control_world_generation(
    keyboard: Res<ButtonInput<KeyCode>>,
    dialog: Res<VoxelGenerationDialogState>,
    mut world: ResMut<VoxelViewerWorld>,
    mut controls: ResMut<VoxelViewerLiveControls>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut commands: Commands,
) {
    if dialog.open {
        return;
    }

    let action = if keyboard.just_pressed(KeyCode::KeyP) {
        Some(LiveControlAction::NextPreset)
    } else if keyboard.just_pressed(KeyCode::KeyB) {
        Some(LiveControlAction::NextBiome)
    } else if keyboard.just_pressed(KeyCode::KeyT) {
        Some(LiveControlAction::NextWeather)
    } else if keyboard.just_pressed(KeyCode::BracketRight) {
        Some(LiveControlAction::IncreaseCrystalRatio)
    } else if keyboard.just_pressed(KeyCode::BracketLeft) {
        Some(LiveControlAction::DecreaseCrystalRatio)
    } else if keyboard.just_pressed(KeyCode::KeyM) {
        Some(LiveControlAction::NextNumericInput)
    } else if keyboard.just_pressed(KeyCode::Equal) {
        Some(LiveControlAction::IncreaseNumericInput)
    } else if keyboard.just_pressed(KeyCode::Minus) {
        Some(LiveControlAction::DecreaseNumericInput)
    } else if keyboard.just_pressed(KeyCode::KeyN) {
        Some(LiveControlAction::NextSeed)
    } else if keyboard.just_pressed(KeyCode::Digit0) {
        Some(LiveControlAction::ResetGenerationSettings)
    } else {
        None
    };

    let Some(action) = action else {
        return;
    };

    if apply_live_control_action(action, &mut controls, &mut world.settings) {
        reload_loaded_chunks(&mut commands, &mut loaded);
    }
}

pub fn apply_live_control_action(
    action: LiveControlAction,
    controls: &mut VoxelViewerLiveControls,
    settings: &mut VoxelWorldSettings,
) -> bool {
    match action {
        LiveControlAction::NextPreset => {
            controls.preset_index = (controls.preset_index + 1) % VIEWER_PRESETS.len();
            let preset = VIEWER_PRESETS[controls.preset_index];
            settings.composition =
                VoxelWorldComposition::preset(preset).expect("viewer preset should exist");
            controls.forced_biome_index = None;
            controls.forced_weather_index = None;
            controls.crystal_ratio = settings.composition.resource_ratios.crystal;
            controls.last_change = format!("preset {preset}");
        }
        LiveControlAction::NextBiome => {
            let index = controls
                .forced_biome_index
                .map(|index| (index + 1) % VoxelBiome::ALL.len())
                .unwrap_or(0);
            let biome = VoxelBiome::ALL[index];
            settings.composition.force_biome(biome);
            controls.forced_biome_index = Some(index);
            controls.last_change = format!("forced biome {}", biome.name());
        }
        LiveControlAction::NextWeather => {
            let index = controls
                .forced_weather_index
                .map(|index| (index + 1) % VoxelWeather::ALL.len())
                .unwrap_or(0);
            let weather = VoxelWeather::ALL[index];
            settings.composition.force_weather(weather);
            controls.forced_weather_index = Some(index);
            controls.last_change = format!("forced weather {}", weather.name());
        }
        LiveControlAction::IncreaseCrystalRatio => {
            controls.crystal_ratio = (controls.crystal_ratio + LIVE_CONTROL_RESOURCE_STEP)
                .clamp(0.0, LIVE_CONTROL_MAX_CRYSTAL_RATIO);
            settings
                .composition
                .resource_ratios
                .set_named("crystal", controls.crystal_ratio);
            controls.last_change = format!("crystal ratio {:.1}", controls.crystal_ratio);
        }
        LiveControlAction::DecreaseCrystalRatio => {
            controls.crystal_ratio = (controls.crystal_ratio - LIVE_CONTROL_RESOURCE_STEP)
                .clamp(0.0, LIVE_CONTROL_MAX_CRYSTAL_RATIO);
            settings
                .composition
                .resource_ratios
                .set_named("crystal", controls.crystal_ratio);
            controls.last_change = format!("crystal ratio {:.1}", controls.crystal_ratio);
        }
        LiveControlAction::NextNumericInput => {
            controls.numeric_input_index =
                (controls.numeric_input_index + 1) % LiveNumericInput::ALL.len();
            let input = LiveNumericInput::ALL[controls.numeric_input_index];
            controls.last_change = format!("selected {}", input.label());
        }
        LiveControlAction::IncreaseNumericInput => {
            let input = LiveNumericInput::ALL[controls.numeric_input_index];
            input.apply_delta(settings, 1);
            controls.last_change = format!(
                "{} {}",
                input.label(),
                input.value_text(settings.sanitized())
            );
        }
        LiveControlAction::DecreaseNumericInput => {
            let input = LiveNumericInput::ALL[controls.numeric_input_index];
            input.apply_delta(settings, -1);
            controls.last_change = format!(
                "{} {}",
                input.label(),
                input.value_text(settings.sanitized())
            );
        }
        LiveControlAction::NextSeed => {
            settings.seed = settings.seed.wrapping_add(LIVE_CONTROL_SEED_STEP);
            controls.last_change = format!("seed 0x{:016X}", settings.seed);
        }
        LiveControlAction::ResetGenerationSettings => {
            let seed = settings.seed;
            *settings = VoxelWorldSettings {
                seed,
                ..VoxelWorldSettings::default()
            };
            *controls = VoxelViewerLiveControls::from_composition(settings.composition);
            controls.last_change = "reset generation settings".to_string();
        }
    }

    *settings = settings.sanitized();
    true
}
