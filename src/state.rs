use std::collections::{BTreeMap, BTreeSet, VecDeque};
use astra_voxel_world::prelude::*;
use bevy::prelude::*;

pub const BLOCK_SIZE: f32 = VOXEL_VIEWER_BLOCK_SIZE;
pub const HEIGHT_SCALE: f32 = VOXEL_VIEWER_HEIGHT_SCALE;
pub const TERRAIN_EDGE_WIDTH: f32 = BLOCK_SIZE * 0.085;
#[allow(dead_code)]
pub const TERRAIN_EDGE_LIFT: f32 = 0.050;
#[allow(dead_code)]
pub const SLOPE_EDGE_HEIGHT_DELTA: i32 = 2;
pub const SURFACE_TARGET_Y_OFFSET: f32 = 7.5;
pub const LOAD_RADIUS_DEFAULT: i64 = 18;
pub const LOAD_RADIUS_MAX: i64 = 32;
pub const LOAD_RADIUS_MARGIN_CHUNKS: f32 = 2.0;
pub const CHUNK_STREAM_BUDGET_PER_FRAME: usize = 6;
pub const CHUNK_UNLOAD_BUDGET_PER_FRAME: usize = 32;
pub const LOD_MEDIUM_CAMERA_HEIGHT: f32 = 360.0;
pub const LOD_LOW_CAMERA_HEIGHT: f32 = 560.0;
pub const CAMERA_MIN_HEIGHT: f32 = 115.0;
pub const CAMERA_MAX_HEIGHT: f32 = 720.0;
pub const CAMERA_DEFAULT_HEIGHT: f32 = 310.0;
pub const CAMERA_VERTICAL_FOV_RADIANS: f32 = 0.733_038_3;
pub const DEFAULT_VIEWPORT_ASPECT: f32 = 1.60;
pub const CAMERA_PITCH: f32 = 0.88;
pub const CAMERA_MOVE_SPEED: f32 = 78.0;
pub const CAMERA_FAST_MULTIPLIER: f32 = 3.0;
pub const CAMERA_ROTATE_SPEED: f32 = 1.55;
pub const VIEWER_PRESETS: [&str; 7] = [
    "balanced", "lush", "dry", "frozen", "volcanic", "crystal", "crater",
];
pub const LIVE_CONTROL_SEED_STEP: u64 = 0x9E37_79B9_7F4A_7C15;
pub const LIVE_CONTROL_RESOURCE_STEP: f64 = 0.5;
pub const LIVE_CONTROL_MAX_CRYSTAL_RATIO: f64 = 6.0;
pub const SEA_LEVEL_STEP: i32 = 2;
pub const HEIGHT_STEP: i32 = 2;
pub const DENSITY_STEP: f64 = 0.01;

#[derive(Resource, Default)]
pub struct MiddleClickResetTimer {
    pub last_click_time: f32,
}

#[derive(Resource)]
pub struct ArabicFont(pub Handle<Font>);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Playing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewerOptions {
    pub seed: u64,
    pub start_x: i64,
    pub start_z: i64,
    pub load_radius: i64,
    pub world_height: u16,
    pub sea_level: i32,
    pub base_height: i32,
    pub terrain_amplitude: i32,
    pub mountain_amplitude: i32,
    pub cave_density: f64,
    pub tree_density: f64,
    pub composition: VoxelWorldComposition,
    pub help: bool,
}

impl Default for ViewerOptions {
    fn default() -> Self {
        let settings = VoxelWorldSettings::default();

        Self {
            seed: settings.seed,
            start_x: 0,
            start_z: 0,
            load_radius: LOAD_RADIUS_DEFAULT,
            world_height: settings.world_height,
            sea_level: settings.sea_level,
            base_height: settings.base_height,
            terrain_amplitude: settings.terrain_amplitude,
            mountain_amplitude: settings.mountain_amplitude,
            cave_density: settings.cave_density,
            tree_density: settings.tree_density,
            composition: VoxelWorldComposition::default(),
            help: false,
        }
    }
}

impl ViewerOptions {
    pub fn generation_settings(&self) -> VoxelWorldSettings {
        VoxelWorldSettings {
            seed: self.seed,
            world_height: self.world_height,
            sea_level: self.sea_level,
            base_height: self.base_height,
            terrain_amplitude: self.terrain_amplitude,
            mountain_amplitude: self.mountain_amplitude,
            cave_density: self.cave_density,
            tree_density: self.tree_density,
            composition: self.composition,
        }
        .sanitized()
    }
}

#[derive(Debug, Resource)]
pub struct VoxelViewerWorld {
    pub settings: VoxelWorldSettings,
    pub load_radius: i64,
}

#[derive(Debug, Resource)]
pub struct VoxelViewerCamera {
    pub center: Vec2,
    pub yaw: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Resource)]
pub struct VoxelViewerLiveControls {
    pub preset_index: usize,
    pub forced_biome_index: Option<usize>,
    pub forced_weather_index: Option<usize>,
    pub numeric_input_index: usize,
    pub crystal_ratio: f64,
    pub last_change: String,
}

impl Default for VoxelViewerLiveControls {
    fn default() -> Self {
        Self::from_composition(VoxelWorldComposition::default())
    }
}

