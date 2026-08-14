use std::cmp::Reverse;
use std::collections::BTreeSet;

use astra_voxel_world::prelude::*;
use bevy::{
    anti_alias::smaa::Smaa,
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::{MouseScrollUnit, MouseWheel},
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter},
    prelude::*,
    render::view::{ColorGrading, ColorGradingGlobal, ColorGradingSection, Hdr},
};

use crate::interaction::VoxelWorldEdits;
use crate::player::{PlayerState, PlayerTag};
use crate::state::*;

pub fn setup_viewer_scene(
    world: Res<VoxelViewerWorld>,
    camera_state: Res<VoxelViewerCamera>,
    mut commands: Commands,
) {
    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });

    let surface = sample_voxel_column(world.settings, 0, 0).height as f32 * HEIGHT_SCALE;
    let mut camera_transform = Transform::default();
    apply_viewer_camera_transform(&camera_state, &mut camera_transform, surface + 3.0);

    commands.spawn((
        Name::new("Critical Point Camera"),
        Camera3d::default(),
        SpatialListener::new(1.0),
        Projection::Perspective(PerspectiveProjection {
            fov: 46.0_f32.to_radians(),
            near: 0.1,
            far: 6_000.0,
            ..default()
        }),
        Hdr,
        Msaa::Off,
        Smaa::default(),
        Tonemapping::TonyMcMapface,
        ColorGrading::with_identical_sections(
            ColorGradingGlobal {
                exposure: -0.25,
                post_saturation: 1.10,
                ..default()
            },
            ColorGradingSection {
                saturation: 1.05,
                ..default()
            },
        ),
        Bloom {
            intensity: 0.35,
            low_frequency_boost: 0.50,
            low_frequency_boost_curvature: 0.85,
            high_pass_frequency: 1.0,
            prefilter: BloomPrefilter {
                threshold: 1.15,
                threshold_softness: 0.20,
            },
            composite_mode: BloomCompositeMode::EnergyConserving,
            ..default()
        },
        DistanceFog {
            color: Color::srgba(0.20, 0.25, 0.32, 1.0),
            directional_light_color: Color::srgb(1.0, 0.94, 0.82),
            directional_light_exponent: 32.0,
            falloff: FogFalloff::Linear {
                start: 450.0,
                end: 1_400.0,
            },
        },
        AmbientLight {
            color: Color::srgb(0.55, 0.65, 0.75),
            brightness: 260.0,
            ..default()
        },
        camera_transform,
        VoxelViewerCameraTag,
    ));

    // Primary Sun Directional Light (Astra-Imperium Balanced Sun)
    commands.spawn((
        Name::new("Astra Realistic Sun"),
        DirectionalLight {
            color: Color::srgb(1.0, 0.94, 0.82),
            illuminance: 16_000.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.015,
            shadow_normal_bias: 0.55,
            ..default()
        },
        Transform::from_xyz(-240.0, 380.0, 160.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 80.0,
            maximum_distance: 420.0,
            ..default()
        }
        .build(),
        VoxelViewerSunTag,
    ));

    // Secondary Camera Fill Light (Soft Shadow Illumination)
    commands.spawn((
        Name::new("Astra Camera Fill Light"),
        DirectionalLight {
            color: Color::srgb(0.90, 0.94, 1.0),
            illuminance: 2_400.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(160.0, 220.0, -180.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Name::new("Risk Color Overlay"),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        Pickable::IGNORE,
        ZIndex(1),
        VoxelViewerWeatherOverlay,
    ));
}
pub fn control_viewer_camera(
    app_state: Res<State<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    time: Res<Time>,
    preferences: Res<GamePreferences>,
    session: Res<GameSession>,
    player_state: Res<PlayerState>,
    mut camera_state: ResMut<VoxelViewerCamera>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<VoxelViewerCameraTag>>,
    player_query: Query<&Transform, (With<PlayerTag>, Without<VoxelViewerCameraTag>)>,
) {
    let interactive = matches!(app_state.get(), AppState::Playing);
    if matches!(
        app_state.get(),
        AppState::MainMenu | AppState::RouteChoice | AppState::Loading
    ) {
        camera_state.yaw += time.delta_secs() * 0.055;
    }

    if interactive && mouse.pressed(MouseButton::Middle) {
        for motion in mouse_motion.read() {
            camera_state.yaw += motion.delta.x * 0.0045 * preferences.camera_sensitivity;
        }
    } else {
        mouse_motion.clear();
    }

    let mut scroll = 0.0;
    for event in mouse_wheel.read() {
        scroll += match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.08,
        };
    }

    if interactive {
        if keyboard.pressed(KeyCode::KeyQ) {
            camera_state.yaw += 1.35 * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::KeyE) {
            camera_state.yaw -= 1.35 * time.delta_secs();
        }
        if keyboard.pressed(KeyCode::KeyZ)
            || keyboard.pressed(KeyCode::PageUp)
            || keyboard.pressed(KeyCode::Equal)
        {
            camera_state.height = (camera_state.height - 180.0 * time.delta_secs())
                .clamp(CAMERA_MIN_HEIGHT, CAMERA_MAX_HEIGHT);
        }
        if keyboard.pressed(KeyCode::KeyX)
            || keyboard.pressed(KeyCode::PageDown)
            || keyboard.pressed(KeyCode::Minus)
        {
            camera_state.height = (camera_state.height + 180.0 * time.delta_secs())
                .clamp(CAMERA_MIN_HEIGHT, CAMERA_MAX_HEIGHT);
        }

        let is_builder = session.loadout.selected_tool == ToolSlot::Builder;
        let modifier = keyboard.any_pressed([
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
        ]);
        if scroll.abs() > f32::EPSILON && (!is_builder || modifier) {
            camera_state.height =
                (camera_state.height - scroll * 18.0).clamp(CAMERA_MIN_HEIGHT, CAMERA_MAX_HEIGHT);
        }
    }

    let target = if let Ok(player) = player_query.single() {
        let look_ahead = player_state.horizontal_velocity * 0.16;
        camera_state.center = Vec2::new(
            (player.translation.x + look_ahead.x) / BLOCK_SIZE,
            (player.translation.z + look_ahead.z) / BLOCK_SIZE,
        );
        player.translation + look_ahead * 0.20 + Vec3::Y * 2.0
    } else {
        Vec3::new(
            camera_state.center.x * BLOCK_SIZE,
            80.0,
            camera_state.center.y * BLOCK_SIZE,
        )
    };

    let Ok((mut transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    let mut desired = Transform::default();
    apply_viewer_camera_transform(&camera_state, &mut desired, target.y);
    let smoothing = 1.0 - (-8.5 * time.delta_secs()).exp();
    transform.translation = transform.translation.lerp(desired.translation, smoothing);
    transform.rotation = transform.rotation.slerp(desired.rotation, smoothing);

    if let Projection::Perspective(perspective) = &mut *projection {
        let sprinting =
            player_state.moving && keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let is_weapon = matches!(session.loadout.selected_tool, ToolSlot::Weapon(_));
        let ads_active = interactive && is_weapon && mouse.pressed(MouseButton::Right);
        let target_fov: f32 = if ads_active {
            39.5
        } else if sprinting {
            50.0
        } else {
            46.0
        };
        let target_fov_rad = target_fov.to_radians();
        let fov_blend = 1.0 - (-5.5 * time.delta_secs()).exp();
        perspective.fov += (target_fov_rad - perspective.fov) * fov_blend;
    }

    if camera_state.shake > 0.001 && !preferences.reduced_motion {
        let t = time.elapsed_secs() * 37.0;
        transform.translation +=
            Vec3::new(t.sin(), (t * 1.7).cos(), (t * 0.7).sin()) * camera_state.shake;
        camera_state.shake = (camera_state.shake - time.delta_secs() * 2.2).max(0.0);
    } else {
        camera_state.shake = 0.0;
    }
}

pub fn apply_viewer_camera_transform(
    camera: &VoxelViewerCamera,
    transform: &mut Transform,
    target_y: f32,
) {
    let target = Vec3::new(
        camera.center.x * BLOCK_SIZE,
        target_y + 2.8,
        camera.center.y * BLOCK_SIZE,
    );
    let yaw_rotation = Quat::from_rotation_y(camera.yaw);
    let pitch_rotation = Quat::from_rotation_x(-CAMERA_PITCH);
    let rotation = yaw_rotation * pitch_rotation;
    let eye = target + rotation * Vec3::Z * camera.height;
    *transform = Transform::from_translation(eye).with_rotation(rotation);
}

pub fn update_world_mood(
    session: Res<GameSession>,
    mut clear_color: ResMut<ClearColor>,
    mut camera_query: Query<(&mut AmbientLight, &mut DistanceFog), With<VoxelViewerCameraTag>>,
    mut sun_query: Query<&mut DirectionalLight, With<VoxelViewerSunTag>>,
) {
    let invaded = session.route == PlanetRoute::InvadedPlanet;
    let (sky, ambient_color, ambient_power, sun_color, sun_power) = if invaded {
        (
            Color::srgb(0.09, 0.065, 0.14),
            Color::srgb(0.34, 0.30, 0.46),
            430.0,
            Color::srgb(0.92, 0.66, 0.74),
            11_500.0,
        )
    } else {
        (
            Color::srgb(0.24, 0.32, 0.40),
            Color::srgb(0.52, 0.60, 0.68),
            540.0,
            Color::srgb(1.0, 0.88, 0.70),
            14_000.0,
        )
    };
    clear_color.0 = sky;
    if let Ok((mut ambient, mut fog)) = camera_query.single_mut() {
        ambient.color = ambient_color;
        ambient.brightness = ambient_power;
        fog.color = sky.with_alpha(1.0);
    }
    if let Ok(mut sun) = sun_query.single_mut() {
        sun.color = sun_color;
        sun.illuminance = sun_power;
    }
}

pub fn sync_visible_chunks(
    world: Res<VoxelViewerWorld>,
    camera: Res<VoxelViewerCamera>,
    edits: Res<VoxelWorldEdits>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut render_assets: ResMut<VoxelViewerRenderAssets>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let center_coord = VoxelChunkCoord::new(
        floor_div(camera.center.x.floor() as i64, DEFAULT_CHUNK_SIZE as i64),
        floor_div(camera.center.y.floor() as i64, DEFAULT_CHUNK_SIZE as i64),
    );
    let radius = world.load_radius.clamp(3, LOAD_RADIUS_MAX);
    let signature = ChunkStreamSignature {
        settings: world.settings,
        center: center_coord,
        radius,
    };

    if loaded.signature != Some(signature) {
        refresh_chunk_stream_plan(&mut loaded, signature);
    }
    retire_chunks_outside_plan(&mut commands, &mut loaded);

    let terrain_material = shared_terrain_material(&mut render_assets, &mut materials);
    let mut spawned = 0;
    while spawned < CHUNK_STREAM_BUDGET_PER_FRAME {
        let Some(coord) = loaded.pending.pop_front() else {
            break;
        };
        if loaded.chunks.contains_key(&coord) || !loaded.desired.contains(&coord) {
            continue;
        }
        let chunk = generate_edited_voxel_chunk(world.settings, coord, &edits.edits);
        let distance = (coord.x - center_coord.x)
            .abs()
            .max((coord.z - center_coord.z).abs());
        let mesh_step = if distance >= 6 { 2 } else { 1 };
        let mesh = astra_voxel_world::prelude::voxel_chunk_surface_mesh(
            world.settings,
            &chunk,
            mesh_step,
            VoxelSurfaceMeshStyle::viewer(),
        );
        let entity = commands
            .spawn((
                Name::new(format!("Voxel Chunk {coord}")),
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(terrain_material.clone()),
                Transform::from_xyz(
                    coord.world_x(0) as f32 * BLOCK_SIZE,
                    0.0,
                    coord.world_z(0) as f32 * BLOCK_SIZE,
                ),
                VoxelChunkEntity,
            ))
            .id();
        loaded.voxel_data.insert(coord, chunk);
        loaded.chunks.insert(coord, entity);
        loaded.dirty.remove(&coord);
        spawned += 1;
    }
}

pub fn refresh_chunk_stream_plan(loaded: &mut LoadedVoxelChunks, signature: ChunkStreamSignature) {
    let desired = desired_chunk_coords(signature.center, signature.radius);
    let mut pending = desired
        .iter()
        .filter(|coord| !loaded.chunks.contains_key(coord))
        .copied()
        .collect::<Vec<_>>();
    pending.sort_by_key(|coord| chunk_stream_priority(signature.center, *coord));

    let mut retiring = loaded
        .chunks
        .keys()
        .filter(|coord| !desired.contains(coord))
        .copied()
        .collect::<Vec<_>>();
    retiring.sort_by_key(|coord| Reverse(chunk_stream_priority(signature.center, *coord)));

    loaded.desired = desired;
    loaded.pending = pending.into();
    loaded.retiring = retiring.into();
    loaded.signature = Some(signature);
}

pub fn retire_chunks_outside_plan(commands: &mut Commands, loaded: &mut LoadedVoxelChunks) {
    let mut retired = 0;
    while retired < CHUNK_UNLOAD_BUDGET_PER_FRAME {
        let Some(coord) = loaded.retiring.pop_front() else {
            break;
        };
        if loaded.desired.contains(&coord) {
            continue;
        }
        if let Some(entity) = loaded.chunks.remove(&coord) {
            if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
            loaded.voxel_data.remove(&coord);
            retired += 1;
        }
    }
}

pub fn invalidate_edit(
    commands: &mut Commands,
    loaded: &mut LoadedVoxelChunks,
    center: VoxelBlockPosition,
    radius: i32,
) {
    let size = DEFAULT_CHUNK_SIZE as i64;
    let min_x = floor_div(center.x - i64::from(radius) - 1, size);
    let max_x = floor_div(center.x + i64::from(radius) + 1, size);
    let min_z = floor_div(center.z - i64::from(radius) - 1, size);
    let max_z = floor_div(center.z + i64::from(radius) + 1, size);

    for z in min_z..=max_z {
        for x in min_x..=max_x {
            let coord = VoxelChunkCoord::new(x, z);
            if let Some(entity) = loaded.chunks.remove(&coord) {
                if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
            }
            loaded.voxel_data.remove(&coord);
            loaded.dirty.insert(coord);
            if loaded.desired.contains(&coord) && !loaded.pending.contains(&coord) {
                loaded.pending.push_front(coord);
            }
        }
    }
}

pub fn reload_loaded_chunks(commands: &mut Commands, loaded: &mut LoadedVoxelChunks) {
    for entity in std::mem::take(&mut loaded.chunks).into_values() {
        if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
    }
    loaded.voxel_data.clear();
    loaded.desired.clear();
    loaded.pending.clear();
    loaded.retiring.clear();
    loaded.dirty.clear();
    loaded.signature = None;
}

fn shared_terrain_material(
    render_assets: &mut VoxelViewerRenderAssets,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    render_assets
        .terrain_material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.82, 0.86, 0.84),
                perceptual_roughness: 0.90,
                metallic: 0.0,
                reflectance: 0.18,
                ..default()
            })
        })
        .clone()
}

