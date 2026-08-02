use crate::{
    generator::{VoxelVolcanicSurface, volcanic_surface_for_column},
    model::{BlockKind, VoxelBiome, VoxelColumnSample, VoxelWeather, VoxelWorldSettings},
};

pub type VoxelRgba = [f32; 4];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelTerrainVisual {
    pub top_color: VoxelRgba,
    pub side_color: VoxelRgba,
    pub edge_color: VoxelRgba,
    pub is_water: bool,
}

pub fn voxel_terrain_visual(
    settings: VoxelWorldSettings,
    column: VoxelColumnSample,
) -> VoxelTerrainVisual {
    let settings = settings.sanitized();
    let is_water = column.height <= settings.sea_level;

    if is_water {
        let depth = (settings.sea_level - column.height).clamp(0, 28) as f32 / 28.0;
        let top_color = mix_rgba([0.24, 0.68, 0.92, 1.0], [0.08, 0.34, 0.66, 1.0], depth);

        return apply_weather_to_visual(
            VoxelTerrainVisual {
                top_color,
                side_color: shade_rgba(top_color, 0.68),
                edge_color: [0.62, 0.92, 1.0, 1.0],
                is_water,
            },
            column.weather,
        );
    }

    if column.biome == VoxelBiome::Volcanic {
        let volcanic = volcanic_terrain_visual(settings, column, is_water);
        return apply_weather_to_visual(volcanic, column.weather);
    }

    let height_factor = ((column.height - settings.sea_level) as f32 / 76.0).clamp(0.0, 1.0);
    let mountain_factor = column.mountain_factor as f32;
    let shade = 0.88 + height_factor * 0.16 + mountain_factor * 0.06;
    let coastal = column.height <= settings.sea_level + 3;
    let base = if coastal {
        [0.86, 0.74, 0.48, 1.0]
    } else {
        biome_base_color(column.biome)
    };
    let top_color = clamp_rgba(shade_rgba(base, shade));
    let edge_base = if coastal {
        [0.56, 0.42, 0.20, 1.0]
    } else {
        biome_edge_color(column.biome)
    };
    let side_base = if coastal {
        [0.52, 0.38, 0.20, 1.0]
    } else {
        biome_side_color(column.biome)
    };
    let side_shade = 0.80 + height_factor * 0.08 + mountain_factor * 0.04;

    apply_weather_to_visual(
        VoxelTerrainVisual {
            top_color,
            side_color: mix_rgba(shade_rgba(side_base, side_shade), top_color, 0.12),
            edge_color: mix_rgba(edge_base, top_color, 0.24),
            is_water,
        },
        column.weather,
    )
}

pub fn voxel_block_color(block: BlockKind) -> VoxelRgba {
    match block {
        BlockKind::Air => [0.0, 0.0, 0.0, 0.0],
        BlockKind::Bedrock => [0.08, 0.08, 0.10, 1.0],
        BlockKind::Stone => [0.42, 0.43, 0.40, 1.0],
        BlockKind::Dirt => [0.42, 0.23, 0.10, 1.0],
        BlockKind::Grass => [0.24, 0.74, 0.18, 1.0],
        BlockKind::Sand => [0.96, 0.78, 0.32, 1.0],
        BlockKind::Water => [0.08, 0.54, 0.92, 1.0],
        BlockKind::Snow => [0.88, 0.96, 1.0, 1.0],
        BlockKind::Wood => [0.46, 0.25, 0.10, 1.0],
        BlockKind::Leaves => [0.05, 0.48, 0.14, 1.0],
        BlockKind::CoalOre => [0.12, 0.12, 0.12, 1.0],
        BlockKind::IronOre => [0.88, 0.46, 0.22, 1.0],
        BlockKind::GoldOre => [1.0, 0.76, 0.14, 1.0],
        BlockKind::Mud => [0.28, 0.18, 0.08, 1.0],
        BlockKind::Basalt => [0.055, 0.050, 0.065, 1.0],
        BlockKind::Ice => [0.52, 0.84, 1.0, 1.0],
        BlockKind::CrystalOre => [0.28, 0.92, 1.0, 1.0],
        BlockKind::VolcanicAsh => [0.28, 0.25, 0.22, 1.0],
        BlockKind::Lava => [1.0, 0.22, 0.025, 1.0],
        BlockKind::TitaniumOre => [0.72, 0.92, 1.0, 1.0],
        BlockKind::UraniumOre => [0.58, 1.0, 0.14, 1.0],
        BlockKind::HeliumVent => [0.28, 0.76, 1.0, 1.0],
        BlockKind::BioPlasmaBloom => [0.18, 1.0, 0.38, 1.0],
        BlockKind::AncientRelic => [1.0, 0.58, 0.14, 1.0],
    }
}

