use std::time::Duration;

use astra_voxel_world::prelude::*;
use bevy::prelude::*;

use crate::state::*;

const PLAYER_HALF_HEIGHT: f32 = 2.1;
const PLAYER_STEP_HEIGHT: f32 = 2.0;
const WALK_SPEED: f32 = 22.0;
const SPRINT_MULTIPLIER: f32 = 1.58;
const GROUND_ACCELERATION: f32 = 92.0;
const GROUND_BRAKING: f32 = 112.0;
const AIR_ACCELERATION: f32 = 34.0;
const RISE_GRAVITY: f32 = 27.0;
const FALL_GRAVITY: f32 = 39.0;
const MAX_FALL_SPEED: f32 = 48.0;
const JUMP_SPEED: f32 = 12.4;
const COYOTE_SECONDS: f32 = 0.14;
const JUMP_BUFFER_SECONDS: f32 = 0.15;
const TURN_RESPONSE: f32 = 15.0;

#[derive(Component)]
pub struct PlayerTag;

#[derive(Component)]
pub struct PlayerModelRoot;

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
    pub horizontal_velocity: Vec3,
    pub vertical_speed: f32,
    pub grounded: bool,
    pub current_y: f32,
    pub moving: bool,
    coyote_timer: f32,
    jump_buffer_timer: f32,
    safe_timer: f32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            speed: WALK_SPEED,
            horizontal_velocity: Vec3::ZERO,
            vertical_speed: 0.0,
            grounded: false,
            current_y: 80.0,
            moving: false,
            coyote_timer: 0.0,
            jump_buffer_timer: 0.0,
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
                PlayerModelRoot,
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
            .repeat()
            .set_speed(0.86);
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
        let playback_speed = match desired {
            WardenAnimationMode::Idle => 0.86,
            WardenAnimationMode::Walk => 1.18,
            WardenAnimationMode::Sprint => 1.42,
        };
        transitions
            .play(&mut player, node, Duration::from_millis(145))
            .repeat()
            .set_speed(playback_speed);
        *mode = desired;
    }
}
pub fn reset_player_for_run(
    world: Res<VoxelViewerWorld>,
    mut player_state: ResMut<PlayerState>,
    mut player: Query<(&mut Transform, &mut Visibility), With<PlayerTag>>,
) {
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
}

