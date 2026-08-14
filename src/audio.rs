use std::collections::HashMap;

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::state::*;

#[derive(Resource)]
pub struct GameAudio {
    ambient: Handle<AudioSource>,
    mine: Handle<AudioSource>,
    build: Handle<AudioSource>,
    pulse: Handle<AudioSource>,
    plasma: Handle<AudioSource>,
    ion: Handle<AudioSource>,
    hit: Handle<AudioSource>,
    death: Handle<AudioSource>,
    resource: Handle<AudioSource>,
    craft: Handle<AudioSource>,
    warning: Handle<AudioSource>,
    success: Handle<AudioSource>,
    failure: Handle<AudioSource>,
    click: Handle<AudioSource>,
}

#[derive(Resource, Default)]
pub struct AudioDirector {
    cooldowns: HashMap<GameSound, f32>,
    variation: u32,
}

#[derive(Component)]
pub struct AmbientLayer;

#[derive(Component)]
pub struct OneShotAudio;

pub fn setup_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    preferences: Res<GamePreferences>,
) {
    let audio = GameAudio {
        ambient: asset_server.load("audio/ambient.ogg"),
        mine: asset_server.load("audio/mine_professional.ogg"),
        build: asset_server.load("audio/build_professional.ogg"),
        pulse: asset_server.load("audio/pulse_shot.ogg"),
        plasma: asset_server.load("audio/plasma_shot.ogg"),
        ion: asset_server.load("audio/ion_shot.ogg"),
        hit: asset_server.load("audio/enemy_hit.ogg"),
        death: asset_server.load("audio/enemy_death.ogg"),
        resource: asset_server.load("audio/crystal.ogg"),
        craft: asset_server.load("audio/craft_professional.ogg"),
        warning: asset_server.load("audio/warning.ogg"),
        success: asset_server.load("audio/success.ogg"),
        failure: asset_server.load("audio/failure.ogg"),
        click: asset_server.load("audio/ui_click_professional.ogg"),
    };
    commands.spawn((
        AudioPlayer::new(audio.ambient.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(preferences.master_volume * 0.13)),
        AmbientLayer,
    ));
    commands.insert_resource(audio);
}

pub fn play_ui_clicks(
    time: Res<Time>,
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut cooldown: Local<f32>,
    mut commands: Commands,
) {
    *cooldown = (*cooldown - time.delta_secs()).max(0.0);
    let Some(audio) = audio else {
        return;
    };
    if *cooldown == 0.0
        && buttons
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        play_one_shot(
            &mut commands,
            audio.click.clone(),
            preferences.master_volume * 0.48,
            1.0,
        );
        *cooldown = 0.075;
    }
}

pub fn play_game_sounds(
    time: Res<Time>,
    audio: Option<Res<GameAudio>>,
    preferences: Res<GamePreferences>,
    mut director: ResMut<AudioDirector>,
    one_shots: Query<Entity, With<OneShotAudio>>,
    mut events: MessageReader<GameSound>,
    mut commands: Commands,
) {
    let Some(audio) = audio else {
        return;
    };
    for cooldown in director.cooldowns.values_mut() {
        *cooldown = (*cooldown - time.delta_secs()).max(0.0);
    }
    let mut active_one_shots = one_shots.iter().count();
    for sound in events.read() {
        if active_one_shots >= 20 {
            continue;
        }
        let minimum = sound_cooldown(*sound);
        if director.cooldowns.get(sound).copied().unwrap_or(0.0) > 0.0 {
            continue;
        }
        director.cooldowns.insert(*sound, minimum);
        let (source, volume) = match sound {
            GameSound::Mine => (audio.mine.clone(), 0.55),
            GameSound::Build => (audio.build.clone(), 0.52),
            GameSound::PulseShot => (audio.pulse.clone(), 0.46),
            GameSound::PlasmaShot => (audio.plasma.clone(), 0.66),
            GameSound::IonShot => (audio.ion.clone(), 0.60),
            GameSound::EnemyHit => (audio.hit.clone(), 0.36),
            GameSound::EnemyDeath => (audio.death.clone(), 0.52),
            GameSound::PlayerHit => (audio.warning.clone(), 0.48),
            GameSound::Resource => (audio.resource.clone(), 0.58),
            GameSound::Craft => (audio.craft.clone(), 0.62),
            GameSound::Warning => (audio.warning.clone(), 0.64),
            GameSound::Success => (audio.success.clone(), 0.78),
            GameSound::Failure => (audio.failure.clone(), 0.76),
        };
        let speed = 0.96 + (director.variation % 5) as f32 * 0.018;
        director.variation = director.variation.wrapping_add(1);
        play_one_shot(
            &mut commands,
            source,
            preferences.master_volume * volume,
            speed,
        );
        active_one_shots += 1;
    }
}

pub fn update_combat_audio(
    preferences: Res<GamePreferences>,
    state: Res<State<AppState>>,
    session: Res<GameSession>,
    mut ambient: Query<&mut PlaybackSettings, With<AmbientLayer>>,
    mut finished: MessageReader<RunFinished>,
    mut sounds: MessageWriter<GameSound>,
) {
    let active = matches!(state.get(), AppState::Playing);
    let pressure = (session.active_enemies as f32 / 16.0).clamp(0.0, 1.0);
    let route_mix = if session.route == PlanetRoute::InvadedPlanet {
        1.0
    } else {
        0.78
    };
    for mut settings in &mut ambient {
        settings.volume = Volume::Linear(
            preferences.master_volume
                * route_mix
                * if active {
                    0.11 + pressure * 0.035
                } else {
                    0.07
                },
        );
        settings.speed = 0.98 + pressure * 0.035;
    }
    for event in finished.read() {
        sounds.write(if event.0 == RunOutcome::MissionFailed {
            GameSound::Failure
        } else {
            GameSound::Success
        });
    }
}

fn sound_cooldown(sound: GameSound) -> f32 {
    match sound {
        GameSound::PulseShot => 0.11,
        GameSound::EnemyHit => 0.07,
        GameSound::PlayerHit => 0.16,
        GameSound::Mine | GameSound::Build => 0.20,
        GameSound::PlasmaShot | GameSound::IonShot => 0.28,
        GameSound::EnemyDeath => 0.12,
        _ => 0.05,
    }
}

fn play_one_shot(commands: &mut Commands, source: Handle<AudioSource>, volume: f32, speed: f32) {
    commands.spawn((
        AudioPlayer::new(source),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(volume.clamp(0.0, 1.0)))
            .with_speed(speed.clamp(0.85, 1.15)),
        OneShotAudio,
    ));
}