fn volcanic_terrain_visual(
    settings: VoxelWorldSettings,
    column: VoxelColumnSample,
    is_water: bool,
) -> VoxelTerrainVisual {
    let height_factor = ((column.height - settings.sea_level) as f32 / 92.0).clamp(0.0, 1.0);
    match volcanic_surface_for_column(settings, column) {
        VoxelVolcanicSurface::Lava => VoxelTerrainVisual {
            top_color: [1.0, 0.22, 0.02, 1.0],
            side_color: [0.28, 0.035, 0.018, 1.0],
            edge_color: [1.0, 0.66, 0.08, 1.0],
            is_water,
        },
        VoxelVolcanicSurface::Ash => {
            let top_color = shade_rgba([0.34, 0.29, 0.24, 1.0], 0.86 + height_factor * 0.20);
            VoxelTerrainVisual {
                top_color,
                side_color: shade_rgba([0.16, 0.14, 0.13, 1.0], 0.80),
                edge_color: [0.58, 0.32, 0.16, 1.0],
                is_water,
            }
        }
        VoxelVolcanicSurface::Basalt => {
            let top_color = shade_rgba([0.095, 0.085, 0.10, 1.0], 0.86 + height_factor * 0.24);
            VoxelTerrainVisual {
                top_color,
                side_color: [0.040, 0.036, 0.046, 1.0],
                edge_color: [0.70, 0.22, 0.08, 1.0],
                is_water,
            }
        }
    }
}

fn apply_weather_to_visual(
    visual: VoxelTerrainVisual,
    weather: VoxelWeather,
) -> VoxelTerrainVisual {
    VoxelTerrainVisual {
        top_color: weather_grade_rgba(visual.top_color, weather, visual.is_water),
        side_color: weather_grade_rgba(visual.side_color, weather, visual.is_water),
        edge_color: weather_grade_rgba(visual.edge_color, weather, visual.is_water),
        is_water: visual.is_water,
    }
}

fn weather_grade_rgba(color: VoxelRgba, weather: VoxelWeather, is_water: bool) -> VoxelRgba {
    let lava_like = color[0] > 0.75 && color[1] < 0.38 && color[2] < 0.14;
    if lava_like {
        return match weather {
            VoxelWeather::Rain | VoxelWeather::Storm | VoxelWeather::Snow => {
                mix_rgba(color, [0.76, 0.18, 0.05, 1.0], 0.18)
            }
            VoxelWeather::Ashfall | VoxelWeather::DustStorm => {
                mix_rgba(color, [0.62, 0.24, 0.08, 1.0], 0.20)
            }
            _ => color,
        };
    }

    let graded = match weather {
        VoxelWeather::Clear => color,
        VoxelWeather::Cloudy => mix_rgba(shade_rgba(color, 0.92), [0.62, 0.66, 0.68, 1.0], 0.10),
        VoxelWeather::Rain => mix_rgba(shade_rgba(color, 0.78), [0.18, 0.28, 0.34, 1.0], 0.20),
        VoxelWeather::Storm => mix_rgba(shade_rgba(color, 0.64), [0.10, 0.13, 0.20, 1.0], 0.30),
        VoxelWeather::Snow => mix_rgba(shade_rgba(color, 1.04), [0.80, 0.90, 0.94, 1.0], 0.24),
        VoxelWeather::DustStorm => mix_rgba(shade_rgba(color, 0.86), [0.70, 0.54, 0.30, 1.0], 0.28),
        VoxelWeather::Ashfall => mix_rgba(shade_rgba(color, 0.72), [0.34, 0.30, 0.27, 1.0], 0.34),
        VoxelWeather::IonStorm => mix_rgba(shade_rgba(color, 0.78), [0.18, 0.44, 0.70, 1.0], 0.30),
    };

    if is_water {
        mix_rgba(graded, [0.08, 0.20, 0.34, 1.0], 0.10)
    } else {
        graded
    }
}

