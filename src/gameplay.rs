use astra_voxel_world::prelude::*;
use bevy::prelude::*;

use crate::combat::{CombatTarget, EnemyDirector};
use crate::interaction::VoxelWorldEdits;
use crate::state::*;
use crate::world::reload_loaded_chunks;

const RELAY_COORDS: [(i64, i64); 3] = [(18, 8), (-24, 18), (26, -24)];
const RESOURCE_COORDS: [(BlockKind, i64, i64); 12] = [
    (BlockKind::IronOre, 6, 5),
    (BlockKind::IronOre, 8, -4),
    (BlockKind::IronOre, -7, 7),
    (BlockKind::TitaniumOre, 12, 3),
    (BlockKind::TitaniumOre, -10, 9),
    (BlockKind::TitaniumOre, 14, -10),
    (BlockKind::HeliumVent, 18, -6),
    (BlockKind::HeliumVent, -16, -8),
    (BlockKind::HeliumVent, 9, 15),
    (BlockKind::BioPlasmaBloom, 22, 12),
    (BlockKind::BioPlasmaBloom, -22, 14),
    (BlockKind::BioPlasmaBloom, -12, -17),
];


#[derive(Resource, Default)]
pub struct RunLifecycle {
    pub active: bool,
    pub player_reset_pending: bool,
}

#[derive(Component)]
pub struct RunEntity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionTargetKind {
    HomeCore,
    Relay(u8),
    Ship,
    Gate,
}

#[derive(Component)]
pub struct MissionTarget {
    pub kind: MissionTargetKind,
    pub health: f32,
    pub max_health: f32,
}

#[derive(Component)]
pub struct LandmarkPulse {
    pub base_scale: Vec3,
    pub phase: f32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct RelayDestroyed(pub u8);

#[derive(Resource)]
pub struct GameplayVisualAssets {
    platform_mesh: Handle<Mesh>,
    relay_mesh: Handle<Mesh>,
    gate_mesh: Handle<Mesh>,
    ship_mesh: Handle<Mesh>,
    home_material: Handle<StandardMaterial>,
    alien_material: Handle<StandardMaterial>,
    gate_material: Handle<StandardMaterial>,
    ship_material: Handle<StandardMaterial>,
    ship_scene: Handle<Scene>,
    gate_scene: Handle<Scene>,
    relay_scene: Handle<Scene>,
}

pub fn setup_gameplay_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let platform_mesh = meshes.add(Cylinder::new(6.0, 1.0));
    let relay_mesh = meshes.add(Cuboid::new(4.0, 12.0, 4.0));
    let gate_mesh = meshes.add(Torus::new(6.0, 8.0));
    let ship_mesh = meshes.add(Capsule3d::new(3.4, 8.0));
    let home_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.34, 0.42),
        emissive: LinearRgba::rgb(0.05, 0.65, 0.85),
        metallic: 0.72,
        perceptual_roughness: 0.26,
        ..default()
    });
    let alien_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.34, 0.05, 0.18),
        emissive: LinearRgba::rgb(1.6, 0.03, 0.34),
        metallic: 0.55,
        perceptual_roughness: 0.22,
        ..default()
    });
    let gate_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.38, 0.08, 0.72, 0.86),
        emissive: LinearRgba::rgb(2.6, 0.18, 4.0),
        alpha_mode: AlphaMode::Blend,
        metallic: 0.45,
        perceptual_roughness: 0.14,
        ..default()
    });
    let ship_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.40, 0.46),
        emissive: LinearRgba::rgb(0.02, 0.16, 0.22),
        metallic: 0.82,
        perceptual_roughness: 0.24,
        ..default()
    });
    commands.insert_resource(GameplayVisualAssets {
        platform_mesh,
        relay_mesh,
        gate_mesh,
        ship_mesh,
        home_material,
        alien_material,
        gate_material,
        ship_material,
        ship_scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/kenney-space/craft_miner.glb")),
        gate_scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/kenney-space/gate_complex.glb")),
        relay_scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/kenney-space/machine_generator.glb")),
    });
}

