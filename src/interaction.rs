use astra_voxel_world::prelude::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::player::PlayerTag;
use crate::state::*;
use crate::world::invalidate_edit;

const INTERACTION_DISTANCE: f32 = 48.0;
const RAY_STEP: f32 = 0.55;
const MINE_SECONDS: f32 = 0.62;

#[derive(Component)]
pub struct TargetBlockHighlightTag;

#[derive(Resource, Default)]
pub struct VoxelWorldEdits {
    pub edits: Vec<VoxelTerrainEdit>,
    pub targeted: Option<VoxelHit>,
}

#[derive(Resource, Default)]
pub struct MiningState {
    pub target: Option<VoxelBlockPosition>,
    pub progress: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelHit {
    pub block: VoxelBlockPosition,
    pub placement: Option<VoxelBlockPosition>,
    pub kind: BlockKind,
}

pub fn setup_target_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(
        BLOCK_SIZE * 1.04,
        HEIGHT_SCALE * 1.08,
        BLOCK_SIZE * 1.04,
    ));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.78, 0.16, 0.26),
        emissive: LinearRgba::rgb(2.2, 1.15, 0.12),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Name::new("Voxel Target"),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, -500.0, 0.0),
        Visibility::Hidden,
        TargetBlockHighlightTag,
    ));
}

