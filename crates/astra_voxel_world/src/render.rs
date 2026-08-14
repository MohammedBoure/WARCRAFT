use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use crate::{
    generator::sample_voxel_column,
    model::{
        BlockKind, DEFAULT_CHUNK_SIZE, VoxelChunk, VoxelChunkCoord, VoxelColumnSample,
        VoxelWorldSettings,
    },
    visual::{voxel_block_color, voxel_terrain_visual},
};

pub const VOXEL_VIEWER_BLOCK_SIZE: f32 = 4.0;
pub const VOXEL_VIEWER_HEIGHT_SCALE: f32 = 1.42;
pub const VOXEL_VIEWER_TERRAIN_EDGE_WIDTH_RATIO: f32 = 0.085;
pub const VOXEL_VIEWER_TERRAIN_EDGE_LIFT: f32 = 0.050;
pub const VOXEL_VIEWER_SLOPE_EDGE_HEIGHT_DELTA: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelSurfaceMeshStyle {
    pub block_size: f32,
    pub height_scale: f32,
    pub terrain_edge_width: f32,
    pub terrain_edge_lift: f32,
    pub slope_edge_height_delta: i32,
    pub resource_markers: bool,
    pub trees: bool,
}

impl Default for VoxelSurfaceMeshStyle {
    fn default() -> Self {
        Self::viewer()
    }
}

impl VoxelSurfaceMeshStyle {
    pub const fn viewer() -> Self {
        Self {
            block_size: VOXEL_VIEWER_BLOCK_SIZE,
            height_scale: VOXEL_VIEWER_HEIGHT_SCALE,
            terrain_edge_width: VOXEL_VIEWER_BLOCK_SIZE * VOXEL_VIEWER_TERRAIN_EDGE_WIDTH_RATIO,
            terrain_edge_lift: VOXEL_VIEWER_TERRAIN_EDGE_LIFT,
            slope_edge_height_delta: VOXEL_VIEWER_SLOPE_EDGE_HEIGHT_DELTA,
            resource_markers: true,
            trees: true,
        }
    }

    pub fn scaled(scale: f32) -> Self {
        let scale = scale.max(0.001);
        Self {
            block_size: VOXEL_VIEWER_BLOCK_SIZE * scale,
            height_scale: VOXEL_VIEWER_HEIGHT_SCALE * scale,
            terrain_edge_width: VOXEL_VIEWER_BLOCK_SIZE
                * VOXEL_VIEWER_TERRAIN_EDGE_WIDTH_RATIO
                * scale,
            terrain_edge_lift: VOXEL_VIEWER_TERRAIN_EDGE_LIFT * scale,
            ..Self::viewer()
        }
    }
}

pub fn voxel_chunk_world_translation(coord: VoxelChunkCoord, style: VoxelSurfaceMeshStyle) -> Vec3 {
    Vec3::new(
        coord.world_x(0) as f32 * style.block_size,
        0.0,
        coord.world_z(0) as f32 * style.block_size,
    )
}

pub fn voxel_surface_y_at(
    settings: VoxelWorldSettings,
    world_x: f32,
    world_z: f32,
    style: VoxelSurfaceMeshStyle,
) -> f32 {
    let column = sample_voxel_column(settings, world_x.round() as i64, world_z.round() as i64);

    voxel_display_height_for_column(settings, column) as f32 * style.height_scale
}

pub fn voxel_display_height_for_column(
    settings: VoxelWorldSettings,
    column: VoxelColumnSample,
) -> i32 {
    let settings = settings.sanitized();

    if column.height <= settings.sea_level {
        settings.sea_level
    } else {
        column.height
    }
}

pub fn voxel_display_height_for_chunk_column(
    settings: VoxelWorldSettings,
    chunk: &VoxelChunk,
    local_x: usize,
    local_z: usize,
) -> Option<i32> {
    let settings = settings.sanitized();
    let terrain_height = chunk.highest_terrain_y(local_x, local_z)?;

    Some(if terrain_height <= settings.sea_level {
        settings.sea_level
    } else {
        terrain_height
    })
}

