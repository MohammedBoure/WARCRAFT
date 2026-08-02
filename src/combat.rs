use astra_voxel_world::prelude::*;
use bevy::prelude::*;

use crate::gameplay::{MissionTarget, MissionTargetKind, RelayDestroyed, RunEntity, finish_run};
use crate::interaction::{VoxelWorldEdits, block_world_center};
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
    pub shield: f32,
    speed: f32,
    attack: Timer,
}

#[derive(Component)]
pub struct Projectile {
    from_player: bool,
    velocity: Vec3,
    damage: f32,
    kind: DamageKind,
    life: Timer,
    radius: f32,
    target: Option<Entity>,
    area: f32,
}

#[derive(Resource)]
pub struct EnemyDirector {
    timer: Timer,
    counter: u32,
    boss_spawned: bool,
}
impl Default for EnemyDirector {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            counter: 0,
            boss_spawned: false,
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
    crawler: Handle<Mesh>,
    spitter: Handle<Mesh>,
    drone: Handle<Mesh>,
    brute: Handle<Mesh>,
    boss: Handle<Mesh>,
    shot: Handle<Mesh>,
    flash: Handle<Mesh>,
    beam: Handle<Mesh>,
    ring: Handle<Mesh>,
    spark: Handle<Mesh>,
    materials: Vec<Handle<StandardMaterial>>,
    fx_materials: Vec<Handle<StandardMaterial>>,
    enemy_scenes: Vec<Handle<Scene>>,
    weapon_scenes: Vec<Handle<Scene>>,
}

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
struct ProjectileTrail {
    last_position: Vec3,
    style: usize,
}

#[derive(Component)]
struct CombatFx {
    age: f32,
    duration: f32,
    start_scale: Vec3,
    end_scale: Vec3,
    peak_light: f32,
}

#[derive(Component)]
struct FxMotion {
    velocity: Vec3,
    drag: f32,
    gravity: f32,
}

