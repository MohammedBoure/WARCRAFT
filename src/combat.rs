use astra_voxel_world::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::gameplay::{MissionTarget, MissionTargetKind, RelayDestroyed, RunEntity, finish_run};
use crate::interaction::{
    VoxelWorldEdits, block_world_center, voxel_raycast_loaded, world_to_block,
};
use crate::player::{PlayerModelRoot, PlayerTag};
use crate::state::*;
use crate::world::invalidate_edit;

#[derive(Component)]
pub struct CombatTarget {
    pub radius: f32,
    pub aerial: bool,
    pub targetable: bool,
}

#[derive(Component)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub health: f32,
    pub max_health: f32,
    pub shield: f32,
    pub max_shield: f32,
    speed: f32,
    attack: Timer,
}

#[derive(Component)]
pub struct Projectile {
    from_player: bool,
    velocity: Vec3,
    gravity: f32,
    damage: f32,
    kind: DamageKind,
    life: Timer,
    radius: f32,
    target: Option<Entity>,
    area: f32,
}

#[derive(Resource)]
pub struct EnemyDirector {
    spawn_cooldown: f32,
    intermission: f32,
    counter: u32,
    spawned: u32,
    quota: u32,
    wave_active: bool,
    boss_wave: bool,
}
impl Default for EnemyDirector {
    fn default() -> Self {
        Self {
            spawn_cooldown: 0.0,
            intermission: 3.0,
            counter: 0,
            spawned: 0,
            quota: 0,
            wave_active: false,
            boss_wave: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct CombatRuntime {
    cooldown: f32,
    last_hit: f32,
}
#[derive(Resource, Default)]
pub struct CraftState {
    weapon: Option<WeaponKind>,
    held: f32,
    completed: bool,
}

#[derive(Resource)]
pub struct CombatAssets {
    shield: Handle<Mesh>,
    shield_material: Handle<StandardMaterial>,
    shot: Handle<Mesh>,
    flash: Handle<Mesh>,
    beam: Handle<Mesh>,
    ring: Handle<Mesh>,
    spark: Handle<Mesh>,
    glow_quad: Handle<Mesh>,
    materials: Vec<Handle<StandardMaterial>>,
    fx_materials: Vec<Handle<StandardMaterial>>,
    fx_core_materials: Vec<Handle<StandardMaterial>>,
    glow_materials: Vec<Handle<StandardMaterial>>,
    enemy_scenes: Vec<Handle<Scene>>,
    weapon_scenes: Vec<Handle<Scene>>,
    pub white_outline_material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct TargetOutlineModel;

#[derive(Component)]
pub struct WeaponVisual;

#[derive(Component)]
pub struct WeaponRig {
    weapon: WeaponKind,
    rest: Transform,
    recoil: f32,
}

#[derive(Component)]
pub struct WeaponMuzzle {
    weapon: WeaponKind,
}

#[derive(Component)]
pub(crate) struct ProjectileTrail {
    last_position: Vec3,
    style: usize,
}

#[derive(Component)]
pub(crate) struct ProjectileVisual {
    base_scale: Vec3,
    light_intensity: f32,
    light_range: f32,
    phase: f32,
}

#[derive(Component)]
pub(crate) struct ProjectileHalo {
    base_scale: Vec3,
    spin: f32,
    pulse: f32,
    phase: f32,
}

#[derive(Component)]
pub(crate) struct CombatFx {
    age: f32,
    duration: f32,
    start_scale: Vec3,
    end_scale: Vec3,
    peak_light: f32,
}

#[derive(Component)]
pub(crate) struct FxMotion {
    velocity: Vec3,
    drag: f32,
    gravity: f32,
}

#[derive(Component)]
pub(crate) struct FxSpin {
    axis: Vec3,
    radians_per_second: f32,
}

#[derive(Component)]
pub struct EnemyAura {
    base_scale: Vec3,
    phase: f32,
}

#[derive(Component)]
pub struct EnemyAimReticle {
    base_scale: Vec3,
}

#[derive(Component)]
pub struct BossShieldVisual {
    base_scale: Vec3,
    max_shield: f32,
}

pub fn setup_combat_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let colors = [
        (
            Color::srgb(0.45, 0.95, 0.15),
            LinearRgba::rgb(18.0, 85.0, 12.0),
        ),
        (
            Color::srgb(0.85, 0.20, 0.95),
            LinearRgba::rgb(85.0, 15.0, 110.0),
        ),
        (
            Color::srgb(0.20, 0.75, 1.0),
            LinearRgba::rgb(25.0, 95.0, 220.0),
        ),
        (
            Color::srgb(1.0, 0.30, 0.12),
            LinearRgba::rgb(180.0, 45.0, 15.0),
        ),
        (
            Color::srgb(0.65, 0.15, 1.0),
            LinearRgba::rgb(120.0, 22.0, 190.0),
        ),
        (
            Color::srgb(0.15, 0.95, 1.0),
            LinearRgba::rgb(35.0, 160.0, 260.0),
        ),
        (
            Color::srgb(1.0, 0.25, 0.85),
            LinearRgba::rgb(160.0, 28.0, 130.0),
        ),
        (
            Color::srgb(0.35, 0.65, 1.0),
            LinearRgba::rgb(45.0, 110.0, 250.0),
        ),
        (
            Color::srgb(1.0, 0.22, 0.10),
            LinearRgba::rgb(220.0, 38.0, 18.0),
        ),
    ];
    let created_materials: Vec<_> = colors
        .into_iter()
        .map(|(base_color, emissive)| {
            materials.add(StandardMaterial {
                base_color,
                emissive,
                unlit: true,
                ..default()
            })
        })
        .collect();
    let fx_materials = [
        (
            Color::srgba(0.20, 0.95, 1.0, 0.40),
            LinearRgba::rgb(25.0, 110.0, 160.0),
        ),
        (
            Color::srgba(1.0, 0.25, 0.85, 0.40),
            LinearRgba::rgb(160.0, 35.0, 110.0),
        ),
        (
            Color::srgba(0.30, 0.65, 1.0, 0.40),
            LinearRgba::rgb(35.0, 95.0, 180.0),
        ),
        (
            Color::srgba(1.0, 0.22, 0.10, 0.40),
            LinearRgba::rgb(140.0, 24.0, 12.0),
        ),
    ]
    .map(|(base_color, emissive)| {
        materials.add(StandardMaterial {
            base_color,
            emissive,
            unlit: true,
            alpha_mode: AlphaMode::Add,
            cull_mode: None,
            ..default()
        })
    })
    .to_vec();
    let fx_core_materials = [
        (
            Color::srgba(0.90, 1.0, 1.0, 0.90),
            LinearRgba::rgb(120.0, 320.0, 450.0),
        ),
        (
            Color::srgba(1.0, 0.90, 0.95, 0.90),
            LinearRgba::rgb(450.0, 180.0, 320.0),
        ),
        (
            Color::srgba(0.88, 0.96, 1.0, 0.90),
            LinearRgba::rgb(110.0, 260.0, 480.0),
        ),
        (
            Color::srgba(1.0, 0.85, 0.70, 0.90),
            LinearRgba::rgb(380.0, 95.0, 45.0),
        ),
    ]
    .map(|(base_color, emissive)| {
        materials.add(StandardMaterial {
            base_color,
            emissive,
            unlit: true,
            alpha_mode: AlphaMode::Add,
            cull_mode: None,
            ..default()
        })
    })
    .to_vec();

    let shield_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.15, 0.65, 1.0, 0.08),
        emissive: LinearRgba::rgb(0.2, 1.5, 3.5),
        metallic: 0.1,
        perceptual_roughness: 0.05,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });

    let mut glow_image_data = Vec::with_capacity(128 * 128 * 4);
    for y in 0..128 {
        for x in 0..128 {
            let dx = (x as f32 + 0.5 - 64.0) / 64.0;
            let dy = (y as f32 + 0.5 - 64.0) / 64.0;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist >= 1.0 {
                glow_image_data.extend_from_slice(&[255, 255, 255, 0]);
            } else {
                let factor = (1.0 - dist).powf(1.6);
                let alpha = (factor * 255.0).clamp(0.0, 255.0) as u8;
                glow_image_data.extend_from_slice(&[255, 255, 255, alpha]);
            }
        }
    }
    let glow_texture = asset_server.add(Image::new(
        Extent3d {
            width: 128,
            height: 128,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        glow_image_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    ));

    let glow_colors = [
        (
            Color::srgba(0.20, 0.95, 1.0, 0.95),
            LinearRgba::rgb(45.0, 180.0, 260.0),
        ),
        (
            Color::srgba(1.0, 0.22, 0.85, 0.95),
            LinearRgba::rgb(260.0, 48.0, 160.0),
        ),
        (
            Color::srgba(0.30, 0.65, 1.0, 0.95),
            LinearRgba::rgb(55.0, 140.0, 280.0),
        ),
        (
            Color::srgba(1.0, 0.20, 0.08, 0.95),
            LinearRgba::rgb(220.0, 36.0, 18.0),
        ),
    ];
    let glow_materials = glow_colors
        .into_iter()
        .map(|(base_color, emissive)| {
            materials.add(StandardMaterial {
                base_color,
                base_color_texture: Some(glow_texture.clone()),
                emissive,
                unlit: true,
                alpha_mode: AlphaMode::Add,
                cull_mode: None,
                ..default()
            })
        })
        .collect();

    commands.insert_resource(CombatAssets {
        shield: meshes.add(Sphere::new(3.6).mesh().ico(3).expect("sphere")),
        shield_material,
        shot: meshes.add(Sphere::new(0.45).mesh().ico(1).expect("shot")),
        flash: meshes.add(Sphere::new(0.55).mesh().ico(2).expect("flash")),
        beam: meshes.add(Cuboid::new(0.14, 0.14, 1.0)),
        ring: meshes.add(Annulus::new(0.92, 1.00)),
        spark: meshes.add(Cuboid::new(0.08, 0.08, 0.62)),
        glow_quad: meshes.add(Rectangle::new(1.0, 1.0)),
        materials: created_materials,
        fx_materials,
        fx_core_materials,
        glow_materials,
        enemy_scenes: [
            "models/kenney-space/alien.glb",
            "models/kenney-space/astronautB.glb",
            "models/kenney-space/craft_speederA.glb",
            "models/kenney-space/astronautA.glb",
            "models/kenney-space/craft_miner.glb",
        ]
        .map(|path| asset_server.load(GltfAssetLabel::Scene(0).from_asset(path)))
        .to_vec(),
        weapon_scenes: [
            "models/kenney-blasters/blaster-a.glb",
            "models/kenney-blasters/blaster-c.glb",
            "models/kenney-blasters/blaster-h.glb",
        ]
        .map(|path| asset_server.load(GltfAssetLabel::Scene(0).from_asset(path)))
        .to_vec(),
        white_outline_material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: LinearRgba::rgb(1.8, 1.8, 1.8),
            unlit: true,
            cull_mode: None,
            ..default()
        }),
    });
}
pub fn handle_weapon_crafting(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut craft: ResMut<CraftState>,
    mut session: ResMut<GameSession>,
    mut crafted: MessageWriter<WeaponCrafted>,
    mut sounds: MessageWriter<GameSound>,
) {
    let ToolSlot::Weapon(weapon) = session.loadout.selected_tool else {
        *craft = default();
        return;
    };
    if !keyboard.pressed(KeyCode::KeyR) {
        craft.weapon = Some(weapon);
        craft.held = 0.0;
        craft.completed = false;
        return;
    }
    if craft.weapon != Some(weapon) {
        *craft = CraftState {
            weapon: Some(weapon),
            ..default()
        };
    }
    if craft.completed {
        return;
    }
    craft.held += time.delta_secs();
    if craft.held < 0.55 {
        return;
    }
    let level = session.loadout.weapon_level(weapon);
    if let Some(cost) = weapon_point_cost(weapon, level)
        && session.loadout.spend_points(cost)
    {
        session.loadout.weapon_levels.insert(weapon, level + 1);
        crafted.write(WeaponCrafted(weapon, level + 1));
        sounds.write(GameSound::Craft);
    }
    craft.completed = true;
}