impl VoxelViewerLiveControls {
    pub fn from_composition(composition: VoxelWorldComposition) -> Self {
        Self {
            preset_index: crate::world::preset_index_for(composition).unwrap_or(0),
            forced_biome_index: crate::world::forced_biome_index_for(composition),
            forced_weather_index: crate::world::forced_weather_index_for(composition),
            numeric_input_index: 0,
            crystal_ratio: composition.resource_ratios.crystal,
            last_change: "ready".to_string(),
        }
    }
}

#[derive(Debug, Clone, Resource)]
pub struct VoxelGenerationDialogState {
    pub open: bool,
    pub buffer: String,
    pub status: String,
}

impl Default for VoxelGenerationDialogState {
    fn default() -> Self {
        Self {
            open: false,
            buffer: String::new(),
            status: "Click INPUTS to edit generation arguments.".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Resource)]
pub struct VoxelViewerWeatherState {
    pub biome: VoxelBiome,
    pub weather: VoxelWeather,
}

impl Default for VoxelViewerWeatherState {
    fn default() -> Self {
        Self {
            biome: VoxelBiome::Plains,
            weather: VoxelWeather::Clear,
        }
    }
}

#[derive(Debug, Resource, Default)]
pub struct LoadedVoxelChunks {
    pub chunks: BTreeMap<VoxelChunkCoord, Entity>,
    pub desired: BTreeSet<VoxelChunkCoord>,
    pub pending: VecDeque<VoxelChunkCoord>,
    pub retiring: VecDeque<VoxelChunkCoord>,
    pub signature: Option<ChunkStreamSignature>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkStreamSignature {
    pub settings: VoxelWorldSettings,
    pub center: VoxelChunkCoord,
    pub radius: i64,
}

#[derive(Debug, Resource, Default)]
pub struct VoxelViewerRenderAssets {
    pub terrain_material: Option<Handle<StandardMaterial>>,
}

#[derive(Component)]
pub struct VoxelViewerCameraTag;

#[derive(Component)]
pub struct VoxelViewerSunTag;

#[derive(Component)]
pub struct VoxelChunkEntity;

#[derive(Component)]
pub struct VoxelViewerHudText;

#[derive(Component)]
pub struct VoxelViewerWeatherOverlay;

#[derive(Component)]
pub struct VoxelGenerationDialogRoot;

#[derive(Component)]
pub struct VoxelGenerationDialogInputText;

#[derive(Component)]
pub struct VoxelGenerationDialogStatusText;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelGenerationDialogAction {
    Open,
    Apply,
    Cancel,
}

#[derive(Component)]
pub struct VoxelGenerationDialogButton {
    pub action: VoxelGenerationDialogAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveNumericInput {
    SeaLevel,
    BaseHeight,
    TerrainAmplitude,
    MountainAmplitude,
    TreeDensity,
    CaveDensity,
}

impl LiveNumericInput {
    pub const ALL: [Self; 6] = [
        Self::SeaLevel,
        Self::BaseHeight,
        Self::TerrainAmplitude,
        Self::MountainAmplitude,
        Self::TreeDensity,
        Self::CaveDensity,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::SeaLevel => "sea level",
            Self::BaseHeight => "base height",
            Self::TerrainAmplitude => "terrain amp",
            Self::MountainAmplitude => "mountain amp",
            Self::TreeDensity => "tree density",
            Self::CaveDensity => "cave density",
        }
    }

    pub fn value_text(self, settings: VoxelWorldSettings) -> String {
        match self {
            Self::SeaLevel => settings.sea_level.to_string(),
            Self::BaseHeight => settings.base_height.to_string(),
            Self::TerrainAmplitude => settings.terrain_amplitude.to_string(),
            Self::MountainAmplitude => settings.mountain_amplitude.to_string(),
            Self::TreeDensity => format!("{:.3}", settings.tree_density),
            Self::CaveDensity => format!("{:.2}", settings.cave_density),
        }
    }

    pub fn apply_delta(self, settings: &mut VoxelWorldSettings, direction: i32) {
        match self {
            Self::SeaLevel => {
                settings.sea_level = settings
                    .sea_level
                    .saturating_add(SEA_LEVEL_STEP * direction);
            }
            Self::BaseHeight => {
                settings.base_height = settings.base_height.saturating_add(HEIGHT_STEP * direction);
            }
            Self::TerrainAmplitude => {
                settings.terrain_amplitude = settings
                    .terrain_amplitude
                    .saturating_add(HEIGHT_STEP * direction);
            }
            Self::MountainAmplitude => {
                settings.mountain_amplitude = settings
                    .mountain_amplitude
                    .saturating_add(HEIGHT_STEP * direction);
            }
            Self::TreeDensity => {
                settings.tree_density += DENSITY_STEP * f64::from(direction);
            }
            Self::CaveDensity => {
                settings.cave_density += DENSITY_STEP * f64::from(direction);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveControlAction {
    NextPreset,
    NextBiome,
    NextWeather,
    IncreaseCrystalRatio,
    DecreaseCrystalRatio,
    NextNumericInput,
    IncreaseNumericInput,
    DecreaseNumericInput,
    NextSeed,
    ResetGenerationSettings,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewerWeatherScene {
    pub sky_color: Color,
    pub ambient_color: Color,
    pub ambient_brightness: f32,
    pub sun_color: Color,
    pub sun_illuminance: f32,
    pub overlay_color: Color,
}
