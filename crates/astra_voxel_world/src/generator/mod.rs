use crate::{
    model::{
        BlockKind, DEFAULT_CHUNK_SIZE, VoxelBiome, VoxelChunk, VoxelChunkCoord, VoxelColumnSample,
        VoxelTerrainDiversityReport, VoxelTerrainFeaturePresence, VoxelTerrainFeatureWeights,
        VoxelWeather, VoxelWorldSettings,
    },
    noise::{fbm2, fbm3, hash3, ridged2, unit3},
};

const TERRAIN_CONTINENT_SCALE: f64 = 0.0014;
const TERRAIN_HILL_SCALE: f64 = 0.0085;
const TERRAIN_MOUNTAIN_SCALE: f64 = 0.0028;
const CLIMATE_SCALE: f64 = 0.0019;
const TERRAIN_ANOMALY_SCALE: f64 = 0.0034;
const WEATHER_SCALE: f64 = 0.0031;
const CAVE_SCALE: f64 = 0.045;
const TREE_MARGIN: i64 = 5;
const VOLCANIC_LAVA_SCALE: f64 = 0.014;
const VOLCANIC_ASH_SCALE: f64 = 0.028;

#[derive(Debug, Clone, Copy)]
struct CraterLayer {
    cell_size: f64,
    min_radius: f64,
    max_radius: f64,
    min_depth_ratio: f64,
    max_depth_ratio: f64,
    min_rim_ratio: f64,
    max_rim_ratio: f64,
    presence_bias: f64,
    salt: u64,
}

#[derive(Debug, Clone, Copy)]
struct LinearTerrainLayer {
    cell_size: f64,
    min_length: f64,
    max_length: f64,
    min_width: f64,
    max_width: f64,
    min_depth: f64,
    max_depth: f64,
    shoulder_ratio: f64,
    presence_bias: f64,
    salt: u64,
}

mod biomes;
mod column;
mod features;
mod reports;
mod resources;
mod weather;

pub use column::{
    generate_voxel_chunk, sample_voxel_column, surface_height_at, voxel_biome_at, voxel_weather_at,
};
pub use features::{terrain_feature_presence, terrain_feature_weights_for_biome};
pub use reports::voxel_terrain_diversity_report;
pub use resources::{
    VoxelSurfaceResource, VoxelVolcanicSurface, surface_resource_for_column,
    volcanic_surface_for_column,
};

fn climate_peak(value: f64, target: f64, width: f64) -> f64 {
    (1.0 - ((value - target).abs() / width.max(f64::EPSILON))).clamp(0.0, 1.0)
}

fn lerp(from: f64, to: f64, t: f64) -> f64 {
    from + (to - from) * t.clamp(0.0, 1.0)
}

fn smoothstep(edge0: f64, edge1: f64, value: f64) -> f64 {
    let t = ((value - edge0) / (edge1 - edge0).max(f64::EPSILON)).clamp(0.0, 1.0);

    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests;
