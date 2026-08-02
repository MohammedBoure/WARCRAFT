use astra_voxel_world::prelude::*;
use bevy::prelude::*;

use crate::interaction::{VoxelWorldEdits, block_world_center};
use crate::player::PlayerTag;
use crate::state::*;
use crate::world::{invalidate_edit, reload_loaded_chunks};

const CRYSTAL_COORDS: [(i64, i64); 3] = [(18, 10), (-24, 18), (24, -29)];
const CORE_COORD: (i64, i64) = (-38, -25);
const INTERACT_RADIUS: f32 = 12.0;

#[derive(Resource, Default)]
pub struct RunLifecycle {
    pub active: bool,
}

#[derive(Resource)]
pub struct CollapseDirector {
    pub timer: Timer,
}

impl Default for CollapseDirector {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(8.0, TimerMode::Once),
        }
    }
}

#[derive(Resource)]
pub struct GameplayVisualAssets {
    crystal_mesh: Handle<Mesh>,
    crystal_material: Handle<StandardMaterial>,
    beacon_mesh: Handle<Mesh>,
    beacon_material: Handle<StandardMaterial>,
    core_mesh: Handle<Mesh>,
    core_material: Handle<StandardMaterial>,
    warning_mesh: Handle<Mesh>,
    warning_material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct RunEntity;
#[derive(Component)]
pub struct CrystalMarker(pub u8);
#[derive(Component)]
pub struct EvacuationBeacon;
#[derive(Component)]
pub struct WorldCore;
#[derive(Component)]
pub struct CollapseWarning {
    pub timer: Timer,
    pub center: VoxelBlockPosition,
}
#[derive(Component)]
pub struct WorldSpark {
    pub origin_y: f32,
    pub phase: f32,
}

pub fn setup_gameplay_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let crystal_mesh = meshes.add(Cuboid::new(2.1, 7.0, 2.1));
    let beacon_mesh = meshes.add(Cylinder::new(5.5, 0.75));
    let core_mesh = meshes.add(Sphere::new(4.8).mesh().ico(4).expect("valid icosphere"));
    let warning_mesh = meshes.add(Cylinder::new(10.0, 0.16));
    let crystal_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.38, 0.96, 0.88),
        emissive: LinearRgba::rgb(1.2, 3.0, 2.4),
        metallic: 0.35,
        perceptual_roughness: 0.18,
        ..default()
    });
    let beacon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.72, 0.92),
        emissive: LinearRgba::rgb(0.35, 1.6, 2.2),
        metallic: 0.5,
        perceptual_roughness: 0.28,
        ..default()
    });
    let core_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.82, 0.12, 0.38),
        emissive: LinearRgba::rgb(3.0, 0.12, 0.55),
        metallic: 0.18,
        perceptual_roughness: 0.16,
        ..default()
    });
    let warning_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.08, 0.04, 0.38),
        emissive: LinearRgba::rgb(2.4, 0.02, 0.01),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.insert_resource(GameplayVisualAssets {
        crystal_mesh,
        crystal_material,
        beacon_mesh,
        beacon_material,
        core_mesh,
        core_material,
        warning_mesh,
        warning_material,
    });
}

