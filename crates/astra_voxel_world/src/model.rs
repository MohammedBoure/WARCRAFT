use std::fmt;

pub const DEFAULT_CHUNK_SIZE: usize = 16;
pub const DEFAULT_WORLD_HEIGHT: usize = 160;
pub const MIN_WORLD_HEIGHT: u16 = 64;
pub const MAX_WORLD_HEIGHT: u16 = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BlockKind {
    Air = 0,
    Bedrock = 1,
    Stone = 2,
    Dirt = 3,
    Grass = 4,
    Sand = 5,
    Water = 6,
    Snow = 7,
    Wood = 8,
    Leaves = 9,
    CoalOre = 10,
    IronOre = 11,
    GoldOre = 12,
    Mud = 13,
    Basalt = 14,
    Ice = 15,
    CrystalOre = 16,
    VolcanicAsh = 17,
    Lava = 18,
    TitaniumOre = 19,
    UraniumOre = 20,
    HeliumVent = 21,
    BioPlasmaBloom = 22,
    AncientRelic = 23,
}

impl BlockKind {
    pub const fn is_solid(self) -> bool {
        !matches!(self, Self::Air | Self::Water | Self::Lava)
    }

    pub const fn is_terrain(self) -> bool {
        matches!(
            self,
            Self::Bedrock
                | Self::Stone
                | Self::Dirt
                | Self::Grass
                | Self::Sand
                | Self::Snow
                | Self::Mud
                | Self::Basalt
                | Self::Ice
                | Self::CrystalOre
                | Self::VolcanicAsh
                | Self::Lava
                | Self::CoalOre
                | Self::IronOre
                | Self::GoldOre
                | Self::TitaniumOre
                | Self::UraniumOre
                | Self::HeliumVent
                | Self::BioPlasmaBloom
                | Self::AncientRelic
        )
    }

    pub const fn is_ore(self) -> bool {
        matches!(
            self,
            Self::CoalOre
                | Self::IronOre
                | Self::GoldOre
                | Self::CrystalOre
                | Self::TitaniumOre
                | Self::UraniumOre
                | Self::HeliumVent
                | Self::BioPlasmaBloom
                | Self::AncientRelic
        )
    }

    pub const fn is_surface_resource(self) -> bool {
        matches!(
            self,
            Self::CoalOre
                | Self::IronOre
                | Self::GoldOre
                | Self::CrystalOre
                | Self::TitaniumOre
                | Self::UraniumOre
                | Self::HeliumVent
                | Self::BioPlasmaBloom
                | Self::AncientRelic
        )
    }

