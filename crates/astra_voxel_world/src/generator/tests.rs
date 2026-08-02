use super::{
    biomes::{terraced_height_for_biome, terrain_terrace_step},
    features::{crater_height_delta, rift_height_delta, volcanic_rift_channel_strength},
    resources::surface_resource_candidates_for_biome,
    *,
};
use crate::model::{
    VoxelBiomeWeights, VoxelResourceRatios, VoxelTerrainFeatureWeights, VoxelWeatherWeights,
    VoxelWorldComposition,
};

fn biome_weights_only(target: VoxelBiome) -> VoxelBiomeWeights {
    VoxelBiomeWeights {
        plains: if target == VoxelBiome::Plains {
            1.0
        } else {
            0.0
        },
        forest: if target == VoxelBiome::Forest {
            1.0
        } else {
            0.0
        },
        desert: if target == VoxelBiome::Desert {
            1.0
        } else {
            0.0
        },
        tundra: if target == VoxelBiome::Tundra {
            1.0
        } else {
            0.0
        },
        mountains: if target == VoxelBiome::Mountains {
            1.0
        } else {
            0.0
        },
        wetlands: if target == VoxelBiome::Wetlands {
            1.0
        } else {
            0.0
        },
        badlands: if target == VoxelBiome::Badlands {
            1.0
        } else {
            0.0
        },
        crater_fields: if target == VoxelBiome::CraterField {
            1.0
        } else {
            0.0
        },
        volcanic: if target == VoxelBiome::Volcanic {
            1.0
        } else {
            0.0
        },
        crystal_fields: if target == VoxelBiome::CrystalFields {
            1.0
        } else {
            0.0
        },
    }
}

fn weather_weights_only(target: VoxelWeather) -> VoxelWeatherWeights {
    VoxelWeatherWeights {
        clear: if target == VoxelWeather::Clear {
            1.0
        } else {
            0.0
        },
        cloudy: if target == VoxelWeather::Cloudy {
            1.0
        } else {
            0.0
        },
        rain: if target == VoxelWeather::Rain {
            1.0
        } else {
            0.0
        },
        storm: if target == VoxelWeather::Storm {
            1.0
        } else {
            0.0
        },
        snow: if target == VoxelWeather::Snow {
            1.0
        } else {
            0.0
        },
        dust_storm: if target == VoxelWeather::DustStorm {
            1.0
        } else {
            0.0
        },
        ashfall: if target == VoxelWeather::Ashfall {
            1.0
        } else {
            0.0
        },
        ion_storm: if target == VoxelWeather::IonStorm {
            1.0
        } else {
            0.0
        },
    }
}

