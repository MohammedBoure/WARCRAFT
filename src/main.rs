use bevy::prelude::*;

const ARENA_HALF_WIDTH: f32 = 590.0;
const ARENA_HALF_HEIGHT: f32 = 300.0;
const PLAYER_SPEED: f32 = 360.0;
const PLAYER_RADIUS: f32 = 18.0;
const PROJECTILE_SPEED: f32 = 760.0;

const INK: Color = Color::srgb(0.02, 0.03, 0.08);
const PANEL: Color = Color::srgba(0.035, 0.055, 0.13, 0.92);
const CYAN: Color = Color::srgb(0.20, 0.92, 1.0);
const MAGENTA: Color = Color::srgb(1.0, 0.24, 0.68);
const AMBER: Color = Color::srgb(1.0, 0.70, 0.22);
const MUTED: Color = Color::srgb(0.52, 0.63, 0.78);

fn main() {
    App::new()
        .insert_resource(ClearColor(INK))
        .insert_resource(GameState::default())
        .insert_resource(WaveState::default())
        .insert_resource(FireCooldown(Timer::from_seconds(
            0.16,
            TimerMode::Repeating,
        )))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "NEON // LAST STAND".into(),
                resolution: (1280, 720).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                restart_game,
                player_movement,
                update_player_aim,
                advance_wave,
                spawn_wave_enemies,
                fire_weapon,
                move_enemies,
                move_projectiles_and_resolve_hits,
                update_particles,
                update_hud,
            )
                .chain(),
        )
        .run();
}

#[derive(Resource)]
struct GameState {
    health: f32,
    max_health: f32,
    score: u32,
    kills: u32,
    game_over: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            score: 0,
            kills: 0,
            game_over: false,
        }
    }
}

#[derive(Resource)]
struct FireCooldown(Timer);

#[derive(Resource)]
struct WaveState {
    number: u32,
    remaining: u32,
    spawn_cursor: u32,
    phase: WavePhase,
    timer: Timer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WavePhase {
    Spawning,
    Clearing,
    Break,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            number: 1,
            remaining: 6,
            spawn_cursor: 0,
            phase: WavePhase::Spawning,
            timer: Timer::from_seconds(0.30, TimerMode::Once),
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy {
    health: f32,
    max_health: f32,
    speed: f32,
}

#[derive(Component)]
struct Projectile {
    velocity: Vec2,
    damage: f32,
}

#[derive(Component)]
struct Particle {
    velocity: Vec2,
    life: f32,
    max_life: f32,
}

#[derive(Component)]
struct HudStats;

#[derive(Component)]
struct HudWave;

#[derive(Component)]
struct HealthBarFill;

#[derive(Component)]
struct GameOverLabel;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    spawn_backdrop(&mut commands);
    spawn_player(&mut commands);
    spawn_hud(&mut commands);
}

fn spawn_backdrop(commands: &mut Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.015, 0.025, 0.07), Vec2::new(1280.0, 720.0)),
        Transform::from_xyz(0.0, 0.0, -20.0),
    ));

    for x in (-10..=10).map(|step| step as f32 * 55.0) {
        commands.spawn((
            Sprite::from_color(Color::srgba(0.12, 0.24, 0.42, 0.30), Vec2::new(1.0, 600.0)),
            Transform::from_xyz(x, 0.0, -18.0),
        ));
    }
    for y in (-5..=5).map(|step| step as f32 * 55.0) {
        commands.spawn((
            Sprite::from_color(Color::srgba(0.12, 0.24, 0.42, 0.30), Vec2::new(1180.0, 1.0)),
            Transform::from_xyz(0.0, y, -18.0),
        ));
    }

    let border = Color::srgba(0.18, 0.65, 0.90, 0.55);
    for (position, size) in [
        (
            Vec3::new(0.0, ARENA_HALF_HEIGHT, -15.0),
            Vec2::new(1180.0, 2.0),
        ),
        (
            Vec3::new(0.0, -ARENA_HALF_HEIGHT, -15.0),
            Vec2::new(1180.0, 2.0),
        ),
        (
            Vec3::new(ARENA_HALF_WIDTH, 0.0, -15.0),
            Vec2::new(2.0, 600.0),
        ),
        (
            Vec3::new(-ARENA_HALF_WIDTH, 0.0, -15.0),
            Vec2::new(2.0, 600.0),
        ),
    ] {
        commands.spawn((
            Sprite::from_color(border, size),
            Transform::from_translation(position),
        ));
    }

    for position in [
        Vec3::new(-ARENA_HALF_WIDTH + 10.0, ARENA_HALF_HEIGHT - 10.0, -14.0),
        Vec3::new(ARENA_HALF_WIDTH - 10.0, ARENA_HALF_HEIGHT - 10.0, -14.0),
        Vec3::new(-ARENA_HALF_WIDTH + 10.0, -ARENA_HALF_HEIGHT + 10.0, -14.0),
        Vec3::new(ARENA_HALF_WIDTH - 10.0, -ARENA_HALF_HEIGHT + 10.0, -14.0),
    ] {
        commands.spawn((
            Sprite::from_color(AMBER, Vec2::splat(7.0)),
            Transform::from_translation(position),
        ));
    }
}