pub fn setup_combat_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let colors = [
        (
            Color::srgb(0.36, 0.58, 0.12),
            LinearRgba::rgb(0.12, 0.48, 0.02),
        ),
        (
            Color::srgb(0.54, 0.12, 0.50),
            LinearRgba::rgb(0.9, 0.04, 0.72),
        ),
        (
            Color::srgb(0.12, 0.34, 0.62),
            LinearRgba::rgb(0.04, 0.72, 1.8),
        ),
        (
            Color::srgb(0.42, 0.11, 0.08),
            LinearRgba::rgb(1.2, 0.08, 0.02),
        ),
        (
            Color::srgb(0.28, 0.04, 0.46),
            LinearRgba::rgb(2.8, 0.08, 4.2),
        ),
        (
            Color::srgb(0.08, 0.86, 1.0),
            LinearRgba::rgb(0.05, 2.8, 4.0),
        ),
        (
            Color::srgb(0.92, 0.18, 0.76),
            LinearRgba::rgb(4.0, 0.08, 2.4),
        ),
        (
            Color::srgb(0.28, 0.48, 1.0),
            LinearRgba::rgb(0.18, 1.2, 4.5),
        ),
        (
            Color::srgb(1.0, 0.14, 0.08),
            LinearRgba::rgb(4.0, 0.05, 0.02),
        ),
    ];
    let materials = colors
        .into_iter()
        .map(|(base_color, emissive)| {
            materials.add(StandardMaterial {
                base_color,
                emissive,
                metallic: 0.32,
                perceptual_roughness: 0.28,
                ..default()
            })
        })
        .collect();
    let fx_materials = [
        (
            Color::srgba(0.22, 0.96, 1.0, 0.88),
            LinearRgba::rgb(4.0, 13.0, 18.0),
        ),
        (
            Color::srgba(1.0, 0.18, 0.78, 0.86),
            LinearRgba::rgb(18.0, 1.4, 11.0),
        ),
        (
            Color::srgba(0.38, 0.56, 1.0, 0.90),
            LinearRgba::rgb(4.0, 8.0, 22.0),
        ),
        (
            Color::srgba(1.0, 0.16, 0.06, 0.86),
            LinearRgba::rgb(18.0, 1.2, 0.25),
        ),
    ]
    .map(|(base_color, emissive)| {
        materials.add(StandardMaterial {
            base_color,
            emissive,
            unlit: true,
            alpha_mode: AlphaMode::Add,
            ..default()
        })
    })
    .to_vec();

    commands.insert_resource(CombatAssets {
        crawler: meshes.add(Cuboid::new(4.4, 2.1, 5.2)),
        spitter: meshes.add(Sphere::new(2.6).mesh().ico(2).expect("sphere")),
        drone: meshes.add(Torus::new(1.2, 3.0)),
        brute: meshes.add(Capsule3d::new(3.2, 6.5)),
        boss: meshes.add(Sphere::new(8.5).mesh().ico(3).expect("sphere")),
        shot: meshes.add(Sphere::new(0.45).mesh().ico(1).expect("shot")),
        flash: meshes.add(Sphere::new(0.55).mesh().ico(2).expect("flash")),
        beam: meshes.add(Cuboid::new(0.14, 0.14, 1.0)),
        ring: meshes.add(Torus::new(0.55, 0.78)),
        spark: meshes.add(Cuboid::new(0.08, 0.08, 0.62)),
        materials,
        fx_materials,
        enemy_scenes: [
            "models/kenney-space/alien.glb",
            "models/kenney-space/astronautB.glb",
            "models/kenney-space/craft_speederA.glb",
            "models/kenney-space/meteor_detailed.glb",
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
    if let Some(recipe) = weapon_recipe(weapon, level)
        && session.loadout.spend(recipe)
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
    targets: Query<(&GlobalTransform, &CombatTarget)>,
    player: Query<&Transform, With<PlayerTag>>,
    muzzles: Query<(&GlobalTransform, &WeaponMuzzle)>,
    mut rigs: Query<&mut WeaponRig>,
    mut runtime: ResMut<CombatRuntime>,
    mut session: ResMut<GameSession>,
    mut damage: MessageWriter<DamageEvent>,
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
        targets.get(entity).ok().and_then(|(transform, target)| {
            target
                .targetable
                .then_some((entity, transform.translation(), target.aerial))
        })
    });

    match weapon {
        WeaponKind::PulseRifle => {
            let end = target
                .map(|(_, point, _)| point)
                .or(aim.world_point)
                .unwrap_or(origin + mount_direction * 64.0);
            let direction = (end - origin).normalize_or_zero();
            if let Some((entity, _, _)) = target {
                damage.write(DamageEvent {
                    target: DamageTarget::Enemy(entity),
                    amount: 14.0 + level as f32 * 4.0,
                    kind: DamageKind::Pulse,
                });
            }
            spawn_muzzle_fx(&mut commands, &assets, origin, direction, 0, 1.0);
            spawn_tracer_fx(&mut commands, &assets, origin, end, 0, 0.72, 0.085);
            spawn_impact_fx(&mut commands, &assets, end, 0, 0.72);
            apply_weapon_recoil(&mut rigs, weapon, 0.42);
            camera.shake = camera.shake.max(0.075);
            runtime.cooldown = 0.14;
            session.loadout.heat += 0.105;
            sounds.write(GameSound::PulseShot);
        }
        WeaponKind::PlasmaMortar => {
            let point = target
                .filter(|(_, _, aerial)| !aerial)
                .map(|(_, point, _)| point)
                .or(aim.world_point)
                .unwrap_or(origin + mount_direction * 42.0);
            let velocity = (point - origin).normalize_or_zero() * 31.0 + Vec3::Y * 5.5;
            spawn_shot(
                &mut commands,
                &assets,
                origin,
                velocity,
                6,
                Projectile {
                    from_player: true,
                    velocity,
                    damage: 30.0 + level as f32 * 8.0,
                    kind: DamageKind::Plasma,
                    life: Timer::from_seconds(2.4, TimerMode::Once),
                    radius: 1.0,
                    target: None,
                    area: 7.5,
                },
            );
            apply_weapon_recoil(&mut rigs, weapon, 0.92);
            camera.shake = camera.shake.max(0.22);
            runtime.cooldown = 0.78;
            session.loadout.heat += 0.27;
            sounds.write(GameSound::PlasmaShot);
        }
        WeaponKind::IonLance => {
            let Some((entity, point, true)) = target else {
                return;
            };
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
                    damage: 24.0 + level as f32 * 8.0,
                    kind: DamageKind::Ion,
                    life: Timer::from_seconds(3.0, TimerMode::Once),
                    radius: 1.4,
                    target: Some(entity),
                    area: 0.0,
                },
            );
            apply_weapon_recoil(&mut rigs, weapon, 0.68);
            camera.shake = camera.shake.max(0.14);
            runtime.cooldown = 0.48;
            session.loadout.heat += 0.20;
            sounds.write(GameSound::IonShot);
        }
    }
}
fn weapon_recipe(weapon: WeaponKind, level: u8) -> Option<&'static [(ResourceKind, u16)]> {
    match (weapon, level) {
        (WeaponKind::PulseRifle, 1) => {
            Some(&[(ResourceKind::SpaceIron, 4), (ResourceKind::Titanium, 2)])
        }
        (WeaponKind::PulseRifle, 2) => {
            Some(&[(ResourceKind::SpaceIron, 6), (ResourceKind::Helium3, 2)])
        }
        (WeaponKind::PlasmaMortar, 0) => {
            Some(&[(ResourceKind::Titanium, 5), (ResourceKind::BioPlasma, 3)])
        }
        (WeaponKind::PlasmaMortar, 1) => {
            Some(&[(ResourceKind::SpaceIron, 3), (ResourceKind::BioPlasma, 3)])
        }
        (WeaponKind::PlasmaMortar, 2) => {
            Some(&[(ResourceKind::Titanium, 5), (ResourceKind::BioPlasma, 5)])
        }
        (WeaponKind::IonLance, 0) => {
            Some(&[(ResourceKind::Titanium, 4), (ResourceKind::Helium3, 3)])
        }
        (WeaponKind::IonLance, 1) => {
            Some(&[(ResourceKind::SpaceIron, 3), (ResourceKind::Helium3, 3)])
        }
        (WeaponKind::IonLance, 2) => {
            Some(&[(ResourceKind::Titanium, 5), (ResourceKind::Helium3, 5)])
        }
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
    mut commands: Commands,
) {
    let count = enemies.iter().count();
    session.active_enemies = count;
    if !matches!(
        session.phase,
        MissionPhase::HomeDefense
            | MissionPhase::RelayHunt
            | MissionPhase::Extraction
            | MissionPhase::GateAssault
    ) {
        return;
    }
    if session.phase == MissionPhase::GateAssault && !director.boss_spawned {
        let point = player
            .single()
            .map(|p| p.translation + Vec3::new(0.0, 24.0, -45.0))
            .unwrap_or(Vec3::Y * 35.0);
        spawn_enemy(&mut commands, &assets, EnemyKind::CarrierBoss, point, 1.0);
        director.boss_spawned = true;
    }
    let cap = if session.route == PlanetRoute::HomeDefense {
        10
    } else {
        24
    };
    if count >= cap || !director.timer.tick(time.delta()).just_finished() {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    director.counter = director.counter.wrapping_add(1);
    let kind = choose_enemy(&session, director.counter);
    let angle = director.counter as f32 * 2.399_963;
    let radius = 42.0 + (director.counter % 5) as f32 * 5.0;
    let x = player.translation.x + angle.cos() * radius;
    let z = player.translation.z + angle.sin() * radius;
    let y = if kind.is_aerial() {
        player.translation.y + 17.0
    } else {
        ground_height(&loaded, &world, x, z) + 2.4
    };
    let difficulty = if session.route == PlanetRoute::HomeDefense {
        0.65
    } else {
        1.0 + session.relays_destroyed as f32 * 0.12
    };
    spawn_enemy(&mut commands, &assets, kind, Vec3::new(x, y, z), difficulty);
    let interval = match session.phase {
        MissionPhase::HomeDefense => 6.2 - session.wave as f32 * 0.85,
        MissionPhase::Extraction => 1.8,
        MissionPhase::GateAssault => 2.5,
        _ => 3.8 - session.relays_destroyed as f32 * 0.55,
    }
    .max(1.25);
    director.timer = Timer::from_seconds(interval, TimerMode::Once);
}

fn choose_enemy(session: &GameSession, n: u32) -> EnemyKind {
    if session.route == PlanetRoute::HomeDefense {
        if session.wave >= 3 && n % 4 == 0 {
            EnemyKind::Drone
        } else if session.wave >= 2 && n % 3 == 0 {
            EnemyKind::Spitter
        } else {
            EnemyKind::Crawler
        }
    } else if session.relays_destroyed == 0 {
        if n % 4 == 0 {
            EnemyKind::Spitter
        } else {
            EnemyKind::Crawler
        }
    } else if session.relays_destroyed == 1 {
        if n % 3 == 0 {
            EnemyKind::Drone
        } else {
            EnemyKind::Spitter
        }
    } else if n % 5 == 0 {
        EnemyKind::Brute
    } else if n % 2 == 0 {
        EnemyKind::Drone
    } else {
        EnemyKind::Crawler
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    assets: &CombatAssets,
    kind: EnemyKind,
    position: Vec3,
    difficulty: f32,
) {
    let (mesh, material, health, shield, speed, radius, attack) = match kind {
        EnemyKind::Crawler => (&assets.crawler, 0, 42.0, 0.0, 12.5, 3.0, 0.72),
        EnemyKind::Spitter => (&assets.spitter, 1, 62.0, 0.0, 7.2, 3.2, 1.55),
        EnemyKind::Drone => (&assets.drone, 2, 54.0, 0.0, 10.5, 3.5, 1.35),
        EnemyKind::Brute => (&assets.brute, 3, 190.0, 0.0, 4.6, 4.5, 1.10),
        EnemyKind::CarrierBoss => (&assets.boss, 4, 760.0, 320.0, 6.5, 9.5, 0.85),
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
    commands
        .spawn((
            Name::new(format!("Alien {kind:?}")),
            Transform::from_translation(position),
            Enemy {
                kind,
                health: health * difficulty,
                shield: shield * difficulty,
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
                SceneRoot(assets.enemy_scenes[scene_index].clone()),
                Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(scene_scale)),
            ));
        });
}

pub fn update_enemy_ai(
    time: Res<Time>,
    world: Res<VoxelViewerWorld>,
    loaded: Res<LoadedVoxelChunks>,
    assets: Res<CombatAssets>,
    session: Res<GameSession>,
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
        let objective = if session.route == PlanetRoute::HomeDefense {
            objectives
                .iter()
                .find(|(_, _, t)| t.kind == MissionTargetKind::HomeCore)
        } else if session.phase == MissionPhase::Extraction {
            objectives
                .iter()
                .find(|(_, _, t)| t.kind == MissionTargetKind::Ship)
        } else {
            None
        };
        let (target, damage_target) = objective
            .map(|(e, t, _)| (t.translation, DamageTarget::Enemy(e)))
            .unwrap_or((player.translation, DamageTarget::Player));
        let offset = target - transform.translation;
        let flat = Vec3::new(offset.x, 0.0, offset.z);
        let distance = offset.length();
        if enemy.kind == EnemyKind::CarrierBoss && enemy.shield <= 0.0 {
            target_data.aerial = false;
        }
        if target_data.aerial {
            let height = if enemy.kind == EnemyKind::CarrierBoss && enemy.shield <= 0.0 {
                target.y + 5.0
            } else {
                target.y + 15.0
            };
            let desired = flat.normalize_or_zero() * enemy.speed
                + Vec3::Y * (height - transform.translation.y) * 1.4;
            transform.translation += desired.clamp_length_max(enemy.speed * 1.6) * dt;
        } else if distance > melee_range(enemy.kind) {
            let candidate = transform.translation + flat.normalize_or_zero() * enemy.speed * dt;
            let ground = ground_height(&loaded, &world, candidate.x, candidate.z);
            if (ground + 2.4 - transform.translation.y).abs() <= 4.0 {
                transform.translation = Vec3::new(candidate.x, ground + 2.2, candidate.z);
            }
        }
        if flat.length_squared() > 0.2 {
            let rotation = Quat::from_rotation_y((-flat.x).atan2(-flat.z));
            transform.rotation = transform.rotation.slerp(rotation, 1.0 - (-8.0 * dt).exp());
        }
        enemy.attack.tick(time.delta());
        if !enemy.attack.just_finished() {
            continue;
        }
        match enemy.kind {
            EnemyKind::Crawler if distance <= 5.0 => {
                damage.write(DamageEvent {
                    target: damage_target,
                    amount: 8.0,
                    kind: DamageKind::Enemy,
                });
            }
            EnemyKind::Brute if distance <= 7.0 => {
                damage.write(DamageEvent {
                    target: damage_target,
                    amount: 20.0,
                    kind: DamageKind::Enemy,
                });
            }
            EnemyKind::Spitter | EnemyKind::Drone | EnemyKind::CarrierBoss if distance <= 62.0 => {
                let velocity =
                    (target + Vec3::Y - transform.translation).normalize_or_zero() * 24.0;
                spawn_shot(
                    &mut commands,
                    &assets,
                    transform.translation,
                    velocity,
                    8,
                    Projectile {
                        from_player: false,
                        velocity,
                        damage: if enemy.kind == EnemyKind::CarrierBoss {
                            16.0
                        } else {
                            9.0
                        },
                        kind: DamageKind::Enemy,
                        life: Timer::from_seconds(3.2, TimerMode::Once),
                        radius: 1.2,
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
    time: Res<Time>,
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
    mut damage: MessageWriter<DamageEvent>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut shot, mut trail) in &mut shots {
        shot.life.tick(time.delta());
        if shot.life.just_finished() {
            if shot.from_player && shot.area > 0.0 {
                area_damage(transform.translation, &shot, &targets, &mut damage);
            }
            spawn_impact_fx(
                &mut commands,
                &assets,
                transform.translation,
                trail.style,
                if shot.area > 0.0 { 1.8 } else { 0.55 },
            );
            commands.entity(entity).despawn();
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
                .lerp(desired, 1.0 - (-7.0 * time.delta_secs()).exp());
        }
        if shot.kind == DamageKind::Plasma {
            shot.velocity.y -= 14.0 * time.delta_secs();
        }
        transform.translation += shot.velocity * time.delta_secs();
        if trail.last_position.distance_squared(transform.translation) > 0.10 {
            spawn_tracer_fx(
                &mut commands,
                &assets,
                trail.last_position,
                transform.translation,
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
            trail.last_position = transform.translation;
        }
        let hit = if shot.from_player {
            targets
                .iter()
                .find(|(_, target_transform, data)| {
                    data.targetable
                        && target_transform
                            .translation()
                            .distance(transform.translation)
                            <= data.radius + shot.radius
                })
                .map(|(entity, _, _)| DamageTarget::Enemy(entity))
        } else {
            player
                .single()
                .ok()
                .filter(|player| player.translation.distance(transform.translation) <= 3.4)
                .map(|_| DamageTarget::Player)
        };
        if let Some(target) = hit {
            if shot.area > 0.0 {
                area_damage(transform.translation, &shot, &targets, &mut damage);
            } else {
                damage.write(DamageEvent {
                    target,
                    amount: shot.damage,
                    kind: shot.kind,
                });
            }
            spawn_impact_fx(
                &mut commands,
                &assets,
                transform.translation,
                trail.style,
                if shot.area > 0.0 { 2.1 } else { 0.85 },
            );
            commands.entity(entity).despawn();
        }
    }
}
fn area_damage(
    center: Vec3,
    shot: &Projectile,
    targets: &Query<(Entity, &GlobalTransform, &CombatTarget)>,
    damage: &mut MessageWriter<DamageEvent>,
) {
    for (entity, transform, target) in targets {
        if !target.aerial && transform.translation().distance(center) <= shot.area {
            damage.write(DamageEvent {
                target: DamageTarget::Enemy(entity),
                amount: shot.damage,
                kind: shot.kind,
            });
        }
    }
}

pub fn process_damage(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    mut runtime: ResMut<CombatRuntime>,
    mut session: ResMut<GameSession>,
    mut player: Query<&mut Transform, With<PlayerTag>>,
    mut enemies: Query<(&mut Enemy, &mut CombatTarget)>,
    mut objectives: Query<&mut MissionTarget>,
    mut events: MessageReader<DamageEvent>,
    mut relays: MessageWriter<RelayDestroyed>,
    mut resources: MessageWriter<ResourceCollected>,
    mut sounds: MessageWriter<GameSound>,
    mut finished: MessageWriter<RunFinished>,
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
                sounds.write(GameSound::PlayerHit);
                if session.loadout.health <= 0.0 {
                    if session.phase == MissionPhase::GateAssault {
                        finish_run(
                            &mut session,
                            RunOutcome::MissionFailed,
                            &mut finished,
                            &mut next_state,
                        );
                    } else {
                        session
                            .loadout
                            .lose_raw_resources(balance.respawn_resource_loss);
                        session.loadout.health = 100.0;
                        session.loadout.shield = 50.0;
                        if let Ok(mut player) = player.single_mut() {
                            player.translation = session.safe_position + Vec3::Y;
                        }
                    }
                }
            }
            DamageTarget::Enemy(entity) => {
                if let Ok((mut enemy, mut target)) = enemies.get_mut(entity) {
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
                        }
                    } else {
                        enemy.health -= event.amount;
                    }
                    if enemy.health <= 0.0 {
                        let kind = enemy.kind;
                        commands.entity(entity).despawn();
                        session.loadout.kills += 1;
                        let drop = match kind {
                            EnemyKind::Crawler | EnemyKind::Spitter => ResourceKind::SpaceIron,
                            EnemyKind::Drone => ResourceKind::Helium3,
                            EnemyKind::Brute => ResourceKind::Titanium,
                            EnemyKind::CarrierBoss => ResourceKind::BioPlasma,
                        };
                        session.loadout.add_resource(drop, 1);
                        resources.write(ResourceCollected(drop, 1));
                        sounds.write(GameSound::EnemyDeath);
                        if kind == EnemyKind::CarrierBoss {
                            finish_run(
                                &mut session,
                                RunOutcome::GateDestroyed,
                                &mut finished,
                                &mut next_state,
                            );
                        }
                    } else {
                        sounds.write(GameSound::EnemyHit);
                    }
                } else if let Ok(mut objective) = objectives.get_mut(entity) {
                    objective.health -= event.amount;
                    match objective.kind {
                        MissionTargetKind::Relay(index) if objective.health <= 0.0 => {
                            commands.entity(entity).despawn();
                            relays.write(RelayDestroyed(index));
                        }
                        MissionTargetKind::HomeCore | MissionTargetKind::Ship => {
                            session.base_health = objective.health.max(0.0);
                            if objective.health <= 0.0 {
                                finish_run(
                                    &mut session,
                                    RunOutcome::MissionFailed,
                                    &mut finished,
                                    &mut next_state,
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            DamageTarget::Base => {}
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
    spawn_muzzle_fx(
        commands,
        assets,
        origin,
        direction,
        style,
        if shot.from_player { 1.25 } else { 0.72 },
    );
    commands.spawn((
        Mesh3d(assets.shot.clone()),
        MeshMaterial3d(assets.materials[material].clone()),
        Transform::from_translation(origin).with_scale(Vec3::splat(
            if shot.kind == DamageKind::Plasma {
                1.75
            } else {
                1.0
            },
        )),
        ProjectileTrail {
            last_position: origin,
            style,
        },
        shot,
        RunEntity,
    ));
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
            commands.entity(entity).despawn();
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
        commands.entity(entity).despawn();
    }
    let Some(weapon) = desired else {
        return;
    };
    let scene = match weapon {
        WeaponKind::PulseRifle => 0,
        WeaponKind::PlasmaMortar => 1,
        WeaponKind::IonLance => 2,
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
                translation: Vec3::new(-0.20, -0.71, 0.20),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.10, 0.0, 0.0),
                scale: Vec3::splat(1.10),
            },
            0.47,
        ),
        WeaponKind::PlasmaMortar => (
            Transform {
                translation: Vec3::new(-0.20, -0.82, 0.16),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.08, 0.0, 0.0),
                scale: Vec3::splat(1.45),
            },
            0.29,
        ),
        WeaponKind::IonLance => (
            Transform {
                translation: Vec3::new(-0.20, -0.82, 0.18),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.06, 0.0, 0.0),
                scale: Vec3::splat(1.24),
            },
            0.38,
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
    mut effects: Query<(
        Entity,
        &mut Transform,
        &mut CombatFx,
        Option<&mut FxMotion>,
        Option<&mut PointLight>,
    )>,
    mut commands: Commands,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut transform, mut effect, motion, light) in &mut effects {
        effect.age += dt;
        let progress = (effect.age / effect.duration.max(0.001)).clamp(0.0, 1.0);
        if progress >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let eased = 1.0 - (1.0 - progress).powi(2);
        transform.scale = effect.start_scale.lerp(effect.end_scale, eased);
        if let Some(mut motion) = motion {
            motion.velocity.y -= motion.gravity * dt;
            motion.velocity *= (-motion.drag * dt).exp();
            transform.translation += motion.velocity * dt;
            let direction = motion.velocity.normalize_or_zero();
            if direction != Vec3::ZERO {
                transform.rotation = Quat::from_rotation_arc(Vec3::Z, direction);
            }
        }
        if let Some(mut light) = light {
            light.intensity = effect.peak_light * (1.0 - progress).powi(2);
        }
    }
}

fn fx_style(kind: DamageKind) -> usize {
    match kind {
        DamageKind::Pulse => 0,
        DamageKind::Plasma => 1,
        DamageKind::Ion => 2,
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
    let direction = direction.normalize_or_zero();
    let start_scale = Vec3::splat(0.24 * power);
    commands.spawn((
        Name::new("Muzzle flash"),
        Mesh3d(assets.flash.clone()),
        MeshMaterial3d(assets.fx_materials[style].clone()),
        Transform::from_translation(origin).with_scale(start_scale),
        PointLight {
            color: fx_color(style),
            intensity: 18_000.0 * power,
            range: 16.0 * power,
            radius: 0.25,
            shadows_enabled: false,
            ..default()
        },
        CombatFx {
            age: 0.0,
            duration: 0.095,
            start_scale,
            end_scale: Vec3::splat(1.42 * power),
            peak_light: 18_000.0 * power,
        },
        RunEntity,
    ));
    if direction != Vec3::ZERO {
        spawn_tracer_fx(
            commands,
            assets,
            origin,
            origin + direction * (1.7 * power),
            style,
            1.8 * power,
            0.065,
        );
    }
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
    let delta = end - start;
    let distance = delta.length();
    if distance <= 0.01 {
        return;
    }
    let mut transform = Transform::from_translation(start + delta * 0.5);
    transform.look_at(end, Vec3::Y);
    let start_scale = Vec3::new(thickness, thickness, distance);
    transform.scale = start_scale;
    commands.spawn((
        Name::new("Energy tracer"),
        Mesh3d(assets.beam.clone()),
        MeshMaterial3d(assets.fx_materials[style].clone()),
        transform,
        CombatFx {
            age: 0.0,
            duration: duration.max(0.02),
            start_scale,
            end_scale: Vec3::new(0.03, 0.03, distance * 0.94),
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
    let core_scale = Vec3::splat(0.18 * power);
    commands.spawn((
        Name::new("Energy impact core"),
        Mesh3d(assets.flash.clone()),
        MeshMaterial3d(assets.fx_materials[style].clone()),
        Transform::from_translation(position).with_scale(core_scale),
        PointLight {
            color: fx_color(style),
            intensity: 22_000.0 * power,
            range: 18.0 * power,
            radius: 0.35,
            shadows_enabled: false,
            ..default()
        },
        CombatFx {
            age: 0.0,
            duration: 0.18 + power * 0.035,
            start_scale: core_scale,
            end_scale: Vec3::splat(1.65 * power),
            peak_light: 22_000.0 * power,
        },
        RunEntity,
    ));
    let ring_scale = Vec3::splat(0.22 * power);
    commands.spawn((
        Name::new("Energy impact ring"),
        Mesh3d(assets.ring.clone()),
        MeshMaterial3d(assets.fx_materials[style].clone()),
        Transform::from_translation(position + Vec3::Y * 0.08).with_scale(ring_scale),
        CombatFx {
            age: 0.0,
            duration: 0.24 + power * 0.045,
            start_scale: ring_scale,
            end_scale: Vec3::splat(2.8 * power),
            peak_light: 0.0,
        },
        RunEntity,
    ));
    let spark_count = if power > 1.2 { 10 } else { 6 };
    for index in 0..spark_count {
        let angle = index as f32 * std::f32::consts::TAU / spark_count as f32;
        let direction = Vec3::new(angle.cos(), 0.36 + (index % 3) as f32 * 0.16, angle.sin())
            .normalize_or_zero();
        let start_scale = Vec3::new(0.65, 0.65, 1.0 + power * 0.45);
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
                duration: 0.20 + (index % 4) as f32 * 0.025,
                start_scale,
                end_scale: Vec3::new(0.02, 0.02, 0.16),
                peak_light: 0.0,
            },
            FxMotion {
                velocity: direction * (10.0 + power * 6.0 + (index % 3) as f32 * 2.2),
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
    fn recipes_are_bounded() {
        assert!(weapon_recipe(WeaponKind::PlasmaMortar, 0).is_some());
        assert!(weapon_recipe(WeaponKind::PulseRifle, 3).is_none());
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
}
