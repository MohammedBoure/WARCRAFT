use super::*;
use super::{
    biomes::{biome_from_climate, biome_height_adjustment, terraced_height_for_biome},
    features::{
        crater_coastal_dampening, crater_height_delta, entry_extremity_dampening,
        erosion_adjusted_height, feature_plateau_terraced_height, high_mountain_delta,
        plateau_height_delta, rift_coastal_dampening, rift_height_delta,
    },
    resources::{ore_for_block, should_carve_cave},
    weather::weather_from_climate,
};

pub fn generate_voxel_chunk(settings: VoxelWorldSettings, coord: VoxelChunkCoord) -> VoxelChunk {
    let settings = settings.sanitized();
    let mut chunk = VoxelChunk::new(coord, settings.height_usize());

    for local_z in 0..DEFAULT_CHUNK_SIZE {
        for local_x in 0..DEFAULT_CHUNK_SIZE {
            let world_x = coord.world_x(local_x);
            let world_z = coord.world_z(local_z);
            let column = sample_voxel_column(settings, world_x, world_z);
            fill_column(&mut chunk, settings, local_x, local_z, column);
        }
    }

    add_trees(&mut chunk, settings);
    chunk
}

pub fn sample_voxel_column(
    settings: VoxelWorldSettings,
    world_x: i64,
    world_z: i64,
) -> VoxelColumnSample {
    let settings = settings.sanitized();
    let x = world_x as f64;
    let z = world_z as f64;
    let continent = fbm2(
        settings.seed,
        x * TERRAIN_CONTINENT_SCALE,
        z * TERRAIN_CONTINENT_SCALE,
        5,
        11,
    );
    let hills = fbm2(
        settings.seed,
        x * TERRAIN_HILL_SCALE,
        z * TERRAIN_HILL_SCALE,
        4,
        29,
    );
    let mountain_ridges = ridged2(
        settings.seed,
        x * TERRAIN_MOUNTAIN_SCALE,
        z * TERRAIN_MOUNTAIN_SCALE,
        4,
        47,
    );
    let temperature = fbm2(settings.seed, x * CLIMATE_SCALE, z * CLIMATE_SCALE, 3, 71);
    let moisture = fbm2(
        settings.seed,
        x * CLIMATE_SCALE * 1.17,
        z * CLIMATE_SCALE * 1.17,
        3,
        89,
    );
    let crater_field = ridged2(
        settings.seed,
        x * TERRAIN_ANOMALY_SCALE * 0.74,
        z * TERRAIN_ANOMALY_SCALE * 0.74,
        3,
        157,
    );
    let volcanic_field = ridged2(
        settings.seed,
        x * TERRAIN_ANOMALY_SCALE * 0.58,
        z * TERRAIN_ANOMALY_SCALE * 0.58,
        4,
        173,
    );
    let crystal_field = fbm2(
        settings.seed,
        x * TERRAIN_ANOMALY_SCALE * 1.31,
        z * TERRAIN_ANOMALY_SCALE * 1.31,
        3,
        191,
    );
    let biome_roll = fbm2(
        settings.seed,
        x * TERRAIN_ANOMALY_SCALE * 0.42,
        z * TERRAIN_ANOMALY_SCALE * 0.42,
        3,
        223,
    );
    let weather_field = fbm2(settings.seed, x * WEATHER_SCALE, z * WEATHER_SCALE, 3, 211);
    let mountain_factor =
        smoothstep(0.48, 0.86, mountain_ridges) * smoothstep(0.28, 0.74, continent);
    let broad_height = (continent - 0.48) * f64::from(settings.terrain_amplitude);
    let hill_height = (hills - 0.45) * f64::from(settings.terrain_amplitude) * 0.42;
    let mountain_height = mountain_factor.powf(2.25) * f64::from(settings.mountain_amplitude);
    let high_mountain_height =
        high_mountain_delta(settings, world_x, world_z, mountain_ridges, continent)
            * entry_extremity_dampening(world_x, world_z);
    let mut height = f64::from(settings.base_height)
        + broad_height
        + hill_height
        + mountain_height
        + high_mountain_height;
    let biome = biome_from_climate(
        settings,
        biome_roll,
        height,
        temperature,
        moisture,
        mountain_factor,
        crater_field,
        volcanic_field,
        crystal_field,
    );

    height += biome_height_adjustment(settings, biome, hills, crater_field, volcanic_field);
    height += plateau_height_delta(settings, world_x, world_z, biome)
        * entry_extremity_dampening(world_x, world_z);
    height =
        terraced_height_for_biome(biome, height, mountain_factor, crater_field, volcanic_field);
    height = feature_plateau_terraced_height(settings, biome, height, world_x, world_z);
    height = erosion_adjusted_height(
        settings,
        biome,
        f64::from(settings.base_height) + broad_height * 0.85,
        height,
        world_x,
        world_z,
    );
    height += crater_height_delta(settings, world_x, world_z, biome)
        * crater_coastal_dampening(settings, height);
    height += rift_height_delta(settings, world_x, world_z, biome, volcanic_field)
        * rift_coastal_dampening(settings, height);

    let max_height = i32::from(settings.world_height).saturating_sub(5);
    let height = height.round().clamp(4.0, f64::from(max_height)) as i32;
    let weather = weather_from_climate(
        settings,
        biome,
        temperature,
        moisture,
        mountain_factor,
        weather_field,
    );

    VoxelColumnSample {
        world_x,
        world_z,
        height,
        biome,
        weather,
        temperature,
        moisture,
        mountain_factor,
    }
}

