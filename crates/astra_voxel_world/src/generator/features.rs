use super::*;

pub fn terrain_feature_weights_for_biome(biome: VoxelBiome) -> VoxelTerrainFeatureWeights {
    let mut weights = VoxelTerrainFeatureWeights::default();

    match biome {
        VoxelBiome::Plains => {
            weights.craters = 0.18;
            weights.large_craters = 0.05;
            weights.rifts = 0.05;
            weights.canyons = 0.06;
            weights.high_mountains = 0.08;
            weights.plateaus = 0.12;
            weights.erosion = 0.72;
        }
        VoxelBiome::Forest => {
            weights.craters = 0.10;
            weights.large_craters = 0.03;
            weights.rifts = 0.04;
            weights.canyons = 0.05;
            weights.high_mountains = 0.10;
            weights.plateaus = 0.08;
            weights.erosion = 0.90;
        }
        VoxelBiome::Desert => {
            weights.craters = 0.30;
            weights.large_craters = 0.10;
            weights.rifts = 0.12;
            weights.canyons = 1.10;
            weights.high_mountains = 0.22;
            weights.plateaus = 0.50;
            weights.erosion = 1.15;
        }
        VoxelBiome::Tundra => {
            weights.craters = 0.42;
            weights.large_craters = 0.16;
            weights.rifts = 0.12;
            weights.canyons = 0.10;
            weights.high_mountains = 0.48;
            weights.plateaus = 0.82;
            weights.erosion = 0.20;
        }
        VoxelBiome::Mountains => {
            weights.craters = 0.24;
            weights.large_craters = 0.08;
            weights.rifts = 0.42;
            weights.canyons = 0.30;
            weights.high_mountains = 1.25;
            weights.plateaus = 0.38;
            weights.erosion = 0.55;
        }
        VoxelBiome::Wetlands => {
            weights.craters = 0.08;
            weights.large_craters = 0.02;
            weights.rifts = 0.04;
            weights.canyons = 0.04;
            weights.high_mountains = 0.05;
            weights.plateaus = 0.06;
            weights.erosion = 1.20;
        }
        VoxelBiome::Badlands => {
            weights.craters = 0.44;
            weights.large_craters = 0.16;
            weights.rifts = 0.62;
            weights.canyons = 1.30;
            weights.high_mountains = 0.50;
            weights.plateaus = 0.95;
            weights.erosion = 0.88;
        }
        VoxelBiome::CraterField => {
            weights.craters = 1.45;
            weights.large_craters = 1.10;
            weights.rifts = 0.26;
            weights.canyons = 0.18;
            weights.high_mountains = 0.30;
            weights.plateaus = 0.44;
            weights.erosion = 0.26;
        }
        VoxelBiome::Volcanic => {
            weights.craters = 0.46;
            weights.large_craters = 0.18;
            weights.rifts = 1.45;
            weights.canyons = 0.32;
            weights.high_mountains = 1.10;
            weights.plateaus = 0.40;
            weights.erosion = 0.18;
        }
        VoxelBiome::CrystalFields => {
            weights.craters = 0.56;
            weights.large_craters = 0.24;
            weights.rifts = 0.34;
            weights.canyons = 0.18;
            weights.high_mountains = 0.45;
            weights.plateaus = 1.05;
            weights.erosion = 0.22;
        }
    }

    weights.sanitized()
}

pub fn terrain_feature_presence(
    settings: VoxelWorldSettings,
    world_x: i64,
    world_z: i64,
) -> VoxelTerrainFeaturePresence {
    let settings = settings.sanitized();
    let biome = voxel_biome_at(settings, world_x, world_z);
    terrain_feature_presence_for_biome(settings, world_x, world_z, biome)
}

