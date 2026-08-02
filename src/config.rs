use astra_voxel_world::prelude::*;
use crate::state::*;

impl ViewerOptions {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut options = Self::default();
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => options.help = true,
                "--seed" => options.seed = parse_u64_arg("--seed", args.next())?,
                "--start-x" => options.start_x = parse_i64_arg("--start-x", args.next())?,
                "--start-z" => options.start_z = parse_i64_arg("--start-z", args.next())?,
                "--radius" => options.load_radius = parse_i64_arg("--radius", args.next())?,
                "--world-height" => {
                    options.world_height = parse_u16_arg("--world-height", args.next())?
                }
                "--sea-level" => options.sea_level = parse_i32_arg("--sea-level", args.next())?,
                "--base-height" => {
                    options.base_height = parse_i32_arg("--base-height", args.next())?
                }
                "--terrain-amplitude" => {
                    options.terrain_amplitude = parse_i32_arg("--terrain-amplitude", args.next())?
                }
                "--mountain-amplitude" => {
                    options.mountain_amplitude = parse_i32_arg("--mountain-amplitude", args.next())?
                }
                "--cave-density" => {
                    options.cave_density = parse_f64_arg("--cave-density", args.next())?
                }
                "--tree-density" => {
                    options.tree_density = parse_f64_arg("--tree-density", args.next())?
                }
                "--preset" => apply_preset_arg(&mut options.composition, args.next())?,
                "--biome" => apply_forced_biome_arg(&mut options.composition, args.next())?,
                "--weather" => apply_forced_weather_arg(&mut options.composition, args.next())?,
                "--biome-weight" => apply_biome_weight_arg(&mut options.composition, args.next())?,
                "--weather-weight" => {
                    apply_weather_weight_arg(&mut options.composition, args.next())?
                }
                "--resource-ratio" => {
                    apply_resource_ratio_arg(&mut options.composition, args.next())?
                }
                "--terrain-feature" => {
                    apply_terrain_feature_arg(&mut options.composition, args.next())?
                }
                unknown => return Err(format!("unknown argument `{unknown}`")),
            }
        }

        options.load_radius = options.load_radius.clamp(1, LOAD_RADIUS_MAX);
        Ok(options)
    }

    pub fn help_text() -> &'static str {
        "Usage:
  cargo run -p astra_voxel_world --bin voxel_world_viewer -- [options]

Options:
  --seed <u64>      World seed. Decimal or 0xHEX.
  --start-x <i64>   Starting world block X.
  --start-z <i64>   Starting world block Z.
  --radius <i64>    Maximum chunk streaming radius, default 18, max 32.
  --world-height <u16>
                    Voxel world height.
  --sea-level <i32> Sea level.
  --base-height <i32>
                    Baseline terrain height.
  --terrain-amplitude <i32>
                    Broad terrain amplitude.
  --mountain-amplitude <i32>
                    Mountain height amplitude.
  --cave-density <f64>
                    Cave generation density.
  --tree-density <f64>
                    Tree generation density.
  --preset <name>   balanced, lush, dry, frozen, volcanic, crystal, crater.
  --biome <name>    Force one biome for testing.
  --weather <name>  Force one weather type for testing.
  --biome-weight <name=value>
                    Override one biome weight. Repeatable.
  --weather-weight <name=value>
                    Override one weather weight. Repeatable.
  --resource-ratio <name=value>
                    Override coal, iron, gold, or crystal ratio. Repeatable.
  --terrain-feature <name=value>
                    Override craters, large-craters, rifts, canyons,
                    high-mountains, plateaus, or erosion. Repeatable.
  --help            Show this help.

Controls:
  WASD / arrows     Move over the world.
  Hold Shift        Move faster.
  Mouse wheel       Zoom in/out.
  Q / E             Rotate view.
  Space             Reset zoom.
  P                 Cycle generation preset.
  B                 Force next biome.
  T                 Force next weather.
  [ / ]             Lower/raise crystal resource ratio.
  M                 Select editable numeric generation input.
  - / =             Lower/raise the selected numeric input.
  N                 Advance seed and regenerate.
  0                 Reset generation settings except seed.
  Click INPUTS      Open a dialog and apply argument-style generation inputs.
"
    }
}

