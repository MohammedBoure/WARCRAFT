use super::features::volcanic_rift_channel_strength;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelVolcanicSurface {
    Basalt,
    Ash,
    Lava,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelSurfaceResource {
    BasaltStone,
    SpaceIron,
    Titanium,
    Osmium,
    SilicateCrystal,
    Uranium,
    Helium3,
    BioPlasma,
    AncientRelic,
}

impl VoxelSurfaceResource {
    pub const fn block(self) -> BlockKind {
        match self {
            Self::BasaltStone => BlockKind::CoalOre,
            Self::SpaceIron => BlockKind::IronOre,
            Self::Titanium => BlockKind::TitaniumOre,
            Self::Osmium => BlockKind::GoldOre,
            Self::SilicateCrystal => BlockKind::CrystalOre,
            Self::Uranium => BlockKind::UraniumOre,
            Self::Helium3 => BlockKind::HeliumVent,
            Self::BioPlasma => BlockKind::BioPlasmaBloom,
            Self::AncientRelic => BlockKind::AncientRelic,
        }
    }

    pub const fn catalog_key(self) -> &'static str {
        match self {
            Self::BasaltStone => "basalt_stone",
            Self::SpaceIron => "space_iron",
            Self::Titanium => "titanium",
            Self::Osmium => "osmium",
            Self::SilicateCrystal => "silicate_crystal",
            Self::Uranium => "uranium",
            Self::Helium3 => "helium_3",
            Self::BioPlasma => "bio_plasma",
            Self::AncientRelic => "ancient_relic",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SurfaceResourceCandidate {
    pub(super) resource: VoxelSurfaceResource,
    threshold: f64,
    scale: f64,
    salt: u64,
}

pub fn surface_resource_for_column(
    settings: VoxelWorldSettings,
    column: VoxelColumnSample,
) -> Option<VoxelSurfaceResource> {
    let settings = settings.sanitized();
    if column.height <= settings.sea_level || column.height <= settings.sea_level + 1 {
        return None;
    }
    if column.biome == VoxelBiome::Volcanic
        && volcanic_surface_for_column(settings, column) == VoxelVolcanicSurface::Lava
    {
        return None;
    }

    let altitude = ((column.height - settings.sea_level) as f64 / 80.0).clamp(0.0, 1.0);
    let moisture = column.moisture.clamp(0.0, 1.0);
    let temperature = column.temperature.clamp(0.0, 1.0);
    let mountain = column.mountain_factor.clamp(0.0, 1.0);
    let weather_bonus = match column.weather {
        VoxelWeather::IonStorm => 0.025,
        VoxelWeather::Ashfall => 0.018,
        VoxelWeather::Storm => 0.010,
        _ => 0.0,
    };

    for candidate in surface_resource_candidates_for_biome(column.biome) {
        let ratio = surface_resource_ratio(settings, candidate.resource);
        if ratio <= 0.0 {
            continue;
        }

        let x = column.world_x as f64;
        let z = column.world_z as f64;
        let deposit = surface_resource_cluster_score(settings, column, candidate, ratio);
        if deposit <= f64::EPSILON {
            continue;
        }

        let interior_vein = ridged2(
            settings.seed,
            x * candidate.scale * 0.72,
            z * candidate.scale * 0.72,
            2,
            candidate.salt ^ 0x5A5A,
        );
        let terrain_fit = surface_resource_terrain_fit(
            candidate.resource,
            altitude,
            moisture,
            temperature,
            mountain,
        );
        let signal = deposit * 0.74 + interior_vein * 0.10 + terrain_fit * 0.14 + weather_bonus;
        let threshold = (candidate.threshold - (ratio - 1.0).max(0.0) * 0.020).clamp(0.58, 0.98);

        if signal >= threshold {
            return Some(candidate.resource);
        }
    }

    if let Some(resource) = entry_surface_resource_for_column(settings, column) {
        return Some(resource);
    }

    None
}

fn entry_surface_resource_for_column(
    settings: VoxelWorldSettings,
    column: VoxelColumnSample,
) -> Option<VoxelSurfaceResource> {
    let distance_from_entry =
        ((column.world_x as f64).powi(2) + (column.world_z as f64).powi(2)).sqrt();
    if distance_from_entry > 220.0 {
        return None;
    }

    let ratios = settings.composition.resource_ratios;
    let reserve_candidates = [
        (
            VoxelSurfaceResource::BasaltStone,
            ratios.coal.max(1.0),
            -82.0,
            -42.0,
            42.0,
            0xA57A_E001,
        ),
        (
            VoxelSurfaceResource::SpaceIron,
            ratios.iron.max(1.0),
            76.0,
            36.0,
            38.0,
            0xA57A_E002,
        ),
        (
            VoxelSurfaceResource::SilicateCrystal,
            ratios.crystal.max(1.0),
            28.0,
            108.0,
            34.0,
            0xA57A_E003,
        ),
        (
            VoxelSurfaceResource::Titanium,
            ratios.iron,
            -44.0,
            104.0,
            30.0,
            0xA57A_E004,
        ),
        (
            VoxelSurfaceResource::Helium3,
            ratios.crystal,
            110.0,
            -88.0,
            30.0,
            0xA57A_E005,
        ),
        (
            VoxelSurfaceResource::BioPlasma,
            ratios.coal,
            -124.0,
            88.0,
            28.0,
            0xA57A_E006,
        ),
    ];

    for (resource, ratio, center_x, center_z, radius, salt) in reserve_candidates {
        if ratio < 0.85 {
            continue;
        }

        let radius_scale = (0.86 + (ratio - 1.0).max(0.0) * 0.08).clamp(0.78, 1.22);
        let effective_radius = radius * radius_scale;
        let distance = ((column.world_x as f64 - center_x).powi(2)
            + (column.world_z as f64 - center_z).powi(2))
        .sqrt();
        if distance > effective_radius {
            continue;
        }

        let core = 1.0 - smoothstep(effective_radius * 0.62, effective_radius, distance);
        let edge_grain = fbm2(
            settings.seed,
            column.world_x as f64 * 0.045,
            column.world_z as f64 * 0.045,
            2,
            salt,
        );
        if core + edge_grain * 0.10 > 0.18 {
            return Some(resource);
        }
    }

    None
}

fn surface_resource_cluster_score(
    settings: VoxelWorldSettings,
    column: VoxelColumnSample,
    candidate: &SurfaceResourceCandidate,
    ratio: f64,
) -> f64 {
    let cell_size = surface_resource_cluster_cell_size(candidate);
    let cell_x = (column.world_x as f64 / cell_size).floor() as i64;
    let cell_z = (column.world_z as f64 / cell_size).floor() as i64;
    let presence_threshold = surface_resource_cluster_presence_threshold(candidate, ratio);
    let mut best = 0.0_f64;

    for dz in -1..=1 {
        for dx in -1..=1 {
            let deposit_cell_x = cell_x + dx;
            let deposit_cell_z = cell_z + dz;
            let presence = unit3(
                settings.seed,
                deposit_cell_x,
                0,
                deposit_cell_z,
                candidate.salt ^ 0xD3A0,
            );
            if presence < presence_threshold {
                continue;
            }

            let jitter_x = (unit3(
                settings.seed,
                deposit_cell_x,
                11,
                deposit_cell_z,
                candidate.salt ^ 0xC105,
            ) - 0.5)
                * cell_size
                * 0.70;
            let jitter_z = (unit3(
                settings.seed,
                deposit_cell_x,
                23,
                deposit_cell_z,
                candidate.salt ^ 0x5EED,
            ) - 0.5)
                * cell_size
                * 0.70;
            let radius_roll = unit3(
                settings.seed,
                deposit_cell_x,
                37,
                deposit_cell_z,
                candidate.salt ^ 0xA11,
            );
            let strength_roll = unit3(
                settings.seed,
                deposit_cell_x,
                53,
                deposit_cell_z,
                candidate.salt ^ 0x71E,
            );
            let center_x = (deposit_cell_x as f64 + 0.5) * cell_size + jitter_x;
            let center_z = (deposit_cell_z as f64 + 0.5) * cell_size + jitter_z;
            let radius = cell_size * (0.22 + radius_roll * 0.18);
            let distance = ((column.world_x as f64 - center_x).powi(2)
                + (column.world_z as f64 - center_z).powi(2))
            .sqrt();
            let falloff = 1.0 - smoothstep(radius * 0.56, radius, distance);
            if falloff <= 0.0 {
                continue;
            }

            let edge_texture = fbm2(
                settings.seed,
                column.world_x as f64 * candidate.scale * 2.15,
                column.world_z as f64 * candidate.scale * 2.15,
                2,
                candidate.salt ^ 0xED9E,
            );
            let strength = 0.84 + strength_roll * 0.16;
            best = best.max(falloff * strength * (0.88 + edge_texture * 0.12));
        }
    }

    best.clamp(0.0, 1.0)
}

fn surface_resource_cluster_cell_size(candidate: &SurfaceResourceCandidate) -> f64 {
    (1.0 / candidate.scale * 2.20).clamp(42.0, 112.0)
}

fn surface_resource_cluster_presence_threshold(
    candidate: &SurfaceResourceCandidate,
    ratio: f64,
) -> f64 {
    (0.58 + candidate.threshold * 0.22 - (ratio - 1.0).max(0.0) * 0.035).clamp(0.38, 0.82)
}

pub(super) fn surface_resource_candidates_for_biome(
    biome: VoxelBiome,
) -> &'static [SurfaceResourceCandidate] {
    use VoxelSurfaceResource as Resource;

    match biome {
        VoxelBiome::Plains => &[
            SurfaceResourceCandidate {
                resource: Resource::BasaltStone,
                threshold: 0.795,
                scale: 0.030,
                salt: 0x1001,
            },
            SurfaceResourceCandidate {
                resource: Resource::SpaceIron,
                threshold: 0.835,
                scale: 0.026,
                salt: 0x1002,
            },
            SurfaceResourceCandidate {
                resource: Resource::BioPlasma,
                threshold: 0.915,
                scale: 0.022,
                salt: 0x1003,
            },
        ],
        VoxelBiome::Forest => &[
            SurfaceResourceCandidate {
                resource: Resource::BioPlasma,
                threshold: 0.785,
                scale: 0.030,
                salt: 0x1101,
            },
            SurfaceResourceCandidate {
                resource: Resource::BasaltStone,
                threshold: 0.860,
                scale: 0.026,
                salt: 0x1102,
            },
            SurfaceResourceCandidate {
                resource: Resource::AncientRelic,
                threshold: 0.965,
                scale: 0.015,
                salt: 0x1103,
            },
        ],
        VoxelBiome::Desert => &[
            SurfaceResourceCandidate {
                resource: Resource::Titanium,
                threshold: 0.785,
                scale: 0.028,
                salt: 0x1201,
            },
            SurfaceResourceCandidate {
                resource: Resource::BasaltStone,
                threshold: 0.840,
                scale: 0.032,
                salt: 0x1202,
            },
            SurfaceResourceCandidate {
                resource: Resource::Uranium,
                threshold: 0.930,
                scale: 0.019,
                salt: 0x1203,
            },
        ],
        VoxelBiome::Tundra => &[
            SurfaceResourceCandidate {
                resource: Resource::Helium3,
                threshold: 0.785,
                scale: 0.026,
                salt: 0x1301,
            },
            SurfaceResourceCandidate {
                resource: Resource::SilicateCrystal,
                threshold: 0.835,
                scale: 0.026,
                salt: 0x1302,
            },
            SurfaceResourceCandidate {
                resource: Resource::SpaceIron,
                threshold: 0.890,
                scale: 0.024,
                salt: 0x1303,
            },
        ],
        VoxelBiome::Mountains => &[
            SurfaceResourceCandidate {
                resource: Resource::SpaceIron,
                threshold: 0.755,
                scale: 0.032,
                salt: 0x1401,
            },
            SurfaceResourceCandidate {
                resource: Resource::Titanium,
                threshold: 0.805,
                scale: 0.028,
                salt: 0x1402,
            },
            SurfaceResourceCandidate {
                resource: Resource::Osmium,
                threshold: 0.895,
                scale: 0.020,
                salt: 0x1403,
            },
        ],
        VoxelBiome::Wetlands => &[
            SurfaceResourceCandidate {
                resource: Resource::BioPlasma,
                threshold: 0.760,
                scale: 0.032,
                salt: 0x1501,
            },
            SurfaceResourceCandidate {
                resource: Resource::Helium3,
                threshold: 0.850,
                scale: 0.026,
                salt: 0x1502,
            },
            SurfaceResourceCandidate {
                resource: Resource::BasaltStone,
                threshold: 0.900,
                scale: 0.030,
                salt: 0x1503,
            },
        ],
        VoxelBiome::Badlands => &[
            SurfaceResourceCandidate {
                resource: Resource::Titanium,
                threshold: 0.755,
                scale: 0.032,
                salt: 0x1601,
            },
            SurfaceResourceCandidate {
                resource: Resource::Uranium,
                threshold: 0.870,
                scale: 0.022,
                salt: 0x1602,
            },
            SurfaceResourceCandidate {
                resource: Resource::Osmium,
                threshold: 0.905,
                scale: 0.020,
                salt: 0x1603,
            },
        ],
        VoxelBiome::CraterField => &[
            SurfaceResourceCandidate {
                resource: Resource::SpaceIron,
                threshold: 0.765,
                scale: 0.032,
                salt: 0x1701,
            },
            SurfaceResourceCandidate {
                resource: Resource::SilicateCrystal,
                threshold: 0.835,
                scale: 0.024,
                salt: 0x1702,
            },
            SurfaceResourceCandidate {
                resource: Resource::AncientRelic,
                threshold: 0.930,
                scale: 0.017,
                salt: 0x1703,
            },
        ],
        VoxelBiome::Volcanic => &[
            SurfaceResourceCandidate {
                resource: Resource::Titanium,
                threshold: 0.745,
                scale: 0.034,
                salt: 0x1801,
            },
            SurfaceResourceCandidate {
                resource: Resource::Uranium,
                threshold: 0.835,
                scale: 0.024,
                salt: 0x1802,
            },
            SurfaceResourceCandidate {
                resource: Resource::Osmium,
                threshold: 0.895,
                scale: 0.019,
                salt: 0x1803,
            },
        ],
        VoxelBiome::CrystalFields => &[
            SurfaceResourceCandidate {
                resource: Resource::SilicateCrystal,
                threshold: 0.730,
                scale: 0.034,
                salt: 0x1901,
            },
            SurfaceResourceCandidate {
                resource: Resource::Helium3,
                threshold: 0.855,
                scale: 0.024,
                salt: 0x1902,
            },
            SurfaceResourceCandidate {
                resource: Resource::AncientRelic,
                threshold: 0.940,
                scale: 0.017,
                salt: 0x1903,
            },
        ],
    }
}

fn surface_resource_ratio(settings: VoxelWorldSettings, resource: VoxelSurfaceResource) -> f64 {
    let ratios = settings.composition.resource_ratios;
    match resource {
        VoxelSurfaceResource::BasaltStone => ratios.coal,
        VoxelSurfaceResource::SpaceIron => ratios.iron,
        VoxelSurfaceResource::Osmium => ratios.gold,
        VoxelSurfaceResource::SilicateCrystal => ratios.crystal,
        VoxelSurfaceResource::Titanium => ratios.iron,
        VoxelSurfaceResource::Uranium => (ratios.gold * 0.65 + ratios.crystal * 0.35).max(0.0),
        VoxelSurfaceResource::Helium3 => ratios.crystal.max(0.25),
        VoxelSurfaceResource::BioPlasma => ratios.coal.max(0.35),
        VoxelSurfaceResource::AncientRelic => ratios.crystal,
    }
}

fn surface_resource_terrain_fit(
    resource: VoxelSurfaceResource,
    altitude: f64,
    moisture: f64,
    temperature: f64,
    mountain: f64,
) -> f64 {
    match resource {
        VoxelSurfaceResource::BasaltStone => 0.45 + altitude * 0.25 + mountain * 0.20,
        VoxelSurfaceResource::SpaceIron => 0.38 + mountain * 0.42 + altitude * 0.18,
        VoxelSurfaceResource::Titanium => 0.36 + temperature * 0.38 + mountain * 0.20,
        VoxelSurfaceResource::Osmium => 0.30 + mountain * 0.44 + altitude * 0.24,
        VoxelSurfaceResource::SilicateCrystal => 0.40 + altitude * 0.28 + (1.0 - moisture) * 0.18,
        VoxelSurfaceResource::Uranium => 0.34 + temperature * 0.28 + (1.0 - moisture) * 0.22,
        VoxelSurfaceResource::Helium3 => 0.46 + (1.0 - temperature) * 0.32 + altitude * 0.12,
        VoxelSurfaceResource::BioPlasma => 0.42 + moisture * 0.42 + (1.0 - altitude) * 0.10,
        VoxelSurfaceResource::AncientRelic => 0.28 + altitude * 0.20 + mountain * 0.18,
    }
    .clamp(0.0, 1.0)
}

pub fn volcanic_surface_for_column(
    settings: VoxelWorldSettings,
    column: VoxelColumnSample,
) -> VoxelVolcanicSurface {
    let settings = settings.sanitized();
    if column.biome != VoxelBiome::Volcanic || column.height <= settings.sea_level + 2 {
        return VoxelVolcanicSurface::Basalt;
    }

    let rift_channel = volcanic_rift_channel_strength(settings, column);
    if rift_channel > 0.58
        && column.height <= settings.sea_level + 16
        && unit3(
            settings.seed,
            column.world_x / 2,
            0,
            column.world_z / 2,
            0x4F17,
        ) > 0.24
    {
        return VoxelVolcanicSurface::Lava;
    }

    let x = column.world_x as f64;
    let z = column.world_z as f64;
    let lava_channels = ridged2(
        settings.seed,
        x * VOLCANIC_LAVA_SCALE,
        z * VOLCANIC_LAVA_SCALE,
        3,
        0x1A7A,
    );
    let lava_vents = unit3(
        settings.seed,
        column.world_x / 3,
        0,
        column.world_z / 3,
        0x4A7A,
    );
    let steep_heat = column.mountain_factor.clamp(0.0, 1.0) * 0.14;
    let lava_score = lava_channels + lava_vents * 0.18 + steep_heat;

    if lava_score > 0.96 {
        return VoxelVolcanicSurface::Lava;
    }

    let ash_field = fbm2(
        settings.seed,
        x * VOLCANIC_ASH_SCALE,
        z * VOLCANIC_ASH_SCALE,
        3,
        0xA5A,
    );
    if ash_field > 0.42 || column.weather == VoxelWeather::Ashfall {
        VoxelVolcanicSurface::Ash
    } else {
        VoxelVolcanicSurface::Basalt
    }
}

pub(super) fn should_carve_cave(
    settings: VoxelWorldSettings,
    column: VoxelColumnSample,
    y: i32,
) -> bool {
    if settings.cave_density <= 0.0 || y <= 4 || y >= column.height - 5 {
        return false;
    }

    let depth = (f64::from(column.height - y) / f64::from(column.height.max(1))).clamp(0.0, 1.0);
    let cave_noise = fbm3(
        settings.seed,
        column.world_x as f64 * CAVE_SCALE,
        y as f64 * CAVE_SCALE * 1.28,
        column.world_z as f64 * CAVE_SCALE,
        3,
        131,
    );
    let threshold = 0.72 - settings.cave_density * 0.10 + depth * 0.06;

    cave_noise > threshold
}

pub(super) fn ore_for_block(
    settings: VoxelWorldSettings,
    column: VoxelColumnSample,
    y: i32,
) -> Option<BlockKind> {
    let height = i32::from(settings.world_height).max(1);
    let ratios = settings.composition.resource_ratios;
    let coal_band = y < height - 24;
    let iron_band = y < height / 2 + 18;
    let gold_band = y < height / 3;
    let crystal_band = y < height / 2 + 6;
    let roll = unit3(
        settings.seed,
        column.world_x,
        i64::from(y),
        column.world_z,
        0x0E,
    );
    let crystal_bonus = if column.biome == VoxelBiome::CrystalFields {
        4.0
    } else {
        1.0
    };

    if crystal_band && ore_roll_hits(roll, 0.0016, ratios.crystal * crystal_bonus) {
        Some(BlockKind::CrystalOre)
    } else if gold_band && ore_roll_hits(roll, 0.0035, ratios.gold) {
        Some(BlockKind::GoldOre)
    } else if iron_band && ore_roll_hits(roll, 0.0105, ratios.iron) {
        Some(BlockKind::IronOre)
    } else if coal_band && ore_roll_hits(roll, 0.0220, ratios.coal) {
        Some(BlockKind::CoalOre)
    } else {
        None
    }
}

fn ore_roll_hits(roll: f64, base_chance: f64, ratio: f64) -> bool {
    let chance = (base_chance * ratio).clamp(0.0, 0.80);

    chance > 0.0 && roll > 1.0 - chance
}
