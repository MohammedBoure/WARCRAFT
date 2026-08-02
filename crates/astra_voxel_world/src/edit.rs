use crate::{
    generator::generate_voxel_chunk,
    model::{BlockKind, DEFAULT_CHUNK_SIZE, VoxelChunk, VoxelChunkCoord, VoxelWorldSettings},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VoxelBlockPosition {
    pub x: i64,
    pub y: i32,
    pub z: i64,
}

impl VoxelBlockPosition {
    pub const fn new(x: i64, y: i32, z: i64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelTerrainEdit {
    SetBlock {
        position: VoxelBlockPosition,
        block: BlockKind,
    },
    DigSphere {
        center: VoxelBlockPosition,
        radius: u16,
    },
    FillSphere {
        center: VoxelBlockPosition,
        radius: u16,
        block: BlockKind,
    },
    FlattenDisk {
        center_x: i64,
        center_z: i64,
        radius: u16,
        target_y: i32,
        surface_block: BlockKind,
        fill_block: BlockKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VoxelTerrainEditSummary {
    pub changed_blocks: usize,
    pub affected_columns: usize,
}

pub fn generate_edited_voxel_chunk(
    settings: VoxelWorldSettings,
    coord: VoxelChunkCoord,
    edits: &[VoxelTerrainEdit],
) -> VoxelChunk {
    let mut chunk = generate_voxel_chunk(settings, coord);
    apply_terrain_edits(&mut chunk, edits);
    chunk
}

pub fn apply_terrain_edits(
    chunk: &mut VoxelChunk,
    edits: &[VoxelTerrainEdit],
) -> VoxelTerrainEditSummary {
    let mut summary = VoxelTerrainEditSummary::default();

    for edit in edits {
        match *edit {
            VoxelTerrainEdit::SetBlock { position, block } => {
                if position.y > 0
                    && position.y < chunk.world_height() as i32
                    && chunk.set_world(position.x, position.y, position.z, block)
                {
                    summary.changed_blocks += 1;
                    summary.affected_columns += 1;
                }
            }
            VoxelTerrainEdit::DigSphere { center, radius } => {
                summary += apply_sphere_edit(chunk, center, radius, BlockKind::Air);
            }
            VoxelTerrainEdit::FillSphere {
                center,
                radius,
                block,
            } => {
                if block.is_solid() {
                    summary += apply_sphere_edit(chunk, center, radius, block);
                }
            }
            VoxelTerrainEdit::FlattenDisk {
                center_x,
                center_z,
                radius,
                target_y,
                surface_block,
                fill_block,
            } => {
                if surface_block.is_solid() && fill_block.is_solid() {
                    summary += apply_flatten_disk(
                        chunk,
                        center_x,
                        center_z,
                        radius,
                        target_y,
                        surface_block,
                        fill_block,
                    );
                }
            }
        }
    }

    summary
}

fn apply_sphere_edit(
    chunk: &mut VoxelChunk,
    center: VoxelBlockPosition,
    radius: u16,
    replacement: BlockKind,
) -> VoxelTerrainEditSummary {
    let radius = i32::from(radius);
    if radius == 0 {
        return VoxelTerrainEditSummary::default();
    }

    let radius_sq = i64::from(radius) * i64::from(radius);
    let mut summary = VoxelTerrainEditSummary::default();
    let mut touched_columns = [[false; DEFAULT_CHUNK_SIZE]; DEFAULT_CHUNK_SIZE];
    let min_y = (center.y - radius).max(1);
    let max_y = (center.y + radius).min(chunk.world_height() as i32 - 1);

    for local_z in 0..DEFAULT_CHUNK_SIZE {
        let world_z = chunk.world_z(local_z);
        let dz = world_z - center.z;
        for local_x in 0..DEFAULT_CHUNK_SIZE {
            let world_x = chunk.world_x(local_x);
            let dx = world_x - center.x;
            let horizontal_sq = dx * dx + dz * dz;
            if horizontal_sq > radius_sq {
                continue;
            }

            for y in min_y..=max_y {
                let dy = i64::from(y - center.y);
                if horizontal_sq + dy * dy > radius_sq {
                    continue;
                }
                if set_if_changed(chunk, local_x, y as usize, local_z, replacement) {
                    summary.changed_blocks += 1;
                    touched_columns[local_z][local_x] = true;
                }
            }
        }
    }

    summary.affected_columns = count_touched_columns(touched_columns);
    summary
}

fn apply_flatten_disk(
    chunk: &mut VoxelChunk,
    center_x: i64,
    center_z: i64,
    radius: u16,
    target_y: i32,
    surface_block: BlockKind,
    fill_block: BlockKind,
) -> VoxelTerrainEditSummary {
    let radius = i64::from(radius);
    if radius == 0 || target_y <= 0 || target_y >= chunk.world_height() as i32 {
        return VoxelTerrainEditSummary::default();
    }

    let radius_sq = radius * radius;
    let mut summary = VoxelTerrainEditSummary::default();
    let mut touched_columns = [[false; DEFAULT_CHUNK_SIZE]; DEFAULT_CHUNK_SIZE];

    for local_z in 0..DEFAULT_CHUNK_SIZE {
        let world_z = chunk.world_z(local_z);
        let dz = world_z - center_z;
        for local_x in 0..DEFAULT_CHUNK_SIZE {
            let world_x = chunk.world_x(local_x);
            let dx = world_x - center_x;
            if dx * dx + dz * dz > radius_sq {
                continue;
            }

            let current_top = chunk.highest_terrain_y(local_x, local_z).unwrap_or(0);
            if current_top > target_y {
                for y in (target_y + 1)..=current_top {
                    if set_if_changed(chunk, local_x, y as usize, local_z, BlockKind::Air) {
                        summary.changed_blocks += 1;
                        touched_columns[local_z][local_x] = true;
                    }
                }
            } else if current_top < target_y {
                for y in (current_top + 1).max(1)..target_y {
                    if set_if_changed(chunk, local_x, y as usize, local_z, fill_block) {
                        summary.changed_blocks += 1;
                        touched_columns[local_z][local_x] = true;
                    }
                }
            }

            if set_if_changed(chunk, local_x, target_y as usize, local_z, surface_block) {
                summary.changed_blocks += 1;
                touched_columns[local_z][local_x] = true;
            }
        }
    }

    summary.affected_columns = count_touched_columns(touched_columns);
    summary
}

fn set_if_changed(
    chunk: &mut VoxelChunk,
    local_x: usize,
    y: usize,
    local_z: usize,
    block: BlockKind,
) -> bool {
    if chunk.get(local_x, y, local_z) == Some(block) {
        return false;
    }

    chunk.set(local_x, y, local_z, block)
}

fn count_touched_columns(
    touched_columns: [[bool; DEFAULT_CHUNK_SIZE]; DEFAULT_CHUNK_SIZE],
) -> usize {
    touched_columns
        .iter()
        .flat_map(|row| row.iter())
        .filter(|touched| **touched)
        .count()
}

impl core::ops::AddAssign for VoxelTerrainEditSummary {
    fn add_assign(&mut self, rhs: Self) {
        self.changed_blocks += rhs.changed_blocks;
        self.affected_columns += rhs.affected_columns;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dig_sphere_removes_blocks_without_touching_bedrock() {
        let settings = VoxelWorldSettings::default();
        let coord = VoxelChunkCoord::ZERO;
        let mut chunk = generate_voxel_chunk(settings, coord);
        let target_y = chunk
            .highest_terrain_y(8, 8)
            .expect("generated surface")
            .saturating_sub(2)
            .max(2);

        let summary = apply_terrain_edits(
            &mut chunk,
            &[VoxelTerrainEdit::DigSphere {
                center: VoxelBlockPosition::new(8, target_y, 8),
                radius: 3,
            }],
        );

        assert!(summary.changed_blocks > 0);
        assert!(summary.affected_columns > 0);
        assert_eq!(chunk.get(8, target_y as usize, 8), Some(BlockKind::Air));
        assert_eq!(chunk.get(8, 0, 8), Some(BlockKind::Bedrock));
    }

    #[test]
    fn flatten_disk_creates_level_build_pad() {
        let settings = VoxelWorldSettings::default();
        let mut chunk = generate_voxel_chunk(settings, VoxelChunkCoord::ZERO);
        let target_y = settings.base_height;

        let summary = apply_terrain_edits(
            &mut chunk,
            &[VoxelTerrainEdit::FlattenDisk {
                center_x: 8,
                center_z: 8,
                radius: 4,
                target_y,
                surface_block: BlockKind::Stone,
                fill_block: BlockKind::Dirt,
            }],
        );

        assert!(summary.changed_blocks > 0);
        for local_z in 5..=11 {
            for local_x in 5..=11 {
                let dx = local_x as i64 - 8;
                let dz = local_z as i64 - 8;
                if dx * dx + dz * dz <= 16 {
                    assert_eq!(chunk.highest_terrain_y(local_x, local_z), Some(target_y));
                    assert_eq!(
                        chunk.get(local_x, target_y as usize, local_z),
                        Some(BlockKind::Stone)
                    );
                }
            }
        }
    }

    #[test]
    fn edited_chunk_generation_is_deterministic() {
        let settings = VoxelWorldSettings::default();
        let coord = VoxelChunkCoord::new(3, -2);
        let edits = [
            VoxelTerrainEdit::DigSphere {
                center: VoxelBlockPosition::new(coord.world_x(7), 64, coord.world_z(6)),
                radius: 5,
            },
            VoxelTerrainEdit::FlattenDisk {
                center_x: coord.world_x(9),
                center_z: coord.world_z(9),
                radius: 4,
                target_y: 70,
                surface_block: BlockKind::Basalt,
                fill_block: BlockKind::Stone,
            },
        ];

        let first = generate_edited_voxel_chunk(settings, coord, &edits);
        let second = generate_edited_voxel_chunk(settings, coord, &edits);

        assert_eq!(first, second);
    }
}