fn chunk_stream_priority(center: VoxelChunkCoord, coord: VoxelChunkCoord) -> (i64, i64, i64) {
    let dx = coord.x - center.x;
    let dz = coord.z - center.z;
    (dx * dx + dz * dz, coord.z, coord.x)
}

fn desired_chunk_coords(center: VoxelChunkCoord, radius: i64) -> BTreeSet<VoxelChunkCoord> {
    let mut coords = BTreeSet::new();
    let radius = radius.clamp(1, LOAD_RADIUS_MAX);
    let radius_squared = radius * radius;
    for z in -radius..=radius {
        for x in -radius..=radius {
            if x * x + z * z <= radius_squared {
                coords.insert(VoxelChunkCoord::new(center.x + x, center.z + z));
            }
        }
    }
    coords
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_plan_is_circular_and_contains_center() {
        let center = VoxelChunkCoord::new(4, -3);
        let coords = desired_chunk_coords(center, 3);
        assert!(coords.contains(&center));
        assert!(coords.contains(&VoxelChunkCoord::new(7, -3)));
        assert!(!coords.contains(&VoxelChunkCoord::new(7, 0)));
    }

    #[test]
    fn negative_floor_division_maps_chunks_correctly() {
        assert_eq!(floor_div(-1, 16), -1);
        assert_eq!(floor_div(-16, 16), -1);
        assert_eq!(floor_div(-17, 16), -2);
    }
}
