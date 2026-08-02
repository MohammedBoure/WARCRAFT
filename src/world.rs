use std::cmp::Reverse;
use std::collections::BTreeSet;
use astra_voxel_world::prelude::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use crate::interaction::*;
use crate::player::*;
use crate::state::*;
use crate::ui::*;

pub fn setup_viewer_scene(
    world: Res<VoxelViewerWorld>,
    camera_state: Res<VoxelViewerCamera>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let font_handle: Handle<Font> = asset_server.load("fonts/arabic.ttf");
    let mut camera_transform = Transform::default();
    apply_viewer_camera_transform(world.settings, &camera_state, &mut camera_transform, 75.0);

    commands.spawn((
        Name::new("Voxel Viewer Camera"),
        Camera3d::default(),
        Camera::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 42.0_f32.to_radians(),
            near: 0.1,
            far: 20_000.0,
            ..default()
        }),
        AmbientLight {
            color: Color::srgb(0.60, 0.68, 0.76),
            brightness: 650.0,
            ..default()
        },
        camera_transform,
        VoxelViewerCameraTag,
    ));

    commands.spawn((
        Name::new("Voxel Viewer Sun"),
        DirectionalLight {
            color: Color::srgb(1.0, 0.94, 0.82),
            illuminance: 24_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-280.0, 430.0, 180.0).looking_at(Vec3::ZERO, Vec3::Y),
        VoxelViewerSunTag,
    ));

    commands.spawn((
        Name::new("Voxel Viewer Weather Overlay"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        Pickable::IGNORE,
        ZIndex(0),
        VoxelViewerWeatherOverlay,
    ));

    commands
        .spawn((
            Name::new("Voxel Viewer Generation HUD"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                left: Val::Px(16.0),
                width: Val::Px(430.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.040, 0.052, 0.76)),
            BorderColor::all(Color::srgba(0.28, 0.48, 0.58, 0.55)),
            ZIndex(10),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(""),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.93, 0.95)),
                VoxelViewerHudText,
            ));
            spawn_generation_dialog_button(panel, VoxelGenerationDialogAction::Open, "INPUTS", font_handle.clone());
        });

    spawn_generation_dialog(&mut commands, font_handle);
}

pub fn control_viewer_camera(
    world: Res<VoxelViewerWorld>,
    dialog: Res<VoxelGenerationDialogState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    time: Res<Time>,
    player_state: Res<PlayerState>,
    mut reset_timer: ResMut<MiddleClickResetTimer>,
    mut camera_state: ResMut<VoxelViewerCamera>,
    mut camera_query: Query<&mut Transform, With<VoxelViewerCameraTag>>,
    player_query: Query<&Transform, (With<PlayerTag>, Without<VoxelViewerCameraTag>)>,
) {
    if dialog.open {
        mouse_wheel.clear();
        mouse_motion.clear();
        return;
    }

    // كشف النقر المزدوج بالزر الأوسط لإعادة ضبط زاوية وارتفاع الرؤية للافتراضي
    if mouse.just_pressed(MouseButton::Middle) {
        let current_time = time.elapsed_secs();
        if current_time - reset_timer.last_click_time < 0.35 {
            camera_state.yaw = -0.72;
            camera_state.height = CAMERA_DEFAULT_HEIGHT;
        }
        reset_timer.last_click_time = current_time;
    }

    // تدوير الكاميرا عند الضغط والسحب بالزر الأوسط للماوس
    if mouse.pressed(MouseButton::Middle) {
        for motion in mouse_motion.read() {
            camera_state.yaw += motion.delta.x * 0.005;
        }
    } else {
        mouse_motion.clear();
    }

    // الكاميرا تتبع موقع البطل بالكامل عند حركته
    if let Ok(player_transform) = player_query.single() {
        camera_state.center = Vec2::new(
            player_transform.translation.x / BLOCK_SIZE,
            player_transform.translation.z / BLOCK_SIZE,
        );
    } else {
        let forward = Vec2::new(-camera_state.yaw.sin(), -camera_state.yaw.cos());
        let right = Vec2::new(camera_state.yaw.cos(), -camera_state.yaw.sin());
        let mut movement = Vec2::ZERO;

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

        if movement.length_squared() > 0.0 {
            let speed = CAMERA_MOVE_SPEED
                * if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                    CAMERA_FAST_MULTIPLIER
                } else {
                    1.0
                }
                * (camera_state.height / CAMERA_DEFAULT_HEIGHT).clamp(0.50, 2.80);
            camera_state.center += movement.normalize() * speed * time.delta_secs();
        }
    }

    if keyboard.pressed(KeyCode::KeyQ) {
        camera_state.yaw += CAMERA_ROTATE_SPEED * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::KeyE) {
        camera_state.yaw -= CAMERA_ROTATE_SPEED * time.delta_secs();
    }
    if keyboard.just_pressed(KeyCode::Space) {
        camera_state.height = CAMERA_DEFAULT_HEIGHT;
    }

    let mut scroll = 0.0;
    for event in mouse_wheel.read() {
        scroll += match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.08,
        };
    }
    if scroll.abs() > f32::EPSILON {
        camera_state.height = (camera_state.height * (1.0 - scroll * 0.10))
            .clamp(CAMERA_MIN_HEIGHT, CAMERA_MAX_HEIGHT);
    }

    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };
    apply_viewer_camera_transform(world.settings, &camera_state, &mut transform, player_state.current_y);
}

