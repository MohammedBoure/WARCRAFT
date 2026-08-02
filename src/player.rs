use std::time::Duration;

use astra_voxel_world::prelude::*;
use bevy::prelude::*;

use crate::gameplay::RunLifecycle;
use crate::state::*;

const PLAYER_HALF_HEIGHT: f32 = 2.1;
const PLAYER_STEP_HEIGHT: f32 = 2.0;
const GRAVITY: f32 = 25.0;
const JUMP_SPEED: f32 = 10.0;

#[derive(Component)]
pub struct PlayerTag;

const WARDEN_MODEL: &str = "models/kenney-blocky/warden.glb";

#[derive(Resource)]
pub struct WardenAnimations {
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    sprint: AnimationNodeIndex,
    graph: Handle<AnimationGraph>,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum WardenAnimationMode {
    Idle,
    Walk,
    Sprint,
}

#[derive(Component)]
pub struct WardenLamp;

#[derive(Resource, Debug)]
pub struct PlayerState {
    pub speed: f32,
    pub vertical_speed: f32,
    pub grounded: bool,
    pub current_y: f32,
    pub moving: bool,
    safe_timer: f32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            speed: 18.0,
            vertical_speed: 0.0,
            grounded: false,
            current_y: 80.0,
            moving: false,
            safe_timer: 0.0,
        }
    }
}

pub fn spawn_player_character(
    world: Res<VoxelViewerWorld>,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut session: ResMut<GameSession>,
    mut player_state: ResMut<PlayerState>,
    mut commands: Commands,
) {
    let surface = sample_voxel_column(world.settings, 0, 0).height as f32 * HEIGHT_SCALE;
    let spawn = Vec3::new(
        BLOCK_SIZE * 0.5,
        surface + PLAYER_HALF_HEIGHT,
        BLOCK_SIZE * 0.5,
    );
    session.safe_position = spawn;
    player_state.current_y = spawn.y;

    let (graph, nodes) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(WARDEN_MODEL)),
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(WARDEN_MODEL)),
        asset_server.load(GltfAssetLabel::Animation(3).from_asset(WARDEN_MODEL)),
    ]);
    commands.insert_resource(WardenAnimations {
        idle: nodes[0],
        walk: nodes[1],
        sprint: nodes[2],
        graph: animation_graphs.add(graph),
    });

    commands
        .spawn((
            Name::new("Last Warden"),
            PlayerTag,
            Transform::from_translation(spawn),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Kenney Blocky Warden"),
                SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(WARDEN_MODEL))),
                Transform {
                    translation: Vec3::new(0.0, -PLAYER_HALF_HEIGHT, 0.0),
                    rotation: Quat::from_rotation_y(std::f32::consts::PI),
                    scale: Vec3::splat(2.15),
                },
            ));
            parent.spawn((
                Name::new("Stability Lamp"),
                SpotLight {
                    color: Color::srgb(0.32, 0.82, 0.92),
                    intensity: 32_000.0,
                    range: 34.0,
                    inner_angle: 0.20,
                    outer_angle: 0.62,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_xyz(0.0, 1.55, -0.65)
                    .looking_at(Vec3::new(0.0, -0.5, -10.0), Vec3::Y),
                WardenLamp,
            ));
        });
}

pub fn setup_warden_animation(
    mut commands: Commands,
    animations: Res<WardenAnimations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, animations.idle, Duration::ZERO)
            .repeat();
        commands.entity(entity).insert((
            AnimationGraphHandle(animations.graph.clone()),
            transitions,
            WardenAnimationMode::Idle,
        ));
    }
}

pub fn update_warden_animation(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_state: Res<PlayerState>,
    animations: Res<WardenAnimations>,
    mut players: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &mut WardenAnimationMode,
    )>,
) {
    let desired = if !player_state.moving {
        WardenAnimationMode::Idle
    } else if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        WardenAnimationMode::Sprint
    } else {
        WardenAnimationMode::Walk
    };
    let node = match desired {
        WardenAnimationMode::Idle => animations.idle,
        WardenAnimationMode::Walk => animations.walk,
        WardenAnimationMode::Sprint => animations.sprint,
    };
    for (mut player, mut transitions, mut mode) in &mut players {
        if *mode == desired {
            continue;
        }
        transitions
            .play(&mut player, node, Duration::from_millis(160))
            .repeat();
        *mode = desired;
    }
}
pub fn reset_player_for_run(
    lifecycle: Res<RunLifecycle>,
    world: Res<VoxelViewerWorld>,
    mut session: ResMut<GameSession>,
    mut player_state: ResMut<PlayerState>,
    mut player: Query<(&mut Transform, &mut Visibility), With<PlayerTag>>,
) {
    if lifecycle.active {
        return;
    }
    let Ok((mut transform, mut visibility)) = player.single_mut() else {
        return;
    };
    let surface = sample_voxel_column(world.settings, 0, 0).height as f32 * HEIGHT_SCALE;
    let spawn = Vec3::new(
        BLOCK_SIZE * 0.5,
        surface + PLAYER_HALF_HEIGHT,
        BLOCK_SIZE * 0.5,
    );
    transform.translation = spawn;
    transform.rotation = Quat::IDENTITY;
    *visibility = Visibility::Visible;
    *player_state = PlayerState {
        current_y: spawn.y,
        ..default()
    };
    session.reset(spawn);
}