pub(super) fn terrain_feature_presence_for_biome(
    settings: VoxelWorldSettings,
    world_x: i64,
    world_z: i64,
    biome: VoxelBiome,
) -> VoxelTerrainFeaturePresence {
    let global = settings.composition.terrain_features;
    let biome_weights = terrain_feature_weights_for_biome(biome);
    let x = world_x as f64;
    let z = world_z as f64;

    VoxelTerrainFeaturePresence {
        craters: weighted_feature_presence(
            global.craters,
            biome_weights.craters,
            ridged2(settings.seed, x * 0.0026, z * 0.0026, 3, 0xC7A7),
        ),
        large_craters: weighted_feature_presence(
            global.large_craters,
            biome_weights.large_craters,
            ridged2(settings.seed, x * 0.00072, z * 0.00072, 3, 0x1A12),
        ),
        rifts: weighted_feature_presence(
            global.rifts,
            biome_weights.rifts,
            ridged2(settings.seed, x * 0.0012, z * 0.0012, 4, 0x71F7),
        ),
        canyons: weighted_feature_presence(
            global.canyons,
            biome_weights.canyons,
            ridged2(settings.seed, x * 0.0018, z * 0.0018, 3, 0xCA71),
        ),
        high_mountains: weighted_feature_presence(
            global.high_mountains,
            biome_weights.high_mountains,
            ridged2(settings.seed, x * 0.0010, z * 0.0010, 4, 0xA17E),
        ),
        plateaus: weighted_feature_presence(
            global.plateaus,
            biome_weights.plateaus,
            fbm2(settings.seed, x * 0.0016, z * 0.0016, 3, 0x91A7),
        ),
        erosion: weighted_feature_presence(
            global.erosion,
            biome_weights.erosion,
            fbm2(settings.seed, x * 0.0024, z * 0.0024, 3, 0xE205),
        ),
    }
}

fn weighted_feature_presence(global_weight: f64, biome_weight: f64, signal: f64) -> f64 {
    let weight = (global_weight * biome_weight).clamp(0.0, 4.0);
    if weight <= f64::EPSILON {
        return 0.0;
    }

    (signal.clamp(0.0, 1.0) * (0.35 + weight * 0.25)).clamp(0.0, 1.0)
}

pub(super) fn high_mountain_delta(
    settings: VoxelWorldSettings,
    world_x: i64,
    world_z: i64,
    mountain_ridges: f64,
    continent: f64,
) -> f64 {
    let settings = settings.sanitized();
    let weight = settings
        .composition
        .terrain_features
        .high_mountains
        .clamp(0.0, 4.0);
    if weight <= f64::EPSILON || settings.mountain_amplitude <= 0 {
        return 0.0;
    }

    let x = world_x as f64;
    let z = world_z as f64;
    let broad_cluster = ridged2(settings.seed, x * 0.00082, z * 0.00082, 4, 0xA17E_4101);
    let peak_detail = ridged2(settings.seed, x * 0.0024, z * 0.0024, 3, 0xA17E_4102);
    let cluster_mask = smoothstep(0.42, 0.86, broad_cluster);
    let ridge_mask = smoothstep(0.34, 0.82, mountain_ridges);
    let continent_mask = smoothstep(0.22, 0.74, continent);
    let peak_mask =
        (cluster_mask * ridge_mask * continent_mask).powf(1.18) * (0.62 + peak_detail * 0.38);
    let amplitude = (f64::from(settings.mountain_amplitude) * 1.18 + 34.0).clamp(54.0, 132.0);

    peak_mask * amplitude * (weight / 4.0).clamp(0.0, 1.0)
}