pub fn prepare_new_run(
    world: Res<VoxelViewerWorld>,
    assets: Res<GameplayVisualAssets>,
    mut lifecycle: ResMut<RunLifecycle>,
    mut session: ResMut<GameSession>,
    mut edits: ResMut<VoxelWorldEdits>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    old_entities: Query<Entity, With<RunEntity>>,
    mut commands: Commands,
) {
    if lifecycle.active {
        return;
    }
    for entity in &old_entities {
        commands.entity(entity).despawn();
    }
    edits.edits.clear();
    reload_loaded_chunks(&mut commands, &mut loaded);
    lifecycle.active = true;

    for (index, (x, z)) in CRYSTAL_COORDS.into_iter().enumerate() {
        let top = sample_voxel_column(world.settings, x, z).height + 1;
        let block = VoxelBlockPosition::new(x, top, z);
        edits.edits.push(VoxelTerrainEdit::SetBlock {
            position: block,
            block: BlockKind::CrystalOre,
        });
        let marker_y = block_world_center(block).y + 7.0;
        commands.spawn((
            Name::new(format!("Crystal Signal {}", index + 1)),
            Mesh3d(assets.crystal_mesh.clone()),
            MeshMaterial3d(assets.crystal_material.clone()),
            Transform::from_xyz(x as f32 * BLOCK_SIZE, marker_y, z as f32 * BLOCK_SIZE)
                .with_rotation(Quat::from_rotation_z(0.28)),
            PointLight {
                color: Color::srgb(0.38, 1.0, 0.88),
                intensity: 380_000.0,
                range: 36.0,
                shadows_enabled: false,
                ..default()
            },
            CrystalMarker((index + 1) as u8),
            RunEntity,
        ));
    }

    let beacon_top = sample_voxel_column(world.settings, 0, 0).height as f32 * HEIGHT_SCALE;
    commands.spawn((
        Name::new("Evacuation Beacon"),
        Mesh3d(assets.beacon_mesh.clone()),
        MeshMaterial3d(assets.beacon_material.clone()),
        Transform::from_xyz(0.0, beacon_top + 0.5, 0.0),
        PointLight {
            color: Color::srgb(0.20, 0.84, 1.0),
            intensity: 520_000.0,
            range: 52.0,
            shadows_enabled: false,
            ..default()
        },
        EvacuationBeacon,
        RunEntity,
    ));

    let core_top = sample_voxel_column(world.settings, CORE_COORD.0, CORE_COORD.1).height as f32
        * HEIGHT_SCALE;
    commands.spawn((
        Name::new("World Core"),
        Mesh3d(assets.core_mesh.clone()),
        MeshMaterial3d(assets.core_material.clone()),
        Transform::from_xyz(
            CORE_COORD.0 as f32 * BLOCK_SIZE,
            core_top + 5.2,
            CORE_COORD.1 as f32 * BLOCK_SIZE,
        ),
        PointLight {
            color: Color::srgb(1.0, 0.08, 0.28),
            intensity: 900_000.0,
            range: 68.0,
            shadows_enabled: true,
            ..default()
        },
        Visibility::Hidden,
        WorldCore,
        RunEntity,
    ));

    for index in 0..26 {
        let angle = index as f32 * 2.399_963;
        let radius = 18.0 + (index % 7) as f32 * 8.5;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        let bx = (x / BLOCK_SIZE).round() as i64;
        let bz = (z / BLOCK_SIZE).round() as i64;
        let y = sample_voxel_column(world.settings, bx, bz).height as f32 * HEIGHT_SCALE + 5.0;
        commands.spawn((
            Mesh3d(assets.crystal_mesh.clone()),
            MeshMaterial3d(assets.crystal_material.clone()),
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(0.09)),
            WorldSpark {
                origin_y: y,
                phase: angle,
            },
            RunEntity,
        ));
    }

    session.objective_hint = "اتبع الإشارة الفيروزية واستخرج أول شظية".to_string();
}

pub fn process_crystal_events(
    mut events: MessageReader<CrystalCollected>,
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<AppState>>,
    mut markers: Query<(Entity, &CrystalMarker)>,
    mut commands: Commands,
) {
    for _event in events.read() {
        let unlocked = session.collect_crystal();
        let collected = session.crystals;
        for (entity, marker) in &mut markers {
            if marker.0 == collected {
                commands.entity(entity).despawn();
            }
        }
        if unlocked {
            session.criticality = session.criticality.max(70.0);
            next_state.set(AppState::Decision);
        }
    }
}

pub fn update_run_clock(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<AppState>>,
    mut criticality_events: MessageWriter<CriticalityChanged>,
    mut finish_events: MessageWriter<RunFinished>,
) {
    if session.phase == GamePhase::Finished {
        return;
    }
    let delta = time.delta_secs();
    session.elapsed += delta;
    let rate = if matches!(
        session.phase,
        GamePhase::Evacuating | GamePhase::Stabilizing
    ) {
        balance.final_risk_per_second
    } else {
        balance.passive_risk_per_second
    };
    session.add_criticality(rate * delta);
    criticality_events.write(CriticalityChanged(session.criticality));

    if let Some(remaining) = &mut session.phase_time_remaining {
        *remaining = (*remaining - delta).max(0.0);
    }
    let timed_out = session.phase_time_remaining == Some(0.0);
    if session.criticality >= 100.0 || timed_out {
        session.finish(RunOutcome::Collapse);
        finish_events.write(RunFinished(RunOutcome::Collapse));
        next_state.set(AppState::Ending);
    }
}

pub fn handle_final_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Query<&Transform, With<PlayerTag>>,
    beacon: Query<
        &Transform,
        (
            With<EvacuationBeacon>,
            Without<PlayerTag>,
            Without<WorldCore>,
        ),
    >,
    core: Query<
        &Transform,
        (
            With<WorldCore>,
            Without<PlayerTag>,
            Without<EvacuationBeacon>,
        ),
    >,
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<AppState>>,
    mut finish_events: MessageWriter<RunFinished>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let outcome = match session.phase {
        GamePhase::Evacuating
            if beacon.single().is_ok_and(|target| {
                player.translation.distance(target.translation) <= INTERACT_RADIUS
            }) =>
        {
            Some(RunOutcome::PeopleSaved)
        }
        GamePhase::Stabilizing
            if core.single().is_ok_and(|target| {
                player.translation.distance(target.translation) <= INTERACT_RADIUS
            }) =>
        {
            Some(RunOutcome::WorldSaved)
        }
        _ => None,
    };
    if let Some(outcome) = outcome {
        session.finish(outcome);
        finish_events.write(RunFinished(outcome));
        next_state.set(AppState::Ending);
    }
}

