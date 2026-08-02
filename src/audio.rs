use bevy::audio::{SpatialScale, Volume};
use bevy::prelude::*;

use crate::state::*;

#[derive(Resource)]
pub struct GameAudio {
    ambient: Handle<AudioSource>,
    risk_pulse: Handle<AudioSource>,
    risk_urgent: Handle<AudioSource>,
    mine: Handle<AudioSource>,
    build: Handle<AudioSource>,
    crystal: Handle<AudioSource>,
    warning: Handle<AudioSource>,
    collapse: Handle<AudioSource>,
    success: Handle<AudioSource>,
    failure: Handle<AudioSource>,
    click: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct AudioRiskState(pub RiskBand);

impl Default for AudioRiskState {
    fn default() -> Self {
        Self(RiskBand::Calm)
    }
}

#[derive(Component, Clone, Copy)]
pub enum AdaptiveAudioLayer {
    Calm,
    Pulse,
    Urgent,
}

pub fn setup_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    preferences: Res<GamePreferences>,
) {
    let audio = GameAudio {
        ambient: asset_server.load("audio/ambient.ogg"),
        risk_pulse: asset_server.load("audio/risk_pulse.ogg"),
        risk_urgent: asset_server.load("audio/risk_urgent.ogg"),
        mine: asset_server.load("audio/mine.ogg"),
        build: asset_server.load("audio/build.ogg"),
        crystal: asset_server.load("audio/crystal.ogg"),
        warning: asset_server.load("audio/warning.ogg"),
        collapse: asset_server.load("audio/collapse.ogg"),
        success: asset_server.load("audio/success.ogg"),
        failure: asset_server.load("audio/failure.ogg"),
        click: asset_server.load("audio/click.ogg"),
    };
    for (source, layer, volume) in [
        (
            audio.ambient.clone(),
            AdaptiveAudioLayer::Calm,
            preferences.master_volume * 0.14,
        ),
        (audio.risk_pulse.clone(), AdaptiveAudioLayer::Pulse, 0.0),
        (audio.risk_urgent.clone(), AdaptiveAudioLayer::Urgent, 0.0),
    ] {
        commands.spawn((
            AudioPlayer::new(source),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
            layer,
        ));
    }
    commands.insert_resource(audio);
}

pub fn play_ui_clicks(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut commands: Commands,
) {
    let Some(audio) = audio else {
        return;
    };
    if buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        play_one_shot(
            &mut commands,
            audio.click.clone(),
            preferences.master_volume * 0.65,
        );
    }
}

pub fn play_voxel_actions(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut events: MessageReader<VoxelActionSound>,
    mut variation: Local<u32>,
    mut commands: Commands,
) {
    let Some(audio) = audio else {
        return;
    };
    for event in events.read() {
        let handle = match event {
            VoxelActionSound::Mine => audio.mine.clone(),
            VoxelActionSound::Build => audio.build.clone(),
        };
        let speed = match event {
            VoxelActionSound::Mine => 0.94 + (*variation % 5) as f32 * 0.025,
            VoxelActionSound::Build => 0.97 + (*variation % 3) as f32 * 0.025,
        };
        *variation = variation.wrapping_add(1);
        play_one_shot_varied(
            &mut commands,
            handle,
            preferences.master_volume * 0.68,
            speed,
        );
    }
}

pub fn play_crystal_audio(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut events: MessageReader<CrystalCollected>,
    mut commands: Commands,
) {
    let Some(audio) = audio else {
        return;
    };
    for event in events.read() {
        let volume = (preferences.master_volume * (0.76 + f32::from(event.0) * 0.03)).min(1.0);
        play_one_shot(&mut commands, audio.crystal.clone(), volume);
    }
}

pub fn play_choice_audio(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut events: MessageReader<ChoiceCommitted>,
    mut commands: Commands,
) {
    let Some(audio) = audio else {
        return;
    };
    for event in events.read() {
        let urgency = if event.0 == CriticalChoice::Stabilize {
            0.92
        } else {
            0.82
        };
        play_one_shot(
            &mut commands,
            audio.warning.clone(),
            preferences.master_volume * urgency,
        );
    }
}