fn biome_base_color(biome: VoxelBiome) -> VoxelRgba {
    match biome {
        VoxelBiome::Plains => [0.36, 0.72, 0.22, 1.0],
        VoxelBiome::Forest => [0.10, 0.48, 0.15, 1.0],
        VoxelBiome::Desert => [0.96, 0.74, 0.30, 1.0],
        VoxelBiome::Tundra => [0.78, 0.92, 1.0, 1.0],
        VoxelBiome::Mountains => [0.58, 0.54, 0.46, 1.0],
        VoxelBiome::Wetlands => [0.16, 0.58, 0.20, 1.0],
        VoxelBiome::Badlands => [0.78, 0.40, 0.18, 1.0],
        VoxelBiome::CraterField => [0.42, 0.36, 0.30, 1.0],
        VoxelBiome::Volcanic => [0.12, 0.10, 0.11, 1.0],
        VoxelBiome::CrystalFields => [0.30, 0.76, 0.88, 1.0],
    }
}

fn biome_edge_color(biome: VoxelBiome) -> VoxelRgba {
    match biome {
        VoxelBiome::Plains => [0.18, 0.46, 0.10, 1.0],
        VoxelBiome::Forest => [0.04, 0.26, 0.06, 1.0],
        VoxelBiome::Desert => [0.74, 0.50, 0.12, 1.0],
        VoxelBiome::Tundra => [0.42, 0.72, 0.88, 1.0],
        VoxelBiome::Mountains => [0.38, 0.34, 0.27, 1.0],
        VoxelBiome::Wetlands => [0.06, 0.34, 0.08, 1.0],
        VoxelBiome::Badlands => [0.58, 0.22, 0.08, 1.0],
        VoxelBiome::CraterField => [0.24, 0.20, 0.17, 1.0],
        VoxelBiome::Volcanic => [0.72, 0.20, 0.06, 1.0],
        VoxelBiome::CrystalFields => [0.18, 0.56, 0.84, 1.0],
    }
}

fn biome_side_color(biome: VoxelBiome) -> VoxelRgba {
    match biome {
        VoxelBiome::Plains => [0.22, 0.42, 0.12, 1.0],
        VoxelBiome::Forest => [0.05, 0.25, 0.07, 1.0],
        VoxelBiome::Desert => [0.64, 0.38, 0.12, 1.0],
        VoxelBiome::Tundra => [0.48, 0.66, 0.74, 1.0],
        VoxelBiome::Mountains => [0.36, 0.32, 0.25, 1.0],
        VoxelBiome::Wetlands => [0.08, 0.31, 0.10, 1.0],
        VoxelBiome::Badlands => [0.50, 0.20, 0.07, 1.0],
        VoxelBiome::CraterField => [0.23, 0.20, 0.18, 1.0],
        VoxelBiome::Volcanic => [0.045, 0.038, 0.050, 1.0],
        VoxelBiome::CrystalFields => [0.13, 0.42, 0.64, 1.0],
    }
}

fn shade_rgba(color: VoxelRgba, factor: f32) -> VoxelRgba {
    [
        (color[0] * factor).clamp(0.0, 1.0),
        (color[1] * factor).clamp(0.0, 1.0),
        (color[2] * factor).clamp(0.0, 1.0),
        color[3].clamp(0.0, 1.0),
    ]
}