pub fn parse_u64_arg(name: &str, value: Option<String>) -> Result<u64, String> {
    let value = value.ok_or_else(|| format!("missing value for {name}"))?;
    parse_u64_str(&value).map_err(|error| format!("invalid value for {name}: {error}"))
}

pub fn parse_u64_str(value: &str) -> Result<u64, String> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).map_err(|error| error.to_string())
    } else {
        trimmed.parse::<u64>().map_err(|error| error.to_string())
    }
}

pub fn parse_i64_arg(name: &str, value: Option<String>) -> Result<i64, String> {
    let value = value.ok_or_else(|| format!("missing value for {name}"))?;
    value.parse::<i64>().map_err(|error| format!("invalid value for {name}: {error}"))
}

pub fn parse_u16_arg(name: &str, value: Option<String>) -> Result<u16, String> {
    let value = value.ok_or_else(|| format!("missing value for {name}"))?;
    value.parse::<u16>().map_err(|error| format!("invalid value for {name}: {error}"))
}

pub fn parse_i32_arg(name: &str, value: Option<String>) -> Result<i32, String> {
    let value = value.ok_or_else(|| format!("missing value for {name}"))?;
    value.parse::<i32>().map_err(|error| format!("invalid value for {name}: {error}"))
}

pub fn parse_f64_arg(name: &str, value: Option<String>) -> Result<f64, String> {
    let value = value.ok_or_else(|| format!("missing value for {name}"))?;
    value.parse::<f64>().map_err(|error| format!("invalid value for {name}: {error}"))
}

pub fn apply_preset_arg(composition: &mut VoxelWorldComposition, value: Option<String>) -> Result<(), String> {
    let name = value.ok_or_else(|| "missing value for --preset".to_string())?;
    let preset = VoxelWorldComposition::preset(&name)
        .ok_or_else(|| format!("unknown preset `{name}`"))?;
    *composition = preset;
    Ok(())
}

pub fn apply_forced_biome_arg(composition: &mut VoxelWorldComposition, value: Option<String>) -> Result<(), String> {
    let name = value.ok_or_else(|| "missing value for --biome".to_string())?;
    let biome = VoxelBiome::from_name(&name).ok_or_else(|| format!("unknown biome `{name}`"))?;
    composition.force_biome(biome);
    Ok(())
}

pub fn apply_forced_weather_arg(composition: &mut VoxelWorldComposition, value: Option<String>) -> Result<(), String> {
    let name = value.ok_or_else(|| "missing value for --weather".to_string())?;
    let weather = VoxelWeather::from_name(&name).ok_or_else(|| format!("unknown weather `{name}`"))?;
    composition.force_weather(weather);
    Ok(())
}

pub fn apply_biome_weight_arg(composition: &mut VoxelWorldComposition, value: Option<String>) -> Result<(), String> {
    let value = value.ok_or_else(|| "missing value for --biome-weight".to_string())?;
    let (name, weight) = parse_name_weight_pair("--biome-weight", &value)?;
    let biome = VoxelBiome::from_name(name).ok_or_else(|| format!("unknown biome `{name}`"))?;
    composition.biome_weights.set(biome, weight);
    Ok(())
}

pub fn apply_weather_weight_arg(composition: &mut VoxelWorldComposition, value: Option<String>) -> Result<(), String> {
    let value = value.ok_or_else(|| "missing value for --weather-weight".to_string())?;
    let (name, weight) = parse_name_weight_pair("--weather-weight", &value)?;
    let weather = VoxelWeather::from_name(name).ok_or_else(|| format!("unknown weather `{name}`"))?;
    composition.weather_weights.set(weather, weight);
    Ok(())
}

pub fn apply_resource_ratio_arg(composition: &mut VoxelWorldComposition, value: Option<String>) -> Result<(), String> {
    let value = value.ok_or_else(|| "missing value for --resource-ratio".to_string())?;
    let (name, ratio) = parse_name_weight_pair("--resource-ratio", &value)?;
    if !composition.resource_ratios.set_named(name, ratio) {
        return Err(format!("unknown resource `{name}`"));
    }
    Ok(())
}

pub fn apply_terrain_feature_arg(composition: &mut VoxelWorldComposition, value: Option<String>) -> Result<(), String> {
    let value = value.ok_or_else(|| "missing value for --terrain-feature".to_string())?;
    let (name, weight) = parse_name_weight_pair("--terrain-feature", &value)?;
    if !composition.terrain_features.set_named(name, weight) {
        return Err(format!("unknown terrain feature `{name}`"));
    }
    Ok(())
}