pub(super) fn plateau_height_delta(
    settings: VoxelWorldSettings,
    world_x: i64,
    world_z: i64,
    biome: VoxelBiome,
) -> f64 {
    let settings = settings.sanitized();
    let global = settings.composition.terrain_features;
    if global.plateaus <= f64::EPSILON {
        return 0.0;
    }

    let biome_weight = terrain_feature_weights_for_biome(biome).plateaus;
    let weight = (global.plateaus * biome_weight).clamp(0.0, 4.0);
    if weight <= f64::EPSILON {
        return 0.0;
    }

    let cell_size = 420.0;
    let cell_x = (world_x as f64 / cell_size).floor() as i64;
    let cell_z = (world_z as f64 / cell_size).floor() as i64;
    let presence_threshold = (0.885 - weight * 0.120).clamp(0.32, 0.985);
    let mut best = 0.0_f64;

    for dz in -1..=1 {
        for dx in -1..=1 {
            let plateau_cell_x = cell_x + dx;
            let plateau_cell_z = cell_z + dz;
            let presence = unit3(
                settings.seed,
                plateau_cell_x,
                0,
                plateau_cell_z,
                0x91A7_4201,
            );
            if presence < presence_threshold {
                continue;
            }

            let center_x = (plateau_cell_x as f64 + 0.5) * cell_size
                + (unit3(
                    settings.seed,
                    plateau_cell_x,
                    11,
                    plateau_cell_z,
                    0x91A7_4202,
                ) - 0.5)
                    * cell_size
                    * 0.56;
            let center_z = (plateau_cell_z as f64 + 0.5) * cell_size
                + (unit3(
                    settings.seed,
                    plateau_cell_x,
                    23,
                    plateau_cell_z,
                    0x91A7_4203,
                ) - 0.5)
                    * cell_size
                    * 0.56;
            let radius = lerp(
                58.0,
                178.0,
                unit3(
                    settings.seed,
                    plateau_cell_x,
                    37,
                    plateau_cell_z,
                    0x91A7_4204,
                ),
            );
            let lift = lerp(
                10.0,
                34.0,
                unit3(
                    settings.seed,
                    plateau_cell_x,
                    41,
                    plateau_cell_z,
                    0x91A7_4205,
                ),
            );
            let distance =
                ((world_x as f64 - center_x).powi(2) + (world_z as f64 - center_z).powi(2)).sqrt();
            let core = radius * 0.66;
            let falloff = if distance < radius {
                1.0 - smoothstep(core, radius, distance)
            } else {
                0.0
            };

            best = best.max(lift * falloff);
        }
    }

    best.clamp(0.0, f64::from(settings.world_height) * 0.18)
}

pub(super) fn feature_plateau_terraced_height(
    settings: VoxelWorldSettings,
    biome: VoxelBiome,
    height: f64,
    world_x: i64,
    world_z: i64,
) -> f64 {
    let settings = settings.sanitized();
    let global = settings.composition.terrain_features;
    let biome_weight = terrain_feature_weights_for_biome(biome).plateaus;
    let weight = (global.plateaus * biome_weight).clamp(0.0, 4.0);
    if weight < 0.55 {
        return height;
    }

    let feature = terrain_feature_presence_for_biome(settings, world_x, world_z, biome).plateaus;
    if feature < 0.34 {
        return height;
    }

    let step = lerp(4.0, 9.0, (weight / 4.0).clamp(0.0, 1.0));
    let strength = (0.46 + feature * 0.42).clamp(0.0, 0.88);
    let stepped = (height / step).round() * step;

    height + (stepped - height) * strength
}

pub(super) fn erosion_adjusted_height(
    settings: VoxelWorldSettings,
    biome: VoxelBiome,
    reference_height: f64,
    height: f64,
    world_x: i64,
    world_z: i64,
) -> f64 {
    let settings = settings.sanitized();
    let global = settings.composition.terrain_features;
    if global.erosion <= f64::EPSILON {
        return height;
    }

    let biome_weight = terrain_feature_weights_for_biome(biome).erosion;
    let weight = (global.erosion * biome_weight).clamp(0.0, 4.0);
    if weight <= f64::EPSILON {
        return height;
    }

    let erosion_presence =
        terrain_feature_presence_for_biome(settings, world_x, world_z, biome).erosion;
    if erosion_presence <= f64::EPSILON {
        return height;
    }

    let strength = (erosion_presence * (0.22 + weight * 0.13)).clamp(0.0, 0.68);
    let relief = height - reference_height;
    let softened = reference_height + relief * (1.0 - strength * 0.62);
    let sediment_noise = fbm2(
        settings.seed,
        world_x as f64 * 0.013,
        world_z as f64 * 0.013,
        3,
        0xE705_5101,
    ) - 0.5;
    let sediment =
        sediment_noise * f64::from(settings.terrain_amplitude).max(4.0) * strength * 0.10;

    softened + sediment
}

