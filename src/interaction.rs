use std::collections::HashMap;

use astra_voxel_world::prelude::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::combat::CombatTarget;
use crate::player::PlayerTag;
use crate::state::*;
use crate::world::invalidate_edit;

const INTERACTION_DISTANCE: f32 = 52.0;
const RAY_STEP: f32 = 0.45;
const MINE_SECONDS: f32 = 0.48;
const BUILDABLE_BLOCKS: [BlockKind; 13] = [
    BlockKind::Stone,
    BlockKind::Dirt,
    BlockKind::Grass,
    BlockKind::Sand,
    BlockKind::Snow,
    BlockKind::Wood,
    BlockKind::Leaves,
    BlockKind::Mud,
    BlockKind::Basalt,
    BlockKind::Ice,
    BlockKind::VolcanicAsh,
    BlockKind::CoalOre,
    BlockKind::CrystalOre,
];

#[derive(Component)]
pub struct TargetBlockHighlightTag;
#[derive(Component)]
pub struct PlacementBlockHighlightTag;

#[derive(Component)]
pub struct HighlightFramePiece {
    placement: bool,
}

#[derive(Resource)]
pub struct PlacementHighlightMaterials {
    valid: Handle<StandardMaterial>,
    invalid: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct VoxelWorldEdits {
    pub edits: Vec<VoxelTerrainEdit>,
    pub placed_durability: HashMap<VoxelBlockPosition, f32>,
}

#[derive(Resource, Default)]
pub struct MiningState {
    pub target: Option<VoxelBlockPosition>,
    pub progress: f32,
}

pub fn setup_target_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let thickness = 0.11;
    let x_bar = meshes.add(Cuboid::new(BLOCK_SIZE * 1.04, thickness, thickness));
    let y_bar = meshes.add(Cuboid::new(thickness, HEIGHT_SCALE * 1.08, thickness));
    let z_bar = meshes.add(Cuboid::new(thickness, thickness, BLOCK_SIZE * 1.04));
    let mining_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.34, 0.08),
        emissive: LinearRgba::rgb(1.8, 0.18, 0.02),
        unlit: true,
        ..default()
    });
    let build_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.05, 0.92, 0.74, 0.72),
        emissive: LinearRgba::rgb(0.02, 1.4, 0.82),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let invalid_build_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.10, 0.08, 0.78),
        emissive: LinearRgba::rgb(2.1, 0.02, 0.01),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.insert_resource(PlacementHighlightMaterials {
        valid: build_material.clone(),
        invalid: invalid_build_material,
    });
    spawn_highlight_frame(
        &mut commands,
        &x_bar,
        &y_bar,
        &z_bar,
        mining_material,
        "Mining Target",
        TargetBlockHighlightTag,
        false,
    );
    spawn_highlight_frame(
        &mut commands,
        &x_bar,
        &y_bar,
        &z_bar,
        build_material,
        "Build Preview",
        PlacementBlockHighlightTag,
        true,
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
    placement_piece: bool,
) {
    let half_x = BLOCK_SIZE * 0.52;
    let half_y = HEIGHT_SCALE * 0.54;
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
                        HighlightFramePiece {
                            placement: placement_piece,
                        },
                    ));
                }
            }
            for x in [-half_x, half_x] {
                for z in [-half_x, half_x] {
                    parent.spawn((
                        Mesh3d(y_bar.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_xyz(x, 0.0, z),
                        HighlightFramePiece {
                            placement: placement_piece,
                        },
                    ));
                }
            }
            for x in [-half_x, half_x] {
                for y in [-half_y, half_y] {
                    parent.spawn((
                        Mesh3d(z_bar.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_xyz(x, y, 0.0),
                        HighlightFramePiece {
                            placement: placement_piece,
                        },
                    ));
                }
            }
        });
}

pub fn handle_tool_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut session: ResMut<GameSession>,
) {
    let selected = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(ToolSlot::Weapon(WeaponKind::PulseRifle))
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(ToolSlot::Weapon(WeaponKind::PlasmaMortar))
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(ToolSlot::Weapon(WeaponKind::IonLance))
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some(ToolSlot::Weapon(WeaponKind::QuantumTesla))
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        Some(ToolSlot::Weapon(WeaponKind::NukeMortar))
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        Some(ToolSlot::MiningLaser)
    } else if keyboard.just_pressed(KeyCode::Digit7) {
        Some(ToolSlot::Builder)
    } else {
        None
    };
    if let Some(tool) = selected {
        session.loadout.selected_tool = tool;
    }

    let mut scroll = 0.0;
    for event in mouse_wheel.read() {
        scroll += match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.08,
        };
    }
    if session.loadout.selected_tool == ToolSlot::Builder && scroll.abs() > f32::EPSILON {
        cycle_build_block(&mut session.loadout, if scroll > 0.0 { 1 } else { -1 });
    }
}