pub fn update_target_block_highlight(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<VoxelViewerCameraTag>>,
    loaded: Res<LoadedVoxelChunks>,
    mut edits: ResMut<VoxelWorldEdits>,
    mining: Res<MiningState>,
    mut highlight_query: Query<(&mut Transform, &mut Visibility), With<TargetBlockHighlightTag>>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok(window) = primary_window.single() else {
        return;
    };
    let Ok((mut transform, mut visibility)) = highlight_query.single_mut() else {
        return;
    };

    edits.targeted = window.cursor_position().and_then(|cursor| {
        camera
            .viewport_to_world(camera_transform, cursor)
            .ok()
            .and_then(|ray| {
                voxel_raycast_loaded(&loaded, ray.origin, *ray.direction, INTERACTION_DISTANCE)
            })
    });

    if let Some(hit) = edits.targeted {
        transform.translation = block_world_center(hit.block);
        let pulse = 1.0 + mining.progress.clamp(0.0, 1.0) * 0.10;
        transform.scale = Vec3::splat(pulse);
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

pub fn handle_voxel_digging_and_building(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    ui_buttons: Query<&Interaction, With<Button>>,
    player_query: Query<&Transform, With<PlayerTag>>,
    balance: Res<BalanceConfig>,
    mut session: ResMut<GameSession>,
    mut mining: ResMut<MiningState>,
    mut edits: ResMut<VoxelWorldEdits>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut crystal_events: MessageWriter<CrystalCollected>,
    mut criticality_events: MessageWriter<CriticalityChanged>,
    mut action_sounds: MessageWriter<VoxelActionSound>,
    mut commands: Commands,
) {
    let pointer_over_ui = ui_buttons
        .iter()
        .any(|interaction| !matches!(interaction, Interaction::None));
    if pointer_over_ui {
        mining.target = None;
        mining.progress = 0.0;
        return;
    }

    if mouse.just_pressed(MouseButton::Right)
        && let Some(hit) = edits.targeted
        && let Some(position) = hit.placement
    {
        let player_occupies = player_query.single().is_ok_and(|player| {
            let player_block = world_to_block(player.translation);
            (player_block.x - position.x).abs() <= 1
                && (player_block.z - position.z).abs() <= 1
                && (player_block.y - position.y).abs() <= 2
        });
        if session.supports > 0
            && !player_occupies
            && loaded
                .block_at(position)
                .is_none_or(|kind| !kind.is_solid())
        {
            edits.edits.push(VoxelTerrainEdit::SetBlock {
                position,
                block: BlockKind::Stone,
            });
            session.supports -= 1;
            invalidate_edit(&mut commands, &mut loaded, position, 1);
            action_sounds.write(VoxelActionSound::Build);
        }
    }

    if !mouse.pressed(MouseButton::Left) {
        mining.target = None;
        mining.progress = 0.0;
        return;
    }

    let Some(hit) = edits.targeted else {
        mining.target = None;
        mining.progress = 0.0;
        return;
    };
    if matches!(
        hit.kind,
        BlockKind::Bedrock | BlockKind::Water | BlockKind::Lava
    ) {
        mining.target = None;
        mining.progress = 0.0;
        return;
    }

    if mining.target != Some(hit.block) {
        mining.target = Some(hit.block);
        mining.progress = 0.0;
    }
    mining.progress += time.delta_secs() / MINE_SECONDS;
    if mining.progress < 1.0 {
        return;
    }

    edits.edits.push(VoxelTerrainEdit::SetBlock {
        position: hit.block,
        block: BlockKind::Air,
    });
    invalidate_edit(&mut commands, &mut loaded, hit.block, 1);
    action_sounds.write(VoxelActionSound::Mine);
    if hit.kind == BlockKind::CrystalOre && session.crystals < 3 {
        crystal_events.write(CrystalCollected(session.crystals.saturating_add(1)));
    } else {
        session.add_criticality(balance.dig_risk);
        criticality_events.write(CriticalityChanged(session.criticality));
    }
    mining.target = None;
    mining.progress = 0.0;
}

pub fn voxel_raycast_loaded(
    loaded: &LoadedVoxelChunks,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<VoxelHit> {
    let direction = direction.normalize_or_zero();
    if direction == Vec3::ZERO {
        return None;
    }
    let mut previous_air = None;
    let mut previous_position = None;
    let steps = (max_distance / RAY_STEP).ceil() as usize;
    for step in 0..=steps {
        let point = origin + direction * (step as f32 * RAY_STEP);
        let position = world_to_block(point);
        if previous_position == Some(position) {
            continue;
        }
        previous_position = Some(position);
        match loaded.block_at(position) {
            Some(kind) if kind.is_solid() => {
                return Some(VoxelHit {
                    block: position,
                    placement: previous_air,
                    kind,
                });
            }
            _ => previous_air = Some(position),
        }
    }
    None
}

pub fn world_to_block(point: Vec3) -> VoxelBlockPosition {
    VoxelBlockPosition::new(
        (point.x / BLOCK_SIZE).round() as i64,
        (point.y / HEIGHT_SCALE).round() as i32,
        (point.z / BLOCK_SIZE).round() as i64,
    )
}

pub fn block_world_center(position: VoxelBlockPosition) -> Vec3 {
    Vec3::new(
        position.x as f32 * BLOCK_SIZE,
        position.y as f32 * HEIGHT_SCALE,
        position.z as f32 * BLOCK_SIZE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raycast_observes_cached_voxel_data() {
        let coord = VoxelChunkCoord::new(0, 0);
        let mut chunk = VoxelChunk::new(coord, 64);
        chunk.set(2, 10, 2, BlockKind::Stone);
        let mut loaded = LoadedVoxelChunks::default();
        loaded.voxel_data.insert(coord, chunk);
        let origin = Vec3::new(2.0 * BLOCK_SIZE, 16.0 * HEIGHT_SCALE, 2.0 * BLOCK_SIZE);
        let hit = voxel_raycast_loaded(&loaded, origin, Vec3::NEG_Y, 20.0).unwrap();
        assert_eq!(hit.block, VoxelBlockPosition::new(2, 10, 2));
        assert_eq!(hit.kind, BlockKind::Stone);
    }

    #[test]
    fn zero_direction_never_hits() {
        assert!(
            voxel_raycast_loaded(&LoadedVoxelChunks::default(), Vec3::ZERO, Vec3::ZERO, 10.0)
                .is_none()
        );
    }
}