fn spawn_player(commands: &mut Commands) {
    commands.spawn((
        Player,
        Sprite::from_color(CYAN, Vec2::new(30.0, 24.0)),
        Transform::from_xyz(0.0, -40.0, 5.0),
    ));
}

fn spawn_hud(commands: &mut Commands) {
    let title_font = TextFont {
        font_size: 22.0,
        ..default()
    };
    let body_font = TextFont {
        font_size: 16.0,
        ..default()
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(22.0),
                top: Val::Px(18.0),
                width: Val::Px(245.0),
                height: Val::Px(105.0),
                padding: UiRect::all(Val::Px(14.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(PANEL),
            BorderColor::all(Color::srgba(0.20, 0.65, 0.92, 0.55)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("NEON // LAST STAND"),
                title_font.clone(),
                TextColor(CYAN),
            ));
            parent.spawn((
                Text::new("WASD / ARROWS  ·  SPACE FIRE  ·  R RESTART"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(MUTED),
            ));
            parent.spawn((
                Text::new("HEALTH"),
                body_font.clone(),
                TextColor(Color::WHITE),
            ));
            parent
                .spawn((
                    Node {
                        width: Val::Px(215.0),
                        height: Val::Px(7.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.10, 0.14, 0.24, 0.95)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        HealthBarFill,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(CYAN),
                    ));
                });
        });

    commands.spawn((
        HudStats,
        Text::new("SCORE  000000\nKILLS  000"),
        body_font.clone(),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(26.0),
            top: Val::Px(24.0),
            ..default()
        },
    ));

    commands.spawn((
        HudWave,
        Text::new("WAVE  01\nINCOMING"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(AMBER),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(26.0),
            bottom: Val::Px(28.0),
            ..default()
        },
    ));

    commands.spawn((
        GameOverLabel,
        Visibility::Hidden,
        Text::new("SIGNAL LOST\n\nPRESS R TO REBOOT"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(MAGENTA),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            ..default()
        },
    ));
}

fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    if state.game_over {
        return;
    }
    let Ok(mut transform) = player.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }

    let movement = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_secs();
    transform.translation.x = (transform.translation.x + movement.x).clamp(
        -ARENA_HALF_WIDTH + PLAYER_RADIUS,
        ARENA_HALF_WIDTH - PLAYER_RADIUS,
    );
    transform.translation.y = (transform.translation.y + movement.y).clamp(
        -ARENA_HALF_HEIGHT + PLAYER_RADIUS,
        ARENA_HALF_HEIGHT - PLAYER_RADIUS,
    );
}

