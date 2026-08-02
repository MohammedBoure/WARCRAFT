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

#[derive(Component)]
pub struct PlayerLimb {
    phase: f32,
}

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
    mut session: ResMut<GameSession>,
    mut player_state: ResMut<PlayerState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let surface = sample_voxel_column(world.settings, 0, 0).height as f32 * HEIGHT_SCALE;
    let spawn = Vec3::new(0.0, surface + PLAYER_HALF_HEIGHT, 0.0);
    session.safe_position = spawn;
    player_state.current_y = spawn.y;

    let body_mesh = meshes.add(Cuboid::new(2.4, 2.8, 1.7));
    let head_mesh = meshes.add(Cuboid::new(2.0, 1.65, 1.85));
    let limb_mesh = meshes.add(Cuboid::new(0.62, 2.1, 0.62));
    let pack_mesh = meshes.add(Cuboid::new(1.65, 2.0, 0.72));
    let visor_mesh = meshes.add(Cuboid::new(1.48, 0.55, 0.22));

    let suit = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.62, 0.82),
        metallic: 0.22,
        perceptual_roughness: 0.42,
        ..default()
    });
    let dark = materials.add(StandardMaterial {
        base_color: Color::srgb(0.035, 0.09, 0.14),
        metallic: 0.45,
        perceptual_roughness: 0.30,
        ..default()
    });
    let glow = materials.add(StandardMaterial {
        base_color: Color::srgb(0.68, 1.0, 0.94),
        emissive: LinearRgba::rgb(3.8, 7.0, 6.2),
        metallic: 0.15,
        perceptual_roughness: 0.2,
        ..default()
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
                Mesh3d(body_mesh),
                MeshMaterial3d(suit.clone()),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            parent.spawn((
                Mesh3d(head_mesh),
                MeshMaterial3d(dark.clone()),
                Transform::from_xyz(0.0, 2.05, 0.0),
            ));
            parent.spawn((
                Mesh3d(visor_mesh),
                MeshMaterial3d(glow),
                Transform::from_xyz(0.0, 2.12, -1.0),
            ));
            parent.spawn((
                Mesh3d(pack_mesh),
                MeshMaterial3d(dark),
                Transform::from_xyz(0.0, 0.15, 1.05),
            ));
            for (x, phase) in [(-1.48, 0.0), (1.48, std::f32::consts::PI)] {
                parent.spawn((
                    Mesh3d(limb_mesh.clone()),
                    MeshMaterial3d(suit.clone()),
                    Transform::from_xyz(x, -0.25, 0.0),
                    PlayerLimb { phase },
                ));
            }
            parent.spawn((
                SpotLight {
                    color: Color::srgb(0.62, 0.95, 1.0),
                    intensity: 650_000.0,
                    range: 42.0,
                    inner_angle: 0.22,
                    outer_angle: 0.72,
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_xyz(0.0, 2.2, -0.7).looking_at(Vec3::new(0.0, 0.0, -8.0), Vec3::Y),
            ));
        });
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
    let spawn = Vec3::new(0.0, surface + PLAYER_HALF_HEIGHT, 0.0);
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
        let world_x = (candidate.x / BLOCK_SIZE).round() as i64;
        let world_z = (candidate.z / BLOCK_SIZE).round() as i64;
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

    let foot_x = (next.x / BLOCK_SIZE).round() as i64;
    let foot_z = (next.z / BLOCK_SIZE).round() as i64;
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
    time: Res<Time>,
    state: Res<PlayerState>,
    mut limbs: Query<(&PlayerLimb, &mut Transform)>,
) {
    let amount = if state.moving { 0.55 } else { 0.06 };
    for (limb, mut transform) in &mut limbs {
        let swing = (time.elapsed_secs() * 8.0 + limb.phase).sin() * amount;
        transform.rotation = Quat::from_rotation_x(swing);
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