pub fn sync_player_visibility(
    state: Res<State<AppState>>,
    mut player: Query<&mut Visibility, With<PlayerTag>>,
) {
    let Ok(mut visibility) = player.single_mut() else {
        return;
    };
    *visibility = if matches!(state.get(), AppState::Loading | AppState::MainMenu) {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
}

pub fn update_player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    world: Res<VoxelViewerWorld>,
    loaded: Res<LoadedVoxelChunks>,
    camera: Res<VoxelViewerCamera>,
    balance: Res<BalanceConfig>,
    mut session: ResMut<GameSession>,
    mut player_state: ResMut<PlayerState>,
    mut criticality_events: MessageWriter<CriticalityChanged>,
    mut player_query: Query<&mut Transform, With<PlayerTag>>,
) {
    let Ok(mut transform) = player_query.single_mut() else {
        return;
    };

    let forward = Vec3::new(-camera.yaw.sin(), 0.0, -camera.yaw.cos());
    let right = Vec3::new(camera.yaw.cos(), 0.0, -camera.yaw.sin());
    let mut movement = Vec3::ZERO;
    if keyboard.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        movement += forward;
    }
    if keyboard.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        movement -= forward;
    }
    if keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        movement += right;
    }
    if keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        movement -= right;
    }

    let speed = player_state.speed
        * if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            1.55
        } else {
            1.0
        };
    player_state.moving = movement.length_squared() > 0.01;

    let current_foot = transform.translation.y - PLAYER_HALF_HEIGHT;
    let mut next = transform.translation;
    if player_state.moving {
        let direction = movement.normalize();
        let candidate = transform.translation + direction * speed * time.delta_secs();
        let world_x = (candidate.x / BLOCK_SIZE).floor() as i64;
        let world_z = (candidate.z / BLOCK_SIZE).floor() as i64;
        let max_y = ((current_foot + PLAYER_STEP_HEIGHT) / HEIGHT_SCALE).ceil() as i32;
        let ground = ground_world_y(&loaded, &world, world_x, world_z, max_y);
        if ground - current_foot <= PLAYER_STEP_HEIGHT {
            next.x = candidate.x;
            next.z = candidate.z;
            if player_state.grounded && ground > current_foot - 0.5 {
                next.y = ground + PLAYER_HALF_HEIGHT;
            }
            transform.rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.z));
        }
    }

    let foot_x = (next.x / BLOCK_SIZE).floor() as i64;
    let foot_z = (next.z / BLOCK_SIZE).floor() as i64;
    let scan_y = ((next.y - PLAYER_HALF_HEIGHT + PLAYER_STEP_HEIGHT) / HEIGHT_SCALE).ceil() as i32;
    let ground = ground_world_y(&loaded, &world, foot_x, foot_z, scan_y);
    let foot = next.y - PLAYER_HALF_HEIGHT;

    if player_state.grounded && keyboard.just_pressed(KeyCode::Space) {
        player_state.vertical_speed = JUMP_SPEED;
        player_state.grounded = false;
    }
    if !player_state.grounded {
        player_state.vertical_speed -= GRAVITY * time.delta_secs();
        next.y += player_state.vertical_speed * time.delta_secs();
    }
    if next.y - PLAYER_HALF_HEIGHT <= ground + 0.12 && player_state.vertical_speed <= 0.0 {
        next.y = ground + PLAYER_HALF_HEIGHT;
        player_state.vertical_speed = 0.0;
        player_state.grounded = true;
    } else if foot > ground + 0.3 {
        player_state.grounded = false;
    }

    player_state.safe_timer += time.delta_secs();
    if player_state.grounded && player_state.safe_timer >= 1.5 {
        session.safe_position = next;
        player_state.safe_timer = 0.0;
    }

    if next.y < 5.0 || next.y < session.safe_position.y - 55.0 {
        next = session.safe_position;
        next.y += 1.0;
        player_state.vertical_speed = 0.0;
        player_state.grounded = false;
        session.add_criticality(balance.fall_risk);
        criticality_events.write(CriticalityChanged(session.criticality));
    }

    transform.translation = next;
    player_state.current_y = next.y;
}

pub fn animate_player(
    session: Res<GameSession>,
    mut lamps: Query<&mut SpotLight, With<WardenLamp>>,
) {
    let (color, intensity) = match session.risk_band() {
        RiskBand::Calm => (Color::srgb(0.32, 0.82, 0.92), 32_000.0),
        RiskBand::Warning => (Color::srgb(0.95, 0.56, 0.16), 36_000.0),
        RiskBand::Critical => (Color::srgb(0.86, 0.18, 0.25), 40_000.0),
        RiskBand::Terminal => (Color::srgb(1.0, 0.05, 0.08), 44_000.0),
    };
    for mut lamp in &mut lamps {
        lamp.color = color;
        lamp.intensity = intensity;
    }
}
fn ground_world_y(
    loaded: &LoadedVoxelChunks,
    world: &VoxelViewerWorld,
    x: i64,
    z: i64,
    max_y: i32,
) -> f32 {
    loaded
        .ground_below(x, z, max_y)
        .map(|y| y as f32 * HEIGHT_SCALE)
        .unwrap_or_else(|| sample_voxel_column(world.settings, x, z).height as f32 * HEIGHT_SCALE)
}