pub fn apply_viewer_camera_transform(
    settings: VoxelWorldSettings,
    camera: &VoxelViewerCamera,
    transform: &mut Transform,
    target_y: f32,
) {
    let surface_y = voxel_surface_y_at(
        settings,
        camera.center.x,
        camera.center.y,
        VoxelSurfaceMeshStyle::viewer(),
    );
    let focus_y = target_y.min(surface_y);
    let target = Vec3::new(
        camera.center.x * BLOCK_SIZE,
        focus_y + SURFACE_TARGET_Y_OFFSET,
        camera.center.y * BLOCK_SIZE,
    );

    let yaw_rotation = Quat::from_rotation_y(camera.yaw);
    let pitch_rotation = Quat::from_rotation_x(-CAMERA_PITCH);
    let rotation = yaw_rotation * pitch_rotation;

    let local_back = rotation * Vec3::Z;
    let eye = target + local_back * camera.height;

    *transform = Transform::from_translation(eye).with_rotation(rotation);
}

pub fn update_viewer_weather_scene(
    world: Res<VoxelViewerWorld>,
    camera: Res<VoxelViewerCamera>,
    mut weather_state: ResMut<VoxelViewerWeatherState>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient_query: Query<&mut AmbientLight, With<VoxelViewerCameraTag>>,
    mut sun_query: Query<&mut DirectionalLight, With<VoxelViewerSunTag>>,
    mut overlay_query: Query<&mut BackgroundColor, With<VoxelViewerWeatherOverlay>>,
) {
    let column = sample_voxel_column(
        world.settings,
        camera.center.x.round() as i64,
        camera.center.y.round() as i64,
    );
    weather_state.biome = column.biome;
    weather_state.weather = column.weather;

    let scene = viewer_weather_scene(column.weather);
    clear_color.0 = scene.sky_color;

    if let Ok(mut ambient) = ambient_query.single_mut() {
        ambient.color = scene.ambient_color;
        ambient.brightness = scene.ambient_brightness;
    }
    if let Ok(mut sun) = sun_query.single_mut() {
        sun.color = scene.sun_color;
        sun.illuminance = scene.sun_illuminance;
    }
    if let Ok(mut overlay) = overlay_query.single_mut() {
        overlay.0 = scene.overlay_color;
    }
}