fn mix_rgba(a: VoxelRgba, b: VoxelRgba, t: f32) -> VoxelRgba {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

fn clamp_rgba(color: VoxelRgba) -> VoxelRgba {
    [
        color[0].clamp(0.0, 1.0),
        color[1].clamp(0.0, 1.0),
        color[2].clamp(0.0, 1.0),
        color[3].clamp(0.0, 1.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_for_biome(biome: VoxelBiome) -> VoxelColumnSample {
        VoxelColumnSample {
            world_x: 0,
            world_z: 0,
            height: 72,
            biome,
            weather: VoxelWeather::Clear,
            temperature: 0.5,
            moisture: 0.5,
            mountain_factor: if biome == VoxelBiome::Mountains {
                0.8
            } else {
                0.1
            },
        }
    }

    fn luminance(color: VoxelRgba) -> f32 {
        color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722
    }

    fn channel_spread(color: VoxelRgba) -> f32 {
        let min_channel = color[0].min(color[1]).min(color[2]);
        let max_channel = color[0].max(color[1]).max(color[2]);

        max_channel - min_channel
    }

    #[test]
    fn terrain_visual_uses_distinct_biome_colors() {
        let settings = VoxelWorldSettings::default();
        let plains = voxel_terrain_visual(settings, sample_for_biome(VoxelBiome::Plains));
        let desert = voxel_terrain_visual(settings, sample_for_biome(VoxelBiome::Desert));
        let tundra = voxel_terrain_visual(settings, sample_for_biome(VoxelBiome::Tundra));

        assert_ne!(plains.top_color, desert.top_color);
        assert_ne!(desert.top_color, tundra.top_color);
    }

    #[test]
    fn terrain_visual_edge_color_is_readable() {
        let settings = VoxelWorldSettings::default();
        let visual = voxel_terrain_visual(settings, sample_for_biome(VoxelBiome::Forest));

        assert_ne!(visual.top_color, visual.edge_color);
        for channel in visual.top_color.into_iter().chain(visual.edge_color) {
            assert!((0.0..=1.0).contains(&channel));
        }
    }

    #[test]
    fn dry_strategy_theme_uses_bright_tops_and_warm_canyon_sides() {
        let settings = VoxelWorldSettings::default();
        let desert = voxel_terrain_visual(settings, sample_for_biome(VoxelBiome::Desert));
        let badlands = voxel_terrain_visual(settings, sample_for_biome(VoxelBiome::Badlands));

        assert!(luminance(desert.top_color) > luminance(desert.side_color));
        assert!(desert.top_color[0] > desert.top_color[2] * 1.7);
        assert!(badlands.side_color[0] > badlands.side_color[2] * 2.4);
        assert_ne!(desert.side_color, desert.edge_color);
    }

    #[test]
    fn water_visual_uses_blue_edge_hint() {
        let settings = VoxelWorldSettings::default();
        let water = voxel_terrain_visual(
            settings,
            VoxelColumnSample {
                height: settings.sea_level - 8,
                ..sample_for_biome(VoxelBiome::Plains)
            },
        );

        assert!(water.is_water);
        assert!(water.edge_color[2] > water.edge_color[0]);
    }

    #[test]
    fn shallow_land_visual_is_not_blue_water() {
        let settings = VoxelWorldSettings::default();
        let land = voxel_terrain_visual(
            settings,
            VoxelColumnSample {
                height: settings.sea_level + 2,
                ..sample_for_biome(VoxelBiome::Plains)
            },
        );

        assert!(!land.is_water);
        assert!(land.top_color[0] > land.top_color[2]);
        assert!(land.top_color[1] > land.top_color[2]);
    }

    #[test]
    fn volcanic_ashfall_visual_uses_ash_and_rock_tones() {
        let settings = VoxelWorldSettings::default();
        let visual = voxel_terrain_visual(
            settings,
            VoxelColumnSample {
                weather: VoxelWeather::Ashfall,
                ..sample_for_biome(VoxelBiome::Volcanic)
            },
        );

        assert!(!visual.is_water);
        assert!(visual.top_color[0] >= visual.top_color[2]);
        assert!(visual.edge_color[0] > visual.edge_color[2]);
    }

    #[test]
    fn crystal_fields_use_saturated_cyan_terrain() {
        let settings = VoxelWorldSettings::default();
        let visual = voxel_terrain_visual(settings, sample_for_biome(VoxelBiome::CrystalFields));

        assert!(visual.top_color[2] > visual.top_color[0] * 1.7);
        assert!(visual.top_color[1] > visual.top_color[0] * 1.35);
        assert!(visual.edge_color[2] > visual.edge_color[0] * 2.8);
        assert!(channel_spread(visual.top_color) > 0.26);
    }

    #[test]
    fn block_palette_makes_resources_and_lava_readable() {
        let lava = voxel_block_color(BlockKind::Lava);
        let crystal = voxel_block_color(BlockKind::CrystalOre);
        let uranium = voxel_block_color(BlockKind::UraniumOre);
        let basalt = voxel_block_color(BlockKind::Basalt);

        assert!(lava[0] > 0.95);
        assert!(lava[1] < 0.30);
        assert!(crystal[2] > 0.95);
        assert!(crystal[1] > 0.86);
        assert!(uranium[1] > uranium[0] * 1.45);
        assert!(basalt[0] < 0.08 && basalt[1] < 0.08 && basalt[2] < 0.08);
        assert!(channel_spread(lava) > 0.70);
        assert!(channel_spread(crystal) > 0.60);
    }

    #[test]
    fn weather_changes_visible_terrain_grade() {
        let settings = VoxelWorldSettings::default();
        let clear = voxel_terrain_visual(settings, sample_for_biome(VoxelBiome::Plains));
        let storm = voxel_terrain_visual(
            settings,
            VoxelColumnSample {
                weather: VoxelWeather::Storm,
                ..sample_for_biome(VoxelBiome::Plains)
            },
        );

        assert_ne!(clear.top_color, storm.top_color);
        assert!(storm.top_color[2] >= storm.top_color[0] * 0.50);
    }
}