pub fn play_collapse_audio(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut events: MessageReader<CollapseTriggered>,
    mut commands: Commands,
) {
    let Some(audio) = audio else {
        return;
    };
    for event in events.read() {
        let distance_mix = (event.0.length() / 800.0).clamp(0.0, 0.12);
        play_spatial_one_shot(
            &mut commands,
            audio.collapse.clone(),
            preferences.master_volume * (0.88 - distance_mix),
            event.0,
        );
    }
}

pub fn play_criticality_audio(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut risk_state: ResMut<AudioRiskState>,
    mut events: MessageReader<CriticalityChanged>,
    mut commands: Commands,
) {
    let Some(audio) = audio else {
        return;
    };
    for event in events.read() {
        let band = RiskBand::from_value(event.0);
        if band != risk_state.0 {
            risk_state.0 = band;
            if matches!(band, RiskBand::Critical | RiskBand::Terminal) {
                play_one_shot(
                    &mut commands,
                    audio.warning.clone(),
                    preferences.master_volume * 0.85,
                );
            }
        }
    }
}

pub fn play_finish_audio(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut events: MessageReader<RunFinished>,
    mut commands: Commands,
) {
    let Some(audio) = audio else {
        return;
    };
    for event in events.read() {
        let handle = match event.0 {
            RunOutcome::PeopleSaved | RunOutcome::WorldSaved => audio.success.clone(),
            RunOutcome::Collapse | RunOutcome::None => audio.failure.clone(),
        };
        play_one_shot(&mut commands, handle, preferences.master_volume * 0.90);
    }
}

pub fn update_ambient_audio(
    preferences: Res<GamePreferences>,
    state: Res<State<AppState>>,
    session: Res<GameSession>,
    mut layers: Query<(&AdaptiveAudioLayer, &mut PlaybackSettings)>,
) {
    let risk = (session.criticality / 100.0).clamp(0.0, 1.0);
    let active_mix = if matches!(state.get(), AppState::Playing) {
        1.0
    } else if matches!(state.get(), AppState::Decision | AppState::Ending) {
        0.72
    } else {
        0.52
    };
    let calm = 1.0 - smoothstep(0.32, 0.72, risk);
    let pulse = smoothstep(0.30, 0.66, risk) * (1.0 - smoothstep(0.82, 0.98, risk));
    let urgent = smoothstep(0.70, 0.96, risk);

    for (layer, mut settings) in &mut layers {
        let layer_volume = match layer {
            AdaptiveAudioLayer::Calm => calm * 0.16,
            AdaptiveAudioLayer::Pulse => pulse * 0.17,
            AdaptiveAudioLayer::Urgent => urgent * 0.20,
        };
        settings.volume = Volume::Linear(preferences.master_volume * active_mix * layer_volume);
        settings.speed = 1.0;
    }
}

fn smoothstep(start: f32, end: f32, value: f32) -> f32 {
    let t = ((value - start) / (end - start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn play_one_shot(commands: &mut Commands, source: Handle<AudioSource>, volume: f32) {
    play_one_shot_varied(commands, source, volume, 1.0);
}

fn play_one_shot_varied(
    commands: &mut Commands,
    source: Handle<AudioSource>,
    volume: f32,
    speed: f32,
) {
    commands.spawn((
        AudioPlayer::new(source),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(volume.clamp(0.0, 1.0)))
            .with_speed(speed.clamp(0.75, 1.25)),
    ));
}

fn play_spatial_one_shot(
    commands: &mut Commands,
    source: Handle<AudioSource>,
    volume: f32,
    position: Vec3,
) {
    commands.spawn((
        AudioPlayer::new(source),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(volume.clamp(0.0, 1.0)))
            .with_spatial(true)
            .with_spatial_scale(SpatialScale::new(0.065)),
        Transform::from_translation(position),
    ));
}