pub fn viewer_weather_scene(weather: VoxelWeather) -> ViewerWeatherScene {
    match weather {
        VoxelWeather::Clear => ViewerWeatherScene {
            sky_color: Color::srgb(0.34, 0.43, 0.55),
            ambient_color: Color::srgb(0.60, 0.68, 0.76),
            ambient_brightness: 650.0,
            sun_color: Color::srgb(1.0, 0.94, 0.82),
            sun_illuminance: 24_000.0,
            overlay_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
        },
        VoxelWeather::Cloudy => ViewerWeatherScene {
            sky_color: Color::srgb(0.42, 0.47, 0.51),
            ambient_color: Color::srgb(0.64, 0.68, 0.70),
            ambient_brightness: 560.0,
            sun_color: Color::srgb(0.86, 0.88, 0.84),
            sun_illuminance: 16_000.0,
            overlay_color: Color::srgba(0.50, 0.56, 0.60, 0.045),
        },
        VoxelWeather::Rain => ViewerWeatherScene {
            sky_color: Color::srgb(0.25, 0.33, 0.40),
            ambient_color: Color::srgb(0.46, 0.55, 0.62),
            ambient_brightness: 470.0,
            sun_color: Color::srgb(0.68, 0.78, 0.84),
            sun_illuminance: 11_000.0,
            overlay_color: Color::srgba(0.12, 0.22, 0.32, 0.095),
        },
        VoxelWeather::Storm => ViewerWeatherScene {
            sky_color: Color::srgb(0.13, 0.16, 0.24),
            ambient_color: Color::srgb(0.32, 0.39, 0.52),
            ambient_brightness: 390.0,
            sun_color: Color::srgb(0.48, 0.58, 0.76),
            sun_illuminance: 7_500.0,
            overlay_color: Color::srgba(0.05, 0.07, 0.14, 0.18),
        },
        VoxelWeather::Snow => ViewerWeatherScene {
            sky_color: Color::srgb(0.62, 0.72, 0.78),
            ambient_color: Color::srgb(0.80, 0.88, 0.92),
            ambient_brightness: 690.0,
            sun_color: Color::srgb(0.84, 0.92, 0.96),
            sun_illuminance: 18_000.0,
            overlay_color: Color::srgba(0.76, 0.88, 0.94, 0.075),
        },
        VoxelWeather::DustStorm => ViewerWeatherScene {
            sky_color: Color::srgb(0.58, 0.43, 0.26),
            ambient_color: Color::srgb(0.70, 0.55, 0.36),
            ambient_brightness: 520.0,
            sun_color: Color::srgb(0.98, 0.70, 0.40),
            sun_illuminance: 12_500.0,
            overlay_color: Color::srgba(0.72, 0.50, 0.24, 0.14),
        },
        VoxelWeather::Ashfall => ViewerWeatherScene {
            sky_color: Color::srgb(0.34, 0.28, 0.25),
            ambient_color: Color::srgb(0.58, 0.50, 0.45),
            ambient_brightness: 430.0,
            sun_color: Color::srgb(0.94, 0.48, 0.28),
            sun_illuminance: 10_000.0,
            overlay_color: Color::srgba(0.30, 0.24, 0.20, 0.17),
        },
        VoxelWeather::IonStorm => ViewerWeatherScene {
            sky_color: Color::srgb(0.18, 0.22, 0.39),
            ambient_color: Color::srgb(0.36, 0.58, 0.82),
            ambient_brightness: 520.0,
            sun_color: Color::srgb(0.46, 0.86, 1.0),
            sun_illuminance: 13_000.0,
            overlay_color: Color::srgba(0.08, 0.34, 0.62, 0.12),
        },
    }
}

pub fn sync_visible_chunks(
    world: Res<VoxelViewerWorld>,
    camera: Res<VoxelViewerCamera>,
    edits_res: Option<Res<VoxelWorldEdits>>,
    primary_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
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
    let viewport_aspect = primary_window
        .single()
        .map(window_aspect)
        .unwrap_or(DEFAULT_VIEWPORT_ASPECT);
    let active_radius = active_load_radius(camera.height, viewport_aspect, world.load_radius);
    let signature = ChunkStreamSignature {
        settings: world.settings,
        center: center_coord,
        radius: active_radius,
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

        let chunk = if let Some(ref edits_res) = edits_res {
            generate_edited_voxel_chunk(world.settings, coord, &edits_res.edits)
        } else {
            generate_voxel_chunk(world.settings, coord)
        };

        let mesh_step = chunk_mesh_step(camera.height, center_coord, coord);
        let mesh = voxel_chunk_surface_mesh(world.settings, &chunk, mesh_step);
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
        loaded.chunks.insert(coord, entity);
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
            commands.entity(entity).despawn();
            retired += 1;
        }
    }
}

pub fn shared_terrain_material(
    render_assets: &mut VoxelViewerRenderAssets,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    render_assets
        .terrain_material
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: false,
                perceptual_roughness: 0.85,
                metallic: 0.0,
                ..default()
            })
        })
        .clone()
}