fn update_player_aim(
    enemies: Query<&Transform, (With<Enemy>, Without<Player>)>,
    mut player: Query<&mut Transform, (With<Player>, Without<Enemy>)>,
) {
    let Ok(mut player_transform) = player.single_mut() else {
        return;
    };
    let player_position = player_transform.translation.truncate();
    let target = enemies
        .iter()
        .min_by(|a, b| {
            player_position
                .distance_squared(a.translation.truncate())
                .partial_cmp(&player_position.distance_squared(b.translation.truncate()))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|transform| transform.translation.truncate());

    if let Some(target) = target {
        let direction = target - player_position;
        player_transform.rotation = Quat::from_rotation_z(direction.y.atan2(direction.x));
    }
}

fn advance_wave(
    time: Res<Time>,
    mut wave: ResMut<WaveState>,
    enemies: Query<(), With<Enemy>>,
    state: Res<GameState>,
) {
    if state.game_over {
        return;
    }

    match wave.phase {
        WavePhase::Spawning => {}
        WavePhase::Clearing if enemies.is_empty() => {
            wave.phase = WavePhase::Break;
            wave.timer = Timer::from_seconds(2.2, TimerMode::Once);
        }
        WavePhase::Clearing => {}
        WavePhase::Break => {
            wave.timer.tick(time.delta());
            if wave.timer.just_finished() {
                wave.number += 1;
                wave.remaining = 5 + wave.number * 2;
                wave.spawn_cursor = 0;
                wave.phase = WavePhase::Spawning;
                wave.timer = Timer::from_seconds(0.25, TimerMode::Once);
            }
        }
    }
}

fn spawn_wave_enemies(
    time: Res<Time>,
    mut wave: ResMut<WaveState>,
    state: Res<GameState>,
    mut commands: Commands,
) {
    if state.game_over || wave.phase != WavePhase::Spawning {
        return;
    }
    wave.timer.tick(time.delta());
    if !wave.timer.just_finished() {
        return;
    }

    if wave.remaining == 0 {
        wave.phase = WavePhase::Clearing;
        return;
    }

    let angle = wave.spawn_cursor as f32 * 2.399963;
    let edge = wave.spawn_cursor % 4;
    let position = match edge {
        0 => Vec2::new(angle.cos() * ARENA_HALF_WIDTH, ARENA_HALF_HEIGHT - 12.0),
        1 => Vec2::new(ARENA_HALF_WIDTH - 12.0, angle.sin() * ARENA_HALF_HEIGHT),
        2 => Vec2::new(angle.cos() * ARENA_HALF_WIDTH, -ARENA_HALF_HEIGHT + 12.0),
        _ => Vec2::new(-ARENA_HALF_WIDTH + 12.0, angle.sin() * ARENA_HALF_HEIGHT),
    };
    let health = 2.0 + wave.number as f32 * 0.65;
    let speed = 42.0 + wave.number as f32 * 5.0;
    let color = if wave.number % 3 == 0 { AMBER } else { MAGENTA };

    commands.spawn((
        Enemy {
            health,
            max_health: health,
            speed,
        },
        Sprite::from_color(color, Vec2::new(24.0, 24.0)),
        Transform::from_xyz(position.x, position.y, 3.0),
    ));
    wave.remaining -= 1;
    wave.spawn_cursor += 1;
    wave.timer = Timer::from_seconds(
        (0.55 - wave.number as f32 * 0.015).max(0.16),
        TimerMode::Once,
    );
}

fn fire_weapon(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    mut cooldown: ResMut<FireCooldown>,
    player: Query<&Transform, With<Player>>,
    enemies: Query<&Transform, With<Enemy>>,
    mut commands: Commands,
) {
    cooldown.0.tick(time.delta());
    if state.game_over || !keyboard.pressed(KeyCode::Space) || !cooldown.0.just_finished() {
        return;
    }
    let Ok(player_transform) = player.single() else {
        return;
    };
    let origin = player_transform.translation.truncate();
    let target = enemies
        .iter()
        .min_by(|a, b| {
            origin
                .distance_squared(a.translation.truncate())
                .partial_cmp(&origin.distance_squared(b.translation.truncate()))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|transform| transform.translation.truncate())
        .unwrap_or(origin + Vec2::X);
    let direction = (target - origin).normalize_or_zero();

    commands.spawn((
        Projectile {
            velocity: direction * PROJECTILE_SPEED,
            damage: 1.0,
        },
        Sprite::from_color(Color::srgb(0.72, 1.0, 0.98), Vec2::new(18.0, 4.0)),
        Transform::from_xyz(
            origin.x + direction.x * 23.0,
            origin.y + direction.y * 23.0,
            4.0,
        )
        .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
    ));
}

fn move_enemies(
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemies: Query<(Entity, &mut Transform, &Enemy), (With<Enemy>, Without<Player>)>,
    mut commands: Commands,
) {
    if game_state.game_over {
        return;
    }
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_position = player_transform.translation.truncate();
    for (entity, mut transform, enemy) in &mut enemies {
        let position = transform.translation.truncate();
        let direction = (player_position - position).normalize_or_zero();
        transform.translation += (direction * enemy.speed * time.delta_secs()).extend(0.0);
        transform.rotate_z(time.delta_secs() * 1.8);

        if position.distance(player_position) < PLAYER_RADIUS + 12.0 {
            game_state.health = (game_state.health - 16.0).max(0.0);
            spawn_burst(&mut commands, position, MAGENTA, 8);
            commands.entity(entity).despawn();
            if game_state.health <= 0.0 {
                game_state.game_over = true;
            }
        }
    }
}

fn move_projectiles_and_resolve_hits(
    time: Res<Time>,
    mut projectiles: Query<
        (Entity, &mut Transform, &Projectile),
        (With<Projectile>, Without<Enemy>),
    >,
    mut enemies: Query<
        (Entity, &mut Enemy, &mut Sprite, &Transform),
        (With<Enemy>, Without<Projectile>),
    >,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
) {
    if game_state.game_over {
        return;
    }
    for (projectile_entity, mut projectile_transform, projectile) in &mut projectiles {
        projectile_transform.translation += (projectile.velocity * time.delta_secs()).extend(0.0);
        let projectile_position = projectile_transform.translation.truncate();
        if projectile_position.x.abs() > ARENA_HALF_WIDTH + 60.0
            || projectile_position.y.abs() > ARENA_HALF_HEIGHT + 60.0
        {
            commands.entity(projectile_entity).despawn();
            continue;
        }

        for (enemy_entity, mut enemy, mut sprite, enemy_transform) in &mut enemies {
            if projectile_position.distance(enemy_transform.translation.truncate()) > 18.0 {
                continue;
            }
            enemy.health -= projectile.damage;
            commands.entity(projectile_entity).despawn();
            if enemy.health <= 0.0 {
                game_state.score += 100 + game_state.health as u32;
                game_state.kills += 1;
                spawn_burst(
                    &mut commands,
                    enemy_transform.translation.truncate(),
                    sprite.color,
                    14,
                );
                commands.entity(enemy_entity).despawn();
            } else {
                let health_ratio = (enemy.health / enemy.max_health).clamp(0.0, 1.0);
                sprite.color = MAGENTA.mix(&AMBER, 1.0 - health_ratio);
            }
            break;
        }
    }
}

fn spawn_burst(commands: &mut Commands, center: Vec2, color: Color, count: u32) {
    for index in 0..count {
        let angle = index as f32 * 2.399963;
        let speed = 50.0 + (index % 4) as f32 * 22.0;
        commands.spawn((
            Particle {
                velocity: Vec2::from_angle(angle) * speed,
                life: 0.55 + (index % 3) as f32 * 0.08,
                max_life: 0.55 + (index % 3) as f32 * 0.08,
            },
            Sprite::from_color(color, Vec2::splat(4.0 + (index % 3) as f32 * 2.0)),
            Transform::from_xyz(center.x, center.y, 6.0),
        ));
    }
}

fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut particle, mut transform, mut sprite) in &mut particles {
        particle.life -= time.delta_secs();
        transform.translation += (particle.velocity * time.delta_secs()).extend(0.0);
        particle.velocity *= 0.94_f32.powf(time.delta_secs() * 60.0);
        sprite.color = sprite
            .color
            .with_alpha((particle.life / particle.max_life).max(0.0));
        if particle.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn update_hud(
    state: Res<GameState>,
    wave: Res<WaveState>,
    enemies: Query<(), With<Enemy>>,
    mut stats: Query<&mut Text, (With<HudStats>, Without<HudWave>, Without<GameOverLabel>)>,
    mut wave_text: Query<&mut Text, (With<HudWave>, Without<HudStats>, Without<GameOverLabel>)>,
    mut health_bar: Query<&mut Node, With<HealthBarFill>>,
    mut game_over: Query<
        (&mut Visibility, &mut Text),
        (With<GameOverLabel>, Without<HudStats>, Without<HudWave>),
    >,
) {
    if let Ok(mut text) = stats.single_mut() {
        text.0 = format!("SCORE  {:06}\nKILLS  {:03}", state.score, state.kills);
    }
    if let Ok(mut text) = wave_text.single_mut() {
        let phase = match wave.phase {
            WavePhase::Spawning => format!("{} SIGNALS", wave.remaining),
            WavePhase::Clearing => format!("{} HOSTILES", enemies.iter().count()),
            WavePhase::Break => "SECTOR CLEAR".to_string(),
        };
        text.0 = format!("WAVE  {:02}\n{phase}", wave.number);
    }
    if let Ok(mut node) = health_bar.single_mut() {
        node.width = Val::Percent((state.health / state.max_health * 100.0).clamp(0.0, 100.0));
    }
    if let Ok((mut visibility, mut text)) = game_over.single_mut() {
        *visibility = if state.game_over {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if state.game_over {
            text.0 = format!("SIGNAL LOST\n\nSCORE {:06}\nPRESS R TO REBOOT", state.score);
        }
    }
}

fn restart_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<GameState>,
    mut wave: ResMut<WaveState>,
    mut cooldown: ResMut<FireCooldown>,
    mut player: Query<&mut Transform, With<Player>>,
    enemies: Query<Entity, With<Enemy>>,
    projectiles: Query<Entity, With<Projectile>>,
    particles: Query<Entity, With<Particle>>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) || !state.game_over {
        return;
    }
    for entity in enemies
        .iter()
        .chain(projectiles.iter())
        .chain(particles.iter())
    {
        commands.entity(entity).despawn();
    }
    state.health = state.max_health;
    state.score = 0;
    state.kills = 0;
    state.game_over = false;
    *wave = WaveState::default();
    cooldown.0.reset();
    if let Ok(mut transform) = player.single_mut() {
        transform.translation = Vec3::new(0.0, -40.0, 5.0);
        transform.rotation = Quat::IDENTITY;
    }
}