pub(super) fn entry_extremity_dampening(world_x: i64, world_z: i64) -> f64 {
    let distance = ((world_x as f64).powi(2) + (world_z as f64).powi(2)).sqrt();
    0.18 + smoothstep(24.0, 96.0, distance) * 0.82
}

pub(super) fn crater_height_delta(
    settings: VoxelWorldSettings,
    world_x: i64,
    world_z: i64,
    biome: VoxelBiome,
) -> f64 {
    let settings = settings.sanitized();
    let global = settings.composition.terrain_features;
    if global.craters <= f64::EPSILON && global.large_craters <= f64::EPSILON {
        return 0.0;
    }

    let biome_weights = terrain_feature_weights_for_biome(biome);
    let small_weight = global.craters * biome_weights.craters;
    let medium_weight = (global.craters * 0.72 + global.large_craters * 0.28)
        * (biome_weights.craters * 0.72 + biome_weights.large_craters * 0.28);
    let large_weight = global.large_craters * biome_weights.large_craters;
    let mut delta = 0.0_f64;

    delta += crater_layer_height_delta(
        settings,
        world_x,
        world_z,
        small_weight,
        CraterLayer {
            cell_size: 72.0,
            min_radius: 7.0,
            max_radius: 18.0,
            min_depth_ratio: 0.18,
            max_depth_ratio: 0.28,
            min_rim_ratio: 0.045,
            max_rim_ratio: 0.090,
            presence_bias: 0.935,
            salt: 0xC2A7_5101,
        },
    );
    delta += crater_layer_height_delta(
        settings,
        world_x,
        world_z,
        medium_weight,
        CraterLayer {
            cell_size: 184.0,
            min_radius: 22.0,
            max_radius: 54.0,
            min_depth_ratio: 0.21,
            max_depth_ratio: 0.32,
            min_rim_ratio: 0.055,
            max_rim_ratio: 0.105,
            presence_bias: 0.955,
            salt: 0xC2A7_5201,
        },
    );
    delta += crater_layer_height_delta(
        settings,
        world_x,
        world_z,
        large_weight,
        CraterLayer {
            cell_size: 560.0,
            min_radius: 76.0,
            max_radius: 172.0,
            min_depth_ratio: 0.19,
            max_depth_ratio: 0.34,
            min_rim_ratio: 0.050,
            max_rim_ratio: 0.115,
            presence_bias: 0.972,
            salt: 0xC2A7_5301,
        },
    );

    let max_dent = f64::from(settings.world_height) * 0.28;
    let max_rim = f64::from(settings.world_height) * 0.08;
    delta.clamp(-max_dent, max_rim)
}

