use super::*;

pub(super) fn biome_from_climate(
    settings: VoxelWorldSettings,
    biome_roll: f64,
    height: f64,
    temperature: f64,
    moisture: f64,
    mountain_factor: f64,
    crater_field: f64,
    volcanic_field: f64,
    crystal_field: f64,
) -> VoxelBiome {
    let weights = settings.composition.biome_weights;
    let dry = 1.0 - moisture;
    let cold = 1.0 - temperature;
    let coastal_lowland = 1.0
        - smoothstep(
            f64::from(settings.sea_level + 3),
            f64::from(settings.sea_level + 24),
            height,
        );
    let low_mountain = 1.0 - mountain_factor.clamp(0.0, 1.0);
    let candidates = [
        (
            VoxelBiome::Plains,
            weighted_biome_score(
                weights.plains,
                low_mountain
                    * (0.35 + climate_peak(temperature, 0.52, 0.55) * 0.35)
                    * (0.40 + climate_peak(moisture, 0.46, 0.52) * 0.40),
            ),
        ),
        (
            VoxelBiome::Forest,
            weighted_biome_score(
                weights.forest,
                low_mountain
                    * smoothstep(0.45, 0.78, moisture)
                    * (0.35 + climate_peak(temperature, 0.56, 0.48) * 0.65),
            ),
        ),
        (
            VoxelBiome::Desert,
            weighted_biome_score(
                weights.desert,
                low_mountain * smoothstep(0.54, 0.86, temperature) * smoothstep(0.44, 0.80, dry),
            ),
        ),
        (
            VoxelBiome::Tundra,
            weighted_biome_score(
                weights.tundra,
                (0.70 + mountain_factor * 0.30) * smoothstep(0.50, 0.82, cold),
            ),
        ),
        (
            VoxelBiome::Mountains,
            weighted_biome_score(
                weights.mountains,
                (mountain_factor * 0.82
                    + smoothstep(
                        f64::from(settings.base_height + 34),
                        f64::from(settings.base_height + 78),
                        height,
                    ) * 0.36)
                    .clamp(0.0, 1.0),
            ),
        ),
        (
            VoxelBiome::Wetlands,
            weighted_biome_score(
                weights.wetlands,
                coastal_lowland * smoothstep(0.52, 0.86, moisture) * (1.0 - mountain_factor * 0.80),
            ),
        ),
        (
            VoxelBiome::Badlands,
            weighted_biome_score(
                weights.badlands,
                smoothstep(0.50, 0.82, temperature)
                    * smoothstep(0.42, 0.76, dry)
                    * (0.35 + mountain_factor * 0.45),
            ),
        ),
        (
            VoxelBiome::CraterField,
            weighted_biome_score(
                weights.crater_fields,
                smoothstep(0.52, 0.88, crater_field) * (0.45 + dry * 0.35 + mountain_factor * 0.20),
            ),
        ),
        (
            VoxelBiome::Volcanic,
            weighted_biome_score(
                weights.volcanic,
                smoothstep(0.58, 0.90, volcanic_field)
                    * (0.35 + mountain_factor * 0.45 + temperature * 0.20),
            ),
        ),
        (
            VoxelBiome::CrystalFields,
            weighted_biome_score(
                weights.crystal_fields,
                smoothstep(0.66, 0.92, crystal_field)
                    * (0.30 + mountain_factor * 0.35 + dry * 0.20),
            ),
        ),
    ];

    choose_weighted_biome(candidates, biome_roll)
}

fn weighted_biome_score(weight: f64, score: f64) -> f64 {
    if weight <= 0.0 {
        0.0
    } else {
        score.max(0.015) * weight
    }
}

fn choose_weighted_biome(candidates: [(VoxelBiome, f64); 10], roll: f64) -> VoxelBiome {
    let total = candidates.iter().map(|(_, score)| *score).sum::<f64>();
    if total <= f64::EPSILON {
        return VoxelBiome::Plains;
    }

    let mut cursor = roll.clamp(0.0, 1.0) * total;
    for (biome, score) in candidates.iter().copied() {
        cursor -= score;
        if cursor <= 0.0 {
            return biome;
        }
    }

    candidates
        .last()
        .map(|(biome, _)| *biome)
        .unwrap_or(VoxelBiome::Plains)
}

pub(super) fn biome_height_adjustment(
    settings: VoxelWorldSettings,
    biome: VoxelBiome,
    hills: f64,
    crater_field: f64,
    volcanic_field: f64,
) -> f64 {
    match biome {
        VoxelBiome::Desert => -2.0,
        VoxelBiome::Tundra => 1.5,
        VoxelBiome::Wetlands => -4.0,
        VoxelBiome::Badlands => (hills - 0.45) * f64::from(settings.terrain_amplitude) * 0.35,
        VoxelBiome::CraterField => (crater_field - 0.62) * 18.0,
        VoxelBiome::Volcanic => volcanic_field.powf(2.0) * 20.0,
        VoxelBiome::CrystalFields => 3.0 + (crater_field - 0.50) * 6.0,
        _ => 0.0,
    }
}

pub(super) fn terraced_height_for_biome(
    biome: VoxelBiome,
    height: f64,
    mountain_factor: f64,
    crater_field: f64,
    volcanic_field: f64,
) -> f64 {
    let step = terrain_terrace_step(biome, mountain_factor, crater_field, volcanic_field);
    if step <= 1.0 {
        return height;
    }

    let stepped = (height / step).round() * step;
    let strength = terrain_terrace_strength(biome, mountain_factor);

    height + (stepped - height) * strength
}

pub(super) fn terrain_terrace_step(
    biome: VoxelBiome,
    mountain_factor: f64,
    crater_field: f64,
    volcanic_field: f64,
) -> f64 {
    match biome {
        VoxelBiome::Mountains if mountain_factor > 0.62 => 4.0,
        VoxelBiome::Mountains => 3.0,
        VoxelBiome::Badlands | VoxelBiome::Desert => 3.0,
        VoxelBiome::CraterField if crater_field > 0.68 => 4.0,
        VoxelBiome::Volcanic if volcanic_field > 0.66 => 4.0,
        VoxelBiome::CraterField | VoxelBiome::Volcanic => 3.0,
        VoxelBiome::CrystalFields => 2.0,
        VoxelBiome::Tundra if mountain_factor > 0.48 => 2.0,
        VoxelBiome::Plains | VoxelBiome::Forest | VoxelBiome::Wetlands
            if mountain_factor > 0.52 =>
        {
            2.0
        }
        _ => 1.0,
    }
}

fn terrain_terrace_strength(biome: VoxelBiome, mountain_factor: f64) -> f64 {
    match biome {
        VoxelBiome::Desert | VoxelBiome::Badlands | VoxelBiome::Mountains => 1.0,
        VoxelBiome::CraterField | VoxelBiome::Volcanic => 0.92,
        VoxelBiome::CrystalFields => 0.82,
        _ => (0.34 + mountain_factor * 0.46).clamp(0.34, 0.76),
    }
}