fn cycle_build_block(loadout: &mut PlayerLoadout, direction: i32) {
    let available = BUILDABLE_BLOCKS
        .iter()
        .copied()
        .filter(|kind| loadout.block_count(*kind) > 0)
        .collect::<Vec<_>>();
    if available.is_empty() {
        return;
    }
    let current = available
        .iter()
        .position(|kind| *kind == loadout.selected_block)
        .unwrap_or(0) as i32;
    let next = (current + direction).rem_euclid(available.len() as i32) as usize;
    loadout.selected_block = available[next];
}

pub fn compute_aim_solution(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<VoxelViewerCameraTag>>,
    ui_buttons: Query<&Interaction, With<Button>>,
    targets: Query<(Entity, &GlobalTransform, &CombatTarget)>,
    player_query: Query<&Transform, (With<PlayerTag>, Without<VoxelViewerCameraTag>)>,
    loaded: Res<LoadedVoxelChunks>,
    mut aim: ResMut<AimSolution>,
) {
    aim.pointer_over_ui = ui_buttons
        .iter()
        .any(|interaction| *interaction != Interaction::None);
    aim.voxel = None;
    aim.enemy = None;
    aim.world_point = None;
    aim.aim_point = None;
    if aim.pointer_over_ui {
        return;
    }
    let Some(cursor) = primary_window
        .single()
        .ok()
        .and_then(Window::cursor_position)
    else {
        return;
    };
    let Some((camera, camera_global_transform)) = camera_query.single().ok() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_global_transform, cursor) else {
        return;
    };
    let origin = ray.origin;
    let direction = *ray.direction;

    let voxel = voxel_raycast_loaded(&loaded, origin, direction, INTERACTION_DISTANCE);
    aim.voxel = voxel;
    aim.world_point = voxel.map(|hit| block_world_center(hit.block));
    let mut closest_enemy_distance = 120.0;
    for (entity, transform, target) in &targets {
        if !target.targetable {
            continue;
        }
        if let Some(distance) =
            ray_sphere_distance(origin, direction, transform.translation(), target.radius + 1.8)
        {
            if distance <= closest_enemy_distance {
                closest_enemy_distance = distance;
                aim.enemy = Some(entity);
                aim.world_point = Some(origin + direction * distance);
            }
        }
    }

    let player_y = player_query
        .single()
        .map(|p| p.translation.y)
        .unwrap_or(30.0);
    let plane_aim = if direction.y.abs() > 1e-4 {
        let t = (player_y + 1.0 - origin.y) / direction.y;
        if t > 0.0 {
            Some(origin + direction * t)
        } else {
            None
        }
    } else {
        None
    };

    aim.aim_point = aim
        .world_point
        .or(plane_aim)
        .or(Some(origin + direction * 60.0));
}

