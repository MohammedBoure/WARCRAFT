use std::collections::{BTreeMap, BTreeSet, VecDeque};

use astra_voxel_world::prelude::*;
use bevy::prelude::*;

pub const BLOCK_SIZE: f32 = VOXEL_VIEWER_BLOCK_SIZE;
pub const HEIGHT_SCALE: f32 = VOXEL_VIEWER_HEIGHT_SCALE;
pub const LOAD_RADIUS_DEFAULT: i64 = 7;
pub const LOAD_RADIUS_MAX: i64 = 12;
pub const CHUNK_STREAM_BUDGET_PER_FRAME: usize = 4;
pub const CHUNK_UNLOAD_BUDGET_PER_FRAME: usize = 16;
pub const CAMERA_MIN_HEIGHT: f32 = 36.0;
pub const CAMERA_MAX_HEIGHT: f32 = 112.0;
pub const CAMERA_DEFAULT_HEIGHT: f32 = 64.0;
pub const CAMERA_PITCH: f32 = 0.88;
pub const GAME_SEED: u64 = 0x4352_4954_4943_414C;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
    Decision,
    Ending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GamePhase {
    #[default]
    Tutorial,
    Scavenge,
    Evacuating,
    Stabilizing,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CriticalChoice {
    #[default]
    None,
    Evacuate,
    Stabilize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunOutcome {
    #[default]
    None,
    PeopleSaved,
    WorldSaved,
    Collapse,
}

#[derive(Resource, Debug, Clone)]
pub struct GameSession {
    pub phase: GamePhase,
    pub criticality: f32,
    pub crystals: u8,
    pub supports: u8,
    pub choice: CriticalChoice,
    pub outcome: RunOutcome,
    pub phase_time_remaining: Option<f32>,
    pub elapsed: f32,
    pub safe_position: Vec3,
    pub objective_hint: String,
    pub collapse_count: u32,
}

impl Default for GameSession {
    fn default() -> Self {
        Self {
            phase: GamePhase::Tutorial,
            criticality: 10.0,
            crystals: 0,
            supports: 6,
            choice: CriticalChoice::None,
            outcome: RunOutcome::None,
            phase_time_remaining: None,
            elapsed: 0.0,
            safe_position: Vec3::ZERO,
            objective_hint: "تحرّك نحو أول إشارة بلورية".to_string(),
            collapse_count: 0,
        }
    }
}

impl GameSession {
    pub fn reset(&mut self, safe_position: Vec3) {
        *self = Self {
            safe_position,
            ..default()
        };
    }

    pub fn add_criticality(&mut self, amount: f32) {
        self.criticality = (self.criticality + amount).clamp(0.0, 100.0);
    }

    pub fn collect_crystal(&mut self) -> bool {
        self.crystals = self.crystals.saturating_add(1).min(3);
        self.add_criticality(10.0);
        self.phase = GamePhase::Scavenge;
        self.objective_hint = match self.crystals {
            0 => "تحرّك نحو أول إشارة بلورية".to_string(),
            1 => "اعثر على الشظية الثانية".to_string(),
            2 => "بقيت شظية واحدة قبل النقطة الحرجة".to_string(),
            _ => "اتخذ القرار الذي سيغيّر مصير العالم".to_string(),
        };
        self.crystals == 3
    }

    pub fn choose(&mut self, choice: CriticalChoice) {
        self.choice = choice;
        self.criticality = self.criticality.max(match choice {
            CriticalChoice::Evacuate => 70.0,
            CriticalChoice::Stabilize => 78.0,
            CriticalChoice::None => self.criticality,
        });
        match choice {
            CriticalChoice::Evacuate => {
                self.phase = GamePhase::Evacuating;
                self.phase_time_remaining = Some(90.0);
                self.objective_hint = "عد إلى منارة الإخلاء واضغط E".to_string();
            }
            CriticalChoice::Stabilize => {
                self.phase = GamePhase::Stabilizing;
                self.phase_time_remaining = Some(120.0);
                self.objective_hint = "اتجه إلى قلب الصدع وثبّت النواة بالضغط على E".to_string();
            }
            CriticalChoice::None => {}
        }
    }

    pub fn finish(&mut self, outcome: RunOutcome) {
        self.outcome = outcome;
        self.phase = GamePhase::Finished;
        self.phase_time_remaining = None;
    }

    pub fn risk_band(&self) -> RiskBand {
        RiskBand::from_value(self.criticality)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskBand {
    Calm,
    Warning,
    Critical,
    Terminal,
}

impl RiskBand {
    pub fn from_value(value: f32) -> Self {
        if value < 40.0 {
            Self::Calm
        } else if value < 70.0 {
            Self::Warning
        } else if value < 90.0 {
            Self::Critical
        } else {
            Self::Terminal
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct BalanceConfig {
    pub passive_risk_per_second: f32,
    pub final_risk_per_second: f32,
    pub dig_risk: f32,
    pub fall_risk: f32,
    pub evacuation_seconds: f32,
    pub stabilization_seconds: f32,
}

impl Default for BalanceConfig {
    fn default() -> Self {
        Self {
            passive_risk_per_second: 0.08,
            final_risk_per_second: 0.15,
            dig_risk: 1.25,
            fall_risk: 10.0,
            evacuation_seconds: 90.0,
            stabilization_seconds: 120.0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct GamePreferences {
    pub master_volume: f32,
    pub camera_sensitivity: f32,
    pub reduced_motion: bool,
}

impl Default for GamePreferences {
    fn default() -> Self {
        Self {
            master_volume: 0.75,
            camera_sensitivity: 1.0,
            reduced_motion: false,
        }
    }
}

#[derive(Resource, Clone)]
pub struct ArabicFont(pub Handle<Font>);

impl FromWorld for ArabicFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self(asset_server.load("fonts/arabic.ttf"))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewerOptions {
    pub seed: u64,
    pub load_radius: i64,
    pub composition: VoxelWorldComposition,
    pub help: bool,
    pub dev_world: bool,
}

impl Default for ViewerOptions {
    fn default() -> Self {
        let mut composition = VoxelWorldComposition::preset("balanced").unwrap_or_default();
        composition.resource_ratios.set_named("crystal", 0.0);
        Self {
            seed: GAME_SEED,
            load_radius: LOAD_RADIUS_DEFAULT,
            composition,
            help: false,
            dev_world: false,
        }
    }
}

impl ViewerOptions {
    pub fn generation_settings(&self) -> VoxelWorldSettings {
        VoxelWorldSettings {
            seed: self.seed,
            composition: self.composition,
            cave_density: 0.11,
            tree_density: 0.025,
            ..default()
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
    pub shake: f32,
}

#[derive(Debug, Resource, Default)]
pub struct LoadedVoxelChunks {
    pub chunks: BTreeMap<VoxelChunkCoord, Entity>,
    pub voxel_data: BTreeMap<VoxelChunkCoord, VoxelChunk>,
    pub desired: BTreeSet<VoxelChunkCoord>,
    pub pending: VecDeque<VoxelChunkCoord>,
    pub retiring: VecDeque<VoxelChunkCoord>,
    pub dirty: BTreeSet<VoxelChunkCoord>,
    pub signature: Option<ChunkStreamSignature>,
}

impl LoadedVoxelChunks {
    pub fn block_at(&self, position: VoxelBlockPosition) -> Option<BlockKind> {
        if position.y < 0 {
            return None;
        }
        let size = DEFAULT_CHUNK_SIZE as i64;
        let coord = VoxelChunkCoord::new(floor_div(position.x, size), floor_div(position.z, size));
        let chunk = self.voxel_data.get(&coord)?;
        let local_x = position.x - coord.world_x(0);
        let local_z = position.z - coord.world_z(0);
        chunk.get(local_x as usize, position.y as usize, local_z as usize)
    }

    pub fn ground_below(&self, world_x: i64, world_z: i64, max_y: i32) -> Option<i32> {
        let max_y = max_y.max(1);
        (1..=max_y).rev().find(|y| {
            self.block_at(VoxelBlockPosition::new(world_x, *y, world_z))
                .is_some_and(BlockKind::is_solid)
        })
    }
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
pub struct VoxelViewerWeatherOverlay;

#[derive(Message, Debug, Clone, Copy)]
pub struct CrystalCollected(pub u8);
#[derive(Message, Debug, Clone, Copy)]
pub struct CriticalityChanged(pub f32);
#[derive(Message, Debug, Clone, Copy)]
pub struct CollapseTriggered(pub Vec3);
#[derive(Message, Debug, Clone, Copy)]
pub struct ChoiceCommitted(pub CriticalChoice);
#[derive(Message, Debug, Clone, Copy)]
pub struct RunFinished(pub RunOutcome);
#[derive(Message, Debug, Clone, Copy)]
pub enum VoxelActionSound {
    Mine,
    Build,
}

#[derive(Component)]
pub struct ScreenRoot;
#[derive(Component)]
pub struct HudRoot;
#[derive(Component)]
pub struct RiskFill;
#[derive(Component)]
pub struct RiskText;
#[derive(Component)]
pub struct ObjectiveText;
#[derive(Component)]
pub struct InventoryText;
#[derive(Component)]
pub struct TimerText;

pub fn floor_div(a: i64, b: i64) -> i64 {
    let d = a / b;
    let r = a % b;
    if r != 0 && ((a < 0) != (b < 0)) {
        d - 1
    } else {
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn criticality_is_clamped() {
        let mut session = GameSession::default();
        session.add_criticality(500.0);
        assert_eq!(session.criticality, 100.0);
        session.add_criticality(-500.0);
        assert_eq!(session.criticality, 0.0);
    }

    #[test]
    fn third_crystal_unlocks_decision() {
        let mut session = GameSession::default();
        assert!(!session.collect_crystal());
        assert!(!session.collect_crystal());
        assert!(session.collect_crystal());
        assert_eq!(session.crystals, 3);
    }

    #[test]
    fn choices_have_distinct_pressure() {
        let mut evacuate = GameSession::default();
        evacuate.choose(CriticalChoice::Evacuate);
        assert_eq!(evacuate.phase, GamePhase::Evacuating);
        assert_eq!(evacuate.phase_time_remaining, Some(90.0));

        let mut stabilize = GameSession::default();
        stabilize.choose(CriticalChoice::Stabilize);
        assert_eq!(stabilize.phase, GamePhase::Stabilizing);
        assert_eq!(stabilize.phase_time_remaining, Some(120.0));
        assert!(stabilize.criticality > evacuate.criticality);
    }

    #[test]
    fn risk_bands_match_visual_thresholds() {
        assert_eq!(RiskBand::from_value(39.9), RiskBand::Calm);
        assert_eq!(RiskBand::from_value(40.0), RiskBand::Warning);
        assert_eq!(RiskBand::from_value(70.0), RiskBand::Critical);
        assert_eq!(RiskBand::from_value(90.0), RiskBand::Terminal);
    }
}