fn crater_layer_height_delta(
    settings: VoxelWorldSettings,
    world_x: i64,
    world_z: i64,
    weight: f64,
    layer: CraterLayer,
) -> f64 {
    let weight = weight.clamp(0.0, 4.0);
    if weight <= f64::EPSILON {
        return 0.0;
    }

    let cell_x = (world_x as f64 / layer.cell_size).floor() as i64;
    let cell_z = (world_z as f64 / layer.cell_size).floor() as i64;
    let mut layer_delta = 0.0_f64;
    let presence_threshold = (layer.presence_bias - weight * 0.105).clamp(0.36, 0.992);

    for dz in -1..=1 {
        for dx in -1..=1 {
            let crater_cell_x = cell_x + dx;
            let crater_cell_z = cell_z + dz;
            let presence = unit3(
                settings.seed,
                crater_cell_x,
                0,
                crater_cell_z,
                layer.salt ^ 0xA17E,
            );
            if presence < presence_threshold {
                continue;
            }

            let jitter_x = (unit3(
                settings.seed,
                crater_cell_x,
                11,
                crater_cell_z,
                layer.salt ^ 0x51A7,
            ) - 0.5)
                * layer.cell_size
                * 0.62;
            let jitter_z = (unit3(
                settings.seed,
                crater_cell_x,
                23,
                crater_cell_z,
                layer.salt ^ 0xB0A1,
            ) - 0.5)
                * layer.cell_size
                * 0.62;
            let radius_roll = unit3(
                settings.seed,
                crater_cell_x,
                37,
                crater_cell_z,
                layer.salt ^ 0xA771,
            );
            let depth_roll = unit3(
                settings.seed,
                crater_cell_x,
                41,
                crater_cell_z,
                layer.salt ^ 0xD331,
            );
            let rim_roll = unit3(
                settings.seed,
                crater_cell_x,
                53,
                crater_cell_z,
                layer.salt ^ 0x71F7,
            );
            let center_x = (crater_cell_x as f64 + 0.5) * layer.cell_size + jitter_x;
            let center_z = (crater_cell_z as f64 + 0.5) * layer.cell_size + jitter_z;
            let radius = lerp(layer.min_radius, layer.max_radius, radius_roll);
            let depth_ratio = lerp(layer.min_depth_ratio, layer.max_depth_ratio, depth_roll);
            let rim_ratio = lerp(layer.min_rim_ratio, layer.max_rim_ratio, rim_roll);
            let distance =
                ((world_x as f64 - center_x).powi(2) + (world_z as f64 - center_z).powi(2)).sqrt();
            let crater_delta = single_crater_height_delta(radius, depth_ratio, rim_ratio, distance);

            if crater_delta < 0.0 {
                layer_delta = layer_delta.min(crater_delta);
            } else {
                layer_delta = layer_delta.max(crater_delta);
            }
        }
    }

    layer_delta
}

fn single_crater_height_delta(radius: f64, depth_ratio: f64, rim_ratio: f64, distance: f64) -> f64 {
    let depth = radius * depth_ratio;
    let rim_height = radius * rim_ratio;
    let floor_radius = radius * 0.22;
    let wall_end = radius * 0.92;
    let rim_center = radius * 1.02;
    let rim_width = radius * 0.22;

    let depression = if distance < wall_end {
        let wall = smoothstep(floor_radius, wall_end, distance);
        -depth * (1.0 - wall) * (0.82 + wall * 0.18)
    } else {
        0.0
    };
    let rim_distance = (distance - rim_center).abs();
    let rim = if rim_distance < rim_width {
        let rim_falloff = 1.0 - smoothstep(0.0, rim_width, rim_distance);
        rim_height * rim_falloff.powf(1.35)
    } else {
        0.0
    };

    depression + rim
}

pub(super) fn crater_coastal_dampening(settings: VoxelWorldSettings, height: f64) -> f64 {
    if height < f64::from(settings.sea_level - 8) {
        0.42
    } else if height < f64::from(settings.sea_level + 5) {
        0.62
    } else {
        1.0
    }
}