pub fn parse_name_weight_pair<'a>(flag_name: &str, pair: &'a str) -> Result<(&'a str, f64), String> {
    let (name, weight_str) = pair
        .split_once('=')
        .ok_or_else(|| format!("expected `name=value` for {flag_name}"))?;
    let weight = weight_str
        .trim()
        .parse::<f64>()
        .map_err(|error| format!("invalid weight for {flag_name}: {error}"))?;
    Ok((name.trim(), weight))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenerationDialogUpdate {
    pub settings: VoxelWorldSettings,
    pub load_radius: i64,
}

pub fn parse_generation_arguments(
    buffer: &str,
    current_settings: VoxelWorldSettings,
    _current_radius: i64,
) -> Result<GenerationDialogUpdate, String> {
    let mut tokens = Vec::new();
    for line in buffer.lines() {
        let text = line.split('#').next().unwrap_or("").trim();
        if text.is_empty() {
            continue;
        }
        for token in text.split_whitespace() {
            tokens.push(token.to_string());
        }
    }

    let options = ViewerOptions::parse(tokens)?;
    let mut settings = current_settings;
    settings.seed = options.seed;
    settings.world_height = options.world_height;
    settings.sea_level = options.sea_level;
    settings.base_height = options.base_height;
    settings.terrain_amplitude = options.terrain_amplitude;
    settings.mountain_amplitude = options.mountain_amplitude;
    settings.cave_density = options.cave_density;
    settings.tree_density = options.tree_density;
    settings.composition = options.composition;

    Ok(GenerationDialogUpdate {
        settings: settings.sanitized(),
        load_radius: options.load_radius.clamp(1, LOAD_RADIUS_MAX),
    })
}

pub fn generation_arguments_text(settings: VoxelWorldSettings, load_radius: i64) -> String {
    let settings = settings.sanitized();
    let biomes = VoxelBiome::ALL
        .into_iter()
        .map(|biome| {
            format!(
                "--biome-weight {}={:.3}",
                biome.name(),
                settings.composition.biome_weights.get(biome)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let weather = VoxelWeather::ALL
        .into_iter()
        .map(|weather| {
            format!(
                "--weather-weight {}={:.3}",
                weather.name(),
                settings.composition.weather_weights.get(weather)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let terrain_features = terrain_feature_arguments_text(settings.composition.terrain_features);

    format!(
        "# Edit these arguments, then Apply.\n--seed 0x{seed:016X}\n--radius {load_radius}\n--world-height {world_height}\n--sea-level {sea_level}\n--base-height {base_height}\n--terrain-amplitude {terrain_amplitude}\n--mountain-amplitude {mountain_amplitude}\n--cave-density {cave_density:.3}\n--tree-density {tree_density:.3}\n{biomes}\n{weather}\n--resource-ratio coal={coal:.3}\n--resource-ratio iron={iron:.3}\n--resource-ratio gold={gold:.3}\n--resource-ratio crystal={crystal:.3}\n{terrain_features}\n",
        seed = settings.seed,
        world_height = settings.world_height,
        sea_level = settings.sea_level,
        base_height = settings.base_height,
        terrain_amplitude = settings.terrain_amplitude,
        mountain_amplitude = settings.mountain_amplitude,
        cave_density = settings.cave_density,
        tree_density = settings.tree_density,
        coal = settings.composition.resource_ratios.coal,
        iron = settings.composition.resource_ratios.iron,
        gold = settings.composition.resource_ratios.gold,
        crystal = settings.composition.resource_ratios.crystal,
    )
}

pub fn terrain_feature_arguments_text(features: VoxelTerrainFeatureWeights) -> String {
    format!(
        "--terrain-feature craters={:.3}\n--terrain-feature large-craters={:.3}\n--terrain-feature rifts={:.3}\n--terrain-feature canyons={:.3}\n--terrain-feature high-mountains={:.3}\n--terrain-feature plateaus={:.3}\n--terrain-feature erosion={:.3}",
        features.craters,
        features.large_craters,
        features.rifts,
        features.canyons,
        features.high_mountains,
        features.plateaus,
        features.erosion,
    )
}
