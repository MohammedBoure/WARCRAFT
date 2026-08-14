use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

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
pub const HOME_SEED: u64 = GAME_SEED ^ 0x484F_4D45;
pub const INVASION_SEED: u64 = GAME_SEED ^ 0x414C_4945_4E;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    RouteChoice,
    Playing,
    Paused,
    FinalDecision,
    Ending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlanetRoute {
    #[default]
    Undecided,
    HomeDefense,
    InvadedPlanet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MissionPhase {
    #[default]
    AwaitingRoute,
    HomePreparation,
    HomeDefense,
    AlienLanding,
    RelayHunt,
    Extraction,
    GateAssault,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FinalChoice {
    #[default]
    None,
    Extract,
    AssaultGate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunOutcome {
    #[default]
    None,
    HomeDefended,
    Extracted,
    GateDestroyed,
    MissionFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceKind {
    SpaceIron,
    Titanium,
    Helium3,
    BioPlasma,
}

impl ResourceKind {
    pub const ALL: [Self; 4] = [
        Self::SpaceIron,
        Self::Titanium,
        Self::Helium3,
        Self::BioPlasma,
    ];

    pub const fn arabic_name(self) -> &'static str {
        match self {
            Self::SpaceIron => "حديد فضائي",
            Self::Titanium => "تيتانيوم",
            Self::Helium3 => "هيليوم-3",
            Self::BioPlasma => "بلازما حيوية",
        }
    }

    pub const fn from_block(block: BlockKind) -> Option<Self> {
        match block {
            BlockKind::IronOre => Some(Self::SpaceIron),
            BlockKind::TitaniumOre => Some(Self::Titanium),
            BlockKind::HeliumVent => Some(Self::Helium3),
            BlockKind::BioPlasmaBloom => Some(Self::BioPlasma),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WeaponKind {
    PulseRifle,
    PlasmaMortar,
    IonLance,
    QuantumTesla,
    NukeMortar,
}

impl WeaponKind {
    pub const ALL: [Self; 5] = [
        Self::PulseRifle,
        Self::PlasmaMortar,
        Self::IonLance,
        Self::QuantumTesla,
        Self::NukeMortar,
    ];

    pub const fn arabic_name(self) -> &'static str {
        match self {
            Self::PulseRifle => "بندقية النبض",
            Self::PlasmaMortar => "مدفع البلازما",
            Self::IonLance => "رمح الأيون",
            Self::QuantumTesla => "تسلا الكوانتم",
            Self::NukeMortar => "مدفع الدمار الشامل",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DamageKind {
    Pulse,
    Plasma,
    Ion,
    Tesla,
    Nuke,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSlot {
    Weapon(WeaponKind),
    MiningLaser,
    Builder,
}

impl Default for ToolSlot {
    fn default() -> Self {
        Self::Weapon(WeaponKind::PulseRifle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Crawler,
    Spitter,
    Drone,
    Brute,
    CarrierBoss,
}

impl EnemyKind {
    pub const fn is_aerial(self) -> bool {
        matches!(self, Self::Drone | Self::CarrierBoss)
    }

    pub const fn arabic_name(self) -> &'static str {
        match self {
            Self::Crawler => "زاحف فضائي",
            Self::Spitter => "باصق الحمض",
            Self::Drone => "مسيّرة هجومية",
            Self::Brute => "عملاق درع",
            Self::CarrierBoss => "حاملة الغزو",
        }
    }

    pub const fn point_value(self) -> u32 {
        match self {
            Self::Crawler => 100,
            Self::Spitter => 150,
            Self::Drone => 200,
            Self::Brute => 350,
            Self::CarrierBoss => 1000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerkOption {
    UnlockWeapon(WeaponKind),
    UpgradeWeapon(WeaponKind),
    AutoDroneSwarm,
    OrbitalAutoStrike,
    VoxelCraterBooster,
    HealthBoost,
    ShieldBoost,
    AttackSpeedBoost,
}

impl PerkOption {
    pub fn title(self) -> &'static str {
        match self {
            PerkOption::UnlockWeapon(WeaponKind::PlasmaMortar) => "فتح مدفع البلازما الخارق",
            PerkOption::UnlockWeapon(WeaponKind::IonLance) => "فتح رمح الأيون الاختراقي",
            PerkOption::UnlockWeapon(WeaponKind::QuantumTesla) => "فتح تسلا الكوانتم العاصف",
            PerkOption::UnlockWeapon(WeaponKind::NukeMortar) => "فتح مدفع الدمار الشامل النووي",
            PerkOption::UnlockWeapon(WeaponKind::PulseRifle) => "ترقية بندقية النبض",
            PerkOption::UpgradeWeapon(WeaponKind::PulseRifle) => "ترقية بندقية النبض (Level Up)",
            PerkOption::UpgradeWeapon(WeaponKind::PlasmaMortar) => "ترقية مدفع البلازما (Level Up)",
            PerkOption::UpgradeWeapon(WeaponKind::IonLance) => "ترقية رمح الأيون (Level Up)",
            PerkOption::UpgradeWeapon(WeaponKind::QuantumTesla) => "ترقية تسلا الكوانتم (Level Up)",
            PerkOption::UpgradeWeapon(WeaponKind::NukeMortar) => "ترقية المدفع النووي المداري",
            PerkOption::AutoDroneSwarm => "سرب الدرونات التلقائي",
            PerkOption::OrbitalAutoStrike => "الضربة المدارية التلقائية",
            PerkOption::VoxelCraterBooster => "متفجرات الانقسام الجزيئي",
            PerkOption::HealthBoost => "تعزيز درع البدن الخارق (+40 HP)",
            PerkOption::ShieldBoost => "تعزيز درع الطاقة (+40 Shield)",
            PerkOption::AttackSpeedBoost => "معجل سرعة الإطلاق (+25% Fire Speed)",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            PerkOption::UnlockWeapon(WeaponKind::PlasmaMortar) => "فتح سلاح إطلاق قذائف البلازما الانفجارية وحفر الأرض",
            PerkOption::UnlockWeapon(WeaponKind::IonLance) => "فتح شعاع الأيون الاختراقي لتدمير أدرع الأعداء الأقوياء",
            PerkOption::UnlockWeapon(WeaponKind::QuantumTesla) => "إطلاق صعقات كهربائية متسلسلة تتفرع وتصهر عدة أعداء فوراً",
            PerkOption::UnlockWeapon(WeaponKind::NukeMortar) => "سلاح مداري تدميري يطلق قذائف أنتي-مايتر تصنع حفرة عملاقة في الأرض",
            PerkOption::UnlockWeapon(WeaponKind::PulseRifle) => "زيادة مدى وقوة رصاص بندقية النبض الكهرومغناطيسية",
            PerkOption::UpgradeWeapon(WeaponKind::PulseRifle) => "زيادة القوة التدميرية ونطاق رمي بندقية النبض",
            PerkOption::UpgradeWeapon(WeaponKind::PlasmaMortar) => "توسيع نطاق الانفجار وتدمير التضاريس لمدفع البلازما",
            PerkOption::UpgradeWeapon(WeaponKind::IonLance) => "زيادة سرعة وسعة الطاقة لشعاع الأيون الخارق",
            PerkOption::UpgradeWeapon(WeaponKind::QuantumTesla) => "زيادة مدى صعق الكهرباء وعدد الأهداف المتفرعة",
            PerkOption::UpgradeWeapon(WeaponKind::NukeMortar) => "مضاعفة قطر حفرة الأرض والضرر الانفجاري للمدفع النووي",
            PerkOption::AutoDroneSwarm => "تفعيل درونات دفاعية تطوف وتطلق صواريخ طاقة موجهة تلقائياً تجاه الوحوش",
            PerkOption::OrbitalAutoStrike => "قصف مداري زلزالي آلي ينهال كل 7 ثوانٍ فوق تجمعات الأعداء ليخلق حفرة تدميرية",
            PerkOption::VoxelCraterBooster => "مضاعفة حجم حفر الأرض الانفجارية ونطاق الضرر بنسبة +100%",
            PerkOption::HealthBoost => "إصلاح وزيادة حد الصحة الأقصى للبطل بمقدار +40 نقطة",
            PerkOption::ShieldBoost => "إعادة شحن وزيادة سعة درع الطاقة الخارق بمقدار +40 نقطة",
            PerkOption::AttackSpeedBoost => "زيادة معدل إطلاق النار لجميع الأسلحة المتاحة بنسبة 25%",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerLoadout {
    pub health: f32,
    pub shield: f32,
    pub resources: BTreeMap<ResourceKind, u16>,
    pub blocks: HashMap<BlockKind, u16>,
    pub weapon_levels: BTreeMap<WeaponKind, u8>,
    pub auto_drones_level: u8,
    pub orbital_strike_level: u8,
    pub crater_booster_level: u8,
    pub selected_tool: ToolSlot,
    pub selected_block: BlockKind,
    pub points: u32,
    pub level: u32,
    pub next_level_threshold: u32,
    pub fire_rate_multiplier: f32,
    pub max_health_bonus: f32,
    pub max_shield_bonus: f32,
    pub heat: f32,
    pub kills: u32,
    pub blocks_placed: u32,
}

impl Default for PlayerLoadout {
    fn default() -> Self {
        let mut weapon_levels = BTreeMap::new();
        weapon_levels.insert(WeaponKind::PulseRifle, 1);
        let mut blocks = HashMap::new();
        blocks.insert(BlockKind::Stone, 8);
        Self {
            health: 100.0,
            shield: 50.0,
            resources: BTreeMap::new(),
            blocks,
            weapon_levels,
            auto_drones_level: 0,
            orbital_strike_level: 0,
            crater_booster_level: 0,
            selected_tool: ToolSlot::default(),
            selected_block: BlockKind::Stone,
            points: 0,
            level: 1,
            next_level_threshold: 300,
            fire_rate_multiplier: 1.0,
            max_health_bonus: 0.0,
            max_shield_bonus: 0.0,
            heat: 0.0,
            kills: 0,
            blocks_placed: 0,
        }
    }
}

impl PlayerLoadout {
    pub fn add_points(&mut self, amount: u32) {
        self.points = self.points.saturating_add(amount);
    }

    pub fn spend_points(&mut self, amount: u32) -> bool {
        if self.points < amount {
            return false;
        }
        self.points -= amount;
        true
    }
    pub fn resource_count(&self, kind: ResourceKind) -> u16 {
        self.resources.get(&kind).copied().unwrap_or(0)
    }

    pub fn add_resource(&mut self, kind: ResourceKind, amount: u16) {
        let count = self.resources.entry(kind).or_default();
        *count = count.saturating_add(amount);
    }

    pub fn add_block(&mut self, kind: BlockKind, amount: u16) {
        let count = self.blocks.entry(kind).or_default();
        *count = count.saturating_add(amount);
    }

    pub fn block_count(&self, kind: BlockKind) -> u16 {
        self.blocks.get(&kind).copied().unwrap_or(0)
    }

    pub fn consume_block(&mut self, kind: BlockKind) -> bool {
        let Some(count) = self.blocks.get_mut(&kind) else {
            return false;
        };
        if *count == 0 {
            return false;
        }
        *count -= 1;
        true
    }

    pub fn weapon_level(&self, weapon: WeaponKind) -> u8 {
        self.weapon_levels.get(&weapon).copied().unwrap_or(0)
    }

    pub fn can_afford(&self, recipe: &[(ResourceKind, u16)]) -> bool {
        recipe
            .iter()
            .all(|(kind, amount)| self.resource_count(*kind) >= *amount)
    }

    pub fn spend(&mut self, recipe: &[(ResourceKind, u16)]) -> bool {
        if !self.can_afford(recipe) {
            return false;
        }
        for (kind, amount) in recipe {
            if let Some(count) = self.resources.get_mut(kind) {
                *count -= *amount;
            }
        }
        true
    }

    pub fn lose_raw_resources(&mut self, fraction: f32) {
        for count in self.resources.values_mut() {
            let loss = (*count as f32 * fraction).ceil() as u16;
            *count = count.saturating_sub(loss);
        }
    }
    pub fn apply_perk(&mut self, perk: PerkOption) {
        match perk {
            PerkOption::UnlockWeapon(kind) | PerkOption::UpgradeWeapon(kind) => {
                let current = self.weapon_level(kind);
                if current < 3 {
                    self.weapon_levels.insert(kind, current + 1);
                }
            }
            PerkOption::AutoDroneSwarm => {
                self.auto_drones_level = (self.auto_drones_level + 1).min(3);
            }
            PerkOption::OrbitalAutoStrike => {
                self.orbital_strike_level = (self.orbital_strike_level + 1).min(3);
            }
            PerkOption::VoxelCraterBooster => {
                self.crater_booster_level = (self.crater_booster_level + 1).min(3);
            }
            PerkOption::HealthBoost => {
                self.max_health_bonus += 40.0;
                self.health = (self.health + 40.0).min(100.0 + self.max_health_bonus);
            }
            PerkOption::ShieldBoost => {
                self.max_shield_bonus += 40.0;
                self.shield = (self.shield + 40.0).min(50.0 + self.max_shield_bonus);
            }
            PerkOption::AttackSpeedBoost => {
                self.fire_rate_multiplier *= 1.25;
            }
        }
    }

    pub fn generate_perks(&self) -> [PerkOption; 3] {
        let mut pool = Vec::new();

        let tesla_lvl = self.weapon_level(WeaponKind::QuantumTesla);
        if tesla_lvl == 0 {
            pool.push(PerkOption::UnlockWeapon(WeaponKind::QuantumTesla));
        } else if tesla_lvl < 3 {
            pool.push(PerkOption::UpgradeWeapon(WeaponKind::QuantumTesla));
        }

        let nuke_lvl = self.weapon_level(WeaponKind::NukeMortar);
        if nuke_lvl == 0 {
            pool.push(PerkOption::UnlockWeapon(WeaponKind::NukeMortar));
        } else if nuke_lvl < 3 {
            pool.push(PerkOption::UpgradeWeapon(WeaponKind::NukeMortar));
        }

        let mortar_lvl = self.weapon_level(WeaponKind::PlasmaMortar);
        if mortar_lvl == 0 {
            pool.push(PerkOption::UnlockWeapon(WeaponKind::PlasmaMortar));
        } else if mortar_lvl < 3 {
            pool.push(PerkOption::UpgradeWeapon(WeaponKind::PlasmaMortar));
        }

        let ion_lvl = self.weapon_level(WeaponKind::IonLance);
        if ion_lvl == 0 {
            pool.push(PerkOption::UnlockWeapon(WeaponKind::IonLance));
        } else if ion_lvl < 3 {
            pool.push(PerkOption::UpgradeWeapon(WeaponKind::IonLance));
        }

        let pulse_lvl = self.weapon_level(WeaponKind::PulseRifle);
        if pulse_lvl < 3 {
            pool.push(PerkOption::UpgradeWeapon(WeaponKind::PulseRifle));
        }

        if self.auto_drones_level < 3 {
            pool.push(PerkOption::AutoDroneSwarm);
        }
        if self.orbital_strike_level < 3 {
            pool.push(PerkOption::OrbitalAutoStrike);
        }
        if self.crater_booster_level < 3 {
            pool.push(PerkOption::VoxelCraterBooster);
        }

        pool.push(PerkOption::HealthBoost);
        pool.push(PerkOption::ShieldBoost);
        pool.push(PerkOption::AttackSpeedBoost);

        let len = pool.len();
        [pool[0], pool[1 % len], pool[2 % len]]
    }
}

#[derive(Resource, Debug, Clone)]
pub struct GameSession {
    pub route: PlanetRoute,
    pub phase: MissionPhase,
    pub final_choice: FinalChoice,
    pub outcome: RunOutcome,
    pub phase_time_remaining: Option<f32>,
    pub elapsed: f32,
    pub safe_position: Vec3,
    pub objective_hint: String,
    pub wave: u32,
    pub relays_destroyed: u8,
    pub base_health: f32,
    pub loadout: PlayerLoadout,
    pub active_enemies: usize,
    pub pending_perk_choices: Option<[PerkOption; 3]>,
}

impl Default for GameSession {
    fn default() -> Self {
        Self {
            route: PlanetRoute::Undecided,
            phase: MissionPhase::AwaitingRoute,
            final_choice: FinalChoice::None,
            outcome: RunOutcome::None,
            phase_time_remaining: None,
            elapsed: 0.0,
            safe_position: Vec3::ZERO,
            objective_hint: "اختر مسار المهمة".to_string(),
            wave: 0,
            relays_destroyed: 0,
            base_health: 500.0,
            loadout: PlayerLoadout::default(),
            active_enemies: 0,
            pending_perk_choices: None,
        }
    }
}

impl GameSession {
    pub fn check_level_up(&mut self) -> bool {
        if self.pending_perk_choices.is_some() {
            return true;
        }
        if self.loadout.points >= self.loadout.next_level_threshold {
            self.loadout.level += 1;
            self.loadout.next_level_threshold += 300 + self.loadout.level * 200;
            self.pending_perk_choices = Some(self.loadout.generate_perks());
            true
        } else {
            false
        }
    }
    pub fn begin_route(&mut self, route: PlanetRoute, safe_position: Vec3) {
        *self = Self {
            route,
            safe_position,
            ..default()
        };
        self.phase = match route {
            PlanetRoute::HomeDefense => MissionPhase::HomePreparation,
            PlanetRoute::InvadedPlanet => MissionPhase::AlienLanding,
            PlanetRoute::Undecided => MissionPhase::AwaitingRoute,
        };
        self.phase_time_remaining = (route == PlanetRoute::HomeDefense).then_some(600.0);
        self.objective_hint = match route {
            PlanetRoute::HomeDefense => "اجمع البلوكات وحصّن مولد المستعمرة".into(),
            PlanetRoute::InvadedPlanet => "اجمع الموارد ودمّر أبراج الغزو الثلاثة".into(),
            PlanetRoute::Undecided => "اختر مسار المهمة".into(),
        };
    }

    pub fn finish(&mut self, outcome: RunOutcome) {
        self.outcome = outcome;
        self.phase = MissionPhase::Finished;
        self.phase_time_remaining = None;
    }
}

#[derive(Resource, Debug, Clone)]
pub struct BalanceConfig {
    pub home_duration: f32,
    pub extraction_seconds: f32,
    pub gate_assault_seconds: f32,
    pub respawn_resource_loss: f32,
}

impl Default for BalanceConfig {
    fn default() -> Self {
        Self {
            home_duration: 600.0,
            extraction_seconds: 90.0,
            gate_assault_seconds: 180.0,
            respawn_resource_loss: 0.25,
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
        Self(world.resource::<AssetServer>().load("fonts/arabic.ttf"))
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
        (1..=max_y.max(1)).rev().find(|y| {
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

#[derive(Debug, Clone, Copy)]
pub struct VoxelHit {
    pub block: VoxelBlockPosition,
    pub placement: Option<VoxelBlockPosition>,
    pub kind: BlockKind,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct AimSolution {
    pub voxel: Option<VoxelHit>,
    pub enemy: Option<Entity>,
    pub world_point: Option<Vec3>,
    pub aim_point: Option<Vec3>,
    pub pointer_over_ui: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum DamageTarget {
    Player,
    Enemy(Entity),
    Base(Entity),
}

#[derive(Message, Debug, Clone, Copy)]
pub struct DamageEvent {
    pub target: DamageTarget,
    pub amount: f32,
    pub kind: DamageKind,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ResourceCollected(pub ResourceKind, pub u16);
#[derive(Message, Debug, Clone, Copy)]
pub struct BlockCollected(pub BlockKind, pub u16);
#[derive(Message, Debug, Clone, Copy)]
pub struct EnemyKilled(pub EnemyKind);
#[derive(Message, Debug, Clone, Copy)]
pub struct WeaponCrafted(pub WeaponKind, pub u8);
#[derive(Message, Debug, Clone, Copy)]
pub struct RouteCommitted(pub PlanetRoute);
#[derive(Message, Debug, Clone, Copy)]
pub struct FinalChoiceCommitted(pub FinalChoice);
#[derive(Message, Debug, Clone, Copy)]
pub struct RunFinished(pub RunOutcome);
#[derive(Message, Debug, Clone, Copy)]
pub struct WaveStarted(pub u32);
#[derive(Message, Debug, Clone, Copy)]
pub struct PlayerRespawned;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSound {
    Mine,
    Build,
    PulseShot,
    PlasmaShot,
    IonShot,
    EnemyHit,
    EnemyDeath,
    PlayerHit,
    Resource,
    Craft,
    Warning,
    Success,
    Failure,
}

#[derive(Component)]
pub struct ScreenRoot;
#[derive(Component)]
pub struct HudRoot;
#[derive(Component)]
pub struct HealthFill;
#[derive(Component)]
pub struct ShieldFill;
#[derive(Component)]
pub struct ObjectiveText;
#[derive(Component)]
pub struct ResourceText;
#[derive(Component)]
pub struct HotbarText;
#[derive(Component)]
pub struct TimerText;
#[derive(Component)]
pub struct BaseHealthText;
#[derive(Component)]
pub struct PlayerHealthText;
#[derive(Component)]
pub struct PlayerShieldText;
#[derive(Component)]
pub struct TargetEnemyPanelRoot;
#[derive(Component)]
pub struct TargetEnemyText;

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
    fn inventory_spend_is_atomic() {
        let mut loadout = PlayerLoadout::default();
        loadout.add_resource(ResourceKind::Titanium, 4);
        let recipe = [(ResourceKind::Titanium, 5), (ResourceKind::BioPlasma, 3)];
        assert!(!loadout.spend(&recipe));
        assert_eq!(loadout.resource_count(ResourceKind::Titanium), 4);
    }

    #[test]
    fn building_only_consumes_collected_blocks() {
        let mut loadout = PlayerLoadout::default();
        assert!(loadout.consume_block(BlockKind::Stone));
        assert!(!loadout.consume_block(BlockKind::Basalt));
        loadout.add_block(BlockKind::Basalt, 1);
        assert!(loadout.consume_block(BlockKind::Basalt));
    }

    #[test]
    fn points_accumulation_and_spending_works() {
        let mut loadout = PlayerLoadout::default();
        assert_eq!(loadout.points, 0);
        loadout.add_points(EnemyKind::Crawler.point_value());
        assert_eq!(loadout.points, 100);
        assert!(!loadout.spend_points(250));
        loadout.add_points(200);
        assert!(loadout.spend_points(250));
        assert_eq!(loadout.points, 50);
    }
}