pub fn update_landmarks(
    session: Res<GameSession>,
    time: Res<Time>,
    mut crystals: Query<(&CrystalMarker, &mut Transform), Without<WorldCore>>,
    mut core: Query<(&mut Transform, &mut Visibility), (With<WorldCore>, Without<CrystalMarker>)>,
    mut sparks: Query<(&WorldSpark, &mut Transform), (Without<WorldCore>, Without<CrystalMarker>)>,
) {
    for (marker, mut transform) in &mut crystals {
        let t = time.elapsed_secs() * 1.7 + marker.0 as f32;
        transform.rotation *= Quat::from_rotation_y(time.delta_secs() * 0.75);
        transform.scale = Vec3::splat(1.0 + t.sin() * 0.07);
    }
    if let Ok((mut transform, mut visibility)) = core.single_mut() {
        *visibility = if session.phase == GamePhase::Stabilizing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.rotation *= Quat::from_rotation_y(time.delta_secs() * 1.4);
        let pulse = (time.elapsed_secs() * 3.0).sin() * 0.09 + 1.0;
        transform.scale = Vec3::splat(pulse);
    }
    for (spark, mut transform) in &mut sparks {
        let wave = (time.elapsed_secs() * 0.8 + spark.phase).sin();
        transform.translation.y = spark.origin_y + wave * 2.0;
        transform.rotation *= Quat::from_rotation_y(time.delta_secs());
    }
}

pub fn drive_collapses(
    time: Res<Time>,
    session: Res<GameSession>,
    assets: Res<GameplayVisualAssets>,
    player: Query<&Transform, With<PlayerTag>>,
    mut director: ResMut<CollapseDirector>,
    mut commands: Commands,
) {
    if session.criticality < 70.0
        || !matches!(
            session.phase,
            GamePhase::Evacuating | GamePhase::Stabilizing
        )
    {
        director.timer.reset();
        return;
    }
    if !director.timer.tick(time.delta()).just_finished() {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let count = session.collapse_count as f32 + session.elapsed * 0.07;
    let angle = count * 2.399_963;
    let distance = 16.0 + (count.sin().abs() * 14.0);
    let center_world =
        player.translation + Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);
    let center = VoxelBlockPosition::new(
        (center_world.x / BLOCK_SIZE).round() as i64,
        (player.translation.y / HEIGHT_SCALE).round() as i32 - 1,
        (center_world.z / BLOCK_SIZE).round() as i64,
    );
    commands.spawn((
        Name::new("Collapse Telegraph"),
        Mesh3d(assets.warning_mesh.clone()),
        MeshMaterial3d(assets.warning_material.clone()),
        Transform::from_xyz(center_world.x, center_world.y - 1.7, center_world.z),
        CollapseWarning {
            timer: Timer::from_seconds(1.5, TimerMode::Once),
            center,
        },
        RunEntity,
    ));
    let interval = (11.0 - session.criticality * 0.065).clamp(4.4, 7.0);
    director.timer = Timer::from_seconds(interval, TimerMode::Once);
}

pub fn resolve_collapse_warnings(
    time: Res<Time>,
    mut session: ResMut<GameSession>,
    mut camera: ResMut<VoxelViewerCamera>,
    mut edits: ResMut<VoxelWorldEdits>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut warnings: Query<(Entity, &mut CollapseWarning, &mut Transform)>,
    mut events: MessageWriter<CollapseTriggered>,
    mut commands: Commands,
) {
    for (entity, mut warning, mut transform) in &mut warnings {
        warning.timer.tick(time.delta());
        let progress = warning.timer.fraction();
        let pulse = 1.0 + (progress * 18.0).sin().abs() * 0.22;
        transform.scale = Vec3::splat(pulse);
        if warning.timer.just_finished() {
            edits.edits.push(VoxelTerrainEdit::DigSphere {
                center: warning.center,
                radius: 2,
            });
            invalidate_edit(&mut commands, &mut loaded, warning.center, 2);
            camera.shake = 0.72;
            session.collapse_count = session.collapse_count.saturating_add(1);
            events.write(CollapseTriggered(block_world_center(warning.center)));
            commands.entity(entity).despawn();
        }
    }
}

pub fn enter_main_menu(mut session: ResMut<GameSession>) {
    *session = GameSession::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objective_coordinates_are_distinct_and_deterministic() {
        let unique = CRYSTAL_COORDS
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), 3);
        assert_eq!(CRYSTAL_COORDS[0], (18, 10));
    }
}