pub fn voxel_chunk_surface_mesh(
    settings: VoxelWorldSettings,
    chunk: &VoxelChunk,
    mesh_step: usize,
    style: VoxelSurfaceMeshStyle,
) -> Mesh {
    let mut builder = VoxelChunkMeshBuilder::default();
    let mesh_step = mesh_step.clamp(1, 4);

    for local_z in (0..DEFAULT_CHUNK_SIZE).step_by(mesh_step) {
        for local_x in (0..DEFAULT_CHUNK_SIZE).step_by(mesh_step) {
            let cell_blocks = mesh_step
                .min(DEFAULT_CHUNK_SIZE - local_x)
                .min(DEFAULT_CHUNK_SIZE - local_z);
            append_column_surface(
                &mut builder,
                settings,
                chunk,
                local_x,
                local_z,
                cell_blocks,
                style,
            );
            if style.trees && mesh_step == 1 {
                append_tree_geometry(&mut builder, chunk, local_x, local_z, style);
            }
        }
    }

    builder.finish()
}

fn append_column_surface(
    builder: &mut VoxelChunkMeshBuilder,
    settings: VoxelWorldSettings,
    chunk: &VoxelChunk,
    local_x: usize,
    local_z: usize,
    cell_blocks: usize,
    style: VoxelSurfaceMeshStyle,
) {
    let sample_x = (local_x + cell_blocks / 2).min(DEFAULT_CHUNK_SIZE - 1);
    let sample_z = (local_z + cell_blocks / 2).min(DEFAULT_CHUNK_SIZE - 1);
    let world_x = chunk.world_x(sample_x);
    let world_z = chunk.world_z(sample_z);
    let column = sample_voxel_column(settings, world_x, world_z);
    let top_height = voxel_display_height_for_chunk_column(settings, chunk, sample_x, sample_z)
        .unwrap_or_else(|| voxel_display_height_for_column(settings, column));
    let y = top_height as f32 * style.height_scale;
    let x0 = local_x as f32 * style.block_size;
    let z0 = local_z as f32 * style.block_size;
    let cell_size = cell_blocks as f32 * style.block_size;
    let x1 = x0 + cell_size;
    let z1 = z0 + cell_size;
    let surface_block = chunk.get(sample_x, top_height as usize, sample_z);
    let visual = voxel_terrain_visual(settings, column);
    let top_color = surface_block
        .filter(|block| block.is_surface_resource())
        .map(voxel_block_color)
        .unwrap_or(visual.top_color);
    let color = lit_terrain_color(
        top_color,
        terrain_light_factor(settings, world_x, world_z, column),
    );

    builder.push_quad(
        [
            Vec3::new(x0, y, z0),
            Vec3::new(x1, y, z0),
            Vec3::new(x1, y, z1),
            Vec3::new(x0, y, z1),
        ],
        Vec3::Y,
        color,
    );

    if style.resource_markers
        && cell_blocks == 1
        && let Some(block) = surface_block.filter(|block| block.is_surface_resource())
    {
        append_surface_resource_marker(builder, block, x0, z0, y, world_x, world_z, style);
    }

    let mut bottom_solid_y = top_height;
    while bottom_solid_y > 0
        && chunk
            .get(sample_x, (bottom_solid_y - 1) as usize, sample_z)
            .is_some_and(|block| block.is_solid())
    {
        bottom_solid_y -= 1;
    }

    let sample_step = cell_blocks as i64;
    for (dx, dz, face) in [
        (-sample_step, 0, FaceDirection::West),
        (sample_step, 0, FaceDirection::East),
        (0, -sample_step, FaceDirection::North),
        (0, sample_step, FaceDirection::South),
    ] {
        let neighbor_column = sample_voxel_column(settings, world_x + dx, world_z + dz);
        let neighbor_height =
            neighbor_local(sample_x, sample_z, dx, dz).and_then(|(neighbor_x, neighbor_z)| {
                voxel_display_height_for_chunk_column(settings, chunk, neighbor_x, neighbor_z)
            });
        let neighbor_height = neighbor_height
            .unwrap_or_else(|| voxel_display_height_for_column(settings, neighbor_column));

        if should_draw_terrain_edge(
            settings,
            column,
            neighbor_column,
            top_height,
            neighbor_height,
            style,
        ) {
            append_top_edge(
                builder,
                face,
                x0,
                z0,
                cell_size,
                y + style.terrain_edge_lift,
                lit_terrain_color(visual.edge_color, 0.94),
                style,
            );
        }

        let effective_bottom = neighbor_height.max(bottom_solid_y);
        if effective_bottom < top_height {
            append_side_face(
                builder,
                face,
                x0,
                z0,
                cell_size,
                y,
                effective_bottom as f32 * style.height_scale,
                lit_terrain_color(visual.side_color, 0.95),
            );
        }
    }
}