pub(super) fn rift_height_delta(
    settings: VoxelWorldSettings,
    world_x: i64,
    world_z: i64,
    biome: VoxelBiome,
    volcanic_field: f64,
) -> f64 {
    let settings = settings.sanitized();
    let global = settings.composition.terrain_features;
    if global.rifts <= f64::EPSILON && global.canyons <= f64::EPSILON {
        return 0.0;
    }

    let biome_weights = terrain_feature_weights_for_biome(biome);
    let volcanic_boost = if biome == VoxelBiome::Volcanic {
        0.80 + volcanic_field.clamp(0.0, 1.0) * 0.55
    } else {
        1.0
    };
    let rift_weight = global.rifts * biome_weights.rifts * volcanic_boost;
    let canyon_weight = global.canyons * biome_weights.canyons;
    let mut delta = 0.0_f64;

    delta += linear_terrain_layer_height_delta(
        settings,
        world_x,
        world_z,
        rift_weight,
        LinearTerrainLayer {
            cell_size: 980.0,
            min_length: 640.0,
            max_length: 1_480.0,
            min_width: 12.0,
            max_width: 42.0,
            min_depth: 16.0,
            max_depth: 58.0,
            shoulder_ratio: 0.16,
            presence_bias: 0.900,
            salt: 0x71F7_2101,
        },
    );
    delta += linear_terrain_layer_height_delta(
        settings,
        world_x,
        world_z,
        canyon_weight,
        LinearTerrainLayer {
            cell_size: 760.0,
            min_length: 420.0,
            max_length: 1_060.0,
            min_width: 72.0,
            max_width: 168.0,
            min_depth: 14.0,
            max_depth: 36.0,
            shoulder_ratio: 0.10,
            presence_bias: 0.620,
            salt: 0xCA71_2201,
        },
    );

    let max_cut = f64::from(settings.world_height) * 0.36;
    let max_shoulder = f64::from(settings.world_height) * 0.07;
    delta.clamp(-max_cut, max_shoulder)
}

fn linear_terrain_layer_height_delta(
    settings: VoxelWorldSettings,
    world_x: i64,
    world_z: i64,
    weight: f64,
    layer: LinearTerrainLayer,
) -> f64 {
    let weight = weight.clamp(0.0, 4.0);
    if weight <= f64::EPSILON {
        return 0.0;
    }

    let cell_x = (world_x as f64 / layer.cell_size).floor() as i64;
    let cell_z = (world_z as f64 / layer.cell_size).floor() as i64;
    let presence_threshold = (layer.presence_bias - weight * 0.120).clamp(0.34, 0.990);
    let mut layer_delta = 0.0_f64;

    for dz in -1..=1 {
        for dx in -1..=1 {
            let feature_cell_x = cell_x + dx;
            let feature_cell_z = cell_z + dz;
            let presence = unit3(
                settings.seed,
                feature_cell_x,
                0,
                feature_cell_z,
                layer.salt ^ 0xA771,
            );
            if presence < presence_threshold {
                continue;
            }

            let center_x = (feature_cell_x as f64 + 0.5) * layer.cell_size
                + (unit3(
                    settings.seed,
                    feature_cell_x,
                    11,
                    feature_cell_z,
                    layer.salt ^ 0x51A7,
                ) - 0.5)
                    * layer.cell_size
                    * 0.60;
            let center_z = (feature_cell_z as f64 + 0.5) * layer.cell_size
                + (unit3(
                    settings.seed,
                    feature_cell_x,
                    23,
                    feature_cell_z,
                    layer.salt ^ 0x5EED,
                ) - 0.5)
                    * layer.cell_size
                    * 0.60;
            let angle = unit3(
                settings.seed,
                feature_cell_x,
                37,
                feature_cell_z,
                layer.salt ^ 0xA11,
            ) * std::f64::consts::TAU;
            let length = lerp(
                layer.min_length,
                layer.max_length,
                unit3(
                    settings.seed,
                    feature_cell_x,
                    41,
                    feature_cell_z,
                    layer.salt ^ 0x1EAF,
                ),
            );
            let width = lerp(
                layer.min_width,
                layer.max_width,
                unit3(
                    settings.seed,
                    feature_cell_x,
                    43,
                    feature_cell_z,
                    layer.salt ^ 0xF17D,
                ),
            );
            let depth = lerp(
                layer.min_depth,
                layer.max_depth,
                unit3(
                    settings.seed,
                    feature_cell_x,
                    47,
                    feature_cell_z,
                    layer.salt ^ 0xD331,
                ),
            );
            let dir_x = angle.cos();
            let dir_z = angle.sin();
            let half_length = length * 0.5;
            let ax = center_x - dir_x * half_length;
            let az = center_z - dir_z * half_length;
            let bx = center_x + dir_x * half_length;
            let bz = center_z + dir_z * half_length;
            let distance = distance_to_segment(world_x as f64, world_z as f64, ax, az, bx, bz);
            let feature_delta =
                single_linear_cut_height_delta(distance, width, depth, layer.shoulder_ratio);

            if feature_delta < 0.0 {
                layer_delta = layer_delta.min(feature_delta);
            } else {
                layer_delta = layer_delta.max(feature_delta);
            }
        }
    }

    layer_delta
}