pub fn handle_weapon_fire(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    aim: Res<AimSolution>,
    assets: Res<CombatAssets>,
    mut camera: ResMut<VoxelViewerCamera>,
    targets: Query<(Entity, &GlobalTransform, &CombatTarget)>,
    player: Query<&Transform, With<PlayerTag>>,
    muzzles: Query<(&GlobalTransform, &WeaponMuzzle)>,
    mut rigs: Query<&mut WeaponRig>,
    mut runtime: ResMut<CombatRuntime>,
    mut session: ResMut<GameSession>,
    mut sounds: MessageWriter<GameSound>,
    mut commands: Commands,
) {
    runtime.cooldown = (runtime.cooldown - time.delta_secs()).max(0.0);
    session.loadout.heat = (session.loadout.heat - time.delta_secs() * 0.30).max(0.0);
    let ToolSlot::Weapon(weapon) = session.loadout.selected_tool else {
        return;
    };
    let level = session.loadout.weapon_level(weapon);
    if level == 0
        || aim.pointer_over_ui
        || !mouse.pressed(MouseButton::Left)
        || runtime.cooldown > 0.0
        || session.loadout.heat >= 0.96
    {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let fallback_direction = *player.forward();
    let (origin, mount_direction) = muzzles
        .iter()
        .find(|(_, muzzle)| muzzle.weapon == weapon)
        .map(|(muzzle, _)| {
            let transform = muzzle.compute_transform();
            (
                transform.translation,
                (transform.rotation * Vec3::Z).normalize_or_zero(),
            )
        })
        .unwrap_or((player.translation + Vec3::Y * 0.65, fallback_direction));
    let target = aim.enemy.and_then(|entity| {
        targets.get(entity).ok().and_then(|(_, transform, target)| {
            (target.targetable && weapon_accepts_target(weapon, target.aerial)).then_some((
                entity,
                transform.translation(),
                target.aerial,
            ))
        })
    });

    let aim_target_point = target
        .map(|(_, point, _)| point)
        .or(aim.aim_point)
        .or(aim.world_point)
        .unwrap_or(origin + mount_direction * 64.0);

    match weapon {
        WeaponKind::PulseRifle => {
            let end = aim_target_point;
            let direction = (end - origin).normalize_or_zero();
            let right = Vec3::new(-direction.z, 0.0, direction.x).normalize_or_zero();
            let speed = 64.0;
            let damage_amount = 16.0 + level as f32 * 5.0;

            for &side_sign in &[-1.0f32, 1.0f32] {
                let offset = right * (side_sign * 0.42);
                let bolt_origin = origin + offset;
                let bolt_velocity = (end - bolt_origin).normalize_or_zero() * speed;

                spawn_shot(
                    &mut commands,
                    &assets,
                    bolt_origin,
                    bolt_velocity,
                    0,
                    Projectile {
                        from_player: true,
                        velocity: bolt_velocity,
                        gravity: 0.0,
                        damage: damage_amount * 0.5,
                        kind: DamageKind::Pulse,
                        life: Timer::from_seconds(1.8, TimerMode::Once),
                        radius: 1.1,
                        target: target.map(|(e, _, _)| e),
                        area: 0.0,
                    },
                );
                spawn_muzzle_fx(&mut commands, &assets, bolt_origin, direction, 0, 0.85);
            }
            apply_weapon_recoil(&mut rigs, weapon, 0.38);
            camera.shake = camera.shake.max(0.08);
            runtime.cooldown = 0.12;
            session.loadout.heat += 0.08;
            sounds.write(GameSound::PulseShot);
        }
        WeaponKind::PlasmaMortar => {
            let point = aim_target_point;
            let speed = 68.0;
            let velocity = (point - origin).normalize_or_zero() * speed;
            spawn_shot(
                &mut commands,
                &assets,
                origin,
                velocity,
                1,
                Projectile {
                    from_player: true,
                    velocity,
                    gravity: 0.0,
                    damage: 32.0 + level as f32 * 9.0,
                    kind: DamageKind::Plasma,
                    life: Timer::from_seconds(2.5, TimerMode::Once),
                    radius: 1.2,
                    target: target.map(|(e, _, _)| e),
                    area: 7.5,
                },
            );
            spawn_muzzle_fx(&mut commands, &assets, origin, velocity.normalize_or_zero(), 1, 1.4);
            apply_weapon_recoil(&mut rigs, weapon, 0.85);
            camera.shake = camera.shake.max(0.20);
            runtime.cooldown = 0.65;
            session.loadout.heat += 0.24;
            sounds.write(GameSound::PlasmaShot);
        }
        WeaponKind::IonLance => {
            let point = aim_target_point;
            let locked_entity = target.map(|(entity, _, _)| entity);
            let velocity = (point - origin).normalize_or_zero() * 42.0;
            spawn_shot(
                &mut commands,
                &assets,
                origin,
                velocity,
                7,
                Projectile {
                    from_player: true,
                    velocity,
                    gravity: 0.0,
                    damage: 24.0 + level as f32 * 8.0,
                    kind: DamageKind::Ion,
                    life: Timer::from_seconds(3.0, TimerMode::Once),
                    radius: 1.4,
                    target: locked_entity,
                    area: 0.0,
                },
            );
            apply_weapon_recoil(&mut rigs, weapon, 0.68);
            camera.shake = camera.shake.max(0.14);
            runtime.cooldown = 0.48;
            session.loadout.heat += 0.20;
            sounds.write(GameSound::IonShot);
        }
        WeaponKind::QuantumTesla => {
            let point = aim_target_point;
            let locked_entity = target.map(|(entity, _, _)| entity);
            let speed = 80.0;
            let velocity = (point - origin).normalize_or_zero() * speed;
            spawn_shot(
                &mut commands,
                &assets,
                origin,
                velocity,
                0,
                Projectile {
                    from_player: true,
                    velocity,
                    gravity: 0.0,
                    damage: 20.0 + level as f32 * 7.0,
                    kind: DamageKind::Tesla,
                    life: Timer::from_seconds(1.5, TimerMode::Once),
                    radius: 1.5,
                    target: locked_entity,
                    area: 4.0,
                },
            );
            spawn_muzzle_fx(&mut commands, &assets, origin, velocity.normalize_or_zero(), 0, 1.2);
            apply_weapon_recoil(&mut rigs, weapon, 0.50);
            camera.shake = camera.shake.max(0.12);
            runtime.cooldown = 0.30;
            session.loadout.heat += 0.15;
            sounds.write(GameSound::PulseShot);
        }
        WeaponKind::NukeMortar => {
            let point = aim_target_point;
            let speed = 50.0;
            let velocity = (point - origin).normalize_or_zero() * speed;
            spawn_shot(
                &mut commands,
                &assets,
                origin,
                velocity,
                1,
                Projectile {
                    from_player: true,
                    velocity,
                    gravity: 0.0,
                    damage: 90.0 + level as f32 * 35.0,
                    kind: DamageKind::Nuke,
                    life: Timer::from_seconds(3.5, TimerMode::Once),
                    radius: 2.0,
                    target: target.map(|(e, _, _)| e),
                    area: 15.0,
                },
            );
            spawn_muzzle_fx(&mut commands, &assets, origin, velocity.normalize_or_zero(), 1, 2.2);
            apply_weapon_recoil(&mut rigs, weapon, 1.20);
            camera.shake = camera.shake.max(0.45);
            runtime.cooldown = 1.80;
            session.loadout.heat += 0.45;
            sounds.write(GameSound::PlasmaShot);
        }
    }
}
fn projectile_accepts_target(kind: DamageKind, aerial: bool) -> bool {
    match kind {
        DamageKind::Pulse | DamageKind::Enemy => true,
        DamageKind::Plasma => true, // hits both aerial and ground after shield down
        DamageKind::Ion => aerial,  // Ion only locks aerial targets
        DamageKind::Tesla => true,
        DamageKind::Nuke => true,
    }
}

fn weapon_accepts_target(weapon: WeaponKind, aerial: bool) -> bool {
    match weapon {
        WeaponKind::PulseRifle => true,
        WeaponKind::PlasmaMortar => !aerial,
        WeaponKind::IonLance => aerial,
        WeaponKind::QuantumTesla => true,
        WeaponKind::NukeMortar => !aerial,
    }
}

pub(crate) fn weapon_point_cost(weapon: WeaponKind, level: u8) -> Option<u32> {
    match (weapon, level) {
        (WeaponKind::PulseRifle, 1) => Some(250),
        (WeaponKind::PulseRifle, 2) => Some(550),
        (WeaponKind::PlasmaMortar, 0) => Some(400),
        (WeaponKind::PlasmaMortar, 1) => Some(700),
        (WeaponKind::PlasmaMortar, 2) => Some(1200),
        (WeaponKind::IonLance, 0) => Some(500),
        (WeaponKind::IonLance, 1) => Some(850),
        (WeaponKind::IonLance, 2) => Some(1500),
        (WeaponKind::QuantumTesla, 0) => Some(600),
        (WeaponKind::QuantumTesla, 1) => Some(1000),
        (WeaponKind::QuantumTesla, 2) => Some(1800),
        (WeaponKind::NukeMortar, 0) => Some(800),
        (WeaponKind::NukeMortar, 1) => Some(1400),
        (WeaponKind::NukeMortar, 2) => Some(2500),
        _ => None,
    }
}

pub fn drive_enemy_spawns(
    time: Res<Time>,
    world: Res<VoxelViewerWorld>,
    loaded: Res<LoadedVoxelChunks>,
    assets: Res<CombatAssets>,
    mut director: ResMut<EnemyDirector>,
    mut session: ResMut<GameSession>,
    player: Query<&Transform, With<PlayerTag>>,
    enemies: Query<&Enemy>,
    mut waves: MessageWriter<WaveStarted>,
    mut sounds: MessageWriter<GameSound>,
    mut commands: Commands,
) {
    let count = enemies.iter().count();
    session.active_enemies = count;
    if session.route == PlanetRoute::Undecided || session.phase == MissionPhase::Finished {
        return;
    }
    let dt = time.delta_secs().min(0.10);
    if !director.wave_active {
        director.intermission = (director.intermission - dt).max(0.0);
        session.phase_time_remaining = Some(director.intermission);
        session.objective_hint = if session.wave == 0 {
            "استعد: أول موجة تقترب من القاعدة".into()
        } else {
            format!(
                "تم تطهير الموجة {} — طوّر أسلحتك بالنقاط قبل الهجوم التالي",
                session.wave
            )
        };
        if director.intermission <= 0.0 {
            begin_wave(&mut director, &mut session, &mut waves, &mut sounds);
        }
        return;
    }
    session.phase_time_remaining = None;
    let Ok(player_transform) = player.single() else {
        return;
    };
    if director.spawned >= director.quota && count == 0 {
        let reward = wave_clear_reward(session.wave);
        session.loadout.add_points(reward);
        session.objective_hint = format!(
            "الموجة {} انتهت — ربحت {} نقطة. الاستعداد للموجة التالية",
            session.wave, reward
        );
        director.wave_active = false;
        director.intermission = if director.boss_wave { 8.0 } else { 5.5 };
        sounds.write(GameSound::Craft);
        return;
    }
    director.spawn_cooldown = (director.spawn_cooldown - dt).max(0.0);
    if director.spawned >= director.quota
        || count >= wave_enemy_cap(session.wave)
        || director.spawn_cooldown > 0.0
    {
        return;
    }
    let is_boss = director.boss_wave && director.spawned + 1 == director.quota;
    let kind = if is_boss {
        EnemyKind::CarrierBoss
    } else {
        choose_wave_enemy(session.wave, director.counter)
    };
    let angle = director.counter as f32 * 2.399_963 + session.wave as f32 * 0.37;
    let radius = 28.0 + (director.counter % 5) as f32 * 4.0;
    let x = player_transform.translation.x + angle.cos() * radius;
    let z = player_transform.translation.z + angle.sin() * radius;
    let y = if kind.is_aerial() {
        player_transform.translation.y + if is_boss { 26.0 } else { 15.0 }
    } else {
        ground_height(&loaded, &world, x, z) + 0.9
    };
    let difficulty = wave_difficulty(session.wave, session.route)
        * if is_boss { 1.12 } else { 1.0 };
    spawn_enemy(&mut commands, &assets, kind, Vec3::new(x, y, z), difficulty);
    director.spawned += 1;
    director.counter = director.counter.wrapping_add(1);
    director.spawn_cooldown = wave_spawn_interval(session.wave);
}

fn begin_wave(
    director: &mut EnemyDirector,
    session: &mut GameSession,
    waves: &mut MessageWriter<WaveStarted>,
    sounds: &mut MessageWriter<GameSound>,
) {
    session.wave = session.wave.saturating_add(1);
    director.spawned = 0;
    director.quota = wave_spawn_total(session.wave);
    director.wave_active = true;
    director.boss_wave = wave_is_boss(session.wave);
    director.spawn_cooldown = 0.15;
    session.objective_hint = if director.boss_wave {
        format!(
            "موجة الزعيم {} — اخترق الدرع الأيوني ثم دمّر الحاملة",
            session.wave
        )
    } else {
        format!(
            "الموجة {} بدأت — أوقف {} من الغزاة",
            session.wave, director.quota
        )
    };
    waves.write(WaveStarted(session.wave));
    sounds.write(GameSound::Warning);
}

fn wave_is_boss(wave: u32) -> bool {
    wave > 0 && wave % 4 == 0
}

fn wave_spawn_total(wave: u32) -> u32 {
    4 + wave * 2 + if wave_is_boss(wave) { 1 } else { 0 }
}

fn wave_enemy_cap(wave: u32) -> usize {
    (5 + wave as usize * 2).clamp(7, 28)
}

fn wave_clear_reward(wave: u32) -> u32 {
    90 + wave * 75
}

fn wave_spawn_interval(wave: u32) -> f32 {
    (0.78 - wave as f32 * 0.028).max(0.16)
}

fn wave_difficulty(wave: u32, route: PlanetRoute) -> f32 {
    let base = 1.0 + (wave.saturating_sub(1)) as f32 * 0.12;
    let route_bonus = if route == PlanetRoute::InvadedPlanet { 0.10 } else { 0.0 };
    // Every 8 waves apply an extra multiplier for exponential ramp-up
    let tier_multiplier = 1.0 + (wave / 8) as f32 * 0.35;
    (base + route_bonus) * tier_multiplier
}

fn choose_wave_enemy(wave: u32, n: u32) -> EnemyKind {
    if wave <= 1 {
        if n % 5 == 4 { EnemyKind::Spitter } else { EnemyKind::Crawler }
    } else if wave <= 3 {
        if n % 4 == 3 {
            EnemyKind::Drone
        } else if n % 3 == 2 {
            EnemyKind::Spitter
        } else {
            EnemyKind::Crawler
        }
    } else if wave <= 6 {
        if n % 5 == 4 {
            EnemyKind::Brute
        } else if n % 3 == 2 {
            EnemyKind::Drone
        } else if n % 2 == 0 {
            EnemyKind::Spitter
        } else {
            EnemyKind::Crawler
        }
    } else {
        // Higher waves: more Brutes and Drones
        let roll = n % 6;
        match roll {
            0 => EnemyKind::Brute,
            1 => EnemyKind::Drone,
            2 => EnemyKind::Spitter,
            3 => EnemyKind::Brute,
            4 => EnemyKind::Drone,
            _ => EnemyKind::Crawler,
        }
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    assets: &CombatAssets,
    kind: EnemyKind,
    position: Vec3,
    difficulty: f32,
) {
    let (_material, health, shield, speed, radius, attack) = match kind {
        EnemyKind::Crawler => (0, 42.0, 0.0, 12.5, 1.2, 0.72),
        EnemyKind::Spitter => (1, 62.0, 0.0, 7.2, 1.3, 1.55),
        EnemyKind::Drone => (2, 54.0, 0.0, 10.5, 1.4, 1.35),
        EnemyKind::Brute => (3, 190.0, 0.0, 4.6, 1.8, 1.10),
        EnemyKind::CarrierBoss => (4, 760.0, 320.0, 6.5, 4.5, 0.85),
    };
    let scene_index = match kind {
        EnemyKind::Crawler => 0,
        EnemyKind::Spitter => 1,
        EnemyKind::Drone => 2,
        EnemyKind::Brute => 3,
        EnemyKind::CarrierBoss => 4,
    };
    let scene_scale = match kind {
        EnemyKind::Crawler => 2.2,
        EnemyKind::Spitter => 2.5,
        EnemyKind::Drone => 2.8,
        EnemyKind::Brute => 3.6,
        EnemyKind::CarrierBoss => 6.0,
    };
    let model_y_offset = 0.0;
    let scaled_health = health * difficulty;
    let scaled_shield = shield * difficulty;
    spawn_impact_fx(
        commands,
        assets,
        position,
        if kind == EnemyKind::CarrierBoss { 2 } else { 3 },
        if kind == EnemyKind::CarrierBoss {
            3.4
        } else {
            1.15
        },
    );
    commands
        .spawn((
            Name::new(format!("Alien {kind:?}")),
            Transform::from_translation(position),
            Visibility::default(),
            Enemy {
                kind,
                health: scaled_health,
                max_health: scaled_health,
                shield: scaled_shield,
                max_shield: scaled_shield,
                speed,
                attack: Timer::from_seconds(attack, TimerMode::Repeating),
            },
            CombatTarget {
                radius,
                aerial: kind.is_aerial(),
                targetable: true,
            },
            RunEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Open source alien model"),
                SceneRoot(assets.enemy_scenes[scene_index].clone()),
                Transform::from_xyz(0.0, model_y_offset, 0.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
                    .with_scale(Vec3::splat(scene_scale)),
            ));
            parent.spawn((
                Name::new("Target 3D Outline Shell"),
                TargetOutlineModel,
                SceneRoot(assets.enemy_scenes[scene_index].clone()),
                Transform::from_xyz(0.0, model_y_offset, 0.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
                    .with_scale(Vec3::splat(scene_scale * 1.05)),
                Visibility::Hidden,
            ));
            if kind == EnemyKind::CarrierBoss {
                let shield_scale = Vec3::splat(1.05);
                parent.spawn((
                    Name::new("Carrier ion shield"),
                    Mesh3d(assets.shield.clone()),
                    MeshMaterial3d(assets.shield_material.clone()),
                    Transform::from_xyz(0.0, 0.5, 0.0).with_scale(shield_scale),
                    BossShieldVisual {
                        base_scale: shield_scale,
                        max_shield: scaled_shield.max(1.0),
                    },
                ));
            }
        });
}

pub fn apply_outline_materials(
    assets: Option<Res<CombatAssets>>,
    outlines: Query<&Children, With<TargetOutlineModel>>,
    children: Query<&Children>,
    mut materials: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    let Some(assets) = assets else { return };
    for root_children in &outlines {
        fn set_white_recursive(
            entity: Entity,
            children: &Query<&Children>,
            materials: &mut Query<&mut MeshMaterial3d<StandardMaterial>>,
            white_mat: &Handle<StandardMaterial>,
        ) {
            if let Ok(mut mat) = materials.get_mut(entity) {
                if mat.0 != *white_mat {
                    mat.0 = white_mat.clone();
                }
            }
            if let Ok(child_list) = children.get(entity) {
                for &child in child_list {
                    set_white_recursive(child, children, materials, white_mat);
                }
            }
        }
        for &child in root_children {
            set_white_recursive(child, &children, &mut materials, &assets.white_outline_material);
        }
    }
}

pub fn update_enemy_visuals(
    time: Res<Time>,
    aim: Res<AimSolution>,
    session: Option<Res<GameSession>>,
    loaded: Res<LoadedVoxelChunks>,
    world: Res<VoxelViewerWorld>,
    camera_query: Query<&Transform, (With<VoxelViewerCameraTag>, Without<BossShieldVisual>)>,
    enemies: Query<
        (Entity, &Enemy, &CombatTarget, &Transform),
        (Without<BossShieldVisual>, Without<TargetOutlineModel>, Without<VoxelViewerCameraTag>),
    >,
    mut gizmos: Gizmos,
    mut outlines: Query<
        (&ChildOf, &mut Visibility),
        (With<TargetOutlineModel>, Without<BossShieldVisual>),
    >,
    mut shields: Query<
        (&ChildOf, &BossShieldVisual, &mut Transform, &mut Visibility),
        (Without<Enemy>, Without<TargetOutlineModel>, Without<VoxelViewerCameraTag>),
    >,
) {
    let elapsed = time.elapsed_secs();
    let camera_transform = camera_query.single().ok();
    let camera_right = camera_transform
        .map(|c| c.right().as_vec3())
        .unwrap_or(Vec3::X);
    let camera_rotation = camera_transform
        .map(|c| c.rotation)
        .unwrap_or(Quat::IDENTITY);

    for (parent, mut visibility) in &mut outlines {
        let is_target = aim.enemy == Some(parent.parent());
        *visibility = if is_target {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (entity, enemy, target, transform) in &enemies {
        let is_targeted = aim.enemy == Some(entity);
        let pulse = if is_targeted {
            1.0 + (elapsed * 6.0).sin() * 0.05
        } else {
            1.0
        };

        // Ground-projected target circle (aligned directly on terrain surface)
        let ground_y = ground_height(&loaded, &world, transform.translation.x, transform.translation.z);
        let ground_iso = Isometry3d::new(
            Vec3::new(transform.translation.x, ground_y + 0.12, transform.translation.z),
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        );
        if is_targeted {
            gizmos.circle(ground_iso, target.radius * pulse, Color::srgb(1.0, 0.25, 0.25));
            gizmos.circle(ground_iso, target.radius * 0.5 * pulse, Color::WHITE);

            // For aerial enemies, also draw a camera-facing 3D targeting reticle directly around the model
            if target.aerial {
                let body_iso = Isometry3d::new(transform.translation, camera_rotation);
                gizmos.circle(body_iso, target.radius * 1.15 * pulse, Color::srgba(1.0, 0.35, 0.35, 0.9));
                gizmos.circle(body_iso, target.radius * 0.6 * pulse, Color::srgba(1.0, 1.0, 1.0, 0.8));
            }
        }

        // Overhead 3D Health Bar Billboard (faces camera perfectly)
        let hp_ratio = (enemy.health / enemy.max_health).clamp(0.0, 1.0);
        if is_targeted || enemy.kind == EnemyKind::CarrierBoss || hp_ratio < 0.99 {
            let head_y = match enemy.kind {
                EnemyKind::Crawler => 1.3,
                EnemyKind::Spitter => 1.5,
                EnemyKind::Drone => 1.8,
                EnemyKind::Brute => 2.5,
                EnemyKind::CarrierBoss => 4.2,
            };
            let bar_center = transform.translation + Vec3::Y * head_y;
            let bar_half_width = match enemy.kind {
                EnemyKind::CarrierBoss => 2.5,
                EnemyKind::Brute => 1.4,
                _ => 0.9,
            };
            let start_pos = bar_center - camera_right * bar_half_width;
            let hp_pos = start_pos + camera_right * (bar_half_width * 2.0 * hp_ratio);
            let end_pos = start_pos + camera_right * (bar_half_width * 2.0);

            if hp_ratio > 0.0 {
                gizmos.line(start_pos, hp_pos, Color::srgb(0.2, 0.9, 0.3));
            }
            if hp_ratio < 1.0 {
                gizmos.line(hp_pos, end_pos, Color::srgb(0.85, 0.15, 0.15));
            }
        }
    }
    if let Some(aim_pt) = aim.aim_point {
        let ground_y = ground_height(&loaded, &world, aim_pt.x, aim_pt.z);
        let draw_y = if aim.enemy.is_some() {
            aim_pt.y + 0.1
        } else {
            ground_y + 0.15
        };
        let iso = Isometry3d::new(
            Vec3::new(aim_pt.x, draw_y, aim_pt.z),
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        );
        gizmos.circle(iso, 0.45, Color::srgb(1.0, 0.9, 0.2));
        if let Some(session) = &session {
            if session.loadout.selected_tool == ToolSlot::Weapon(WeaponKind::PlasmaMortar) {
                gizmos.circle(iso, 7.5, Color::srgba(1.0, 0.45, 0.1, 0.65));
            }
        }
    }
    for (parent, shield, mut transform, mut visibility) in &mut shields {
        let Ok((_, enemy, _, _)) = enemies.get(parent.parent()) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let ratio = (enemy.shield / shield.max_shield).clamp(0.0, 1.0);
        *visibility = if ratio > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let pulse = 1.0 + (elapsed * 4.8).sin() * (0.025 + ratio * 0.035);
        transform.scale = shield.base_scale * pulse * (0.90 + ratio * 0.10);
        transform.rotate_y(time.delta_secs() * 0.55);
    }
}

pub fn update_enemy_ai(
    time: Res<Time>,
    world: Res<VoxelViewerWorld>,
    loaded: Res<LoadedVoxelChunks>,
    assets: Res<CombatAssets>,
    _edits: Res<VoxelWorldEdits>,
    player: Query<&Transform, (With<PlayerTag>, Without<Enemy>)>,
    objectives: Query<(Entity, &Transform, &MissionTarget), Without<Enemy>>,
    mut enemies: Query<(&mut Transform, &mut Enemy, &mut CombatTarget), Without<PlayerTag>>,
    mut damage: MessageWriter<DamageEvent>,
    mut commands: Commands,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let dt = time.delta_secs().min(0.05);
    for (mut transform, mut enemy, mut target_data) in &mut enemies {
        let objective = objectives
            .iter()
            .find(|(_, _, target)| target.kind == MissionTargetKind::HomeCore);
        let (target, damage_target) = objective
            .map(|(e, t, _)| (t.translation, DamageTarget::Base(e)))
            .unwrap_or((player.translation, DamageTarget::Player));
        let offset = target - transform.translation;
        let flat = Vec3::new(offset.x, 0.0, offset.z);
        let distance = offset.length();
        if enemy.kind == EnemyKind::CarrierBoss && enemy.shield <= 0.0 {
            target_data.aerial = false;
        }
        let forward = flat.normalize_or_zero();
        let side = Vec3::new(-forward.z, 0.0, forward.x);
        if target_data.aerial {
            // Boss that has lost its shield descends to the ground
            let height = if enemy.kind == EnemyKind::CarrierBoss && enemy.shield <= 0.0 {
                let ground = ground_height(&loaded, &world, transform.translation.x, transform.translation.z);
                ground + 5.5  // land on the ground
            } else {
                target.y + 15.0
            };
            let orbit = side
                * (time.elapsed_secs() * 1.45 + transform.translation.x * 0.03).sin()
                * enemy.speed
                * if enemy.kind == EnemyKind::Drone {
                    0.72
                } else {
                    0.34
                };
            let desired =
                forward * enemy.speed + orbit + Vec3::Y * (height - transform.translation.y) * 1.4;
            transform.translation += desired.clamp_length_max(enemy.speed * 1.65) * dt;

            // Once boss is near the ground after shield loss, set it fully grounded
            if enemy.kind == EnemyKind::CarrierBoss && enemy.shield <= 0.0 {
                let ground = ground_height(&loaded, &world, transform.translation.x, transform.translation.z);
                if transform.translation.y <= ground + 7.0 {
                    target_data.aerial = false;
                    transform.translation.y = ground + 5.5;
                }
            }
        } else {
            let move_dir = match enemy.kind {
                EnemyKind::Spitter if distance < 14.0 => -forward,
                EnemyKind::Spitter if distance <= 26.0 => {
                    side * (time.elapsed_secs() * 2.2 + transform.translation.z * 0.05).sin()
                }
                EnemyKind::Crawler if distance <= 9.0 => forward * 2.4, // Fast lunging leap!
                EnemyKind::Crawler => {
                    let flank = side * (time.elapsed_secs() * 3.2 + transform.translation.x * 0.06).sin() * 0.42;
                    (forward + flank).normalize_or_zero()
                }
                EnemyKind::Brute if distance <= 16.0 => forward * 2.2, // Heavy Enraged Charge!
                EnemyKind::CarrierBoss if distance > 8.0 => forward * 1.35,
                _ if distance > melee_range(enemy.kind) => forward,
                _ => Vec3::ZERO,
            };
            if move_dir != Vec3::ZERO {
                let current_speed = if enemy.kind == EnemyKind::Crawler && distance <= 9.0 {
                    enemy.speed * 2.4
                } else if enemy.kind == EnemyKind::Brute && distance <= 16.0 {
                    enemy.speed * 2.2
                } else {
                    enemy.speed
                };
                let candidate = transform.translation + move_dir.normalize_or_zero() * current_speed * dt;
                let ground = ground_height(&loaded, &world, candidate.x, candidate.z);
                let target_y = ground + if enemy.kind == EnemyKind::CarrierBoss { 5.5 } else { 0.9 };
                let y_blend = 1.0 - (-14.0 * dt).exp();
                let new_y = transform.translation.y + (target_y - transform.translation.y) * y_blend;
                transform.translation = Vec3::new(candidate.x, new_y, candidate.z);
            }
        }
        if flat.length_squared() > 0.05 {
            let target_rotation = Quat::from_rotation_y(flat.x.atan2(flat.z));
            transform.rotation = transform.rotation.slerp(target_rotation, 1.0 - (-12.0 * dt).exp());
        }
        enemy.attack.tick(time.delta());
        if !enemy.attack.just_finished() {
            continue;
        }
        match enemy.kind {
            EnemyKind::Crawler if distance <= 5.5 => {
                damage.write(DamageEvent {
                    target: damage_target,
                    amount: 14.0,
                    kind: DamageKind::Enemy,
                });
            }
            EnemyKind::Brute if distance <= 7.5 => {
                damage.write(DamageEvent {
                    target: damage_target,
                    amount: 28.0,
                    kind: DamageKind::Enemy,
                });
            }
            EnemyKind::Spitter | EnemyKind::Drone | EnemyKind::CarrierBoss if distance <= 65.0 => {
                let lead_target = target + Vec3::Y * 0.8;
                let velocity = (lead_target - transform.translation).normalize_or_zero() * 26.0;
                spawn_shot(
                    &mut commands,
                    &assets,
                    transform.translation + Vec3::Y * 1.2,
                    velocity,
                    3,
                    Projectile {
                        from_player: false,
                        velocity,
                        gravity: 0.0,
                        damage: if enemy.kind == EnemyKind::CarrierBoss {
                            18.0
                        } else if enemy.kind == EnemyKind::Spitter {
                            12.0
                        } else {
                            10.0
                        },
                        kind: DamageKind::Enemy,
                        life: Timer::from_seconds(3.5, TimerMode::Once),
                        radius: 1.3,
                        target: None,
                        area: 0.0,
                    },
                );
            }
            _ => {}
        }
    }
}

pub fn update_projectiles(
    time: Res<Time<Fixed>>,
    assets: Res<CombatAssets>,
    mut shots: Query<
        (
            Entity,
            &mut Transform,
            &mut Projectile,
            &mut ProjectileTrail,
        ),
        Without<PlayerTag>,
    >,
    targets: Query<(Entity, &GlobalTransform, &CombatTarget)>,
    player: Query<&Transform, (With<PlayerTag>, Without<Projectile>)>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut edits: ResMut<VoxelWorldEdits>,
    mut damage: MessageWriter<DamageEvent>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut shot, mut trail) in &mut shots {
        shot.life.tick(time.delta());
        if shot.life.just_finished() {
            if shot.from_player && shot.area > 0.0 {
                area_damage(transform.translation, &shot, &targets, &mut damage);
                let block_pos = world_to_block(transform.translation);
                edits.edits.push(VoxelTerrainEdit::DigSphere {
                    center: block_pos,
                    radius: 2,
                });
                invalidate_edit(&mut commands, &mut loaded, block_pos, 2);
            }
            let impact_power = match shot.kind {
                DamageKind::Nuke => 5.4,
                DamageKind::Plasma => 2.8,
                DamageKind::Ion => 1.6,
                DamageKind::Tesla => 1.4,
                _ => if shot.area > 0.0 { 2.2 } else { 0.95 },
            };
            spawn_impact_fx(
                &mut commands,
                &assets,
                transform.translation,
                trail.style,
                impact_power,
            );
            if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
            continue;
        }
        if let Some(target) = shot.target
            && let Ok((_, target_transform, _)) = targets.get(target)
        {
            let desired = (target_transform.translation() - transform.translation)
                .normalize_or_zero()
                * shot.velocity.length();
            shot.velocity = shot
                .velocity
                .lerp(desired, 1.0 - (-8.5 * time.delta_secs()).exp());
        } else if shot.from_player {
            let speed = shot.velocity.length();
            if speed > 1.0 {
                if let Some((_, near_transform, _)) = targets.iter().find(|(_, t_trans, t_data)| {
                    t_data.targetable
                        && projectile_accepts_target(shot.kind, t_data.aerial)
                        && t_trans.translation().distance(transform.translation) <= 7.5
                }) {
                    let desired = (near_transform.translation() - transform.translation)
                        .normalize_or_zero()
                        * speed;
                    shot.velocity = shot
                        .velocity
                        .lerp(desired, 1.0 - (-6.5 * time.delta_secs()).exp());
                }
            }
        }
        let start = transform.translation;
        let dt = time.delta_secs();
        shot.velocity.y -= shot.gravity * dt;
        let end = start + shot.velocity * dt;
        #[derive(Clone, Copy)]
        enum ProjectileImpact {
            Target(DamageTarget),
            Terrain,
        }
        let mut impact = if shot.from_player {
            targets
                .iter()
                .filter(|(_, _, target)| {
                    target.targetable && projectile_accepts_target(shot.kind, target.aerial)
                })
                .filter_map(|(entity, target_transform, target)| {
                    segment_sphere_hit_fraction(
                        start,
                        end,
                        target_transform.translation(),
                        target.radius + shot.radius + 1.85,
                    )
                    .map(|fraction| {
                        (
                            fraction,
                            ProjectileImpact::Target(DamageTarget::Enemy(entity)),
                        )
                    })
                })
                .min_by(|left, right| left.0.total_cmp(&right.0))
        } else {
            player.single().ok().and_then(|player| {
                segment_sphere_hit_fraction(start, end, player.translation, 3.4 + shot.radius)
                    .map(|fraction| (fraction, ProjectileImpact::Target(DamageTarget::Player)))
            })
        };
        if let Some(fraction) = terrain_hit_fraction(&loaded, start, end, shot.radius) {
            if impact.is_none_or(|(target_fraction, _)| fraction < target_fraction) {
                impact = Some((fraction, ProjectileImpact::Terrain));
            }
        }
        if let Some((fraction, impact)) = impact {
            let impact_position = start.lerp(end, fraction);
            if shot.area > 0.0 && shot.from_player {
                area_damage(impact_position, &shot, &targets, &mut damage);
            } else if let ProjectileImpact::Target(target) = impact {
                damage.write(DamageEvent {
                    target,
                    amount: shot.damage,
                    kind: shot.kind,
                });
            }

            let is_terrain = matches!(impact, ProjectileImpact::Terrain);
            let is_explosion = shot.area > 0.0;
            if is_terrain || is_explosion {
                let crater_radius: u16 = match shot.kind {
                    DamageKind::Plasma => 2,
                    DamageKind::Ion => 1,
                    DamageKind::Pulse => 1,
                    DamageKind::Enemy => 1,
                    DamageKind::Tesla => 1,
                    DamageKind::Nuke => 4,
                };
                let block_pos = world_to_block(impact_position);
                edits.edits.push(VoxelTerrainEdit::DigSphere {
                    center: block_pos,
                    radius: crater_radius,
                });
                invalidate_edit(&mut commands, &mut loaded, block_pos, crater_radius as i32);
            }

            let impact_power = match shot.kind {
                DamageKind::Nuke => 5.4,
                DamageKind::Plasma => 2.8,
                DamageKind::Ion => 1.6,
                DamageKind::Tesla => 1.4,
                _ => if shot.area > 0.0 { 2.2 } else { 0.95 },
            };
            spawn_impact_fx(
                &mut commands,
                &assets,
                impact_position,
                trail.style,
                impact_power,
            );
            if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
            continue;
        }
        transform.translation = end;
        let direction = shot.velocity.normalize_or_zero();
        if direction != Vec3::ZERO {
            transform.rotation = Quat::from_rotation_arc(Vec3::Z, direction);
        }
        if trail.last_position.distance_squared(end) > 0.10 {
            spawn_tracer_fx(
                &mut commands,
                &assets,
                trail.last_position,
                end,
                trail.style,
                if shot.kind == DamageKind::Plasma {
                    1.7
                } else {
                    1.05
                },
                if shot.kind == DamageKind::Plasma {
                    0.19
                } else {
                    0.13
                },
            );
            trail.last_position = end;
        }
    }
}

fn ballistic_velocity(origin: Vec3, target: Vec3, speed: f32, gravity: f32) -> Vec3 {
    let offset = target - origin;
    let horizontal = Vec3::new(offset.x, 0.0, offset.z);
    let horizontal_distance = horizontal.length();
    if horizontal_distance <= f32::EPSILON || gravity <= f32::EPSILON {
        return offset.normalize_or_zero() * speed;
    }
    let speed_squared = speed * speed;
    let discriminant = speed_squared * speed_squared
        - gravity
            * (gravity * horizontal_distance * horizontal_distance
                + 2.0 * offset.y * speed_squared);
    if discriminant < 0.0 {
        return offset.normalize_or_zero() * speed + Vec3::Y * (gravity * 0.16);
    }
    let tangent = (speed_squared - discriminant.sqrt()) / (gravity * horizontal_distance);
    let cosine = 1.0 / (1.0 + tangent * tangent).sqrt();
    horizontal.normalize_or_zero() * (speed * cosine) + Vec3::Y * (speed * cosine * tangent)
}

fn segment_sphere_hit_fraction(start: Vec3, end: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let segment = end - start;
    let segment_length_squared = segment.length_squared();
    if segment_length_squared <= f32::EPSILON {
        return (start.distance_squared(center) <= radius * radius).then_some(0.0);
    }
    let offset = start - center;
    let b = 2.0 * offset.dot(segment);
    let c = offset.length_squared() - radius * radius;
    if c <= 0.0 {
        return Some(0.0);
    }
    let discriminant = b * b - 4.0 * segment_length_squared * c;
    if discriminant < 0.0 {
        return None;
    }
    let fraction = (-b - discriminant.sqrt()) / (2.0 * segment_length_squared);
    (0.0..=1.0).contains(&fraction).then_some(fraction)
}

fn terrain_hit_fraction(
    loaded: &LoadedVoxelChunks,
    start: Vec3,
    end: Vec3,
    radius: f32,
) -> Option<f32> {
    let segment = end - start;
    let distance = segment.length();
    if distance <= f32::EPSILON {
        return None;
    }
    let direction = segment / distance;
    let hit = voxel_raycast_loaded(loaded, start, direction, distance + radius)?;
    let hit_center = block_world_center(hit.block);
    let impact_distance = ((hit_center - start).dot(direction) - radius).max(0.0);
    Some((impact_distance / distance).clamp(0.0, 1.0))
}

fn area_damage(
    center: Vec3,
    shot: &Projectile,
    targets: &Query<(Entity, &GlobalTransform, &CombatTarget)>,
    damage: &mut MessageWriter<DamageEvent>,
) {
    for (entity, transform, target) in targets {
        if target.targetable
            && !target.aerial
            && transform.translation().distance(center) <= shot.area
        {
            damage.write(DamageEvent {
                target: DamageTarget::Enemy(entity),
                amount: shot.damage,
                kind: shot.kind,
            });
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct DamageWriters<'w> {
    pub relays: MessageWriter<'w, RelayDestroyed>,
    pub resources: MessageWriter<'w, ResourceCollected>,
    pub kills: MessageWriter<'w, EnemyKilled>,
    pub respawned: MessageWriter<'w, PlayerRespawned>,
    pub sounds: MessageWriter<'w, GameSound>,
    pub finished: MessageWriter<'w, RunFinished>,
}

pub fn process_damage(
    time: Res<Time>,
    _balance: Res<BalanceConfig>,
    assets: Res<CombatAssets>,
    mut runtime: ResMut<CombatRuntime>,
    mut session: ResMut<GameSession>,
    _player: Query<&Transform, (With<PlayerTag>, Without<Enemy>, Without<MissionTarget>)>,
    mut enemies: Query<(&mut Enemy, &mut CombatTarget, &Transform), Without<PlayerTag>>,
    mut objectives: Query<(&mut MissionTarget, &Transform), (Without<Enemy>, Without<PlayerTag>)>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut edits: ResMut<VoxelWorldEdits>,
    mut events: MessageReader<DamageEvent>,
    mut writers: DamageWriters,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    runtime.last_hit += time.delta_secs();
    for event in events.read() {
        match event.target {
            DamageTarget::Player => {
                runtime.last_hit = 0.0;
                let absorbed = session.loadout.shield.min(event.amount);
                session.loadout.shield -= absorbed;
                session.loadout.health =
                    (session.loadout.health - event.amount + absorbed).max(0.0);
                writers.sounds.write(GameSound::PlayerHit);
                if session.loadout.health <= 0.0 {
                    finish_run(
                        &mut session,
                        RunOutcome::MissionFailed,
                        &mut writers.finished,
                        &mut next_state,
                    );
                }
            }
            DamageTarget::Enemy(entity) => {
                if let Ok((mut enemy, mut target, enemy_transform)) = enemies.get_mut(entity) {
                    if enemy.kind == EnemyKind::CarrierBoss && enemy.shield > 0.0 {
                        enemy.shield = (enemy.shield
                            - event.amount
                                * if event.kind == DamageKind::Ion {
                                    1.0
                                } else {
                                    0.18
                                })
                        .max(0.0);
                        if enemy.shield <= 0.0 {
                            target.aerial = false;
                            session.objective_hint =
                                "درع الحاملة انهار — اضرب النواة بالنبض والبلازما".into();
                            spawn_impact_fx(
                                &mut commands,
                                &assets,
                                enemy_transform.translation + Vec3::Y * 1.8,
                                2,
                                3.2,
                            );
                            writers.sounds.write(GameSound::Warning);
                        }
                    } else {
                        enemy.health -= event.amount;
                    }
                    if enemy.health <= 0.0 {
                        let kind = enemy.kind;
                        let death_position = enemy_transform.translation;
                        spawn_impact_fx(
                            &mut commands,
                            &assets,
                            death_position,
                            if kind == EnemyKind::CarrierBoss { 2 } else { 3 },
                            if kind == EnemyKind::CarrierBoss {
                                5.4
                            } else {
                                2.1
                            },
                        );

                        let crater_radius: u16 = if kind == EnemyKind::CarrierBoss {
                            4
                        } else if kind == EnemyKind::Brute {
                            2
                        } else {
                            1
                        };
                        let block_pos = world_to_block(death_position);
                        edits.edits.push(VoxelTerrainEdit::DigSphere {
                            center: block_pos,
                            radius: crater_radius,
                        });
                        invalidate_edit(&mut commands, &mut loaded, block_pos, crater_radius as i32);
                        if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
                        session.loadout.kills += 1;
                        let points_earned = kind.point_value();
                        session.loadout.add_points(points_earned);
                        session.check_level_up();
                        writers.kills.write(EnemyKilled(kind));
                        let drop = match kind {
                            EnemyKind::Crawler | EnemyKind::Spitter => ResourceKind::SpaceIron,
                            EnemyKind::Drone => ResourceKind::Helium3,
                            EnemyKind::Brute => ResourceKind::Titanium,
                            EnemyKind::CarrierBoss => ResourceKind::BioPlasma,
                        };
                        session.loadout.add_resource(drop, 1);
                        writers.resources.write(ResourceCollected(drop, 1));
                        writers.sounds.write(GameSound::EnemyDeath);
                        if kind == EnemyKind::CarrierBoss {
                            session.objective_hint = "انهار الزعيم — صفِّ بقية الموجة لتحصل على مكافأة التطهير".into();
                        }
                    } else {
                        writers.sounds.write(GameSound::EnemyHit);
                    }
                } else if let Ok((mut objective, objective_transform)) = objectives.get_mut(entity)
                {
                    objective.health -= event.amount;
                    match objective.kind {
                        MissionTargetKind::Relay(index) if objective.health <= 0.0 => {
                            spawn_impact_fx(
                                &mut commands,
                                &assets,
                                objective_transform.translation,
                                1,
                                3.0,
                            );
                            if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
                            writers.relays.write(RelayDestroyed(index));
                        }
                        MissionTargetKind::HomeCore | MissionTargetKind::Ship => {
                            session.base_health = objective.health.max(0.0);
                            if objective.health <= 0.0 {
                                finish_run(
                                    &mut session,
                                    RunOutcome::MissionFailed,
                                    &mut writers.finished,
                                    &mut next_state,
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            DamageTarget::Base(entity) => {
                if let Ok((mut objective, _)) = objectives.get_mut(entity) {
                    objective.health = (objective.health - event.amount).max(0.0);
                    session.base_health = objective.health;
                    if objective.health <= 0.0 {
                        finish_run(
                            &mut session,
                            RunOutcome::MissionFailed,
                            &mut writers.finished,
                            &mut next_state,
                        );
                    }
                }
            }
        }
    }
    if runtime.last_hit > 4.0 {
        session.loadout.shield = (session.loadout.shield + time.delta_secs() * 12.0).min(50.0);
    }
}

pub fn damage_player_blocks(
    time: Res<Time>,
    brutes: Query<(&Transform, &Enemy)>,
    mut edits: ResMut<VoxelWorldEdits>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut timer: Local<f32>,
    mut commands: Commands,
) {
    *timer += time.delta_secs();
    if *timer < 0.6 {
        return;
    }
    *timer = 0.0;
    let mut destroyed = None;
    for (transform, enemy) in &brutes {
        if enemy.kind != EnemyKind::Brute {
            continue;
        }
        if let Some((position, durability)) = edits
            .placed_durability
            .iter_mut()
            .find(|(p, _)| transform.translation.distance(block_world_center(**p)) < 7.0)
        {
            *durability -= 18.0;
            if *durability <= 0.0 {
                destroyed = Some(*position);
                break;
            }
        }
    }
    if let Some(position) = destroyed {
        edits.placed_durability.remove(&position);
        edits.edits.push(VoxelTerrainEdit::SetBlock {
            position,
            block: BlockKind::Air,
        });
        invalidate_edit(&mut commands, &mut loaded, position, 1);
    }
}

fn spawn_shot(
    commands: &mut Commands,
    assets: &CombatAssets,
    origin: Vec3,
    velocity: Vec3,
    material: usize,
    mut shot: Projectile,
) {
    shot.velocity = velocity;
    let style = fx_style(shot.kind);
    let direction = velocity.normalize_or_zero();
    let (core_size, _halo_size, tail_length, light_intensity, light_range) =
        projectile_visual_profile(shot.kind);
    spawn_muzzle_fx(
        commands,
        assets,
        origin,
        direction,
        style,
        if shot.from_player { 1.25 } else { 0.72 },
    );
    let phase = origin.x * 0.17 + origin.y * 0.11 + origin.z * 0.23;
    commands
        .spawn((
            Name::new("Energy projectile core"),
            Mesh3d(assets.shot.clone()),
            MeshMaterial3d(assets.fx_core_materials[style].clone()),
            Transform::from_translation(origin).with_scale(Vec3::splat(core_size * 0.45)),
            PointLight {
                color: fx_color(style),
                intensity: light_intensity,
                range: light_range,
                radius: 0.18 * core_size,
                shadows_enabled: false,
                ..default()
            },
            ProjectileVisual {
                base_scale: Vec3::splat(core_size * 0.45),
                light_intensity,
                light_range,
                phase,
            },
            ProjectileTrail {
                last_position: origin,
                style,
            },
            shot,
            RunEntity,
        ))
        .with_children(|projectile| {
            // Layer 1: Saturated inner plasma glow
            projectile.spawn((
                Name::new("Projectile inner plasma"),
                Mesh3d(assets.shot.clone()),
                MeshMaterial3d(assets.materials[material].clone()),
                Transform::from_scale(Vec3::splat(1.35)),
            ));
            // Layer 2: Soft Radial Gradient Glow Flare (Omnidirectional 3D Aura)
            let flare_size = _halo_size * 2.8;
            projectile.spawn((
                Name::new("Projectile glow flare H"),
                Mesh3d(assets.glow_quad.clone()),
                MeshMaterial3d(assets.glow_materials[style].clone()),
                Transform::from_scale(Vec3::splat(flare_size)),
            ));
            projectile.spawn((
                Name::new("Projectile glow flare V"),
                Mesh3d(assets.glow_quad.clone()),
                MeshMaterial3d(assets.glow_materials[style].clone()),
                Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    scale: Vec3::splat(flare_size),
                },
            ));
            projectile.spawn((
                Name::new("Projectile glow flare Flat"),
                Mesh3d(assets.glow_quad.clone()),
                MeshMaterial3d(assets.glow_materials[style].clone()),
                Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                    scale: Vec3::splat(flare_size),
                },
            ));
            // Layer 3: Sleek tapered plasma tail
            projectile.spawn((
                Name::new("Projectile plasma tail"),
                Mesh3d(assets.beam.clone()),
                MeshMaterial3d(assets.fx_materials[style].clone()),
                Transform::from_translation(Vec3::Z * -tail_length * 0.48).with_scale(Vec3::new(
                    core_size * 0.35,
                    core_size * 0.35,
                    tail_length,
                )),
            ));
            // Layer 4: Hot needle center tail
            projectile.spawn((
                Name::new("Projectile hot core tail"),
                Mesh3d(assets.beam.clone()),
                MeshMaterial3d(assets.fx_core_materials[style].clone()),
                Transform::from_translation(Vec3::Z * -tail_length * 0.44).with_scale(Vec3::new(
                    core_size * 0.12,
                    core_size * 0.12,
                    tail_length * 0.88,
                )),
            ));
        });
}

fn projectile_visual_profile(kind: DamageKind) -> (f32, f32, f32, f32, f32) {
    match kind {
        DamageKind::Pulse => (0.60, 1.40, 3.4, 28_000.0, 14.0),
        DamageKind::Plasma => (1.45, 2.80, 6.2, 75_000.0, 26.0),
        DamageKind::Ion => (0.95, 1.90, 7.2, 44_000.0, 19.0),
        DamageKind::Tesla => (0.80, 1.60, 5.0, 34_000.0, 16.0),
        DamageKind::Nuke => (2.20, 4.20, 10.5, 130_000.0, 38.0),
        DamageKind::Enemy => (0.75, 1.50, 3.8, 22_000.0, 13.0),
    }
}

fn melee_range(kind: EnemyKind) -> f32 {
    match kind {
        EnemyKind::Crawler => 4.4,
        EnemyKind::Brute => 6.2,
        _ => 20.0,
    }
}

fn ground_height(loaded: &LoadedVoxelChunks, world: &VoxelViewerWorld, x: f32, z: f32) -> f32 {
    let bx = (x / BLOCK_SIZE).floor() as i64;
    let bz = (z / BLOCK_SIZE).floor() as i64;
    loaded
        .ground_below(bx, bz, 220)
        .map(|y| y as f32 * HEIGHT_SCALE)
        .unwrap_or_else(|| sample_voxel_column(world.settings, bx, bz).height as f32 * HEIGHT_SCALE)
}

pub fn sync_player_weapon_visual(
    session: Res<GameSession>,
    assets: Res<CombatAssets>,
    model_roots: Query<Entity, With<PlayerModelRoot>>,
    bones: Query<(Entity, &Name)>,
    parents: Query<&ChildOf>,
    visuals: Query<Entity, With<WeaponVisual>>,
    mut active: Local<Option<WeaponKind>>,
    mut commands: Commands,
) {
    let desired = match session.loadout.selected_tool {
        ToolSlot::Weapon(weapon) if session.loadout.weapon_level(weapon) > 0 => Some(weapon),
        _ => None,
    };
    let visual_count = visuals.iter().count();
    if *active == desired && visual_count > 0 {
        return;
    }
    if desired.is_none() {
        for entity in &visuals {
            if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
        }
        *active = None;
        return;
    }
    let Ok(model_root) = model_roots.single() else {
        return;
    };
    let Some(right_arm) = bones.iter().find_map(|(entity, name)| {
        (name.as_str() == "arm-right" && is_descendant_of(entity, model_root, &parents))
            .then_some(entity)
    }) else {
        return;
    };
    for entity in &visuals {
        if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
    }
    let Some(weapon) = desired else {
        return;
    };
    let scene = match weapon {
        WeaponKind::PulseRifle => 0,
        WeaponKind::PlasmaMortar => 1,
        WeaponKind::IonLance => 2,
        WeaponKind::QuantumTesla => 0,
        WeaponKind::NukeMortar => 1,
    };
    let (rest, muzzle_distance) = weapon_mount(weapon);
    commands.entity(right_arm).with_children(|arm| {
        arm.spawn((
            Name::new("Animated hand weapon rig"),
            SceneRoot(assets.weapon_scenes[scene].clone()),
            rest.clone(),
            WeaponVisual,
            WeaponRig {
                weapon,
                rest,
                recoil: 0.0,
            },
        ))
        .with_children(|weapon_root| {
            weapon_root.spawn((
                Name::new("Weapon muzzle socket"),
                Transform::from_xyz(0.0, 0.0, muzzle_distance),
                WeaponMuzzle { weapon },
            ));
        });
    });
    *active = desired;
}

fn is_descendant_of(mut entity: Entity, root: Entity, parents: &Query<&ChildOf>) -> bool {
    for _ in 0..12 {
        if entity == root {
            return true;
        }
        let Ok(parent) = parents.get(entity) else {
            return false;
        };
        entity = parent.parent();
    }
    false
}

fn weapon_mount(weapon: WeaponKind) -> (Transform, f32) {
    match weapon {
        WeaponKind::PulseRifle => (
            Transform {
                translation: Vec3::new(-0.15, -0.65, 0.15),
                rotation: Quat::from_euler(EulerRot::YXZ, std::f32::consts::PI, -0.10, 0.0),
                scale: Vec3::splat(1.15),
            },
            0.47,
        ),
        WeaponKind::PlasmaMortar => (
            Transform {
                translation: Vec3::new(-0.15, -0.70, 0.15),
                rotation: Quat::from_euler(EulerRot::YXZ, std::f32::consts::PI, -0.08, 0.0),
                scale: Vec3::splat(1.35),
            },
            0.29,
        ),
        WeaponKind::IonLance => (
            Transform {
                translation: Vec3::new(-0.15, -0.72, 0.15),
                rotation: Quat::from_euler(EulerRot::YXZ, std::f32::consts::PI, -0.06, 0.0),
                scale: Vec3::splat(1.25),
            },
            0.38,
        ),
        WeaponKind::QuantumTesla => (
            Transform {
                translation: Vec3::new(-0.15, -0.68, 0.15),
                rotation: Quat::from_euler(EulerRot::YXZ, std::f32::consts::PI, -0.08, 0.0),
                scale: Vec3::splat(1.20),
            },
            0.40,
        ),
        WeaponKind::NukeMortar => (
            Transform {
                translation: Vec3::new(-0.15, -0.75, 0.15),
                rotation: Quat::from_euler(EulerRot::YXZ, std::f32::consts::PI, -0.06, 0.0),
                scale: Vec3::splat(1.40),
            },
            0.32,
        ),
    }
}

fn apply_weapon_recoil(rigs: &mut Query<&mut WeaponRig>, weapon: WeaponKind, strength: f32) {
    for mut rig in rigs.iter_mut() {
        if rig.weapon == weapon {
            rig.recoil = (rig.recoil + strength).min(1.35);
        }
    }
}

pub fn update_weapon_recoil(time: Res<Time>, mut rigs: Query<(&mut Transform, &mut WeaponRig)>) {
    let dt = time.delta_secs().min(0.05);
    for (mut transform, mut rig) in &mut rigs {
        rig.recoil *= (-17.0 * dt).exp();
        let rest = rig.rest.clone();
        let recoil = rig.recoil;
        *transform = rest;
        transform.translation -= Vec3::Z * recoil * 0.16;
        transform.rotation *= Quat::from_rotation_x(-recoil * 0.10);
    }
}

pub fn update_combat_effects(
    time: Res<Time>,
    mut effects: Query<
        (
            Entity,
            &mut Transform,
            &mut CombatFx,
            Option<&mut FxMotion>,
            Option<&FxSpin>,
            Option<&mut PointLight>,
        ),
        (
            With<CombatFx>,
            Without<ProjectileVisual>,
            Without<ProjectileHalo>,
        ),
    >,
    mut projectiles: Query<
        (
            &ProjectileVisual,
            &Projectile,
            &mut Transform,
            Option<&mut PointLight>,
        ),
        (Without<CombatFx>, Without<ProjectileHalo>),
    >,
    mut halos: Query<
        (&ChildOf, &ProjectileHalo, &mut Transform),
        (Without<CombatFx>, Without<ProjectileVisual>),
    >,
    mut commands: Commands,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut transform, mut effect, motion, spin, light) in &mut effects {
        effect.age += dt;
        let progress = (effect.age / effect.duration.max(0.001)).clamp(0.0, 1.0);
        if progress >= 1.0 {
            if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
            continue;
        }
        let eased = 1.0 - (1.0 - progress).powi(2);
        transform.scale = effect.start_scale.lerp(effect.end_scale, eased);
        if let Some(mut motion) = motion {
            let drag_decay = (-motion.drag * dt).exp();
            motion.velocity.y -= motion.gravity * dt;
            motion.velocity *= drag_decay;
            transform.translation += motion.velocity * dt;
            let direction = motion.velocity.normalize_or_zero();
            if direction != Vec3::ZERO {
                transform.rotation = Quat::from_rotation_arc(Vec3::Z, direction);
            }
        }
        if let Some(mut light) = light {
            light.intensity = effect.peak_light * (1.0 - progress).powi(2);
        }
        if let Some(spin) = spin {
            transform.rotate(Quat::from_axis_angle(spin.axis, spin.radians_per_second * dt));
        }
    }
    let elapsed = time.elapsed_secs();
    for (visual, shot, mut transform, light) in &mut projectiles {
        let speed_ratio = (shot.velocity.length() / 42.0).clamp(0.55, 1.35);
        let pulse = 1.0 + (elapsed * (8.0 + speed_ratio * 2.0) + visual.phase).sin() * 0.10;
        transform.scale = visual.base_scale * pulse;
        if let Some(mut light) = light {
            light.intensity = visual.light_intensity * (0.84 + pulse * 0.18) * speed_ratio;
            light.range = visual.light_range * (0.90 + pulse * 0.12);
        }
    }
    for (parent, halo, mut transform) in &mut halos {
        let Ok((visual, shot, _, _)) = projectiles.get(parent.parent()) else {
            continue;
        };
        let speed_ratio = (shot.velocity.length() / 42.0).clamp(0.55, 1.35);
        let pulse = 1.0
            + (elapsed * (9.5 + speed_ratio * 2.5) + halo.phase).sin() * halo.pulse * speed_ratio;
        transform.scale = halo.base_scale * pulse;
        transform.rotate_z(halo.spin * dt * (0.75 + speed_ratio * 0.25));
        transform.translation.y =
            (elapsed * 5.5 + visual.phase + halo.phase).sin() * 0.035 * speed_ratio;
    }
}

fn fx_style(kind: DamageKind) -> usize {
    match kind {
        DamageKind::Pulse => 0,
        DamageKind::Plasma => 1,
        DamageKind::Ion => 2,
        DamageKind::Tesla => 0,
        DamageKind::Nuke => 1,
        DamageKind::Enemy => 3,
    }
}

fn fx_color(style: usize) -> Color {
    match style {
        0 => Color::srgb(0.22, 0.96, 1.0),
        1 => Color::srgb(1.0, 0.18, 0.78),
        2 => Color::srgb(0.38, 0.56, 1.0),
        _ => Color::srgb(1.0, 0.16, 0.06),
    }
}

fn spawn_muzzle_fx(
    commands: &mut Commands,
    assets: &CombatAssets,
    origin: Vec3,
    direction: Vec3,
    style: usize,
    power: f32,
) {
    let style = style % assets.fx_core_materials.len();
    let direction = direction.normalize_or_zero();
    if direction == Vec3::ZERO {
        return;
    }
    let ignition_scale = Vec3::splat(0.12 * power);
    commands.spawn((
        Name::new("Muzzle ignition core"),
        Mesh3d(assets.flash.clone()),
        MeshMaterial3d(assets.fx_core_materials[style].clone()),
        Transform::from_translation(origin).with_scale(ignition_scale),
        PointLight {
            color: fx_color(style),
            intensity: 18_000.0 * power,
            range: 12.0 * power,
            radius: 0.25,
            shadows_enabled: false,
            ..default()
        },
        CombatFx {
            age: 0.0,
            duration: 0.075,
            start_scale: ignition_scale,
            end_scale: Vec3::splat(1.65 * power),
            peak_light: 18_000.0 * power,
        },
        RunEntity,
    ));
    let ring_scale = Vec3::splat(0.18 * power);
    commands.spawn((
        Name::new("Muzzle shock iris"),
        Mesh3d(assets.ring.clone()),
        MeshMaterial3d(assets.fx_core_materials[style].clone()),
        Transform {
            translation: origin + direction * 0.08,
            rotation: Quat::from_rotation_arc(Vec3::Y, direction),
            scale: ring_scale,
        },
        CombatFx {
            age: 0.0,
            duration: 0.16,
            start_scale: ring_scale,
            end_scale: Vec3::splat(2.65 * power),
            peak_light: 0.0,
        },
        FxSpin {
            axis: direction,
            radians_per_second: 18.0,
        },
        RunEntity,
    ));
    let rail_length = 3.2 * power;
    let mut rail_transform = Transform::from_translation(origin + direction * rail_length * 0.46);
    rail_transform.look_at(origin + direction * rail_length, Vec3::Y);
    let rail_scale = Vec3::new(0.32 * power, 0.32 * power, rail_length);
    rail_transform.scale = rail_scale;
    commands.spawn((
        Name::new("Muzzle accelerator channel"),
        Mesh3d(assets.beam.clone()),
        MeshMaterial3d(assets.fx_materials[style].clone()),
        rail_transform,
        CombatFx {
            age: 0.0,
            duration: 0.085,
            start_scale: rail_scale,
            end_scale: Vec3::new(0.03, 0.03, rail_length * 0.68),
            peak_light: 0.0,
        },
        RunEntity,
    ));
    let mut core_transform = Transform::from_translation(origin + direction * rail_length * 0.48);
    core_transform.look_at(origin + direction * rail_length, Vec3::Y);
    let core_scale = Vec3::new(0.075 * power, 0.075 * power, rail_length * 0.92);
    core_transform.scale = core_scale;
    commands.spawn((
        Name::new("Muzzle accelerator core"),
        Mesh3d(assets.beam.clone()),
        MeshMaterial3d(assets.fx_core_materials[style].clone()),
        core_transform,
        CombatFx {
            age: 0.0,
            duration: 0.055,
            start_scale: core_scale,
            end_scale: Vec3::new(0.01, 0.01, rail_length * 0.72),
            peak_light: 0.0,
        },
        RunEntity,
    ));
    let reference = if direction.y.abs() > 0.85 { Vec3::Z } else { Vec3::Y };
    let right = direction.cross(reference).normalize_or_zero();
    let up = right.cross(direction).normalize_or_zero();
    for index in 0..3 {
        let angle = index as f32 * std::f32::consts::TAU / 3.0;
        let offset = (right * angle.cos() + up * angle.sin()) * (0.24 * power);
        let start = origin - direction * 0.18 + offset;
        let end = origin + direction * (1.65 * power) + offset * 0.20;
        let delta = end - start;
        let distance = delta.length();
        let mut arc_transform = Transform::from_translation(start + delta * 0.5);
        arc_transform.look_at(end, up);
        let arc_scale = Vec3::new(0.055 * power, 0.055 * power, distance);
        arc_transform.scale = arc_scale;
        commands.spawn((
            Name::new("Muzzle discharge blade"),
            Mesh3d(assets.beam.clone()),
            MeshMaterial3d(assets.fx_core_materials[style].clone()),
            arc_transform,
            CombatFx {
                age: 0.0,
                duration: 0.11 + index as f32 * 0.02,
                start_scale: arc_scale,
                end_scale: Vec3::new(0.01, 0.01, distance * 0.36),
                peak_light: 0.0,
            },
            FxSpin {
                axis: direction,
                radians_per_second: 12.0 + index as f32 * 4.0,
            },
            RunEntity,
        ));
    }
    spawn_tracer_fx(
        commands,
        assets,
        origin,
        origin + direction * (3.6 * power),
        style,
        0.92 * power,
        0.045,
    );
}

fn spawn_tracer_fx(
    commands: &mut Commands,
    assets: &CombatAssets,
    start: Vec3,
    end: Vec3,
    style: usize,
    thickness: f32,
    duration: f32,
) {
    let style = style % assets.fx_materials.len();
    let delta = end - start;
    let distance = delta.length();
    if distance <= 0.01 {
        return;
    }
    let mut glow_transform = Transform::from_translation(start + delta * 0.5);
    glow_transform.look_at(end, Vec3::Y);
    let glow_scale = Vec3::new(thickness, thickness, distance);
    glow_transform.scale = glow_scale;
    commands.spawn((
        Name::new("Energy tracer glow"),
        Mesh3d(assets.beam.clone()),
        MeshMaterial3d(assets.fx_materials[style].clone()),
        glow_transform,
        CombatFx {
            age: 0.0,
            duration: (duration * 1.12).max(0.025),
            start_scale: glow_scale,
            end_scale: Vec3::new(0.04, 0.04, distance * 0.96),
            peak_light: 0.0,
        },
        RunEntity,
    ));
    let mut core_transform = Transform::from_translation(start + delta * 0.5);
    core_transform.look_at(end, Vec3::Y);
    let core_scale = Vec3::new(thickness * 0.30, thickness * 0.30, distance * 0.98);
    core_transform.scale = core_scale;
    commands.spawn((
        Name::new("Energy tracer hot core"),
        Mesh3d(assets.beam.clone()),
        MeshMaterial3d(assets.fx_core_materials[style].clone()),
        core_transform,
        CombatFx {
            age: 0.0,
            duration: (duration * 0.78).max(0.02),
            start_scale: core_scale,
            end_scale: Vec3::new(0.02, 0.02, distance * 0.90),
            peak_light: 0.0,
        },
        RunEntity,
    ));
}

fn spawn_impact_fx(
    commands: &mut Commands,
    assets: &CombatAssets,
    position: Vec3,
    style: usize,
    power: f32,
) {
    let style = style % assets.fx_materials.len();
    let light_intensity = 85_000.0 * power;
    let light_range = 24.0 * power;
    let core_scale = Vec3::splat(0.18 * power);
    let end_core = Vec3::splat(1.6 * power);

    commands.spawn((
        Name::new("Energy impact core"),
        Mesh3d(assets.flash.clone()),
        MeshMaterial3d(assets.fx_core_materials[style].clone()),
        Transform::from_translation(position).with_scale(core_scale),
        PointLight {
            color: fx_color(style),
            intensity: light_intensity,
            range: light_range,
            radius: 0.25 * power,
            shadows_enabled: false,
            ..default()
        },
        CombatFx {
            age: 0.0,
            duration: 0.16 + power * 0.03,
            start_scale: core_scale,
            end_scale: end_core,
            peak_light: light_intensity,
        },
        RunEntity,
    ));

    let ring_scale = Vec3::splat(0.22 * power);
    let end_ring = Vec3::splat(3.6 * power);
    commands.spawn((
        Name::new("Energy impact ring"),
        Mesh3d(assets.ring.clone()),
        MeshMaterial3d(assets.fx_materials[style].clone()),
        Transform::from_translation(position + Vec3::Y * 0.08).with_scale(ring_scale),
        CombatFx {
            age: 0.0,
            duration: 0.22 + power * 0.04,
            start_scale: ring_scale,
            end_scale: end_ring,
            peak_light: 0.0,
        },
        RunEntity,
    ));

    let flare_scale = Vec3::splat(0.50 * power);
    let end_flare = Vec3::splat(5.5 * power);
    commands.spawn((
        Name::new("Energy impact radial flare"),
        Mesh3d(assets.glow_quad.clone()),
        MeshMaterial3d(assets.glow_materials[style].clone()),
        Transform {
            translation: position + Vec3::Y * 0.12,
            rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            scale: flare_scale,
        },
        CombatFx {
            age: 0.0,
            duration: 0.25 + power * 0.04,
            start_scale: flare_scale,
            end_scale: end_flare,
            peak_light: 0.0,
        },
        RunEntity,
    ));

    let spark_count = if power > 2.0 { 16 } else if power > 1.2 { 10 } else { 6 };
    for index in 0..spark_count {
        let angle = index as f32 * std::f32::consts::TAU / spark_count as f32;
        let direction = Vec3::new(angle.cos(), 0.36 + (index % 3) as f32 * 0.16, angle.sin())
            .normalize_or_zero();
        let start_scale = Vec3::new(0.65, 0.65, 1.0 + power * 0.55);
        commands.spawn((
            Name::new("Impact spark"),
            Mesh3d(assets.spark.clone()),
            MeshMaterial3d(assets.fx_materials[style].clone()),
            Transform {
                translation: position,
                rotation: Quat::from_rotation_arc(Vec3::Z, direction),
                scale: start_scale,
            },
            CombatFx {
                age: 0.0,
                duration: 0.22 + (index % 4) as f32 * 0.03,
                start_scale,
                end_scale: Vec3::new(0.02, 0.02, 0.16),
                peak_light: 0.0,
            },
            FxMotion {
                velocity: direction * (12.0 + power * 7.5 + (index % 3) as f32 * 2.5),
                drag: 2.8,
                gravity: 18.0,
            },
            RunEntity,
        ));
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_director_scales_enemy_pressure_and_boss_cadence() {
        assert_eq!(wave_spawn_total(1), 6);
        assert_eq!(wave_spawn_total(4), 13);
        assert!(!wave_is_boss(3));
        assert!(wave_is_boss(4));
        assert!(wave_clear_reward(5) > wave_clear_reward(2));
        assert_eq!(wave_enemy_cap(30), 28);
    }

    #[test]
    fn weapon_point_costs_are_bounded() {
        assert!(weapon_point_cost(WeaponKind::PlasmaMortar, 0).is_some());
        assert!(weapon_point_cost(WeaponKind::PulseRifle, 3).is_none());
    }

    #[test]
    fn every_weapon_mount_sits_near_the_right_hand() {
        for weapon in WeaponKind::ALL {
            let (mount, muzzle) = weapon_mount(weapon);
            assert!(mount.translation.x < 0.0);
            assert!(mount.translation.y < -0.5);
            assert!((0.25..0.55).contains(&muzzle));
            assert!((1.0..1.6).contains(&mount.scale.x));
        }
    }

    #[test]
    fn weapon_effect_styles_are_distinct() {
        assert_ne!(fx_style(DamageKind::Pulse), fx_style(DamageKind::Plasma));
        assert_ne!(fx_style(DamageKind::Plasma), fx_style(DamageKind::Ion));
        assert_ne!(fx_style(DamageKind::Ion), fx_style(DamageKind::Enemy));
    }

    #[test]
    fn fast_projectile_hits_the_first_point_of_a_target() {
        let hit = segment_sphere_hit_fraction(
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 20.0),
            Vec3::new(0.0, 0.0, 10.0),
            1.0,
        );
        assert_eq!(hit, Some(0.45));
    }

    #[test]
    fn plasma_ballistic_solution_reaches_a_level_target() {
        let target = Vec3::new(0.0, 0.0, 24.0);
        let velocity = ballistic_velocity(Vec3::ZERO, target, 38.0, 17.0);
        let flight_time = target.z / velocity.z;
        let height = velocity.y * flight_time - 0.5 * 17.0 * flight_time * flight_time;
        assert!(height.abs() < 0.01, "expected a level impact, got {height}");
    }

    #[test]
    fn projectile_visual_profiles_keep_each_weapon_readable() {
        let plasma = projectile_visual_profile(DamageKind::Plasma);
        let ion = projectile_visual_profile(DamageKind::Ion);
        let enemy = projectile_visual_profile(DamageKind::Enemy);
        assert!(plasma.0 > ion.0);
        assert!(ion.2 > plasma.2);
        assert!(plasma.3 > enemy.3);
        assert!(enemy.4 < plasma.4);
    }
}