pub fn prepare_new_run(
    mut world: ResMut<VoxelViewerWorld>,
    assets: Res<GameplayVisualAssets>,
    mut lifecycle: ResMut<RunLifecycle>,
    mut director: ResMut<EnemyDirector>,
    mut session: ResMut<GameSession>,
    mut edits: ResMut<VoxelWorldEdits>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    old_entities: Query<Entity, With<RunEntity>>,
    mut commands: Commands,
) {
    if lifecycle.active || session.route == PlanetRoute::Undecided {
        return;
    }
    for entity in &old_entities {
        commands.entity(entity).despawn();
    }
    let route = session.route;
    apply_planet_profile(&mut world, route);
    edits.edits.clear();
    edits.placed_durability.clear();
    reload_loaded_chunks(&mut commands, &mut loaded);
    lifecycle.active = true;
    *director = EnemyDirector::default();

    let surface = sample_voxel_column(world.settings, 0, 0).height as f32 * HEIGHT_SCALE;
    let spawn = Vec3::new(BLOCK_SIZE * 0.5, surface + 2.1, BLOCK_SIZE * 0.5);

    if session.phase == MissionPhase::AwaitingRoute {
        session.begin_route(route, spawn);
        lifecycle.player_reset_pending = true;
    }

    for (block, x, z) in RESOURCE_COORDS {
        let y = sample_voxel_column(world.settings, x, z).height + 1;
        edits.edits.push(VoxelTerrainEdit::SetBlock {
            position: VoxelBlockPosition::new(x, y, z),
            block,
        });
    }

    match route {
        PlanetRoute::HomeDefense => {}
        PlanetRoute::InvadedPlanet => {
            for (index, (x, z)) in RELAY_COORDS.into_iter().enumerate() {
                spawn_relay(&mut commands, &assets, world.settings, index as u8 + 1, x, z);
            }
            spawn_gate(&mut commands, &assets, world.settings);
        }
        PlanetRoute::Undecided => {}
    }
}

fn apply_planet_profile(world: &mut VoxelViewerWorld, route: PlanetRoute) {
    world.settings.seed = match route {
        PlanetRoute::HomeDefense => HOME_SEED,
        PlanetRoute::InvadedPlanet => INVASION_SEED,
        PlanetRoute::Undecided => GAME_SEED,
    };
    let ratios = &mut world.settings.composition.resource_ratios;
    ratios.set_named("iron", 0.42);
    ratios.set_named("titanium", if route == PlanetRoute::InvadedPlanet { 0.58 } else { 0.30 });
    ratios.set_named("helium_3", if route == PlanetRoute::InvadedPlanet { 0.48 } else { 0.18 });
    ratios.set_named("bio_plasma", if route == PlanetRoute::InvadedPlanet { 0.52 } else { 0.14 });
    world.settings.cave_density = if route == PlanetRoute::InvadedPlanet { 0.14 } else { 0.08 };
}

fn spawn_ship(commands: &mut Commands, assets: &GameplayVisualAssets, surface: f32) {
    commands.spawn((
        Name::new("Player Landing Ship"),
        Visibility::default(),
        Mesh3d(assets.ship_mesh.clone()),
        MeshMaterial3d(assets.ship_material.clone()),
        Transform::from_xyz(-10.0, surface + 1.2, 2.0),
        MissionTarget { kind: MissionTargetKind::Ship, health: 650.0, max_health: 650.0 },
        CombatTarget { radius: 5.0, aerial: false, targetable: false },
        RunEntity,
    )).with_children(|parent| {
        parent.spawn((
            SceneRoot(assets.ship_scene.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(4.2)),
        ));
    });
}