pub fn surface_height_at(settings: VoxelWorldSettings, world_x: i64, world_z: i64) -> i32 {
    sample_voxel_column(settings, world_x, world_z).height
}

pub fn voxel_biome_at(settings: VoxelWorldSettings, world_x: i64, world_z: i64) -> VoxelBiome {
    sample_voxel_column(settings, world_x, world_z).biome
}

pub fn voxel_weather_at(settings: VoxelWorldSettings, world_x: i64, world_z: i64) -> VoxelWeather {
    sample_voxel_column(settings, world_x, world_z).weather
}

fn fill_column(
    chunk: &mut VoxelChunk,
    settings: VoxelWorldSettings,
    local_x: usize,
    local_z: usize,
    column: VoxelColumnSample,
) {
    let max_y = settings.height_usize().saturating_sub(1);
    for y in 0..=max_y {
        let y_i32 = y as i32;
        let mut block = base_block_for_y(settings, column, y_i32);

        if block == BlockKind::Stone && should_carve_cave(settings, column, y_i32) {
            block = BlockKind::Air;
        }
        if matches!(block, BlockKind::Stone | BlockKind::Basalt) {
            block = ore_for_block(settings, column, y_i32).unwrap_or(block);
        }

        chunk.set(local_x, y, local_z, block);
    }
}

fn base_block_for_y(settings: VoxelWorldSettings, column: VoxelColumnSample, y: i32) -> BlockKind {
    if y == 0 {
        return BlockKind::Bedrock;
    }
    if y > column.height {
        return if y <= settings.sea_level {
            if y == settings.sea_level && column.biome == VoxelBiome::Tundra {
                BlockKind::Ice
            } else {
                BlockKind::Water
            }
        } else {
            BlockKind::Air
        };
    }
    if y == column.height {
        if let Some(resource) = surface_resource_for_column(settings, column) {
            return resource.block();
        }

        return match column.biome {
            VoxelBiome::Desert => BlockKind::Sand,
            VoxelBiome::Tundra => BlockKind::Snow,
            VoxelBiome::Mountains if column.height > settings.sea_level + 24 => BlockKind::Stone,
            VoxelBiome::Wetlands => BlockKind::Mud,
            VoxelBiome::Badlands => BlockKind::Sand,
            VoxelBiome::CraterField => BlockKind::Stone,
            VoxelBiome::Volcanic => match volcanic_surface_for_column(settings, column) {
                VoxelVolcanicSurface::Basalt => BlockKind::Basalt,
                VoxelVolcanicSurface::Ash => BlockKind::VolcanicAsh,
                VoxelVolcanicSurface::Lava => BlockKind::Lava,
            },
            VoxelBiome::CrystalFields => {
                if settings.composition.resource_ratios.crystal > 0.0
                    && unit3(
                        settings.seed,
                        column.world_x,
                        i64::from(y),
                        column.world_z,
                        0xC7,
                    ) > 0.94
                {
                    BlockKind::CrystalOre
                } else {
                    BlockKind::Stone
                }
            }
            _ => BlockKind::Grass,
        };
    }
    if y >= column.height - 4 {
        return match column.biome {
            VoxelBiome::Desert => BlockKind::Sand,
            VoxelBiome::Wetlands => BlockKind::Mud,
            VoxelBiome::Badlands if y >= column.height - 2 => BlockKind::Sand,
            VoxelBiome::Mountains if y < column.height - 1 => BlockKind::Stone,
            VoxelBiome::CraterField | VoxelBiome::CrystalFields => BlockKind::Stone,
            VoxelBiome::Volcanic => match volcanic_surface_for_column(settings, column) {
                VoxelVolcanicSurface::Ash if y >= column.height - 2 => BlockKind::VolcanicAsh,
                _ => BlockKind::Basalt,
            },
            _ => BlockKind::Dirt,
        };
    }

    BlockKind::Stone
}