pub fn sync_player_visibility(
    state: Res<State<AppState>>,
    mut player: Query<&mut Visibility, With<PlayerTag>>,
) {
    let Ok(mut visibility) = player.single_mut() else {
        return;
    };
    *visibility = if matches!(
        state.get(),
        AppState::Loading | AppState::MainMenu | AppState::RouteChoice
    ) {
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
    mut session: ResMut<GameSession>,
    mut player_state: ResMut<PlayerState>,
    mut player_query: Query<&mut Transform, With<PlayerTag>>,
) {
    let Ok(mut transform) = player_query.single_mut() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);

    let forward = Vec3::new(-camera.yaw.sin(), 0.0, -camera.yaw.cos());
    let right = Vec3::new(camera.yaw.cos(), 0.0, -camera.yaw.sin());
    let mut input = Vec3::ZERO;
    if keyboard.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        input += forward;
    }
    if keyboard.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        input -= forward;
    }
    if keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        input += right;
    }
    if keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        input -= right;
    }
    let input_direction = input.normalize_or_zero();
    let sprinting = input_direction != Vec3::ZERO
        && keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let target_speed = player_state.speed * if sprinting { SPRINT_MULTIPLIER } else { 1.0 };
    let desired_velocity = input_direction * target_speed;
    let acceleration = if player_state.grounded {
        if input_direction == Vec3::ZERO {
            GROUND_BRAKING
        } else {
            GROUND_ACCELERATION
        }
    } else {
        AIR_ACCELERATION
    };
    player_state.horizontal_velocity = move_towards(
        player_state.horizontal_velocity,
        desired_velocity,
        acceleration * dt,
    );
    player_state.horizontal_velocity.y = 0.0;
    player_state.moving = player_state.horizontal_velocity.length_squared() > 0.16;

    if keyboard.just_pressed(KeyCode::Space) {
        player_state.jump_buffer_timer = JUMP_BUFFER_SECONDS;
    } else {
        player_state.jump_buffer_timer = (player_state.jump_buffer_timer - dt).max(0.0);
    }
    if player_state.grounded {
        player_state.coyote_timer = COYOTE_SECONDS;
    } else {
        player_state.coyote_timer = (player_state.coyote_timer - dt).max(0.0);
    }
    if player_state.jump_buffer_timer > 0.0 && player_state.coyote_timer > 0.0 {
        player_state.vertical_speed = JUMP_SPEED;
        player_state.grounded = false;
        player_state.coyote_timer = 0.0;
        player_state.jump_buffer_timer = 0.0;
    }
    if keyboard.just_released(KeyCode::Space) && player_state.vertical_speed > 0.0 {
        player_state.vertical_speed *= 0.52;
    }

    let current_foot = transform.translation.y - PLAYER_HALF_HEIGHT;
    let mut next = transform.translation;
    let horizontal_step = player_state.horizontal_velocity * dt;
    let scan_y = ((current_foot + PLAYER_STEP_HEIGHT) / HEIGHT_SCALE).ceil() as i32;

    if horizontal_step.x.abs() > f32::EPSILON {
        let candidate_x = next.x + horizontal_step.x;
        let cell_x = (candidate_x / BLOCK_SIZE).floor() as i64;
        let cell_z = (next.z / BLOCK_SIZE).floor() as i64;
        let ground = ground_world_y(&loaded, &world, cell_x, cell_z, scan_y);
        if ground - current_foot <= PLAYER_STEP_HEIGHT {
            next.x = candidate_x;
            if player_state.grounded && ground >= current_foot - 0.48 {
                next.y = ground + PLAYER_HALF_HEIGHT;
            }
        } else {
            player_state.horizontal_velocity.x = 0.0;
        }
    }
    if horizontal_step.z.abs() > f32::EPSILON {
        let candidate_z = next.z + horizontal_step.z;
        let cell_x = (next.x / BLOCK_SIZE).floor() as i64;
        let cell_z = (candidate_z / BLOCK_SIZE).floor() as i64;
        let ground = ground_world_y(&loaded, &world, cell_x, cell_z, scan_y);
        if ground - current_foot <= PLAYER_STEP_HEIGHT {
            next.z = candidate_z;
            if player_state.grounded && ground >= current_foot - 0.48 {
                next.y = ground + PLAYER_HALF_HEIGHT;
            }
        } else {
            player_state.horizontal_velocity.z = 0.0;
        }
    }

    let foot_x = (next.x / BLOCK_SIZE).floor() as i64;
    let foot_z = (next.z / BLOCK_SIZE).floor() as i64;
    let vertical_scan =
        ((next.y - PLAYER_HALF_HEIGHT + PLAYER_STEP_HEIGHT) / HEIGHT_SCALE).ceil() as i32;
    let ground = ground_world_y(&loaded, &world, foot_x, foot_z, vertical_scan);
    let foot_before_fall = next.y - PLAYER_HALF_HEIGHT;
    if player_state.grounded && foot_before_fall > ground + 0.30 {
        player_state.grounded = false;
    }

    if !player_state.grounded {
        let gravity = if player_state.vertical_speed > 0.0 && keyboard.pressed(KeyCode::Space) {
            RISE_GRAVITY
        } else {
            FALL_GRAVITY
        };
        player_state.vertical_speed =
            (player_state.vertical_speed - gravity * dt).max(-MAX_FALL_SPEED);
        next.y += player_state.vertical_speed * dt;
    }

    let foot_after_fall = next.y - PLAYER_HALF_HEIGHT;
    if foot_after_fall <= ground + 0.14 && player_state.vertical_speed <= 0.0 {
        next.y = ground + PLAYER_HALF_HEIGHT;
        player_state.vertical_speed = 0.0;
        player_state.grounded = true;
    }

    if input_direction != Vec3::ZERO {
        let target_rotation = Quat::from_rotation_y((-input_direction.x).atan2(-input_direction.z));
        let turn_blend = 1.0 - (-TURN_RESPONSE * dt).exp();
        transform.rotation = transform.rotation.slerp(target_rotation, turn_blend);
    }

    player_state.safe_timer += dt;
    if player_state.grounded && player_state.safe_timer >= 1.25 {
        session.safe_position = next;
        player_state.safe_timer = 0.0;
    }

    if next.y < 5.0 || next.y < session.safe_position.y - 55.0 {
        next = session.safe_position;
        next.y += 1.0;
        player_state.horizontal_velocity *= 0.25;
        player_state.vertical_speed = 0.0;
        player_state.grounded = false;
        player_state.coyote_timer = 0.0;
    }

    transform.translation = next;
    player_state.current_y = next.y;
}

fn move_towards(current: Vec3, target: Vec3, max_delta: f32) -> Vec3 {
    let delta = target - current;
    let distance = delta.length();
    if distance <= max_delta || distance <= f32::EPSILON {
        target
    } else {
        current + delta / distance * max_delta
    }
}
pub fn animate_player(mut lamps: Query<&mut SpotLight, With<WardenLamp>>) {
    for mut lamp in &mut lamps {
        lamp.color = Color::srgb(1.0, 0.95, 0.85);
        lamp.intensity = 35_000.0;
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