fn neighbor_local(local_x: usize, local_z: usize, dx: i64, dz: i64) -> Option<(usize, usize)> {
    let neighbor_x = local_x as i64 + dx;
    let neighbor_z = local_z as i64 + dz;
    if (0..DEFAULT_CHUNK_SIZE as i64).contains(&neighbor_x)
        && (0..DEFAULT_CHUNK_SIZE as i64).contains(&neighbor_z)
    {
        Some((neighbor_x as usize, neighbor_z as usize))
    } else {
        None
    }
}

fn append_surface_resource_marker(
    builder: &mut VoxelChunkMeshBuilder,
    block: BlockKind,
    x0: f32,
    z0: f32,
    surface_y: f32,
    world_x: i64,
    world_z: i64,
    style: VoxelSurfaceMeshStyle,
) {
    let base_color = voxel_block_color(block);
    let block_size = style.block_size;
    let center = Vec3::new(x0 + block_size * 0.50, surface_y, z0 + block_size * 0.50);
    let jitter =
        visual_hash_2d(0xA57A_0FE5, world_x, world_z, block as u64) as f32 / u32::MAX as f32;
    let angle = jitter * std::f32::consts::TAU;
    let offset = Vec3::new(angle.cos(), 0.0, angle.sin()) * block_size * 0.11;

    match block {
        BlockKind::CrystalOre => {
            builder.push_box(
                center + offset + Vec3::Y * (block_size * 0.32),
                Vec3::new(block_size * 0.30, block_size * 0.64, block_size * 0.30),
                lit_terrain_color(base_color, 1.22),
            );
            builder.push_box(
                center - offset * 0.8 + Vec3::Y * (block_size * 0.22),
                Vec3::new(block_size * 0.20, block_size * 0.44, block_size * 0.20),
                lit_terrain_color(base_color, 0.95),
            );
        }
        BlockKind::HeliumVent => {
            builder.push_box(
                center + Vec3::Y * (block_size * 0.16),
                Vec3::new(block_size * 0.42, block_size * 0.32, block_size * 0.42),
                lit_terrain_color([0.20, 0.30, 0.36, 1.0], 0.90),
            );
            builder.push_box(
                center + Vec3::Y * (block_size * 0.43),
                Vec3::new(block_size * 0.20, block_size * 0.30, block_size * 0.20),
                lit_terrain_color(base_color, 1.20),
            );
        }
        BlockKind::BioPlasmaBloom => {
            for marker_offset in [
                Vec3::new(-0.20, 0.0, -0.08),
                Vec3::new(0.18, 0.0, 0.10),
                Vec3::new(0.02, 0.0, 0.22),
            ] {
                builder.push_box(
                    center + marker_offset * block_size + Vec3::Y * (block_size * 0.12),
                    Vec3::splat(block_size * 0.24),
                    lit_terrain_color(base_color, 1.12),
                );
            }
        }
        BlockKind::AncientRelic => {
            builder.push_box(
                center + Vec3::Y * (block_size * 0.38),
                Vec3::new(block_size * 0.34, block_size * 0.76, block_size * 0.24),
                lit_terrain_color(base_color, 0.95),
            );
            builder.push_box(
                center + Vec3::Y * (block_size * 0.82),
                Vec3::new(block_size * 0.46, block_size * 0.14, block_size * 0.34),
                lit_terrain_color(base_color, 1.20),
            );
        }
        BlockKind::TitaniumOre
        | BlockKind::UraniumOre
        | BlockKind::GoldOre
        | BlockKind::IronOre => {
            builder.push_box(
                center + offset + Vec3::Y * (block_size * 0.15),
                Vec3::new(block_size * 0.58, block_size * 0.30, block_size * 0.50),
                lit_terrain_color(base_color, 1.10),
            );
            builder.push_box(
                center - offset * 0.7 + Vec3::Y * (block_size * 0.29),
                Vec3::new(block_size * 0.38, block_size * 0.34, block_size * 0.32),
                lit_terrain_color(base_color, 0.92),
            );
        }
        BlockKind::CoalOre => {
            builder.push_box(
                center + Vec3::Y * (block_size * 0.10),
                Vec3::new(block_size * 0.62, block_size * 0.20, block_size * 0.54),
                lit_terrain_color(base_color, 1.08),
            );
        }
        _ => {}
    }
}