fn add_trees(chunk: &mut VoxelChunk, settings: VoxelWorldSettings) {
    let start_x = chunk.coord().world_x(0) - TREE_MARGIN;
    let start_z = chunk.coord().world_z(0) - TREE_MARGIN;
    let end_x = chunk.coord().world_x(DEFAULT_CHUNK_SIZE - 1) + TREE_MARGIN;
    let end_z = chunk.coord().world_z(DEFAULT_CHUNK_SIZE - 1) + TREE_MARGIN;

    for world_z in start_z..=end_z {
        for world_x in start_x..=end_x {
            let column = sample_voxel_column(settings, world_x, world_z);
            if !tree_can_spawn(settings, column) {
                continue;
            }
            place_tree(chunk, settings, world_x, column.height + 1, world_z);
        }
    }
}

fn tree_can_spawn(settings: VoxelWorldSettings, column: VoxelColumnSample) -> bool {
    if column.height <= settings.sea_level {
        return false;
    }
    let biome_multiplier = match column.biome {
        VoxelBiome::Forest => 1.0,
        VoxelBiome::Wetlands => 0.45,
        VoxelBiome::Plains => 0.28,
        _ => 0.0,
    };
    if biome_multiplier <= 0.0 {
        return false;
    }

    let sparse_grid = hash3(
        settings.seed,
        column.world_x / 2,
        0,
        column.world_z / 2,
        0x7AEE,
    );
    if sparse_grid & 0b11 != 0 {
        return false;
    }

    let chance = settings.tree_density * biome_multiplier;
    unit3(settings.seed, column.world_x, 0, column.world_z, 0x7AEE) < chance
}

fn place_tree(
    chunk: &mut VoxelChunk,
    settings: VoxelWorldSettings,
    trunk_x: i64,
    base_y: i32,
    trunk_z: i64,
) {
    let tree_height =
        4 + (hash3(settings.seed, trunk_x, i64::from(base_y), trunk_z, 0x71) % 3) as i32;

    for dy in 0..tree_height {
        chunk.set_world(trunk_x, base_y + dy, trunk_z, BlockKind::Wood);
    }

    let leaf_base = base_y + tree_height - 2;
    for dy in 0..4 {
        let radius = if dy == 3 { 1 } else { 2 };
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let distance = dx * dx + dz * dz + (dy - 1) * (dy - 1);
                if distance > radius * radius + 1 {
                    continue;
                }
                let wx = trunk_x + i64::from(dx);
                let wy = leaf_base + dy;
                let wz = trunk_z + i64::from(dz);
                let existing = if wy >= 0 {
                    chunk.get_world(wx, wy, wz)
                } else {
                    None
                };
                if matches!(existing, Some(BlockKind::Air | BlockKind::Water) | None) {
                    chunk.set_world(wx, wy, wz, BlockKind::Leaves);
                }
            }
        }
    }
}

trait VoxelChunkWorldAccess {
    fn get_world(&self, world_x: i64, y: i32, world_z: i64) -> Option<BlockKind>;
}

impl VoxelChunkWorldAccess for VoxelChunk {
    fn get_world(&self, world_x: i64, y: i32, world_z: i64) -> Option<BlockKind> {
        if y < 0 {
            return None;
        }
        let local_x = world_x.saturating_sub(self.coord().world_x(0));
        let local_z = world_z.saturating_sub(self.coord().world_z(0));
        if !(0..DEFAULT_CHUNK_SIZE as i64).contains(&local_x)
            || !(0..DEFAULT_CHUNK_SIZE as i64).contains(&local_z)
        {
            return None;
        }

        self.get(local_x as usize, y as usize, local_z as usize)
    }
}
