//! نظام الحفظ والتحميل — Critical Point Save / Load System
//!
//! يحفظ: session (موجة, صحة, نقاط, أسلحة, مستوى ...) في ملف JSON
//! ملف الحفظ: <AppData>/Roaming/critical_point/save.json

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::state::{
    AppState, GameSession, MissionPhase, PlanetRoute, PlayerLoadout, ResourceKind, WeaponKind,
};

// ─────────────────────────────────────────────────────────────────────────────
// Save file path
// ─────────────────────────────────────────────────────────────────────────────

fn save_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("critical_point")
}

fn save_path() -> PathBuf {
    save_dir().join("save.json")
}

// ─────────────────────────────────────────────────────────────────────────────
// Serializable snapshot types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SavedResourceKind {
    SpaceIron,
    Titanium,
    Helium3,
    BioPlasma,
}

impl From<ResourceKind> for SavedResourceKind {
    fn from(r: ResourceKind) -> Self {
        match r {
            ResourceKind::SpaceIron => Self::SpaceIron,
            ResourceKind::Titanium => Self::Titanium,
            ResourceKind::Helium3 => Self::Helium3,
            ResourceKind::BioPlasma => Self::BioPlasma,
        }
    }
}

impl From<SavedResourceKind> for ResourceKind {
    fn from(r: SavedResourceKind) -> Self {
        match r {
            SavedResourceKind::SpaceIron => Self::SpaceIron,
            SavedResourceKind::Titanium => Self::Titanium,
            SavedResourceKind::Helium3 => Self::Helium3,
            SavedResourceKind::BioPlasma => Self::BioPlasma,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SavedWeaponKind {
    PulseRifle,
    PlasmaMortar,
    IonLance,
    QuantumTesla,
    NukeMortar,
}

impl From<WeaponKind> for SavedWeaponKind {
    fn from(w: WeaponKind) -> Self {
        match w {
            WeaponKind::PulseRifle => Self::PulseRifle,
            WeaponKind::PlasmaMortar => Self::PlasmaMortar,
            WeaponKind::IonLance => Self::IonLance,
            WeaponKind::QuantumTesla => Self::QuantumTesla,
            WeaponKind::NukeMortar => Self::NukeMortar,
        }
    }
}

impl From<SavedWeaponKind> for WeaponKind {
    fn from(w: SavedWeaponKind) -> Self {
        match w {
            SavedWeaponKind::PulseRifle => Self::PulseRifle,
            SavedWeaponKind::PlasmaMortar => Self::PlasmaMortar,
            SavedWeaponKind::IonLance => Self::IonLance,
            SavedWeaponKind::QuantumTesla => Self::QuantumTesla,
            SavedWeaponKind::NukeMortar => Self::NukeMortar,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedLoadout {
    pub health: f32,
    pub shield: f32,
    pub points: u32,
    pub level: u32,
    pub next_level_threshold: u32,
    pub fire_rate_multiplier: f32,
    pub max_health_bonus: f32,
    pub max_shield_bonus: f32,
    pub kills: u32,
    pub resources: BTreeMap<SavedResourceKind, u16>,
    pub weapon_levels: BTreeMap<SavedWeaponKind, u8>,
}

impl From<&PlayerLoadout> for SavedLoadout {
    fn from(l: &PlayerLoadout) -> Self {
        Self {
            health: l.health,
            shield: l.shield,
            points: l.points,
            level: l.level,
            next_level_threshold: l.next_level_threshold,
            fire_rate_multiplier: l.fire_rate_multiplier,
            max_health_bonus: l.max_health_bonus,
            max_shield_bonus: l.max_shield_bonus,
            kills: l.kills,
            resources: l
                .resources
                .iter()
                .map(|(k, v)| ((*k).into(), *v))
                .collect(),
            weapon_levels: l
                .weapon_levels
                .iter()
                .map(|(k, v)| ((*k).into(), *v))
                .collect(),
        }
    }
}

impl SavedLoadout {
    fn apply_to(&self, loadout: &mut PlayerLoadout) {
        loadout.health = self.health;
        loadout.shield = self.shield;
        loadout.points = self.points;
        loadout.level = self.level;
        loadout.next_level_threshold = self.next_level_threshold;
        loadout.fire_rate_multiplier = self.fire_rate_multiplier;
        loadout.max_health_bonus = self.max_health_bonus;
        loadout.max_shield_bonus = self.max_shield_bonus;
        loadout.kills = self.kills;
        loadout.resources = self
            .resources
            .iter()
            .map(|(k, v)| ((*k).into(), *v))
            .collect();
        loadout.weapon_levels = self
            .weapon_levels
            .iter()
            .map(|(k, v)| ((*k).into(), *v))
            .collect();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavedRoute {
    Undecided,
    HomeDefense,
    InvadedPlanet,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavedPhase {
    AwaitingRoute,
    HomePreparation,
    HomeDefense,
    AlienLanding,
    RelayHunt,
    Extraction,
    GateAssault,
    Finished,
}

impl From<PlanetRoute> for SavedRoute {
    fn from(r: PlanetRoute) -> Self {
        match r {
            PlanetRoute::Undecided => Self::Undecided,
            PlanetRoute::HomeDefense => Self::HomeDefense,
            PlanetRoute::InvadedPlanet => Self::InvadedPlanet,
        }
    }
}
impl From<SavedRoute> for PlanetRoute {
    fn from(r: SavedRoute) -> Self {
        match r {
            SavedRoute::Undecided => Self::Undecided,
            SavedRoute::HomeDefense => Self::HomeDefense,
            SavedRoute::InvadedPlanet => Self::InvadedPlanet,
        }
    }
}
impl From<MissionPhase> for SavedPhase {
    fn from(p: MissionPhase) -> Self {
        match p {
            MissionPhase::AwaitingRoute => Self::AwaitingRoute,
            MissionPhase::HomePreparation => Self::HomePreparation,
            MissionPhase::HomeDefense => Self::HomeDefense,
            MissionPhase::AlienLanding => Self::AlienLanding,
            MissionPhase::RelayHunt => Self::RelayHunt,
            MissionPhase::Extraction => Self::Extraction,
            MissionPhase::GateAssault => Self::GateAssault,
            MissionPhase::Finished => Self::Finished,
        }
    }
}
impl From<SavedPhase> for MissionPhase {
    fn from(p: SavedPhase) -> Self {
        match p {
            SavedPhase::AwaitingRoute => Self::AwaitingRoute,
            SavedPhase::HomePreparation => Self::HomePreparation,
            SavedPhase::HomeDefense => Self::HomeDefense,
            SavedPhase::AlienLanding => Self::AlienLanding,
            SavedPhase::RelayHunt => Self::RelayHunt,
            SavedPhase::Extraction => Self::Extraction,
            SavedPhase::GateAssault => Self::GateAssault,
            SavedPhase::Finished => Self::Finished,
        }
    }
}

/// الحفظ الكامل للجلسة
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveData {
    pub version: u32,
    pub wave: u32,
    pub base_health: f32,
    pub relays_destroyed: u8,
    pub elapsed: f32,
    pub route: SavedRoute,
    pub phase: SavedPhase,
    pub safe_position: [f32; 3],
    pub loadout: SavedLoadout,
}

impl SaveData {
    const CURRENT_VERSION: u32 = 1;

    pub fn from_session(session: &GameSession) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            wave: session.wave,
            base_health: session.base_health,
            relays_destroyed: session.relays_destroyed,
            elapsed: session.elapsed,
            route: session.route.into(),
            phase: session.phase.into(),
            safe_position: [
                session.safe_position.x,
                session.safe_position.y,
                session.safe_position.z,
            ],
            loadout: SavedLoadout::from(&session.loadout),
        }
    }

    pub fn apply_to_session(&self, session: &mut GameSession) {
        session.wave = self.wave;
        session.base_health = self.base_health;
        session.relays_destroyed = self.relays_destroyed;
        session.elapsed = self.elapsed;
        session.route = self.route.into();
        session.phase = self.phase.into();
        session.safe_position = Vec3::from_array(self.safe_position);
        self.loadout.apply_to(&mut session.loadout);
        session.pending_perk_choices = None;
        session.objective_hint = "تم تحميل اللعبة المحفوظة".to_string();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// IO helpers
// ─────────────────────────────────────────────────────────────────────────────

pub fn write_save(data: &SaveData) -> std::io::Result<()> {
    let dir = save_dir();
    fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(save_path(), json)?;
    info!("💾 تم حفظ اللعبة في {:?}", save_path());
    Ok(())
}

pub fn read_save() -> Option<SaveData> {
    let bytes = fs::read(save_path()).ok()?;
    let data: SaveData = serde_json::from_slice(&bytes)
        .map_err(|e| warn!("⚠️ فشل قراءة ملف الحفظ: {e}"))
        .ok()?;
    if data.version != SaveData::CURRENT_VERSION {
        warn!("⚠️ إصدار ملف الحفظ ({}) غير مدعوم", data.version);
        return None;
    }
    Some(data)
}

pub fn delete_save() {
    if let Err(e) = fs::remove_file(save_path()) {
        warn!("لم يمكن حذف الملف: {e}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Bevy Resource — tracks save/load signals
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct SaveLoadState {
    /// عدد الثواني منذ آخر حفظ تلقائي
    pub auto_save_cooldown: f32,
    /// عدد الموجات عند آخر حفظ
    pub last_saved_wave: u32,
    /// رسالة تظهر للمستخدم
    pub status_message: Option<(String, f32)>,
    /// طلب حفظ
    pub save_requested: bool,
    /// طلب تحميل
    pub load_requested: bool,
    /// هل يوجد ملف حفظ؟
    pub save_exists: bool,
}

impl SaveLoadState {
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), 3.0));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems
// ─────────────────────────────────────────────────────────────────────────────

/// يشتغل عند بدء التطبيق — يتحقق هل يوجد ملف حفظ
pub fn check_save_on_startup(mut save_state: ResMut<SaveLoadState>) {
    save_state.save_exists = save_path().exists();
    if save_state.save_exists {
        info!("📂 تم العثور على ملف حفظ في {:?}", save_path());
    }
}

/// يقرأ مدخلات لوحة المفاتيح: Ctrl+S للحفظ، Ctrl+L للتحميل
pub fn handle_save_load_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut save_state: ResMut<SaveLoadState>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl && keys.just_pressed(KeyCode::KeyS) {
        save_state.save_requested = true;
    }
    if ctrl && keys.just_pressed(KeyCode::KeyL) {
        save_state.load_requested = true;
    }
}

/// ينفذ الحفظ والتحميل المطلوب ويحفظ تلقائياً بعد كل موجة
pub fn process_save_load(
    mut save_state: ResMut<SaveLoadState>,
    mut session: ResMut<GameSession>,
    mut lifecycle: ResMut<crate::gameplay::RunLifecycle>,
    mut player_query: Query<(&mut Transform, &mut Visibility), With<crate::player::PlayerTag>>,
    time: Res<Time>,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    use crate::state::AppState;

    let dt = time.delta_secs();
    // تحديث مؤقت رسالة الحالة
    if let Some((_, ref mut ttl)) = save_state.status_message {
        *ttl -= dt;
        if *ttl <= 0.0 {
            save_state.status_message = None;
        }
    }

    // حفظ يدوي (Ctrl+S)
    if save_state.save_requested {
        save_state.save_requested = false;
        let data = SaveData::from_session(&session);
        match write_save(&data) {
            Ok(_) => {
                save_state.save_exists = true;
                save_state.last_saved_wave = session.wave;
                save_state.set_status("✅ تم الحفظ بنجاح");
            }
            Err(e) => {
                save_state.set_status(format!("❌ فشل الحفظ: {e}"));
            }
        }
    }

    // تحميل يدوي (Ctrl+L) أو عبر الزر — يعمل من MainMenu أو Paused أو Playing
    if save_state.load_requested {
        save_state.load_requested = false;
        if let Some(data) = read_save() {
            data.apply_to_session(&mut session);
            lifecycle.active = false;
            lifecycle.player_reset_pending = false;
            if let Ok((mut transform, _)) = player_query.single_mut() {
                transform.translation = session.safe_position;
            }
            save_state.set_status("📂 تم تحميل اللعبة المحفوظة");
            if *app_state.get() != AppState::Playing {
                next_app_state.set(AppState::Playing);
            }
        } else {
            save_state.set_status("❌ لا يوجد ملف حفظ صالح");
        }
    }

    // حفظ تلقائي بعد كل موجة جديدة (cooldown 5 ثواني لتجنب الحفظ المتكرر)
    save_state.auto_save_cooldown = (save_state.auto_save_cooldown - dt).max(0.0);
    let in_game = *app_state.get() == AppState::Playing;
    let new_wave = session.wave > save_state.last_saved_wave;
    let wave_active = session.wave > 0;
    if in_game && new_wave && wave_active && save_state.auto_save_cooldown <= 0.0 {
        let data = SaveData::from_session(&session);
        if write_save(&data).is_ok() {
            save_state.save_exists = true;
            save_state.last_saved_wave = session.wave;
            save_state.auto_save_cooldown = 5.0;
            save_state.set_status(format!("💾 حُفظ تلقائياً — الموجة {}", session.wave));
        }
    }
}