fn spawn_home_core(commands: &mut Commands, assets: &GameplayVisualAssets, surface: f32) {
    commands.spawn((
        Name::new("Home Colony Core"),
        Mesh3d(assets.platform_mesh.clone()),
        MeshMaterial3d(assets.home_material.clone()),
        Transform::from_xyz(12.0, surface + 0.6, 8.0),
        PointLight {
            color: Color::srgb(0.18, 0.82, 1.0),
            intensity: 360_000.0,
            range: 46.0,
            shadows_enabled: false,
            ..default()
        },
        MissionTarget { kind: MissionTargetKind::HomeCore, health: 500.0, max_health: 500.0 },
        CombatTarget { radius: 6.0, aerial: false, targetable: false },
        LandmarkPulse { base_scale: Vec3::ONE, phase: 0.0 },
        RunEntity,
    ));
}

fn spawn_relay(
    commands: &mut Commands,
    assets: &GameplayVisualAssets,
    settings: VoxelWorldSettings,
    index: u8,
    x: i64,
    z: i64,
) {
    let y = sample_voxel_column(settings, x, z).height as f32 * HEIGHT_SCALE;
    commands.spawn((
        Name::new(format!("Alien Relay {index}")),
        Mesh3d(assets.relay_mesh.clone()),
        MeshMaterial3d(assets.alien_material.clone()),
        Transform::from_xyz(
            (x as f32 + 0.5) * BLOCK_SIZE,
            y + 6.0,
            (z as f32 + 0.5) * BLOCK_SIZE,
        ),
        PointLight {
            color: Color::srgb(1.0, 0.04, 0.24),
            intensity: 520_000.0,
            range: 44.0,
            shadows_enabled: false,
            ..default()
        },
        MissionTarget { kind: MissionTargetKind::Relay(index), health: 180.0 + index as f32 * 45.0, max_health: 315.0 },
        CombatTarget { radius: 5.0, aerial: false, targetable: true },
        LandmarkPulse { base_scale: Vec3::ONE, phase: index as f32 * 1.7 },
        RunEntity,
    )).with_children(|parent| {
        parent.spawn((
            SceneRoot(assets.relay_scene.clone()),
            Transform::from_xyz(0.0, -5.5, 0.0).with_scale(Vec3::splat(3.6)),
        ));
    });
}

fn spawn_gate(commands: &mut Commands, assets: &GameplayVisualAssets, settings: VoxelWorldSettings) {
    let (x, z) = (-34, -26);
    let y = sample_voxel_column(settings, x, z).height as f32 * HEIGHT_SCALE;
    commands.spawn((
        Name::new("Alien Invasion Gate"),
        Mesh3d(assets.gate_mesh.clone()),
        MeshMaterial3d(assets.gate_material.clone()),
        Transform::from_xyz(x as f32 * BLOCK_SIZE, y + 10.0, z as f32 * BLOCK_SIZE)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        Visibility::Hidden,
        MissionTarget { kind: MissionTargetKind::Gate, health: 1000.0, max_health: 1000.0 },
        CombatTarget { radius: 9.0, aerial: false, targetable: false },
        LandmarkPulse { base_scale: Vec3::splat(1.35), phase: 0.4 },
        RunEntity,
    )).with_children(|parent| {
        parent.spawn((
            SceneRoot(assets.gate_scene.clone()),
            Transform::from_xyz(0.0, -8.0, 0.0).with_scale(Vec3::splat(4.8)),
        ));
    });
}