fn single_linear_cut_height_delta(
    distance: f64,
    half_width: f64,
    depth: f64,
    shoulder_ratio: f64,
) -> f64 {
    let core_width = half_width * 0.34;
    let wall_end = half_width;
    let shoulder_outer = half_width * 1.55;
    let floor = if distance < wall_end {
        let wall = smoothstep(core_width, wall_end, distance);
        -depth * (1.0 - wall).powf(0.72)
    } else {
        0.0
    };
    let shoulder = if distance > wall_end && distance < shoulder_outer {
        let rim = 1.0 - smoothstep(wall_end, shoulder_outer, distance);
        depth * shoulder_ratio * rim.powf(1.4)
    } else {
        0.0
    };

    floor + shoulder
}

fn distance_to_segment(px: f64, pz: f64, ax: f64, az: f64, bx: f64, bz: f64) -> f64 {
    let ab_x = bx - ax;
    let ab_z = bz - az;
    let ap_x = px - ax;
    let ap_z = pz - az;
    let len_sq = ab_x * ab_x + ab_z * ab_z;
    if len_sq <= f64::EPSILON {
        return ((px - ax).powi(2) + (pz - az).powi(2)).sqrt();
    }

    let t = ((ap_x * ab_x + ap_z * ab_z) / len_sq).clamp(0.0, 1.0);
    let closest_x = ax + ab_x * t;
    let closest_z = az + ab_z * t;
    ((px - closest_x).powi(2) + (pz - closest_z).powi(2)).sqrt()
}

pub(super) fn rift_coastal_dampening(settings: VoxelWorldSettings, height: f64) -> f64 {
    if height < f64::from(settings.sea_level - 10) {
        0.52
    } else if height < f64::from(settings.sea_level + 4) {
        0.72
    } else {
        1.0
    }
}

pub(super) fn volcanic_rift_channel_strength(
    settings: VoxelWorldSettings,
    column: VoxelColumnSample,
) -> f64 {
    if column.biome != VoxelBiome::Volcanic {
        return 0.0;
    }

    let global = settings.sanitized().composition.terrain_features;
    let biome_weights = terrain_feature_weights_for_biome(VoxelBiome::Volcanic);
    let weight = (global.rifts * biome_weights.rifts).clamp(0.0, 4.0);
    if weight <= f64::EPSILON {
        return 0.0;
    }

    let delta = linear_terrain_layer_height_delta(
        settings,
        column.world_x,
        column.world_z,
        weight,
        LinearTerrainLayer {
            cell_size: 980.0,
            min_length: 640.0,
            max_length: 1_480.0,
            min_width: 12.0,
            max_width: 42.0,
            min_depth: 16.0,
            max_depth: 58.0,
            shoulder_ratio: 0.16,
            presence_bias: 0.900,
            salt: 0x71F7_2101,
        },
    );

    (-delta / 48.0).clamp(0.0, 1.0)
}