fn forced_crater_settings() -> VoxelWorldSettings {
    VoxelWorldSettings {
        seed: 0xA57A_C7A7_E000_0001,
        base_height: 86,
        sea_level: 34,
        terrain_amplitude: 18,
        mountain_amplitude: 24,
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::CraterField),
            terrain_features: VoxelTerrainFeatureWeights {
                craters: 4.0,
                large_craters: 4.0,
                rifts: 0.0,
                canyons: 0.0,
                high_mountains: 0.0,
                plateaus: 0.0,
                erosion: 0.0,
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized()
}

fn forced_rift_settings(biome: VoxelBiome) -> VoxelWorldSettings {
    VoxelWorldSettings {
        seed: 0xA57A_71F7_E000_0001,
        base_height: 82,
        sea_level: 56,
        terrain_amplitude: 14,
        mountain_amplitude: 18,
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(biome),
            terrain_features: VoxelTerrainFeatureWeights {
                craters: 0.0,
                large_craters: 0.0,
                rifts: 4.0,
                canyons: 0.0,
                high_mountains: 0.0,
                plateaus: 0.0,
                erosion: 0.0,
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized()
}

fn forced_canyon_settings(biome: VoxelBiome) -> VoxelWorldSettings {
    VoxelWorldSettings {
        seed: 0xA57A_CA71_E000_0001,
        base_height: 82,
        sea_level: 42,
        terrain_amplitude: 12,
        mountain_amplitude: 10,
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(biome),
            terrain_features: VoxelTerrainFeatureWeights {
                craters: 0.0,
                large_craters: 0.0,
                rifts: 0.0,
                canyons: 4.0,
                high_mountains: 0.0,
                plateaus: 0.0,
                erosion: 0.0,
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized()
}

fn forced_mountain_settings(high_mountains: f64) -> VoxelWorldSettings {
    VoxelWorldSettings {
        seed: 0xA57A_A17E_E000_0001,
        base_height: 72,
        sea_level: 32,
        terrain_amplitude: 10,
        mountain_amplitude: 46,
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::Mountains),
            terrain_features: VoxelTerrainFeatureWeights {
                craters: 0.0,
                large_craters: 0.0,
                rifts: 0.0,
                canyons: 0.0,
                high_mountains,
                plateaus: 0.0,
                erosion: 0.0,
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized()
}

fn forced_plateau_settings(plateaus: f64) -> VoxelWorldSettings {
    VoxelWorldSettings {
        seed: 0xA57A_91A7_E000_0001,
        base_height: 72,
        sea_level: 30,
        terrain_amplitude: 18,
        mountain_amplitude: 8,
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::Badlands),
            terrain_features: VoxelTerrainFeatureWeights {
                craters: 0.0,
                large_craters: 0.0,
                rifts: 0.0,
                canyons: 0.0,
                high_mountains: 0.0,
                plateaus,
                erosion: 0.0,
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized()
}

fn forced_erosion_settings(erosion: f64) -> VoxelWorldSettings {
    VoxelWorldSettings {
        seed: 0xA57A_E705_E000_0001,
        base_height: 78,
        sea_level: 36,
        terrain_amplitude: 34,
        mountain_amplitude: 42,
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::Forest),
            terrain_features: VoxelTerrainFeatureWeights {
                craters: 0.0,
                large_craters: 0.0,
                rifts: 0.0,
                canyons: 0.0,
                high_mountains: 0.0,
                plateaus: 0.0,
                erosion,
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized()
}

fn sampled_height_range(settings: VoxelWorldSettings, half_extent: i64, step: usize) -> i32 {
    let mut min_height = i32::MAX;
    let mut max_height = i32::MIN;
    for world_z in (-half_extent..=half_extent).step_by(step) {
        for world_x in (-half_extent..=half_extent).step_by(step) {
            let height = sample_voxel_column(settings, world_x, world_z).height;
            min_height = min_height.min(height);
            max_height = max_height.max(height);
        }
    }

    max_height - min_height
}

fn flat_band_score(settings: VoxelWorldSettings, half_extent: i64, step: usize) -> usize {
    let mut score = 0;
    for world_z in (-half_extent..=half_extent).step_by(step) {
        let mut previous = None;
        for world_x in (-half_extent..=half_extent).step_by(step) {
            let height = sample_voxel_column(settings, world_x, world_z).height;
            if previous == Some(height) {
                score += 1;
            }
            previous = Some(height);
        }
    }

    score
}

#[test]
fn voxel_chunk_generation_is_deterministic() {
    let settings = VoxelWorldSettings::default();
    let coord = VoxelChunkCoord::new(-12, 34);

    let first = generate_voxel_chunk(settings, coord);
    let second = generate_voxel_chunk(settings, coord);

    assert_eq!(first, second);
}

#[test]
fn voxel_chunks_match_world_height_function_at_boundaries() {
    let settings = VoxelWorldSettings::default();
    let left = generate_voxel_chunk(settings, VoxelChunkCoord::new(0, 0));
    let right = generate_voxel_chunk(settings, VoxelChunkCoord::new(1, 0));

    for local_z in 0..DEFAULT_CHUNK_SIZE {
        let left_world_z = left.world_z(local_z);
        let right_world_z = right.world_z(local_z);
        assert_eq!(left_world_z, right_world_z);
        assert_eq!(
            left.highest_terrain_y(DEFAULT_CHUNK_SIZE - 1, local_z),
            Some(surface_height_at(settings, 15, left_world_z))
        );
        assert_eq!(
            right.highest_terrain_y(0, local_z),
            Some(surface_height_at(settings, 16, right_world_z))
        );
    }
}

#[test]
fn voxel_world_supports_far_chunk_coordinates() {
    let settings = VoxelWorldSettings::default();
    let coord = VoxelChunkCoord::new(1_000_000, -1_000_000);
    let chunk = generate_voxel_chunk(settings, coord);

    assert_eq!(chunk.coord(), coord);
    assert_eq!(
        chunk.highest_terrain_y(0, 0),
        Some(surface_height_at(
            settings,
            coord.world_x(0),
            coord.world_z(0)
        ))
    );
    assert!(chunk.count_blocks(BlockKind::Stone) > 0);
    assert!(chunk.count_blocks(BlockKind::Air) > 0);
}

#[test]
fn voxel_world_contains_varied_biomes_and_resources() {
    let settings = VoxelWorldSettings::default();
    let mut biomes = Vec::new();
    let mut ore_count = 0;

    for coord in [
        VoxelChunkCoord::ZERO,
        VoxelChunkCoord::new(12, 7),
        VoxelChunkCoord::new(-42, 18),
        VoxelChunkCoord::new(144, -91),
    ] {
        let chunk = generate_voxel_chunk(settings, coord);
        ore_count += chunk.blocks().iter().filter(|block| block.is_ore()).count();
        biomes.push(voxel_biome_at(
            settings,
            coord.world_x(3),
            coord.world_z(11),
        ));
    }

    biomes.sort();
    biomes.dedup();
    assert!(biomes.len() >= 2);
    assert!(ore_count > 0);
}

#[test]
fn biome_weights_can_force_specific_surface_type() {
    let settings = VoxelWorldSettings {
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::Volcanic),
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    };

    for (x, z) in [(0, 0), (512, -128), (-700, 340), (2_048, 1_024)] {
        assert_eq!(
            sample_voxel_column(settings, x, z).biome,
            VoxelBiome::Volcanic
        );
    }
}

#[test]
fn dry_and_rocky_biomes_use_visible_terrain_terraces() {
    assert_eq!(
        terrain_terrace_step(VoxelBiome::Desert, 0.10, 0.50, 0.50),
        3.0
    );
    assert_eq!(
        terrain_terrace_step(VoxelBiome::Badlands, 0.10, 0.50, 0.50),
        3.0
    );
    assert_eq!(
        terrain_terrace_step(VoxelBiome::Mountains, 0.70, 0.50, 0.50),
        4.0
    );

    let terraced =
        terraced_height_for_biome(VoxelBiome::Desert, 71.6, 0.10, 0.50, 0.50).round() as i32;
    assert_eq!(terraced.rem_euclid(3), 0);

    let settings = VoxelWorldSettings {
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::Desert),
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    };

    for (x, z) in [(0, 0), (128, -64), (-320, 192), (768, 512)] {
        let sample = sample_voxel_column(settings, x, z);
        assert_eq!(sample.biome, VoxelBiome::Desert);
        assert_eq!(sample.height.rem_euclid(3), 0);
    }
}

#[test]
fn forced_volcanic_world_generates_ash_and_lava_materials() {
    let settings = VoxelWorldSettings {
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::Volcanic),
            weather_weights: weather_weights_only(VoxelWeather::Ashfall),
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    };
    let mut ash_count = 0;
    let mut lava_count = 0;

    for coord in [
        VoxelChunkCoord::ZERO,
        VoxelChunkCoord::new(1, 0),
        VoxelChunkCoord::new(-2, 3),
        VoxelChunkCoord::new(8, -5),
    ] {
        let chunk = generate_voxel_chunk(settings, coord);
        ash_count += chunk.count_blocks(BlockKind::VolcanicAsh);
        lava_count += chunk.count_blocks(BlockKind::Lava);
    }

    assert!(ash_count > 0);
    assert!(lava_count > 0);
}

#[test]
fn weather_weights_can_force_specific_weather() {
    let settings = VoxelWorldSettings {
        composition: VoxelWorldComposition {
            weather_weights: weather_weights_only(VoxelWeather::IonStorm),
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    };

    for (x, z) in [(0, 0), (120, -88), (-2_000, 500)] {
        assert_eq!(voxel_weather_at(settings, x, z), VoxelWeather::IonStorm);
    }
}

#[test]
fn resource_ratios_control_crystal_generation() {
    let no_crystal = VoxelWorldSettings {
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::CrystalFields),
            resource_ratios: VoxelResourceRatios {
                crystal: 0.0,
                ..VoxelResourceRatios::default()
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    };
    let rich_crystal = VoxelWorldSettings {
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::CrystalFields),
            resource_ratios: VoxelResourceRatios {
                crystal: 6.0,
                ..VoxelResourceRatios::default()
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    };

    let coord = VoxelChunkCoord::new(3, -2);
    let empty = generate_voxel_chunk(no_crystal, coord);
    let rich = generate_voxel_chunk(rich_crystal, coord);

    assert_eq!(empty.count_blocks(BlockKind::CrystalOre), 0);
    assert!(rich.count_blocks(BlockKind::CrystalOre) > 0);
}

#[test]
fn surface_resources_form_contiguous_deposits() {
    let settings = VoxelWorldSettings {
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::Desert),
            resource_ratios: VoxelResourceRatios {
                coal: 5.0,
                iron: 6.0,
                gold: 5.0,
                crystal: 4.0,
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized();
    let mut resource_columns = 0;
    let mut clustered_columns = 0;

    for world_z in (-160..=160).step_by(2) {
        for world_x in (-160..=160).step_by(2) {
            let column = sample_voxel_column(settings, world_x, world_z);
            let Some(resource) = surface_resource_for_column(settings, column) else {
                continue;
            };

            resource_columns += 1;
            let same_neighbor_count = [(2, 0), (-2, 0), (0, 2), (0, -2)]
                .into_iter()
                .filter(|(dx, dz)| {
                    let neighbor = sample_voxel_column(settings, world_x + dx, world_z + dz);
                    surface_resource_for_column(settings, neighbor) == Some(resource)
                })
                .count();
            if same_neighbor_count > 0 {
                clustered_columns += 1;
            }
        }
    }

    assert!(resource_columns > 24);
    assert!(
        clustered_columns as f32 / resource_columns as f32 > 0.84,
        "surface resources should read as clustered deposits, not isolated dots"
    );
}

#[test]
fn entry_area_keeps_playable_resource_reserves() {
    let settings = VoxelWorldSettings {
        seed: 0xA57A_57A7,
        composition: VoxelWorldComposition {
            resource_ratios: VoxelResourceRatios {
                coal: 2.0,
                iron: 3.5,
                gold: 1.4,
                crystal: 2.5,
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized();
    let mut resource_columns = 0usize;
    let mut clustered_columns = 0usize;

    for world_z in (-192..=192).step_by(4) {
        for world_x in (-192..=192).step_by(4) {
            let column = sample_voxel_column(settings, world_x, world_z);
            let Some(resource) = surface_resource_for_column(settings, column) else {
                continue;
            };

            resource_columns += 1;
            let same_neighbor_count = [(4, 0), (-4, 0), (0, 4), (0, -4)]
                .into_iter()
                .filter(|(dx, dz)| {
                    let neighbor = sample_voxel_column(settings, world_x + dx, world_z + dz);
                    surface_resource_for_column(settings, neighbor) == Some(resource)
                })
                .count();
            if same_neighbor_count > 0 {
                clustered_columns += 1;
            }
        }
    }

    assert!(resource_columns >= 36);
    assert!(
        clustered_columns as f32 / resource_columns as f32 > 0.86,
        "entry reserves should still read as clustered deposits"
    );
}

#[test]
fn surface_resource_candidates_follow_biome_catalog_order() {
    let expected = [
        (
            VoxelBiome::Plains,
            [
                VoxelSurfaceResource::BasaltStone,
                VoxelSurfaceResource::SpaceIron,
                VoxelSurfaceResource::BioPlasma,
            ],
        ),
        (
            VoxelBiome::Forest,
            [
                VoxelSurfaceResource::BioPlasma,
                VoxelSurfaceResource::BasaltStone,
                VoxelSurfaceResource::AncientRelic,
            ],
        ),
        (
            VoxelBiome::Desert,
            [
                VoxelSurfaceResource::Titanium,
                VoxelSurfaceResource::BasaltStone,
                VoxelSurfaceResource::Uranium,
            ],
        ),
        (
            VoxelBiome::Tundra,
            [
                VoxelSurfaceResource::Helium3,
                VoxelSurfaceResource::SilicateCrystal,
                VoxelSurfaceResource::SpaceIron,
            ],
        ),
        (
            VoxelBiome::Mountains,
            [
                VoxelSurfaceResource::SpaceIron,
                VoxelSurfaceResource::Titanium,
                VoxelSurfaceResource::Osmium,
            ],
        ),
        (
            VoxelBiome::Wetlands,
            [
                VoxelSurfaceResource::BioPlasma,
                VoxelSurfaceResource::Helium3,
                VoxelSurfaceResource::BasaltStone,
            ],
        ),
        (
            VoxelBiome::Badlands,
            [
                VoxelSurfaceResource::Titanium,
                VoxelSurfaceResource::Uranium,
                VoxelSurfaceResource::Osmium,
            ],
        ),
        (
            VoxelBiome::CraterField,
            [
                VoxelSurfaceResource::SpaceIron,
                VoxelSurfaceResource::SilicateCrystal,
                VoxelSurfaceResource::AncientRelic,
            ],
        ),
        (
            VoxelBiome::Volcanic,
            [
                VoxelSurfaceResource::Titanium,
                VoxelSurfaceResource::Uranium,
                VoxelSurfaceResource::Osmium,
            ],
        ),
        (
            VoxelBiome::CrystalFields,
            [
                VoxelSurfaceResource::SilicateCrystal,
                VoxelSurfaceResource::Helium3,
                VoxelSurfaceResource::AncientRelic,
            ],
        ),
    ];

    for (biome, expected_resources) in expected {
        let actual: Vec<_> = surface_resource_candidates_for_biome(biome)
            .iter()
            .map(|candidate| candidate.resource)
            .collect();

        assert_eq!(actual, expected_resources);
    }
}

#[test]
fn forced_biomes_expose_their_primary_surface_resources() {
    for biome in VoxelBiome::ALL {
        let settings = VoxelWorldSettings {
            composition: VoxelWorldComposition {
                biome_weights: biome_weights_only(biome),
                resource_ratios: VoxelResourceRatios {
                    coal: 4.0,
                    iron: 4.0,
                    gold: 4.0,
                    crystal: 6.0,
                },
                ..VoxelWorldComposition::default()
            },
            ..VoxelWorldSettings::default()
        }
        .sanitized();
        let primary_resource = surface_resource_candidates_for_biome(biome)[0].resource;
        let mut found_primary = false;
        let mut found_any_resource = false;

        'scan: for world_z in (-384..=384).step_by(8) {
            for world_x in (-384..=384).step_by(8) {
                let column = sample_voxel_column(settings, i64::from(world_x), i64::from(world_z));
                if let Some(resource) = surface_resource_for_column(settings, column) {
                    found_any_resource = true;
                    if resource == primary_resource {
                        found_primary = true;
                        break 'scan;
                    }
                }
            }
        }

        assert!(
            found_any_resource,
            "forced biome {} should expose at least one surface resource",
            biome.name()
        );
        assert!(
            found_primary,
            "forced biome {} should expose primary resource {}",
            biome.name(),
            primary_resource.catalog_key()
        );
    }
}

#[test]
fn terrain_feature_weights_follow_biome_identity() {
    let plains = terrain_feature_weights_for_biome(VoxelBiome::Plains);
    let volcanic = terrain_feature_weights_for_biome(VoxelBiome::Volcanic);
    let crater = terrain_feature_weights_for_biome(VoxelBiome::CraterField);
    let badlands = terrain_feature_weights_for_biome(VoxelBiome::Badlands);

    assert!(volcanic.rifts > plains.rifts);
    assert!(volcanic.high_mountains > plains.high_mountains);
    assert!(crater.large_craters > plains.large_craters);
    assert!(badlands.canyons > plains.canyons);
}

#[test]
fn terrain_feature_presence_is_deterministic_and_normalized() {
    let settings = VoxelWorldSettings {
        seed: 0xA57A_7E22_A1CE,
        composition: VoxelWorldComposition::preset("volcanic").expect("volcanic preset"),
        ..VoxelWorldSettings::default()
    }
    .sanitized();

    let first = terrain_feature_presence(settings, 1_024, -768);
    let second = terrain_feature_presence(settings, 1_024, -768);

    assert_eq!(first, second);
    for value in [
        first.craters,
        first.large_craters,
        first.rifts,
        first.canyons,
        first.high_mountains,
        first.plateaus,
        first.erosion,
    ] {
        assert!(value.is_finite());
        assert!((0.0..=1.0).contains(&value));
    }
}

#[test]
fn forced_crater_world_has_visible_bowls_and_rims() {
    let settings = forced_crater_settings();
    let mut min_delta = 0.0_f64;
    let mut max_delta = 0.0_f64;

    for world_z in (-1_024..=1_024).step_by(8) {
        for world_x in (-1_024..=1_024).step_by(8) {
            let delta = crater_height_delta(
                settings,
                i64::from(world_x),
                i64::from(world_z),
                VoxelBiome::CraterField,
            );
            min_delta = min_delta.min(delta);
            max_delta = max_delta.max(delta);
        }
    }

    assert!(
        min_delta < -8.0,
        "forced crater world should contain deep impact bowls, got {min_delta}"
    );
    assert!(
        max_delta > 2.0,
        "forced crater world should contain raised rims, got {max_delta}"
    );
}

#[test]
fn crater_delta_is_deterministic_for_same_seed() {
    let settings = forced_crater_settings();

    let first = crater_height_delta(settings, 320, -448, VoxelBiome::CraterField);
    let second = crater_height_delta(settings, 320, -448, VoxelBiome::CraterField);

    assert_eq!(first, second);
}

#[test]
fn disabled_crater_features_leave_crater_delta_flat() {
    let settings = VoxelWorldSettings {
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::CraterField),
            terrain_features: VoxelTerrainFeatureWeights {
                craters: 0.0,
                large_craters: 0.0,
                ..VoxelTerrainFeatureWeights::default()
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized();

    for (x, z) in [(0, 0), (256, -128), (-768, 512), (2_048, -1_024)] {
        assert_eq!(
            crater_height_delta(settings, x, z, VoxelBiome::CraterField),
            0.0
        );
    }
}

#[test]
fn large_craters_keep_surface_height_inside_world_bounds() {
    let settings = VoxelWorldSettings {
        world_height: 128,
        base_height: 40,
        sea_level: 18,
        terrain_amplitude: 8,
        mountain_amplitude: 0,
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::CraterField),
            terrain_features: VoxelTerrainFeatureWeights {
                craters: 4.0,
                large_craters: 4.0,
                rifts: 0.0,
                canyons: 0.0,
                high_mountains: 0.0,
                plateaus: 0.0,
                erosion: 0.0,
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized();

    for world_z in (-1_536..=1_536).step_by(16) {
        for world_x in (-1_536..=1_536).step_by(16) {
            let sample = sample_voxel_column(settings, world_x, world_z);
            assert!((4..=i32::from(settings.world_height) - 5).contains(&sample.height));
        }
    }
}

#[test]
fn forced_rift_world_cuts_deep_linear_trenches() {
    let settings = forced_rift_settings(VoxelBiome::Volcanic);
    let mut min_delta = 0.0_f64;
    let mut max_delta = 0.0_f64;
    let mut affected_columns = 0;

    for world_z in (-1_536..=1_536).step_by(8) {
        for world_x in (-1_536..=1_536).step_by(8) {
            let delta = rift_height_delta(
                settings,
                i64::from(world_x),
                i64::from(world_z),
                VoxelBiome::Volcanic,
                1.0,
            );
            min_delta = min_delta.min(delta);
            max_delta = max_delta.max(delta);
            if delta < -6.0 {
                affected_columns += 1;
            }
        }
    }

    assert!(
        min_delta < -18.0,
        "forced rift world should cut deep trenches, got {min_delta}"
    );
    assert!(
        max_delta > 1.5,
        "forced rift world should have raised fracture shoulders, got {max_delta}"
    );
    assert!(
        affected_columns > 24,
        "rift field should cover a long readable line, got {affected_columns} columns"
    );
}

#[test]
fn rift_delta_is_deterministic_for_same_seed() {
    let settings = forced_rift_settings(VoxelBiome::Volcanic);

    let first = rift_height_delta(settings, 640, -512, VoxelBiome::Volcanic, 0.84);
    let second = rift_height_delta(settings, 640, -512, VoxelBiome::Volcanic, 0.84);

    assert_eq!(first, second);
}

#[test]
fn disabled_rift_and_canyon_features_leave_rift_delta_flat() {
    let settings = VoxelWorldSettings {
        composition: VoxelWorldComposition {
            biome_weights: biome_weights_only(VoxelBiome::Volcanic),
            terrain_features: VoxelTerrainFeatureWeights {
                rifts: 0.0,
                canyons: 0.0,
                ..VoxelTerrainFeatureWeights::default()
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized();

    for (x, z) in [(0, 0), (512, -128), (-1_024, 2_048), (3_072, -768)] {
        assert_eq!(
            rift_height_delta(settings, x, z, VoxelBiome::Volcanic, 1.0),
            0.0
        );
    }
}

#[test]
fn canyon_world_is_wide_but_less_extreme_than_rift_world() {
    let canyon_settings = forced_canyon_settings(VoxelBiome::Badlands);
    let rift_settings = forced_rift_settings(VoxelBiome::Volcanic);
    let mut canyon_cut_columns = 0;
    let mut canyon_min_delta = 0.0_f64;
    let mut rift_min_delta = 0.0_f64;

    for world_z in (-1_280..=1_280).step_by(8) {
        for world_x in (-1_280..=1_280).step_by(8) {
            let canyon_delta = rift_height_delta(
                canyon_settings,
                i64::from(world_x),
                i64::from(world_z),
                VoxelBiome::Badlands,
                0.2,
            );
            let rift_delta = rift_height_delta(
                rift_settings,
                i64::from(world_x),
                i64::from(world_z),
                VoxelBiome::Volcanic,
                1.0,
            );

            canyon_min_delta = canyon_min_delta.min(canyon_delta);
            rift_min_delta = rift_min_delta.min(rift_delta);
            if canyon_delta < -4.0 {
                canyon_cut_columns += 1;
            }
        }
    }

    assert!(
        canyon_cut_columns > 36,
        "expected wide canyon coverage, got {canyon_cut_columns} columns and min {canyon_min_delta}"
    );
    assert!(canyon_min_delta < -8.0);
    assert!(
        canyon_min_delta > rift_min_delta,
        "canyons should be shallower than volcanic rifts"
    );
}

#[test]
fn volcanic_rifts_can_expose_lava_channels() {
    let settings = forced_rift_settings(VoxelBiome::Volcanic);
    let mut found_lava = false;

    'scan: for world_z in (-1_536..=1_536).step_by(4) {
        for world_x in (-1_536..=1_536).step_by(4) {
            let column = sample_voxel_column(settings, world_x, world_z);
            if volcanic_rift_channel_strength(settings, column) > 0.58
                && volcanic_surface_for_column(settings, column) == VoxelVolcanicSurface::Lava
            {
                found_lava = true;
                break 'scan;
            }
        }
    }

    assert!(
        found_lava,
        "forced volcanic rift world should expose lava channels"
    );
}

#[test]
fn mountain_feature_boosts_large_scale_height_range() {
    let low = forced_mountain_settings(0.0);
    let high = forced_mountain_settings(4.0);
    let low_range = sampled_height_range(low, 256, 8);
    let high_range = sampled_height_range(high, 256, 8);

    assert!(
        high_range > low_range + 18,
        "high mountain feature should expand range, low={low_range} high={high_range}"
    );
}

#[test]
fn mountain_feature_keeps_entry_area_less_extreme() {
    let settings = forced_mountain_settings(4.0);
    let entry = sample_voxel_column(settings, 0, 0).height;
    let nearby = sampled_height_range(settings, 24, 4);
    let broad = sampled_height_range(settings, 384, 8);

    assert!(nearby < broad);
    assert!(
        entry < settings.base_height + settings.mountain_amplitude,
        "entry column should not spawn on the tallest possible peak"
    );
}

#[test]
fn mountain_prompt_plateaus_create_more_flat_bands_than_default() {
    let default = forced_plateau_settings(0.0);
    let plateau = forced_plateau_settings(4.0);
    let default_score = flat_band_score(default, 256, 4);
    let plateau_score = flat_band_score(plateau, 256, 4);

    assert!(
        plateau_score > default_score + 16,
        "plateaus should add readable flat bands, default={default_score} plateau={plateau_score}"
    );
}

#[test]
fn mountain_and_plateau_worlds_keep_height_inside_bounds() {
    for settings in [forced_mountain_settings(4.0), forced_plateau_settings(4.0)] {
        for world_z in (-1_024..=1_024).step_by(16) {
            for world_x in (-1_024..=1_024).step_by(16) {
                let sample = sample_voxel_column(settings, world_x, world_z);
                assert!((4..=i32::from(settings.world_height) - 5).contains(&sample.height));
            }
        }
    }
}

#[test]
fn diversity_report_is_deterministic_and_counts_samples() {
    let settings = VoxelWorldSettings {
        composition: VoxelWorldComposition::preset("crystal").expect("crystal preset"),
        ..VoxelWorldSettings::default()
    }
    .sanitized();
    let first = voxel_terrain_diversity_report(settings, 128, 16);
    let second = voxel_terrain_diversity_report(settings, 128, 16);

    assert_eq!(first, second);
    assert_eq!(first.sample_count, 17 * 17);
    assert!(first.height_range > 0);
    assert!(first.distinct_biomes >= 1);
    assert!(first.distinct_weather >= 1);
    for value in [
        first.average_features.craters,
        first.average_features.large_craters,
        first.average_features.rifts,
        first.average_features.canyons,
        first.average_features.high_mountains,
        first.average_features.plateaus,
        first.average_features.erosion,
    ] {
        assert!((0.0..=1.0).contains(&value));
    }
}

#[test]
fn diversity_report_tracks_forced_biome_and_weather() {
    let mut composition = VoxelWorldComposition::default();
    composition.force_biome(VoxelBiome::Volcanic);
    composition.force_weather(VoxelWeather::Ashfall);
    let settings = VoxelWorldSettings {
        composition,
        ..VoxelWorldSettings::default()
    }
    .sanitized();
    let report = voxel_terrain_diversity_report(settings, 96, 16);

    assert_eq!(report.distinct_biomes, 1);
    assert_eq!(report.distinct_weather, 1);
}

#[test]
fn diversity_report_reflects_feature_weighted_planets() {
    let volcanic = VoxelWorldSettings {
        composition: VoxelWorldComposition::preset("volcanic").expect("volcanic preset"),
        ..VoxelWorldSettings::default()
    }
    .sanitized();
    let crater = VoxelWorldSettings {
        composition: VoxelWorldComposition::preset("crater").expect("crater preset"),
        ..VoxelWorldSettings::default()
    }
    .sanitized();
    let frozen = VoxelWorldSettings {
        composition: VoxelWorldComposition::preset("frozen").expect("frozen preset"),
        ..VoxelWorldSettings::default()
    }
    .sanitized();

    let volcanic_report = voxel_terrain_diversity_report(volcanic, 384, 16);
    let crater_report = voxel_terrain_diversity_report(crater, 384, 16);
    let frozen_report = voxel_terrain_diversity_report(frozen, 384, 16);

    assert!(
        volcanic_report.average_features.rifts > frozen_report.average_features.rifts,
        "volcanic worlds should expose more rift pressure"
    );
    assert!(
        crater_report.average_features.craters > frozen_report.average_features.craters,
        "crater worlds should expose more impact pressure"
    );
    assert!(
        frozen_report.average_features.plateaus > volcanic_report.average_features.plateaus,
        "frozen worlds should lean into broad plateaus"
    );
}

#[test]
fn erosion_feature_softens_local_height_range() {
    let low = forced_erosion_settings(0.0);
    let high = forced_erosion_settings(4.0);
    let low_range = sampled_height_range(low, 512, 8);
    let high_range = sampled_height_range(high, 512, 8);

    assert!(
        high_range + 8 < low_range,
        "erosion should soften relief, low={low_range} high={high_range}"
    );
}

#[test]
fn sanitized_composition_recovers_from_invalid_zero_weights() {
    let settings = VoxelWorldSettings {
        composition: VoxelWorldComposition {
            biome_weights: VoxelBiomeWeights {
                plains: 0.0,
                forest: 0.0,
                desert: 0.0,
                tundra: 0.0,
                mountains: 0.0,
                wetlands: 0.0,
                badlands: 0.0,
                crater_fields: 0.0,
                volcanic: 0.0,
                crystal_fields: 0.0,
            },
            weather_weights: VoxelWeatherWeights {
                clear: f64::NAN,
                cloudy: 0.0,
                rain: 0.0,
                storm: 0.0,
                snow: 0.0,
                dust_storm: 0.0,
                ashfall: 0.0,
                ion_storm: 0.0,
            },
            ..VoxelWorldComposition::default()
        },
        ..VoxelWorldSettings::default()
    }
    .sanitized();

    assert!(settings.composition.biome_weights.total() > 0.0);
    assert!(settings.composition.weather_weights.total() > 0.0);
}

#[test]
fn regression_samples_capture_stable_generation_outputs() {
    let samples = [
        (
            VoxelWorldSettings {
                seed: 0xA57A_0000_0000_0101,
                ..VoxelWorldSettings::default()
            }
            .sanitized(),
            [
                (0, 0, 64, VoxelBiome::Forest, VoxelWeather::Clear, None),
                (
                    76,
                    36,
                    64,
                    VoxelBiome::Plains,
                    VoxelWeather::Clear,
                    Some(VoxelSurfaceResource::SpaceIron),
                ),
                (-512, 384, 69, VoxelBiome::Plains, VoxelWeather::Clear, None),
            ],
        ),
        (
            VoxelWorldSettings {
                seed: 0xA57A_0000_0000_0202,
                composition: VoxelWorldComposition::preset("volcanic").expect("volcanic preset"),
                ..VoxelWorldSettings::default()
            }
            .sanitized(),
            [
                (
                    110,
                    -88,
                    69,
                    VoxelBiome::Mountains,
                    VoxelWeather::Clear,
                    Some(VoxelSurfaceResource::Helium3),
                ),
                (
                    384,
                    -256,
                    122,
                    VoxelBiome::Volcanic,
                    VoxelWeather::Ashfall,
                    None,
                ),
                (
                    -1_024,
                    768,
                    115,
                    VoxelBiome::Volcanic,
                    VoxelWeather::Ashfall,
                    None,
                ),
            ],
        ),
        (
            VoxelWorldSettings {
                seed: 0xA57A_0000_0000_0303,
                composition: VoxelWorldComposition::preset("crystal").expect("crystal preset"),
                ..VoxelWorldSettings::default()
            }
            .sanitized(),
            [
                (28, 108, 58, VoxelBiome::Forest, VoxelWeather::Clear, None),
                (
                    -320,
                    -448,
                    84,
                    VoxelBiome::Tundra,
                    VoxelWeather::Clear,
                    None,
                ),
                (
                    1_280,
                    640,
                    114,
                    VoxelBiome::CraterField,
                    VoxelWeather::IonStorm,
                    None,
                ),
            ],
        ),
    ];

    for (settings, expected_samples) in samples {
        for (world_x, world_z, height, biome, weather, resource) in expected_samples {
            let column = sample_voxel_column(settings, world_x, world_z);

            assert_eq!(column.height, height);
            assert_eq!(column.biome, biome);
            assert_eq!(column.weather, weather);
            assert_eq!(surface_resource_for_column(settings, column), resource);
        }
    }
}
