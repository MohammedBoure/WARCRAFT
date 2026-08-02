use super::*;

pub(super) fn weather_from_climate(
    settings: VoxelWorldSettings,
    biome: VoxelBiome,
    temperature: f64,
    moisture: f64,
    mountain_factor: f64,
    weather_field: f64,
) -> VoxelWeather {
    let weights = settings.composition.weather_weights;
    let dry = 1.0 - moisture;
    let cold = 1.0 - temperature;
    let turbulence = smoothstep(0.52, 0.90, weather_field);
    let mut best = (VoxelWeather::Clear, 0.0);

    push_weather_score(
        &mut best,
        VoxelWeather::Clear,
        weights.clear,
        0.55 + dry * 0.25 + (1.0 - turbulence) * 0.20,
    );
    push_weather_score(
        &mut best,
        VoxelWeather::Cloudy,
        weights.cloudy,
        0.25 + moisture * 0.45 + turbulence * 0.15,
    );
    push_weather_score(
        &mut best,
        VoxelWeather::Rain,
        weights.rain,
        smoothstep(0.50, 0.86, moisture) * (1.0 - cold * 0.65),
    );
    push_weather_score(
        &mut best,
        VoxelWeather::Storm,
        weights.storm,
        smoothstep(0.56, 0.90, moisture) * (0.35 + turbulence * 0.65),
    );
    push_weather_score(
        &mut best,
        VoxelWeather::Snow,
        weights.snow,
        smoothstep(0.52, 0.86, cold) * (0.35 + moisture * 0.50),
    );
    push_weather_score(
        &mut best,
        VoxelWeather::DustStorm,
        weights.dust_storm,
        biome_weather_bonus(biome, VoxelWeather::DustStorm)
            * smoothstep(0.48, 0.82, dry)
            * (0.35 + turbulence * 0.65),
    );
    push_weather_score(
        &mut best,
        VoxelWeather::Ashfall,
        weights.ashfall,
        biome_weather_bonus(biome, VoxelWeather::Ashfall) * (0.55 + turbulence * 0.45),
    );
    push_weather_score(
        &mut best,
        VoxelWeather::IonStorm,
        weights.ion_storm,
        biome_weather_bonus(biome, VoxelWeather::IonStorm)
            * (0.40 + mountain_factor * 0.25 + turbulence * 0.35),
    );

    best.0
}

fn push_weather_score(
    best: &mut (VoxelWeather, f64),
    weather: VoxelWeather,
    weight: f64,
    score: f64,
) {
    if weight <= 0.0 {
        return;
    }

    let weighted = score.max(0.015) * weight;
    if weighted > best.1 {
        *best = (weather, weighted);
    }
}

fn biome_weather_bonus(biome: VoxelBiome, weather: VoxelWeather) -> f64 {
    match (biome, weather) {
        (
            VoxelBiome::Desert | VoxelBiome::Badlands | VoxelBiome::CraterField,
            VoxelWeather::DustStorm,
        ) => 1.0,
        (VoxelBiome::Volcanic, VoxelWeather::Ashfall) => 1.0,
        (VoxelBiome::CrystalFields | VoxelBiome::CraterField, VoxelWeather::IonStorm) => 1.0,
        _ => 0.05,
    }
}