pub fn update_mission(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<AppState>>,
    mut finished: MessageWriter<RunFinished>,
) {
    if session.phase == MissionPhase::Finished {
        return;
    }
    let dt = time.delta_secs();
    session.elapsed += dt;

    match session.route {
        PlanetRoute::HomeDefense => {
            let remaining = (balance.home_duration - session.elapsed).max(0.0);
            session.phase_time_remaining = Some(remaining);
            session.phase = if session.elapsed < 90.0 { MissionPhase::HomePreparation } else { MissionPhase::HomeDefense };
            session.wave = if session.elapsed < 90.0 { 0 } else if session.elapsed < 240.0 { 1 } else if session.elapsed < 420.0 { 2 } else { 3 };
            session.objective_hint = if session.phase == MissionPhase::HomePreparation {
                "حصّن المولد قبل وصول أول غارة".into()
            } else {
                format!("دافع عن مولد المستعمرة — الموجة {}/3", session.wave)
            };
            if remaining <= 0.0 {
                finish_run(&mut session, RunOutcome::HomeDefended, &mut finished, &mut next_state);
            }
        }
        PlanetRoute::InvadedPlanet => {
            if session.phase == MissionPhase::AlienLanding && session.elapsed >= 4.0 {
                session.phase = MissionPhase::RelayHunt;
            }
            if matches!(session.phase, MissionPhase::Extraction | MissionPhase::GateAssault) {
                let default_remaining = match session.phase {
                    MissionPhase::Extraction => balance.extraction_seconds,
                    _ => balance.gate_assault_seconds,
                };
                let remaining = session.phase_time_remaining.get_or_insert(default_remaining);
                *remaining = (*remaining - dt).max(0.0);
                if *remaining <= 0.0 {
                    let outcome = if session.phase == MissionPhase::Extraction {
                        RunOutcome::Extracted
                    } else {
                        RunOutcome::MissionFailed
                    };
                    finish_run(&mut session, outcome, &mut finished, &mut next_state);
                }
            }
        }
        PlanetRoute::Undecided => {}
    }
}

pub fn handle_relay_destroyed(
    mut events: MessageReader<RelayDestroyed>,
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for RelayDestroyed(index) in events.read() {
        session.relays_destroyed = session.relays_destroyed.max(*index).min(3);
        session.safe_position = match *index {
            1 => Vec3::new(18.5 * BLOCK_SIZE, session.safe_position.y, 8.5 * BLOCK_SIZE),
            2 => Vec3::new(-23.5 * BLOCK_SIZE, session.safe_position.y, 18.5 * BLOCK_SIZE),
            _ => session.safe_position,
        };
        session.objective_hint = match session.relays_destroyed {
            1 => "البرج الأرضي دُمّر — اصنع مدفع البلازما وتابع التقدم".into(),
            2 => "البرج الجوي دُمّر — رمح الأيون ضروري للمرحلة التالية".into(),
            _ => "أبراج الغزو متوقفة. اتخذ قرارك النهائي.".into(),
        };
        if session.relays_destroyed >= 3 {
            next_state.set(AppState::FinalDecision);
        }
    }
}

pub fn animate_landmarks(
    time: Res<Time>,
    session: Res<GameSession>,
    mut landmarks: Query<(&LandmarkPulse, &MissionTarget, &mut Transform, &mut Visibility)>,
) {
    for (pulse, target, mut transform, mut visibility) in &mut landmarks {
        if target.kind == MissionTargetKind::Gate {
            *visibility = if session.phase == MissionPhase::GateAssault { Visibility::Visible } else { Visibility::Hidden };
        }
        let scale = 1.0 + (time.elapsed_secs() * 2.0 + pulse.phase).sin() * 0.045;
        transform.scale = pulse.base_scale * scale;
        transform.rotate_y(time.delta_secs() * 0.18);
    }
}

pub fn finish_run(
    session: &mut GameSession,
    outcome: RunOutcome,
    events: &mut MessageWriter<RunFinished>,
    next_state: &mut NextState<AppState>,
) {
    session.finish(outcome);
    events.write(RunFinished(outcome));
    next_state.set(AppState::Ending);
}

pub fn enter_main_menu(mut session: ResMut<GameSession>, mut lifecycle: ResMut<RunLifecycle>) {
    *session = GameSession::default();
    lifecycle.active = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objectives_are_deterministic_and_distinct() {
        let unique = RELAY_COORDS.into_iter().collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn resource_guarantees_cover_all_recipe_types() {
        for expected in [BlockKind::IronOre, BlockKind::TitaniumOre, BlockKind::HeliumVent, BlockKind::BioPlasmaBloom] {
            assert!(RESOURCE_COORDS.iter().any(|(kind, _, _)| *kind == expected));
        }
    }
}