pub fn update_target_highlights(
    session: Res<GameSession>,
    aim: Res<AimSolution>,
    player: Query<
        &Transform,
        (
            With<PlayerTag>,
            Without<TargetBlockHighlightTag>,
            Without<PlacementBlockHighlightTag>,
        ),
    >,
    loaded: Res<LoadedVoxelChunks>,
    mining: Res<MiningState>,
    blockers: Query<(&GlobalTransform, &CombatTarget), Without<PlayerTag>>,
    placement_materials: Res<PlacementHighlightMaterials>,
    mut highlight_pieces: Query<(&HighlightFramePiece, &mut MeshMaterial3d<StandardMaterial>)>,
    mut delete_highlight: Query<
        (&mut Transform, &mut Visibility),
        (
            With<TargetBlockHighlightTag>,
            Without<PlacementBlockHighlightTag>,
            Without<PlayerTag>,
        ),
    >,
    mut placement_highlight: Query<
        (&mut Transform, &mut Visibility),
        (
            With<PlacementBlockHighlightTag>,
            Without<TargetBlockHighlightTag>,
            Without<PlayerTag>,
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
    *delete_visibility = Visibility::Hidden;
    *placement_visibility = Visibility::Hidden;

    let Some(hit) = aim.voxel else {
        return;
    };
    match session.loadout.selected_tool {
        ToolSlot::MiningLaser => {
            delete_transform.translation = block_world_center(hit.block);
            delete_transform.scale = Vec3::splat(1.0 + mining.progress.clamp(0.0, 1.0) * 0.08);
            *delete_visibility = Visibility::Visible;
        }
        ToolSlot::Builder => {
            if let Some(position) = hit.placement {
                let can_place = session.loadout.block_count(session.loadout.selected_block) > 0
                    && valid_placement(position, &loaded, &player, &blockers);
                placement_transform.translation = block_world_center(position);
                placement_transform.scale = Vec3::ONE;
                *placement_visibility = Visibility::Visible;
                let material = if can_place {
                    &placement_materials.valid
                } else {
                    &placement_materials.invalid
                };
                for (piece, mut piece_material) in &mut highlight_pieces {
                    if piece.placement {
                        piece_material.0 = material.clone();
                    }
                }
            }
        }
        ToolSlot::Weapon(_) => {}
    }
}

pub fn handle_voxel_actions(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    aim: Res<AimSolution>,
    player: Query<&Transform, With<PlayerTag>>,
    blockers: Query<(&GlobalTransform, &CombatTarget), Without<PlayerTag>>,
    mut session: ResMut<GameSession>,
    mut mining: ResMut<MiningState>,
    mut edits: ResMut<VoxelWorldEdits>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut resource_events: MessageWriter<ResourceCollected>,
    mut block_events: MessageWriter<BlockCollected>,
    mut sounds: MessageWriter<GameSound>,
    mut commands: Commands,
) {
    if aim.pointer_over_ui {
        mining.target = None;
        mining.progress = 0.0;
        return;
    }
    match session.loadout.selected_tool {
        ToolSlot::Builder if mouse.just_pressed(MouseButton::Left) => {
            let Some(position) = aim.voxel.and_then(|hit| hit.placement) else {
                return;
            };
            if !valid_placement(position, &loaded, &player, &blockers) {
                return;
            }
            let block = session.loadout.selected_block;
            if !session.loadout.consume_block(block) {
                return;
            }
            edits
                .edits
                .push(VoxelTerrainEdit::SetBlock { position, block });
            edits.placed_durability.insert(position, 120.0);
            session.loadout.blocks_placed = session.loadout.blocks_placed.saturating_add(1);
            invalidate_edit(&mut commands, &mut loaded, position, 1);
            sounds.write(GameSound::Build);
        }
        ToolSlot::MiningLaser => {
            if !mouse.pressed(MouseButton::Left) {
                mining.target = None;
                mining.progress = 0.0;
                return;
            }
            let Some(hit) = aim.voxel else {
                mining.target = None;
                mining.progress = 0.0;
                return;
            };
            if matches!(
                hit.kind,
                BlockKind::Bedrock | BlockKind::Water | BlockKind::Lava
            ) {
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
            edits.placed_durability.remove(&hit.block);
            invalidate_edit(&mut commands, &mut loaded, hit.block, 1);
            if let Some(resource) = ResourceKind::from_block(hit.kind) {
                session.loadout.add_resource(resource, 2);
                resource_events.write(ResourceCollected(resource, 2));
                sounds.write(GameSound::Resource);
            } else if hit.kind.is_solid() {
                session.loadout.add_block(hit.kind, 1);
                block_events.write(BlockCollected(hit.kind, 1));
                sounds.write(GameSound::Mine);
            }
            mining.target = None;
            mining.progress = 0.0;
        }
        ToolSlot::Weapon(_) => {
            mining.target = None;
            mining.progress = 0.0;
        }
        ToolSlot::Builder => {}
    }
}

fn valid_placement<F: bevy::ecs::query::QueryFilter>(
    position: VoxelBlockPosition,
    loaded: &LoadedVoxelChunks,
    player: &Query<&Transform, F>,
    blockers: &Query<(&GlobalTransform, &CombatTarget), Without<PlayerTag>>,
) -> bool {
    if loaded.block_at(position).is_some_and(BlockKind::is_solid) {
        return false;
    }
    let center = block_world_center(position);
    if player
        .single()
        .is_ok_and(|transform| transform.translation.distance(center) < BLOCK_SIZE * 1.15)
    {
        return false;
    }
    !blockers.iter().any(|(transform, target)| {
        transform.translation().distance(center) < target.radius + BLOCK_SIZE * 0.62
    })
}

pub(crate) fn ray_sphere_distance(
    origin: Vec3,
    direction: Vec3,
    center: Vec3,
    radius: f32,
) -> Option<f32> {
    let offset = origin - center;
    let b = offset.dot(direction);
    let c = offset.length_squared() - radius * radius;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return None;
    }
    let near = -b - discriminant.sqrt();
    let far = -b + discriminant.sqrt();
    if near >= 0.0 {
        Some(near)
    } else if far >= 0.0 {
        Some(far)
    } else {
        None
    }
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
    fn zero_direction_never_hits() {
        assert!(
            voxel_raycast_loaded(&LoadedVoxelChunks::default(), Vec3::ZERO, Vec3::ZERO, 10.0)
                .is_none()
        );
    }

    #[test]
    fn ray_sphere_uses_nearest_positive_hit() {
        let hit = ray_sphere_distance(Vec3::ZERO, Vec3::Z, Vec3::new(0.0, 0.0, 10.0), 2.0);
        assert_eq!(hit, Some(8.0));
    }

    #[test]
    fn block_center_round_trip() {
        let position = VoxelBlockPosition::new(-2, 7, 3);
        assert_eq!(world_to_block(block_world_center(position)), position);
    }
}