pub fn chunk_stream_priority(center: VoxelChunkCoord, coord: VoxelChunkCoord) -> (i64, i64, i64) {
    let dx = coord.x - center.x;
    let dz = coord.z - center.z;

    (dx * dx + dz * dz, coord.z, coord.x)
}

pub fn chunk_mesh_step(camera_height: f32, center: VoxelChunkCoord, coord: VoxelChunkCoord) -> usize {
    let distance = (coord.x - center.x).abs().max((coord.z - center.z).abs());

    if camera_height >= LOD_LOW_CAMERA_HEIGHT || distance >= 12 {
        4
    } else if camera_height >= LOD_MEDIUM_CAMERA_HEIGHT || distance >= 7 {
        2
    } else {
        1
    }
}

pub fn window_aspect(window: &Window) -> f32 {
    let height = window.resolution.height().max(1.0);

    (window.resolution.width() / height).clamp(1.0, 2.60)
}

pub fn active_load_radius(camera_height: f32, viewport_aspect: f32, max_radius: i64) -> i64 {
    let camera_height = camera_height.clamp(CAMERA_MIN_HEIGHT, CAMERA_MAX_HEIGHT);
    let viewport_aspect = viewport_aspect.clamp(1.0, 2.60);
    let half_fov_tan = (CAMERA_VERTICAL_FOV_RADIANS * 0.5).tan();
    let half_view_height = camera_height * half_fov_tan;
    let half_view_width = half_view_height * viewport_aspect;
    let visible_half_diagonal = Vec2::new(half_view_width, half_view_height).length();
    let ground_offset = camera_height * CAMERA_PITCH.cos();
    let visible_blocks = (visible_half_diagonal + ground_offset) / BLOCK_SIZE;
    let radius =
        (visible_blocks / DEFAULT_CHUNK_SIZE as f32 + LOAD_RADIUS_MARGIN_CHUNKS).ceil() as i64;

    radius.clamp(1, max_radius.clamp(1, LOAD_RADIUS_MAX))
}

pub fn desired_chunk_coords(center: VoxelChunkCoord, radius: i64) -> BTreeSet<VoxelChunkCoord> {
    let mut coords = BTreeSet::new();
    let radius = radius.clamp(1, LOAD_RADIUS_MAX);
    let radius_squared = radius * radius;

    for z in -radius..=radius {
        for x in -radius..=radius {
            if x * x + z * z > radius_squared {
                continue;
            }
            coords.insert(VoxelChunkCoord::new(center.x + x, center.z + z));
        }
    }

    coords
}

pub fn voxel_chunk_surface_mesh(
    settings: VoxelWorldSettings,
    chunk: &VoxelChunk,
    mesh_step: usize,
) -> Mesh {
    astra_voxel_world::prelude::voxel_chunk_surface_mesh(
        settings,
        chunk,
        mesh_step,
        VoxelSurfaceMeshStyle::viewer(),
    )
}

pub fn reload_loaded_chunks(commands: &mut Commands, loaded: &mut LoadedVoxelChunks) {
    for entity in std::mem::take(&mut loaded.chunks).into_values() {
        commands.entity(entity).despawn();
    }
    loaded.desired.clear();
    loaded.pending.clear();
    loaded.retiring.clear();
    loaded.signature = None;
}

pub fn preset_index_for(composition: VoxelWorldComposition) -> Option<usize> {
    VIEWER_PRESETS.iter().position(|preset| {
        VoxelWorldComposition::preset(preset).is_some_and(|candidate| candidate == composition)
    })
}

pub fn forced_biome_index_for(composition: VoxelWorldComposition) -> Option<usize> {
    if (composition.biome_weights.total() - 1.0).abs() > f64::EPSILON {
        return None;
    }

    VoxelBiome::ALL
        .iter()
        .position(|biome| composition.biome_weights.get(*biome) == 1.0)
}

pub fn forced_weather_index_for(composition: VoxelWorldComposition) -> Option<usize> {
    if (composition.weather_weights.total() - 1.0).abs() > f64::EPSILON {
        return None;
    }

    VoxelWeather::ALL
        .iter()
        .position(|weather| composition.weather_weights.get(*weather) == 1.0)
}

pub fn floor_div(a: i64, b: i64) -> i64 {
    let d = a / b;
    let r = a % b;
    if r != 0 && ((a < 0) != (b < 0)) {
        d - 1
    } else {
        d
    }
}