fn append_tree_geometry(
    builder: &mut VoxelChunkMeshBuilder,
    chunk: &VoxelChunk,
    local_x: usize,
    local_z: usize,
    style: VoxelSurfaceMeshStyle,
) {
    let Some(surface_y) = chunk.highest_terrain_y(local_x, local_z) else {
        return;
    };
    if chunk.get(local_x, (surface_y + 1) as usize, local_z) != Some(BlockKind::Wood) {
        return;
    }

    let block_size = style.block_size;
    let x = local_x as f32 * block_size + block_size * 0.50;
    let z = local_z as f32 * block_size + block_size * 0.50;
    let base_y = (surface_y + 1) as f32 * style.height_scale;
    builder.push_box(
        Vec3::new(x, base_y + 2.4 * style.height_scale, z),
        Vec3::new(
            block_size * 0.34,
            4.8 * style.height_scale,
            block_size * 0.34,
        ),
        voxel_block_color(BlockKind::Wood),
    );
    builder.push_box(
        Vec3::new(x, base_y + 6.2 * style.height_scale, z),
        Vec3::new(block_size * 2.15, block_size * 1.35, block_size * 2.15),
        voxel_block_color(BlockKind::Leaves),
    );
}

fn terrain_light_factor(
    settings: VoxelWorldSettings,
    world_x: i64,
    world_z: i64,
    column: VoxelColumnSample,
) -> f32 {
    let west = voxel_display_height_for_column(
        settings,
        sample_voxel_column(settings, world_x.saturating_sub(1), world_z),
    );
    let east = voxel_display_height_for_column(
        settings,
        sample_voxel_column(settings, world_x.saturating_add(1), world_z),
    );
    let north = voxel_display_height_for_column(
        settings,
        sample_voxel_column(settings, world_x, world_z.saturating_sub(1)),
    );
    let south = voxel_display_height_for_column(
        settings,
        sample_voxel_column(settings, world_x, world_z.saturating_add(1)),
    );
    let slope_normal = Vec3::new(
        (west - east) as f32 * 0.32,
        3.20,
        (north - south) as f32 * 0.32,
    )
    .normalize_or_zero();
    let sun_dir = Vec3::new(-0.42, 0.76, -0.34).normalize();
    let direct_light = slope_normal.dot(sun_dir).max(0.0);
    let altitude = ((column.height - settings.sea_level) as f32 / 80.0).clamp(0.0, 1.0);
    let micro_variation =
        visual_hash_2d(settings.seed, world_x, world_z, 0x51A_D0C) as f32 / u32::MAX as f32;

    (0.70 + direct_light * 0.34 + altitude * 0.08 + (micro_variation - 0.5) * 0.08)
        .clamp(0.58, 1.18)
}

fn lit_terrain_color(color: [f32; 4], light: f32) -> [f32; 4] {
    [
        (color[0] * light).clamp(0.0, 1.0),
        (color[1] * light).clamp(0.0, 1.0),
        (color[2] * light).clamp(0.0, 1.0),
        color[3],
    ]
}

