use super::features::terrain_feature_presence_for_biome;
use super::*;

pub fn voxel_terrain_diversity_report(
    settings: VoxelWorldSettings,
    half_extent: i64,
    step: usize,
) -> VoxelTerrainDiversityReport {
    let settings = settings.sanitized();
    let half_extent = half_extent.saturating_abs();
    let step = step.max(1);
    let mut biome_seen = [false; 10];
    let mut weather_seen = [false; 8];
    let mut min_height = i32::MAX;
    let mut max_height = i32::MIN;
    let mut sample_count = 0usize;
    let mut feature_totals = VoxelTerrainFeaturePresence::default();

    for world_z in (-half_extent..=half_extent).step_by(step) {
        for world_x in (-half_extent..=half_extent).step_by(step) {
            let column = sample_voxel_column(settings, world_x, world_z);
            let features =
                terrain_feature_presence_for_biome(settings, world_x, world_z, column.biome);

            min_height = min_height.min(column.height);
            max_height = max_height.max(column.height);
            biome_seen[biome_index(column.biome)] = true;
            weather_seen[weather_index(column.weather)] = true;
            feature_totals.craters += features.craters;
            feature_totals.large_craters += features.large_craters;
            feature_totals.rifts += features.rifts;
            feature_totals.canyons += features.canyons;
            feature_totals.high_mountains += features.high_mountains;
            feature_totals.plateaus += features.plateaus;
            feature_totals.erosion += features.erosion;
            sample_count += 1;
        }
    }

    if sample_count == 0 {
        return VoxelTerrainDiversityReport {
            sample_count,
            min_height: 0,
            max_height: 0,
            height_range: 0,
            distinct_biomes: 0,
            distinct_weather: 0,
            average_features: VoxelTerrainFeaturePresence::default(),
        };
    }

    let inv_samples = 1.0 / sample_count as f64;
    VoxelTerrainDiversityReport {
        sample_count,
        min_height,
        max_height,
        height_range: max_height - min_height,
        distinct_biomes: biome_seen.iter().filter(|seen| **seen).count(),
        distinct_weather: weather_seen.iter().filter(|seen| **seen).count(),
        average_features: VoxelTerrainFeaturePresence {
            craters: feature_totals.craters * inv_samples,
            large_craters: feature_totals.large_craters * inv_samples,
            rifts: feature_totals.rifts * inv_samples,
            canyons: feature_totals.canyons * inv_samples,
            high_mountains: feature_totals.high_mountains * inv_samples,
            plateaus: feature_totals.plateaus * inv_samples,
            erosion: feature_totals.erosion * inv_samples,
        },
    }
}

fn biome_index(biome: VoxelBiome) -> usize {
    match biome {
        VoxelBiome::Plains => 0,
        VoxelBiome::Forest => 1,
        VoxelBiome::Desert => 2,
        VoxelBiome::Tundra => 3,
        VoxelBiome::Mountains => 4,
        VoxelBiome::Wetlands => 5,
        VoxelBiome::Badlands => 6,
        VoxelBiome::CraterField => 7,
        VoxelBiome::Volcanic => 8,
        VoxelBiome::CrystalFields => 9,
    }
}

fn weather_index(weather: VoxelWeather) -> usize {
    match weather {
        VoxelWeather::Clear => 0,
        VoxelWeather::Cloudy => 1,
        VoxelWeather::Rain => 2,
        VoxelWeather::Storm => 3,
        VoxelWeather::Snow => 4,
        VoxelWeather::DustStorm => 5,
        VoxelWeather::Ashfall => 6,
        VoxelWeather::IonStorm => 7,
    }
}