    pub const fn resource_key(self) -> Option<&'static str> {
        match self {
            Self::CoalOre => Some("basalt_stone"),
            Self::IronOre => Some("space_iron"),
            Self::GoldOre => Some("osmium"),
            Self::CrystalOre => Some("silicate_crystal"),
            Self::TitaniumOre => Some("titanium"),
            Self::UraniumOre => Some("uranium"),
            Self::HeliumVent => Some("helium_3"),
            Self::BioPlasmaBloom => Some("bio_plasma"),
            Self::AncientRelic => Some("ancient_relic"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VoxelBiome {
    Plains,
    Forest,
    Desert,
    Tundra,
    Mountains,
    Wetlands,
    Badlands,
    CraterField,
    Volcanic,
    CrystalFields,
}

impl VoxelBiome {
    pub const ALL: [Self; 10] = [
        Self::Plains,
        Self::Forest,
        Self::Desert,
        Self::Tundra,
        Self::Mountains,
        Self::Wetlands,
        Self::Badlands,
        Self::CraterField,
        Self::Volcanic,
        Self::CrystalFields,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Plains => "plains",
            Self::Forest => "forest",
            Self::Desert => "desert",
            Self::Tundra => "tundra",
            Self::Mountains => "mountains",
            Self::Wetlands => "wetlands",
            Self::Badlands => "badlands",
            Self::CraterField => "crater-field",
            Self::Volcanic => "volcanic",
            Self::CrystalFields => "crystal-fields",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match normalized_name(name).as_str() {
            "plains" => Some(Self::Plains),
            "forest" => Some(Self::Forest),
            "desert" => Some(Self::Desert),
            "tundra" => Some(Self::Tundra),
            "mountains" | "mountain" => Some(Self::Mountains),
            "wetlands" | "wetland" | "swamp" => Some(Self::Wetlands),
            "badlands" | "badland" => Some(Self::Badlands),
            "craterfield" | "craterfields" | "crater" => Some(Self::CraterField),
            "volcanic" | "volcano" => Some(Self::Volcanic),
            "crystalfields" | "crystalfield" | "crystal" => Some(Self::CrystalFields),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VoxelWeather {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Snow,
    DustStorm,
    Ashfall,
    IonStorm,
}

impl VoxelWeather {
    pub const ALL: [Self; 8] = [
        Self::Clear,
        Self::Cloudy,
        Self::Rain,
        Self::Storm,
        Self::Snow,
        Self::DustStorm,
        Self::Ashfall,
        Self::IonStorm,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::Cloudy => "cloudy",
            Self::Rain => "rain",
            Self::Storm => "storm",
            Self::Snow => "snow",
            Self::DustStorm => "dust-storm",
            Self::Ashfall => "ashfall",
            Self::IonStorm => "ion-storm",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match normalized_name(name).as_str() {
            "clear" => Some(Self::Clear),
            "cloudy" | "cloud" => Some(Self::Cloudy),
            "rain" | "rainy" => Some(Self::Rain),
            "storm" | "stormy" => Some(Self::Storm),
            "snow" | "snowy" => Some(Self::Snow),
            "duststorm" | "dust" => Some(Self::DustStorm),
            "ashfall" | "ash" => Some(Self::Ashfall),
            "ionstorm" | "ion" => Some(Self::IonStorm),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelBiomeWeights {
    pub plains: f64,
    pub forest: f64,
    pub desert: f64,
    pub tundra: f64,
    pub mountains: f64,
    pub wetlands: f64,
    pub badlands: f64,
    pub crater_fields: f64,
    pub volcanic: f64,
    pub crystal_fields: f64,
}

impl Default for VoxelBiomeWeights {
    fn default() -> Self {
        Self {
            plains: 0.82,
            forest: 0.88,
            desert: 0.58,
            tundra: 0.50,
            mountains: 0.70,
            wetlands: 0.45,
            badlands: 0.30,
            crater_fields: 0.16,
            volcanic: 0.10,
            crystal_fields: 0.08,
        }
    }
}

impl VoxelBiomeWeights {
    pub fn only(biome: VoxelBiome) -> Self {
        let mut weights = Self {
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
        };
        weights.set(biome, 1.0);
        weights
    }

    pub fn get(self, biome: VoxelBiome) -> f64 {
        match biome {
            VoxelBiome::Plains => self.plains,
            VoxelBiome::Forest => self.forest,
            VoxelBiome::Desert => self.desert,
            VoxelBiome::Tundra => self.tundra,
            VoxelBiome::Mountains => self.mountains,
            VoxelBiome::Wetlands => self.wetlands,
            VoxelBiome::Badlands => self.badlands,
            VoxelBiome::CraterField => self.crater_fields,
            VoxelBiome::Volcanic => self.volcanic,
            VoxelBiome::CrystalFields => self.crystal_fields,
        }
    }

    pub fn set(&mut self, biome: VoxelBiome, weight: f64) {
        let weight = sanitize_weight(weight);
        match biome {
            VoxelBiome::Plains => self.plains = weight,
            VoxelBiome::Forest => self.forest = weight,
            VoxelBiome::Desert => self.desert = weight,
            VoxelBiome::Tundra => self.tundra = weight,
            VoxelBiome::Mountains => self.mountains = weight,
            VoxelBiome::Wetlands => self.wetlands = weight,
            VoxelBiome::Badlands => self.badlands = weight,
            VoxelBiome::CraterField => self.crater_fields = weight,
            VoxelBiome::Volcanic => self.volcanic = weight,
            VoxelBiome::CrystalFields => self.crystal_fields = weight,
        }
    }

    pub fn sanitized(self) -> Self {
        let sanitized = Self {
            plains: sanitize_weight(self.plains),
            forest: sanitize_weight(self.forest),
            desert: sanitize_weight(self.desert),
            tundra: sanitize_weight(self.tundra),
            mountains: sanitize_weight(self.mountains),
            wetlands: sanitize_weight(self.wetlands),
            badlands: sanitize_weight(self.badlands),
            crater_fields: sanitize_weight(self.crater_fields),
            volcanic: sanitize_weight(self.volcanic),
            crystal_fields: sanitize_weight(self.crystal_fields),
        };

        if sanitized.total() <= f64::EPSILON {
            Self::default()
        } else {
            sanitized
        }
    }

    pub fn total(self) -> f64 {
        self.plains
            + self.forest
            + self.desert
            + self.tundra
            + self.mountains
            + self.wetlands
            + self.badlands
            + self.crater_fields
            + self.volcanic
            + self.crystal_fields
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelWeatherWeights {
    pub clear: f64,
    pub cloudy: f64,
    pub rain: f64,
    pub storm: f64,
    pub snow: f64,
    pub dust_storm: f64,
    pub ashfall: f64,
    pub ion_storm: f64,
}

impl Default for VoxelWeatherWeights {
    fn default() -> Self {
        Self {
            clear: 1.0,
            cloudy: 0.52,
            rain: 0.24,
            storm: 0.08,
            snow: 0.12,
            dust_storm: 0.07,
            ashfall: 0.035,
            ion_storm: 0.025,
        }
    }
}

impl VoxelWeatherWeights {
    pub fn only(weather: VoxelWeather) -> Self {
        let mut weights = Self {
            clear: 0.0,
            cloudy: 0.0,
            rain: 0.0,
            storm: 0.0,
            snow: 0.0,
            dust_storm: 0.0,
            ashfall: 0.0,
            ion_storm: 0.0,
        };
        weights.set(weather, 1.0);
        weights
    }

    pub fn get(self, weather: VoxelWeather) -> f64 {
        match weather {
            VoxelWeather::Clear => self.clear,
            VoxelWeather::Cloudy => self.cloudy,
            VoxelWeather::Rain => self.rain,
            VoxelWeather::Storm => self.storm,
            VoxelWeather::Snow => self.snow,
            VoxelWeather::DustStorm => self.dust_storm,
            VoxelWeather::Ashfall => self.ashfall,
            VoxelWeather::IonStorm => self.ion_storm,
        }
    }

    pub fn set(&mut self, weather: VoxelWeather, weight: f64) {
        let weight = sanitize_weight(weight);
        match weather {
            VoxelWeather::Clear => self.clear = weight,
            VoxelWeather::Cloudy => self.cloudy = weight,
            VoxelWeather::Rain => self.rain = weight,
            VoxelWeather::Storm => self.storm = weight,
            VoxelWeather::Snow => self.snow = weight,
            VoxelWeather::DustStorm => self.dust_storm = weight,
            VoxelWeather::Ashfall => self.ashfall = weight,
            VoxelWeather::IonStorm => self.ion_storm = weight,
        }
    }

    pub fn sanitized(self) -> Self {
        let sanitized = Self {
            clear: sanitize_weight(self.clear),
            cloudy: sanitize_weight(self.cloudy),
            rain: sanitize_weight(self.rain),
            storm: sanitize_weight(self.storm),
            snow: sanitize_weight(self.snow),
            dust_storm: sanitize_weight(self.dust_storm),
            ashfall: sanitize_weight(self.ashfall),
            ion_storm: sanitize_weight(self.ion_storm),
        };

        if sanitized.total() <= f64::EPSILON {
            Self::default()
        } else {
            sanitized
        }
    }

    pub fn total(self) -> f64 {
        self.clear
            + self.cloudy
            + self.rain
            + self.storm
            + self.snow
            + self.dust_storm
            + self.ashfall
            + self.ion_storm
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelResourceRatios {
    pub coal: f64,
    pub iron: f64,
    pub gold: f64,
    pub crystal: f64,
}

impl Default for VoxelResourceRatios {
    fn default() -> Self {
        Self {
            coal: 1.0,
            iron: 1.0,
            gold: 1.0,
            crystal: 1.0,
        }
    }
}

impl VoxelResourceRatios {
    pub fn set_named(&mut self, resource: &str, ratio: f64) -> bool {
        let ratio = sanitize_ratio(ratio, 6.0);
        match normalized_name(resource).as_str() {
            "coal" => self.coal = ratio.clamp(0.0, 4.0),
            "iron" => self.iron = ratio.clamp(0.0, 4.0),
            "gold" => self.gold = ratio.clamp(0.0, 4.0),
            "crystal" | "crystals" => self.crystal = ratio,
            _ => return false,
        }
        true
    }

    pub fn sanitized(self) -> Self {
        Self {
            coal: sanitize_ratio(self.coal, 4.0),
            iron: sanitize_ratio(self.iron, 4.0),
            gold: sanitize_ratio(self.gold, 4.0),
            crystal: sanitize_ratio(self.crystal, 6.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelTerrainFeatureWeights {
    pub craters: f64,
    pub large_craters: f64,
    pub rifts: f64,
    pub canyons: f64,
    pub high_mountains: f64,
    pub plateaus: f64,
    pub erosion: f64,
}

impl Default for VoxelTerrainFeatureWeights {
    fn default() -> Self {
        Self {
            craters: 0.35,
            large_craters: 0.12,
            rifts: 0.18,
            canyons: 0.20,
            high_mountains: 0.32,
            plateaus: 0.25,
            erosion: 0.45,
        }
    }
}

impl VoxelTerrainFeatureWeights {
    pub fn set_named(&mut self, feature: &str, weight: f64) -> bool {
        let weight = sanitize_feature_weight(weight);
        match normalized_name(feature).as_str() {
            "crater" | "craters" => self.craters = weight,
            "largecrater" | "largecraters" => self.large_craters = weight,
            "rift" | "rifts" => self.rifts = weight,
            "canyon" | "canyons" => self.canyons = weight,
            "highmountain" | "highmountains" | "mountainpeaks" | "peaks" => {
                self.high_mountains = weight
            }
            "plateau" | "plateaus" => self.plateaus = weight,
            "erosion" => self.erosion = weight,
            _ => return false,
        }
        true
    }

    pub fn sanitized(self) -> Self {
        Self {
            craters: sanitize_feature_weight(self.craters),
            large_craters: sanitize_feature_weight(self.large_craters),
            rifts: sanitize_feature_weight(self.rifts),
            canyons: sanitize_feature_weight(self.canyons),
            high_mountains: sanitize_feature_weight(self.high_mountains),
            plateaus: sanitize_feature_weight(self.plateaus),
            erosion: sanitize_feature_weight(self.erosion),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VoxelTerrainFeaturePresence {
    pub craters: f64,
    pub large_craters: f64,
    pub rifts: f64,
    pub canyons: f64,
    pub high_mountains: f64,
    pub plateaus: f64,
    pub erosion: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelTerrainDiversityReport {
    pub sample_count: usize,
    pub min_height: i32,
    pub max_height: i32,
    pub height_range: i32,
    pub distinct_biomes: usize,
    pub distinct_weather: usize,
    pub average_features: VoxelTerrainFeaturePresence,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VoxelWorldComposition {
    pub biome_weights: VoxelBiomeWeights,
    pub weather_weights: VoxelWeatherWeights,
    pub resource_ratios: VoxelResourceRatios,
    pub terrain_features: VoxelTerrainFeatureWeights,
}

impl VoxelWorldComposition {
    pub fn preset(name: &str) -> Option<Self> {
        let mut composition = Self::default();
        match normalized_name(name).as_str() {
            "balanced" | "default" => Some(composition),
            "lush" | "forest" => {
                composition.biome_weights.forest = 2.2;
                composition.biome_weights.wetlands = 1.0;
                composition.biome_weights.desert = 0.15;
                composition.weather_weights.rain = 1.4;
                composition.weather_weights.cloudy = 1.0;
                Some(composition)
            }
            "dry" | "desert" => {
                composition.biome_weights.desert = 2.5;
                composition.biome_weights.badlands = 1.3;
                composition.biome_weights.wetlands = 0.05;
                composition.weather_weights.dust_storm = 1.4;
                composition.weather_weights.rain = 0.05;
                composition.terrain_features.canyons = 0.85;
                composition.terrain_features.erosion = 1.20;
                Some(composition)
            }
            "frozen" | "tundra" => {
                composition.biome_weights.tundra = 2.6;
                composition.biome_weights.mountains = 1.0;
                composition.weather_weights.snow = 2.0;
                composition.weather_weights.rain = 0.05;
                composition.terrain_features.plateaus = 0.75;
                composition.terrain_features.erosion = 0.18;
                Some(composition)
            }
            "volcanic" | "lava" => {
                composition.biome_weights.volcanic = 3.0;
                composition.biome_weights.badlands = 0.9;
                composition.biome_weights.forest = 0.05;
                composition.weather_weights.ashfall = 2.0;
                composition.resource_ratios.coal = 1.5;
                composition.resource_ratios.iron = 1.4;
                composition.terrain_features.rifts = 1.80;
                composition.terrain_features.high_mountains = 1.25;
                Some(composition)
            }
            "crystal" | "crystals" => {
                composition.biome_weights.crystal_fields = 3.0;
                composition.biome_weights.crater_fields = 0.9;
                composition.weather_weights.ion_storm = 2.0;
                composition.resource_ratios.crystal = 5.0;
                composition.terrain_features.plateaus = 0.95;
                composition.terrain_features.craters = 0.50;
                Some(composition)
            }
            "crater" | "craters" => {
                composition.biome_weights.crater_fields = 3.0;
                composition.biome_weights.badlands = 1.1;
                composition.weather_weights.dust_storm = 0.9;
                composition.weather_weights.ion_storm = 0.6;
                composition.terrain_features.craters = 1.80;
                composition.terrain_features.large_craters = 1.10;
                Some(composition)
            }
            _ => None,
        }
    }

    pub fn force_biome(&mut self, biome: VoxelBiome) {
        self.biome_weights = VoxelBiomeWeights::only(biome);
    }

    pub fn force_weather(&mut self, weather: VoxelWeather) {
        self.weather_weights = VoxelWeatherWeights::only(weather);
    }

    pub fn sanitized(self) -> Self {
        Self {
            biome_weights: self.biome_weights.sanitized(),
            weather_weights: self.weather_weights.sanitized(),
            resource_ratios: self.resource_ratios.sanitized(),
            terrain_features: self.terrain_features.sanitized(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VoxelChunkCoord {
    pub x: i64,
    pub z: i64,
}

impl VoxelChunkCoord {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: i64, z: i64) -> Self {
        Self { x, z }
    }

    pub fn world_x(self, local_x: usize) -> i64 {
        self.x
            .saturating_mul(DEFAULT_CHUNK_SIZE as i64)
            .saturating_add(local_x as i64)
    }

    pub fn world_z(self, local_z: usize) -> i64 {
        self.z
            .saturating_mul(DEFAULT_CHUNK_SIZE as i64)
            .saturating_add(local_z as i64)
    }
}

impl fmt::Display for VoxelChunkCoord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "({}, {})", self.x, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelWorldSettings {
    pub seed: u64,
    pub world_height: u16,
    pub sea_level: i32,
    pub base_height: i32,
    pub terrain_amplitude: i32,
    pub mountain_amplitude: i32,
    pub cave_density: f64,
    pub tree_density: f64,
    pub composition: VoxelWorldComposition,
}

impl Default for VoxelWorldSettings {
    fn default() -> Self {
        Self {
            seed: 0xA57A_B10C_0000_0001,
            world_height: DEFAULT_WORLD_HEIGHT as u16,
            sea_level: 58,
            base_height: 66,
            terrain_amplitude: 26,
            mountain_amplitude: 54,
            cave_density: 0.50,
            tree_density: 0.020,
            composition: VoxelWorldComposition::default(),
        }
    }
}

impl VoxelWorldSettings {
    pub fn sanitized(self) -> Self {
        let world_height = self.world_height.clamp(MIN_WORLD_HEIGHT, MAX_WORLD_HEIGHT);
        let max_height = i32::from(world_height).saturating_sub(8);

        Self {
            seed: self.seed,
            world_height,
            sea_level: self.sea_level.clamp(8, max_height),
            base_height: self.base_height.clamp(12, max_height),
            terrain_amplitude: self.terrain_amplitude.clamp(4, max_height),
            mountain_amplitude: self.mountain_amplitude.clamp(0, max_height),
            cave_density: self.cave_density.clamp(0.0, 1.0),
            tree_density: self.tree_density.clamp(0.0, 0.20),
            composition: self.composition.sanitized(),
        }
    }

    pub const fn chunk_size(self) -> usize {
        DEFAULT_CHUNK_SIZE
    }

    pub fn height_usize(self) -> usize {
        usize::from(self.sanitized().world_height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelColumnSample {
    pub world_x: i64,
    pub world_z: i64,
    pub height: i32,
    pub biome: VoxelBiome,
    pub weather: VoxelWeather,
    pub temperature: f64,
    pub moisture: f64,
    pub mountain_factor: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoxelChunk {
    coord: VoxelChunkCoord,
    world_height: usize,
    blocks: Vec<BlockKind>,
}

impl VoxelChunk {
    pub fn new(coord: VoxelChunkCoord, world_height: usize) -> Self {
        let world_height =
            world_height.clamp(usize::from(MIN_WORLD_HEIGHT), usize::from(MAX_WORLD_HEIGHT));
        Self {
            coord,
            world_height,
            blocks: vec![BlockKind::Air; DEFAULT_CHUNK_SIZE * world_height * DEFAULT_CHUNK_SIZE],
        }
    }

    pub const fn coord(&self) -> VoxelChunkCoord {
        self.coord
    }

    pub const fn chunk_size(&self) -> usize {
        DEFAULT_CHUNK_SIZE
    }

    pub const fn world_height(&self) -> usize {
        self.world_height
    }

    pub fn blocks(&self) -> &[BlockKind] {
        &self.blocks
    }

    pub fn get(&self, local_x: usize, y: usize, local_z: usize) -> Option<BlockKind> {
        let index = self.index(local_x, y, local_z)?;
        self.blocks.get(index).copied()
    }

    pub fn set(&mut self, local_x: usize, y: usize, local_z: usize, block: BlockKind) -> bool {
        let Some(index) = self.index(local_x, y, local_z) else {
            return false;
        };
        self.blocks[index] = block;
        true
    }

    pub fn set_world(&mut self, world_x: i64, y: i32, world_z: i64, block: BlockKind) -> bool {
        if y < 0 {
            return false;
        }
        let local_x = world_x.saturating_sub(self.coord.world_x(0));
        let local_z = world_z.saturating_sub(self.coord.world_z(0));
        if !(0..DEFAULT_CHUNK_SIZE as i64).contains(&local_x)
            || !(0..DEFAULT_CHUNK_SIZE as i64).contains(&local_z)
        {
            return false;
        }

        self.set(local_x as usize, y as usize, local_z as usize, block)
    }

    pub fn world_x(&self, local_x: usize) -> i64 {
        self.coord.world_x(local_x)
    }

    pub fn world_z(&self, local_z: usize) -> i64 {
        self.coord.world_z(local_z)
    }

    pub fn highest_terrain_y(&self, local_x: usize, local_z: usize) -> Option<i32> {
        if local_x >= DEFAULT_CHUNK_SIZE || local_z >= DEFAULT_CHUNK_SIZE {
            return None;
        }

        (0..self.world_height).rev().find_map(|y| {
            self.get(local_x, y, local_z)
                .filter(|block| block.is_terrain())
                .map(|_| y as i32)
        })
    }

    pub fn count_blocks(&self, block: BlockKind) -> usize {
        self.blocks
            .iter()
            .filter(|candidate| **candidate == block)
            .count()
    }

    fn index(&self, local_x: usize, y: usize, local_z: usize) -> Option<usize> {
        if local_x >= DEFAULT_CHUNK_SIZE || local_z >= DEFAULT_CHUNK_SIZE || y >= self.world_height
        {
            return None;
        }

        Some((y * DEFAULT_CHUNK_SIZE + local_z) * DEFAULT_CHUNK_SIZE + local_x)
    }
}

fn sanitize_weight(value: f64) -> f64 {
    sanitize_ratio(value, 12.0)
}

fn sanitize_feature_weight(value: f64) -> f64 {
    sanitize_ratio(value, 4.0)
}

fn sanitize_ratio(value: f64, max: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, max)
    } else {
        0.0
    }
}

fn normalized_name(name: &str) -> String {
    name.chars()
        .filter(|ch| !matches!(ch, '-' | '_' | ' ' | '\t'))
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biome_and_weather_names_round_trip() {
        for biome in VoxelBiome::ALL {
            assert_eq!(VoxelBiome::from_name(biome.name()), Some(biome));
        }
        for weather in VoxelWeather::ALL {
            assert_eq!(VoxelWeather::from_name(weather.name()), Some(weather));
        }
    }

    #[test]
    fn composition_presets_are_tunable() {
        let crystal = VoxelWorldComposition::preset("crystal").expect("crystal preset");
        let volcanic = VoxelWorldComposition::preset("volcanic").expect("volcanic preset");
        let crater = VoxelWorldComposition::preset("crater").expect("crater preset");

        assert!(crystal.biome_weights.crystal_fields > VoxelBiomeWeights::default().crystal_fields);
        assert!(crystal.resource_ratios.crystal > VoxelResourceRatios::default().crystal);
        assert!(volcanic.terrain_features.rifts > VoxelTerrainFeatureWeights::default().rifts);
        assert!(
            crater.terrain_features.large_craters
                > VoxelTerrainFeatureWeights::default().large_craters
        );
    }

    #[test]
    fn forced_weights_enable_one_target() {
        let biome_weights = VoxelBiomeWeights::only(VoxelBiome::Volcanic);
        let weather_weights = VoxelWeatherWeights::only(VoxelWeather::Ashfall);

        assert_eq!(biome_weights.get(VoxelBiome::Volcanic), 1.0);
        assert_eq!(biome_weights.total(), 1.0);
        assert_eq!(weather_weights.get(VoxelWeather::Ashfall), 1.0);
        assert_eq!(weather_weights.total(), 1.0);
    }

    #[test]
    fn surface_resource_blocks_have_catalog_keys() {
        let resources = [
            (BlockKind::CoalOre, "basalt_stone"),
            (BlockKind::IronOre, "space_iron"),
            (BlockKind::GoldOre, "osmium"),
            (BlockKind::CrystalOre, "silicate_crystal"),
            (BlockKind::TitaniumOre, "titanium"),
            (BlockKind::UraniumOre, "uranium"),
            (BlockKind::HeliumVent, "helium_3"),
            (BlockKind::BioPlasmaBloom, "bio_plasma"),
            (BlockKind::AncientRelic, "ancient_relic"),
        ];

        for (block, key) in resources {
            assert!(block.is_surface_resource());
            assert!(block.is_ore());
            assert_eq!(block.resource_key(), Some(key));
        }

        assert_eq!(BlockKind::Grass.resource_key(), None);
        assert!(!BlockKind::Grass.is_surface_resource());
    }

    #[test]
    fn terrain_feature_weights_sanitize_into_supported_range() {
        let settings = VoxelWorldSettings {
            composition: VoxelWorldComposition {
                terrain_features: VoxelTerrainFeatureWeights {
                    craters: f64::NAN,
                    large_craters: -1.0,
                    rifts: 99.0,
                    canyons: f64::INFINITY,
                    high_mountains: 2.25,
                    plateaus: 4.5,
                    erosion: 0.75,
                },
                ..VoxelWorldComposition::default()
            },
            ..VoxelWorldSettings::default()
        }
        .sanitized();
        let features = settings.composition.terrain_features;

        for value in [
            features.craters,
            features.large_craters,
            features.rifts,
            features.canyons,
            features.high_mountains,
            features.plateaus,
            features.erosion,
        ] {
            assert!(value.is_finite());
            assert!((0.0..=4.0).contains(&value));
        }
        assert_eq!(features.craters, 0.0);
        assert_eq!(features.large_craters, 0.0);
        assert_eq!(features.rifts, 4.0);
        assert_eq!(features.canyons, 0.0);
        assert_eq!(features.plateaus, 4.0);
    }
}
