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

#[derive(Component)]
pub struct PlacementBlockHighlightTag;

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
    let thickness = 0.13;
    let x_bar = meshes.add(Cuboid::new(BLOCK_SIZE * 1.05, thickness, thickness));
    let y_bar = meshes.add(Cuboid::new(thickness, HEIGHT_SCALE * 1.12, thickness));
    let z_bar = meshes.add(Cuboid::new(thickness, thickness, BLOCK_SIZE * 1.05));
    let delete_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.58, 0.08),
        emissive: LinearRgba::rgb(1.4, 0.38, 0.03),
        unlit: true,
        ..default()
    });
    let placement_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.86, 0.94),
        emissive: LinearRgba::rgb(0.04, 1.2, 1.5),
        unlit: true,
        ..default()
    });

    spawn_highlight_frame(
        &mut commands,
        &x_bar,
        &y_bar,
        &z_bar,
        delete_material,
        "Mining Target",
        TargetBlockHighlightTag,
    );
    spawn_highlight_frame(
        &mut commands,
        &x_bar,
        &y_bar,
        &z_bar,
        placement_material,
        "Support Preview",
        PlacementBlockHighlightTag,
    );
}

fn spawn_highlight_frame(
    commands: &mut Commands,
    x_bar: &Handle<Mesh>,
    y_bar: &Handle<Mesh>,
    z_bar: &Handle<Mesh>,
    material: Handle<StandardMaterial>,
    name: &'static str,
    marker: impl Component,
) {
    let half_x = BLOCK_SIZE * 0.525;
    let half_y = HEIGHT_SCALE * 0.56;
    commands
        .spawn((
            Name::new(name),
            Transform::from_xyz(0.0, -500.0, 0.0),
            Visibility::Hidden,
            marker,
        ))
        .with_children(|parent| {
            for y in [-half_y, half_y] {
                for z in [-half_x, half_x] {
                    parent.spawn((
                        Mesh3d(x_bar.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_xyz(0.0, y, z),
                    ));
                }
            }
            for x in [-half_x, half_x] {
                for z in [-half_x, half_x] {
                    parent.spawn((
                        Mesh3d(y_bar.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_xyz(x, 0.0, z),
                    ));
                }
            }
            for x in [-half_x, half_x] {
                for y in [-half_y, half_y] {
                    parent.spawn((
                        Mesh3d(z_bar.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_xyz(x, y, 0.0),
                    ));
                }
            }
        });
}

pub fn update_target_block_highlight(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<VoxelViewerCameraTag>>,
    ui_buttons: Query<&Interaction, With<Button>>,
    session: Res<GameSession>,
    loaded: Res<LoadedVoxelChunks>,
    mut edits: ResMut<VoxelWorldEdits>,
    mining: Res<MiningState>,
    mut delete_highlight: Query<
        (&mut Transform, &mut Visibility),
        (
            With<TargetBlockHighlightTag>,
            Without<PlacementBlockHighlightTag>,
        ),
    >,
    mut placement_highlight: Query<
        (&mut Transform, &mut Visibility),
        (
            With<PlacementBlockHighlightTag>,
            Without<TargetBlockHighlightTag>,
        ),
    >,
) {
    let Ok((mut delete_transform, mut delete_visibility)) = delete_highlight.single_mut() else {
        return;
    };
    let Ok((mut placement_transform, mut placement_visibility)) = placement_highlight.single_mut()
    else {
        return;
    };

    let pointer_over_ui = ui_buttons
        .iter()
        .any(|interaction| !matches!(interaction, Interaction::None));
    let ray_hit = if pointer_over_ui {
        None
    } else {
        primary_window
            .single()
            .ok()
            .and_then(|window| window.cursor_position())
            .and_then(|cursor| {
                camera_query
                    .single()
                    .ok()
                    .and_then(|(camera, camera_transform)| {
                        camera
                            .viewport_to_world(camera_transform, cursor)
                            .ok()
                            .and_then(|ray| {
                                voxel_raycast_loaded(
                                    &loaded,
                                    ray.origin,
                                    *ray.direction,
                                    INTERACTION_DISTANCE,
                                )
                            })
                    })
            })
    };
    edits.targeted = ray_hit;

    let Some(hit) = ray_hit else {
        *delete_visibility = Visibility::Hidden;
        *placement_visibility = Visibility::Hidden;
        return;
    };

    delete_transform.translation = block_world_center(hit.block);
    let pulse = 1.0 + mining.progress.clamp(0.0, 1.0) * 0.10;
    delete_transform.scale = Vec3::splat(pulse);
    *delete_visibility = Visibility::Visible;

    let valid_placement = hit.placement.filter(|position| {
        session.supports > 0
            && loaded
                .block_at(*position)
                .is_none_or(|kind| !kind.is_solid())
    });
    if let Some(position) = valid_placement {
        placement_transform.translation = block_world_center(position);
        placement_transform.scale = Vec3::ONE;
        *placement_visibility = Visibility::Visible;
    } else {
        *placement_visibility = Visibility::Hidden;
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
        (point.x / BLOCK_SIZE).floor() as i64,
        (point.y / HEIGHT_SCALE).ceil() as i32,
        (point.z / BLOCK_SIZE).floor() as i64,
    )
}

pub fn block_world_center(position: VoxelBlockPosition) -> Vec3 {
    Vec3::new(
        (position.x as f32 + 0.5) * BLOCK_SIZE,
        (position.y as f32 - 0.5) * HEIGHT_SCALE,
        (position.z as f32 + 0.5) * BLOCK_SIZE,
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
    fn block_center_round_trips_for_positive_and_negative_coordinates() {
        for position in [
            VoxelBlockPosition::new(0, 10, 0),
            VoxelBlockPosition::new(-2, 7, 3),
            VoxelBlockPosition::new(4, 1, -5),
        ] {
            assert_eq!(world_to_block(block_world_center(position)), position);
        }
    }

    #[test]
    fn world_cells_follow_rendered_mesh_bounds() {
        assert_eq!(
            world_to_block(Vec3::new(BLOCK_SIZE * 0.99, HEIGHT_SCALE * 9.01, 0.01)),
            VoxelBlockPosition::new(0, 10, 0)
        );
        assert_eq!(
            world_to_block(Vec3::new(-0.01, HEIGHT_SCALE * 9.99, -0.01)),
            VoxelBlockPosition::new(-1, 10, -1)
        );
    }

    #[test]
    fn zero_direction_never_hits() {
        assert!(
            voxel_raycast_loaded(&LoadedVoxelChunks::default(), Vec3::ZERO, Vec3::ZERO, 10.0)
                .is_none()
        );
    }
}