fn visual_hash_2d(seed: u64, x: i64, z: i64, salt: u64) -> u32 {
    let mut value = seed
        ^ salt
        ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (z as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;

    (value >> 32) as u32
}

fn should_draw_terrain_edge(
    settings: VoxelWorldSettings,
    column: VoxelColumnSample,
    neighbor: VoxelColumnSample,
    top_height: i32,
    neighbor_height: i32,
    style: VoxelSurfaceMeshStyle,
) -> bool {
    let current_visual = voxel_terrain_visual(settings, column);
    let neighbor_visual = voxel_terrain_visual(settings, neighbor);
    let height_delta = top_height.abs_diff(neighbor_height);

    height_delta >= style.slope_edge_height_delta.max(0) as u32
        || column.biome != neighbor.biome
        || current_visual.is_water != neighbor_visual.is_water
}

#[derive(Debug, Clone, Copy)]
enum FaceDirection {
    North,
    South,
    East,
    West,
}

fn append_side_face(
    builder: &mut VoxelChunkMeshBuilder,
    face: FaceDirection,
    x0: f32,
    z0: f32,
    cell_size: f32,
    top: f32,
    bottom: f32,
    color: [f32; 4],
) {
    let x1 = x0 + cell_size;
    let z1 = z0 + cell_size;
    let shade = match face {
        FaceDirection::North | FaceDirection::West => 0.94,
        FaceDirection::South | FaceDirection::East => 0.78,
    };
    let shaded = [
        color[0] * shade,
        color[1] * shade,
        color[2] * shade,
        color[3],
    ];

    match face {
        FaceDirection::North => builder.push_quad(
            [
                Vec3::new(x1, bottom, z0),
                Vec3::new(x0, bottom, z0),
                Vec3::new(x0, top, z0),
                Vec3::new(x1, top, z0),
            ],
            Vec3::NEG_Z,
            shaded,
        ),
        FaceDirection::South => builder.push_quad(
            [
                Vec3::new(x0, bottom, z1),
                Vec3::new(x1, bottom, z1),
                Vec3::new(x1, top, z1),
                Vec3::new(x0, top, z1),
            ],
            Vec3::Z,
            shaded,
        ),
        FaceDirection::East => builder.push_quad(
            [
                Vec3::new(x1, bottom, z1),
                Vec3::new(x1, bottom, z0),
                Vec3::new(x1, top, z0),
                Vec3::new(x1, top, z1),
            ],
            Vec3::X,
            shaded,
        ),
        FaceDirection::West => builder.push_quad(
            [
                Vec3::new(x0, bottom, z0),
                Vec3::new(x0, bottom, z1),
                Vec3::new(x0, top, z1),
                Vec3::new(x0, top, z0),
            ],
            Vec3::NEG_X,
            shaded,
        ),
    }
}

fn append_top_edge(
    builder: &mut VoxelChunkMeshBuilder,
    face: FaceDirection,
    x0: f32,
    z0: f32,
    cell_size: f32,
    y: f32,
    color: [f32; 4],
    style: VoxelSurfaceMeshStyle,
) {
    let x1 = x0 + cell_size;
    let z1 = z0 + cell_size;
    let width = style.terrain_edge_width.min(cell_size * 0.25);

    let vertices = match face {
        FaceDirection::North => [
            Vec3::new(x0, y, z0),
            Vec3::new(x1, y, z0),
            Vec3::new(x1, y, z0 + width),
            Vec3::new(x0, y, z0 + width),
        ],
        FaceDirection::South => [
            Vec3::new(x0, y, z1 - width),
            Vec3::new(x1, y, z1 - width),
            Vec3::new(x1, y, z1),
            Vec3::new(x0, y, z1),
        ],
        FaceDirection::East => [
            Vec3::new(x1 - width, y, z0),
            Vec3::new(x1, y, z0),
            Vec3::new(x1, y, z1),
            Vec3::new(x1 - width, y, z1),
        ],
        FaceDirection::West => [
            Vec3::new(x0, y, z0),
            Vec3::new(x0 + width, y, z0),
            Vec3::new(x0 + width, y, z1),
            Vec3::new(x0, y, z1),
        ],
    };

    builder.push_quad(vertices, Vec3::Y, color);
}

#[derive(Default)]
struct VoxelChunkMeshBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl VoxelChunkMeshBuilder {
    fn push_quad(&mut self, vertices: [Vec3; 4], normal: Vec3, color: [f32; 4]) {
        let start = self.positions.len() as u32;
        let face_normal = (vertices[1] - vertices[0]).cross(vertices[2] - vertices[0]);
        let indices = if face_normal.dot(normal) < 0.0 {
            [start, start + 2, start + 1, start, start + 3, start + 2]
        } else {
            [start, start + 1, start + 2, start, start + 2, start + 3]
        };

        self.positions
            .extend(vertices.map(|vertex| vertex.to_array()));
        self.normals.extend([normal.to_array(); 4]);
        self.colors.extend([color; 4]);
        self.uvs
            .extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        self.indices.extend_from_slice(&indices);
    }

    fn push_box(&mut self, center: Vec3, size: Vec3, color: [f32; 4]) {
        let half = size * 0.5;
        let min = center - half;
        let max = center + half;
        let faces = [
            (
                Vec3::Y,
                [
                    Vec3::new(min.x, max.y, min.z),
                    Vec3::new(max.x, max.y, min.z),
                    Vec3::new(max.x, max.y, max.z),
                    Vec3::new(min.x, max.y, max.z),
                ],
            ),
            (
                Vec3::NEG_Y,
                [
                    Vec3::new(min.x, min.y, max.z),
                    Vec3::new(max.x, min.y, max.z),
                    Vec3::new(max.x, min.y, min.z),
                    Vec3::new(min.x, min.y, min.z),
                ],
            ),
            (
                Vec3::Z,
                [
                    Vec3::new(min.x, min.y, max.z),
                    Vec3::new(min.x, max.y, max.z),
                    Vec3::new(max.x, max.y, max.z),
                    Vec3::new(max.x, min.y, max.z),
                ],
            ),
            (
                Vec3::NEG_Z,
                [
                    Vec3::new(max.x, min.y, min.z),
                    Vec3::new(max.x, max.y, min.z),
                    Vec3::new(min.x, max.y, min.z),
                    Vec3::new(min.x, min.y, min.z),
                ],
            ),
            (
                Vec3::X,
                [
                    Vec3::new(max.x, min.y, max.z),
                    Vec3::new(max.x, max.y, max.z),
                    Vec3::new(max.x, max.y, min.z),
                    Vec3::new(max.x, min.y, min.z),
                ],
            ),
            (
                Vec3::NEG_X,
                [
                    Vec3::new(min.x, min.y, min.z),
                    Vec3::new(min.x, max.y, min.z),
                    Vec3::new(min.x, max.y, max.z),
                    Vec3::new(min.x, min.y, max.z),
                ],
            ),
        ];

        for (normal, vertices) in faces {
            self.push_quad(vertices, normal, color);
        }
    }

    fn finish(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        edit::{VoxelTerrainEdit, apply_terrain_edits},
        generator::generate_voxel_chunk,
        model::{BlockKind, VoxelChunkCoord, VoxelWorldSettings},
    };

    #[test]
    fn shared_surface_mesh_matches_viewer_scale_defaults() {
        let style = VoxelSurfaceMeshStyle::viewer();

        assert_eq!(style.block_size, VOXEL_VIEWER_BLOCK_SIZE);
        assert_eq!(style.height_scale, VOXEL_VIEWER_HEIGHT_SCALE);
        assert!(style.resource_markers);
        assert!(style.trees);
    }

    #[test]
    fn viewer_style_keeps_terrain_blocks_visibly_raised() {
        let style = VoxelSurfaceMeshStyle::viewer();

        assert!(style.height_scale > 1.30);
        assert!(style.terrain_edge_width >= style.block_size * 0.08);
        assert!(style.terrain_edge_lift > 0.0);
    }

    #[test]
    fn shared_surface_mesh_builds_renderable_chunk() {
        let settings = VoxelWorldSettings::default();
        let chunk = generate_voxel_chunk(settings, VoxelChunkCoord::ZERO);
        let mesh = voxel_chunk_surface_mesh(settings, &chunk, 1, VoxelSurfaceMeshStyle::viewer());

        assert!(mesh.count_vertices() >= DEFAULT_CHUNK_SIZE * DEFAULT_CHUNK_SIZE * 4);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some());
        assert!(mesh.indices().is_some());
    }

    #[test]
    fn scaled_style_preserves_viewer_proportions() {
        let scale = 800.0;
        let style = VoxelSurfaceMeshStyle::scaled(scale);
        let coord = VoxelChunkCoord::new(2, -3);
        let translation = voxel_chunk_world_translation(coord, style);

        assert_eq!(style.block_size, VOXEL_VIEWER_BLOCK_SIZE * scale);
        assert_eq!(style.height_scale, VOXEL_VIEWER_HEIGHT_SCALE * scale);
        assert_eq!(
            translation.x,
            coord.world_x(0) as f32 * VOXEL_VIEWER_BLOCK_SIZE * scale
        );
        assert_eq!(
            translation.z,
            coord.world_z(0) as f32 * VOXEL_VIEWER_BLOCK_SIZE * scale
        );
    }

    #[test]
    fn surface_mesh_reads_edited_chunk_height() {
        let settings = VoxelWorldSettings::default();
        let mut chunk = generate_voxel_chunk(settings, VoxelChunkCoord::ZERO);
        let target_y = settings.base_height;
        apply_terrain_edits(
            &mut chunk,
            &[VoxelTerrainEdit::FlattenDisk {
                center_x: 8,
                center_z: 8,
                radius: 3,
                target_y,
                surface_block: BlockKind::Stone,
                fill_block: BlockKind::Dirt,
            }],
        );

        assert_eq!(
            voxel_display_height_for_chunk_column(settings, &chunk, 8, 8),
            Some(target_y)
        );

        let mesh = voxel_chunk_surface_mesh(settings, &chunk, 1, VoxelSurfaceMeshStyle::viewer());
        assert!(mesh.count_vertices() >= DEFAULT_CHUNK_SIZE * DEFAULT_CHUNK_SIZE * 4);
    }
}
