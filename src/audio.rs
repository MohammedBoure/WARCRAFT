use bevy::audio::Volume;
use bevy::prelude::*;

use crate::state::*;

#[derive(Resource)]
pub struct GameAudio {
    ambient: Handle<AudioSource>,
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

#[derive(Component)]
pub struct AmbientLoop;

pub fn setup_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    preferences: Res<GamePreferences>,
) {
    let audio = GameAudio {
        ambient: asset_server.load("audio/ambient.ogg"),
        mine: asset_server.load("audio/mine.ogg"),
        build: asset_server.load("audio/build.ogg"),
        crystal: asset_server.load("audio/crystal.ogg"),
        warning: asset_server.load("audio/warning.ogg"),
        collapse: asset_server.load("audio/collapse.ogg"),
        success: asset_server.load("audio/success.ogg"),
        failure: asset_server.load("audio/failure.ogg"),
        click: asset_server.load("audio/click.ogg"),
    };
    commands.spawn((
        AudioPlayer::new(audio.ambient.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(preferences.master_volume * 0.22)),
        AmbientLoop,
    ));
    commands.insert_resource(audio);
}

pub fn play_ui_clicks(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut commands: Commands,
) {
    let Some(audio) = audio else { return; };
    if buttons.iter().any(|interaction| *interaction == Interaction::Pressed) {
        play_one_shot(&mut commands, audio.click.clone(), preferences.master_volume * 0.65);
    }
}

pub fn play_voxel_actions(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut events: MessageReader<VoxelActionSound>,
    mut commands: Commands,
) {
    let Some(audio) = audio else { return; };
    for event in events.read() {
        let handle = match event {
            VoxelActionSound::Mine => audio.mine.clone(),
            VoxelActionSound::Build => audio.build.clone(),
        };
        play_one_shot(&mut commands, handle, preferences.master_volume * 0.72);
    }
}

pub fn play_crystal_audio(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut events: MessageReader<CrystalCollected>,
    mut commands: Commands,
) {
    let Some(audio) = audio else { return; };
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
    let Some(audio) = audio else { return; };
    for event in events.read() {
        let urgency = if event.0 == CriticalChoice::Stabilize { 0.92 } else { 0.82 };
        play_one_shot(&mut commands, audio.warning.clone(), preferences.master_volume * urgency);
    }
}

pub fn play_collapse_audio(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut events: MessageReader<CollapseTriggered>,
    mut commands: Commands,
) {
    let Some(audio) = audio else { return; };
    for _ in events.read() {
        play_one_shot(&mut commands, audio.collapse.clone(), preferences.master_volume * 0.95);
    }
}

pub fn play_criticality_audio(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut risk_state: ResMut<AudioRiskState>,
    session: Res<GameSession>,
    mut commands: Commands,
) {
    let Some(audio) = audio else { return; };
    let band = session.risk_band();
    if band != risk_state.0 {
        risk_state.0 = band;
        if matches!(band, RiskBand::Critical | RiskBand::Terminal) {
            play_one_shot(&mut commands, audio.warning.clone(), preferences.master_volume * 0.85);
        }
    }
}

pub fn play_finish_audio(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut events: MessageReader<RunFinished>,
    mut commands: Commands,
) {
    let Some(audio) = audio else { return; };
    for event in events.read() {
        let handle = match event.0 {
            RunOutcome::PeopleSaved | RunOutcome::WorldSaved => audio.success.clone(),
            RunOutcome::Collapse | RunOutcome::None => audio.failure.clone(),
        };
        play_one_shot(&mut commands, handle, preferences.master_volume * 0.90);
    }
}

pub fn update_ambient_audio(
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    session: Res<GameSession>,
    mut ambient_query: Query<&mut PlaybackSettings, With<AmbientLoop>>,
) {
    let _ = audio;
    let Ok(mut settings) = ambient_query.get_single_mut() else { return; };
    let intensity = (session.criticality / 100.0).clamp(0.0, 1.0);
    settings.volume = Volume::Linear(preferences.master_volume * (0.18 + intensity * 0.16));
}

fn play_one_shot(commands: &mut Commands, source: Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioPlayer::new(source),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume.clamp(0.0, 1.0))),
    ));
}