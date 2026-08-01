use bevy::{core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom, prelude::*};

const GRID_RADIUS_X: i32 = 6;
const GRID_RADIUS_Z: i32 = 5;
const GRID_STEP: f32 = 1.78;
const PLAYER_SPEED: f32 = 5.6;
const INTERACTION_RANGE: f32 = 2.45;
const CRITICAL_TIME_SECONDS: f32 = 600.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.002, 0.006, 0.015)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.14, 0.19, 0.28),
            brightness: 42.0,
            ..default()
        })
        .insert_resource(GameClock::default())
        .insert_resource(AssistantLog::default())
        .insert_resource(NearbyNode::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GRID // CENTRAL OS".into(),
                resolution: (1440, 900).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GamePhase>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                tick_clock,
                player_movement,
                follow_camera.after(player_movement),
                update_nearby_node.after(player_movement),
                node_interaction.after(update_nearby_node),
                route_power_system.after(node_interaction),
                thermal_dynamics_system.after(route_power_system),
                create_periodic_fault.after(thermal_dynamics_system),
                critical_surge_trigger,
                platform_interaction.after(player_movement),
                update_energy_visuals,
                crisis_pulse,
                update_assistant_ui,
            ),
        )
        .add_systems(OnEnter(GamePhase::CriticalSurge), enter_critical_surge)
        .add_systems(OnEnter(GamePhase::AftermathA), resolve_external_discharge)
        .add_systems(OnEnter(GamePhase::AftermathB), resolve_reactor_meltdown)
        .run();
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum GamePhase {
    #[default]
    StableOperation,
    CriticalSurge,
    AftermathA,
    AftermathB,
}

#[derive(Resource)]
struct GameClock {
    elapsed: f32,
    next_fault_at: f32,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            next_fault_at: 60.0,
        }
    }
}

#[derive(Resource)]
struct AssistantLog {
    message: String,
    glitch_level: f32,
}

impl Default for AssistantLog {
    fn default() -> Self {
        Self {
            message: "[NOTICE] Sector B reports a capacitor leak. Approach the red node and press E to scan it.\n\nA damaged part cannot be repaired directly: rotate a neighbouring relay with Q to open a bypass route.".into(),
            glitch_level: 0.0,
        }
    }
}

#[derive(Resource, Default)]
struct NearbyNode(Option<Entity>);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct EnergyNode {
    id: usize,
    voltage: f32,
    temperature: f32,
    is_blown: bool,
    is_bypassed: bool,
    routing: u8,
    capacity: f32,
    connections: Vec<usize>,
}

#[derive(Component)]
struct EnergyPath {
    from: usize,
    to: usize,
}

#[derive(Component)]
struct EnergyLight;

#[derive(Component)]
struct AssistantText;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct ControlPlatform {
    ending: Ending,
}

#[derive(Clone, Copy)]
enum Ending {
    ExternalDischarge,
    ReactorMeltdown,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.002, 0.006, 0.015)),
            ..default()
        },
        Projection::from(OrthographicProjection {
            scale: 0.72,
            ..OrthographicProjection::default_3d()
        }),
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        Transform::from_xyz(13.0, 17.5, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
    ));

    spawn_grid(&mut commands, &mut meshes, &mut materials);
    spawn_energy_network(&mut commands, &mut meshes, &mut materials);
    spawn_player(&mut commands, &mut meshes, &mut materials);
    spawn_interface(&mut commands);
}

fn spawn_grid(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let tile_mesh = meshes.add(Cuboid::new(1.56, 0.18, 1.56));
    let trim_mesh = meshes.add(Cuboid::new(1.6, 0.045, 0.045));
    let tile_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.025, 0.045, 0.075),
        metallic: 0.88,
        perceptual_roughness: 0.32,
        ..default()
    });
    let trim_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.25, 0.32),
        emissive: LinearRgba::rgb(0.0, 0.17, 0.24),
        metallic: 0.72,
        perceptual_roughness: 0.26,
        ..default()
    });

    for x in -GRID_RADIUS_X..=GRID_RADIUS_X {
        for z in -GRID_RADIUS_Z..=GRID_RADIUS_Z {
            let position = Vec3::new(x as f32 * GRID_STEP, -0.12, z as f32 * GRID_STEP);
            commands.spawn((
                Mesh3d(tile_mesh.clone()),
                MeshMaterial3d(tile_material.clone()),
                Transform::from_translation(position),
            ));
            if (x + z) % 2 == 0 {
                commands.spawn((
                    Mesh3d(trim_mesh.clone()),
                    MeshMaterial3d(trim_material.clone()),
                    Transform::from_translation(position + Vec3::new(0.0, 0.005, 0.72)),
                ));
            }
        }
    }

    let border_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.25, 0.32),
        emissive: LinearRgba::rgb(0.0, 1.5, 2.5),
        metallic: 0.6,
        perceptual_roughness: 0.2,
        ..default()
    });
    let border_mesh = meshes.add(Cuboid::new(
        0.1,
        0.08,
        GRID_RADIUS_Z as f32 * GRID_STEP * 2.0,
    ));
    for x in [
        -GRID_RADIUS_X as f32 * GRID_STEP - 0.9,
        GRID_RADIUS_X as f32 * GRID_STEP + 0.9,
    ] {
        commands.spawn((
            Mesh3d(border_mesh.clone()),
            MeshMaterial3d(border_material.clone()),
            Transform::from_xyz(x, 0.0, 0.0),
        ));
    }
}

fn spawn_energy_network(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let positions = [
        Vec3::new(-7.1, 0.0, -4.8),
        Vec3::new(-2.4, 0.0, -4.8),
        Vec3::new(2.4, 0.0, -4.8),
        Vec3::new(7.1, 0.0, -4.8),
        Vec3::new(-7.1, 0.0, 1.4),
        Vec3::new(-2.4, 0.0, 1.4),
        Vec3::new(2.4, 0.0, 1.4),
        Vec3::new(7.1, 0.0, 1.4),
        Vec3::new(-2.4, 0.0, 6.0),
        Vec3::new(2.4, 0.0, 6.0),
    ];
    let links = [
        (0, 1),
        (1, 2),
        (2, 3),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
        (4, 5),
        (5, 6),
        (6, 7),
        (5, 8),
        (6, 9),
        (8, 9),
    ];
    let connections = build_connections(positions.len(), &links);

    let path_mesh = meshes.add(Cuboid::default());
    for (from, to) in links {
        let start = positions[from] + Vec3::Y * 0.22;
        let end = positions[to] + Vec3::Y * 0.22;
        let direction = end - start;
        let path_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.42, 0.52),
            emissive: LinearRgba::rgb(0.0, 2.4, 4.2),
            metallic: 0.45,
            perceptual_roughness: 0.18,
            ..default()
        });
        commands.spawn((
            EnergyPath { from, to },
            Mesh3d(path_mesh.clone()),
            MeshMaterial3d(path_material),
            Transform::from_translation((start + end) * 0.5)
                .with_rotation(Quat::from_rotation_arc(Vec3::Z, direction.normalize()))
                .with_scale(Vec3::new(0.11, 0.11, direction.length())),
        ));
    }

    let node_mesh = meshes.add(Cylinder::new(0.47, 0.44));
    let cap_mesh = meshes.add(Sphere::new(0.20).mesh().uv(24, 16));
    for (id, position) in positions.into_iter().enumerate() {
        let blown = id == 6;
        let material = materials.add(node_material(blown, false));
        let node_entity = commands
            .spawn((
                EnergyNode {
                    id,
                    voltage: if blown { 4.2 } else { 12.4 },
                    temperature: if blown { 112.0 } else { 31.0 + id as f32 * 1.7 },
                    is_blown: blown,
                    is_bypassed: false,
                    routing: 0,
                    capacity: 92.0,
                    connections: connections[id].clone(),
                },
                Mesh3d(node_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(position + Vec3::Y * 0.42),
            ))
            .id();

        let cap_material = materials.add(StandardMaterial {
            base_color: if blown {
                Color::srgb(0.8, 0.03, 0.01)
            } else {
                Color::srgb(0.02, 0.55, 0.72)
            },
            emissive: if blown {
                LinearRgba::rgb(7.0, 0.0, 0.0)
            } else {
                LinearRgba::rgb(0.0, 2.0, 3.8)
            },
            metallic: 0.32,
            perceptual_roughness: 0.18,
            ..default()
        });
        commands.entity(node_entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(cap_mesh.clone()),
                MeshMaterial3d(cap_material),
                Transform::from_xyz(0.0, 0.38, 0.0),
            ));
        });

        commands.spawn((
            PointLight {
                color: if blown {
                    Color::srgb(1.0, 0.03, 0.01)
                } else {
                    Color::srgb(0.05, 0.75, 1.0)
                },
                intensity: if blown { 210_000.0 } else { 95_000.0 },
                range: 5.8,
                radius: 0.22,
                ..default()
            },
            Transform::from_translation(position + Vec3::Y * 1.1),
            EnergyLight,
        ));
    }
}

fn build_connections(size: usize, links: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut connections = vec![Vec::new(); size];
    for &(from, to) in links {
        connections[from].push(to);
        connections[to].push(from);
    }
    connections
}

fn node_material(blown: bool, bypassed: bool) -> StandardMaterial {
    let (base_color, emissive) = if blown {
        if bypassed {
            (
                Color::srgb(0.30, 0.12, 0.02),
                LinearRgba::rgb(1.8, 0.35, 0.0),
            )
        } else {
            (
                Color::srgb(0.42, 0.01, 0.01),
                LinearRgba::rgb(6.0, 0.0, 0.0),
            )
        }
    } else {
        (
            Color::srgb(0.03, 0.18, 0.24),
            LinearRgba::rgb(0.0, 1.45, 2.8),
        )
    };
    StandardMaterial {
        base_color,
        emissive,
        metallic: 0.92,
        perceptual_roughness: 0.24,
        ..default()
    }
}

fn spawn_player(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let body_mesh = meshes.add(Capsule3d::new(0.33, 0.78));
    let head_mesh = meshes.add(Sphere::new(0.28).mesh().uv(24, 16));
    let arm_mesh = meshes.add(Cuboid::new(0.14, 0.52, 0.14));
    let metal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.23, 0.34, 0.44),
        metallic: 0.9,
        perceptual_roughness: 0.22,
        ..default()
    });
    let visor = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.88, 1.0),
        emissive: LinearRgba::rgb(0.0, 4.5, 7.0),
        metallic: 0.25,
        perceptual_roughness: 0.12,
        ..default()
    });

    commands
        .spawn((Player, Transform::from_xyz(-6.8, 0.25, -7.0)))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(body_mesh),
                MeshMaterial3d(metal.clone()),
                Transform::from_xyz(0.0, 0.64, 0.0),
            ));
            parent.spawn((
                Mesh3d(head_mesh),
                MeshMaterial3d(visor),
                Transform::from_xyz(0.0, 1.34, 0.0),
            ));
            for x in [-0.48, 0.48] {
                parent.spawn((
                    Mesh3d(arm_mesh.clone()),
                    MeshMaterial3d(metal.clone()),
                    Transform::from_xyz(x, 0.65, 0.0)
                        .with_rotation(Quat::from_rotation_z(x.signum() * -0.18)),
                ));
            }
            parent.spawn((
                PointLight {
                    color: Color::srgb(0.1, 0.85, 1.0),
                    intensity: 78_000.0,
                    range: 4.2,
                    ..default()
                },
                Transform::from_xyz(0.0, 1.25, 0.0),
            ));
        });
}

fn spawn_interface(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(29.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                border: UiRect::left(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.006, 0.02, 0.045, 0.92)),
            BorderColor::all(Color::srgba(0.05, 0.72, 0.90, 0.58)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("CENTRAL OS // ASSISTANT"),
                TextFont {
                    font_size: 23.0,
                    ..default()
                },
                TextColor(Color::srgb(0.20, 0.95, 1.0)),
            ));
            panel.spawn((
                Text::new("SYSTEM TERMINAL  /  LIVE DIAGNOSTICS"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.42, 0.60, 0.72)),
            ));
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.58, 0.76, 0.55)),
            ));
            panel.spawn((
                AssistantText,
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.66, 0.91, 0.95)),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            panel.spawn((
                Text::new("WASD / ARROWS  MOVE\nE  SCAN / COMMIT\nQ  ROTATE RELAY\nF9  DEMO CRITICAL SURGE"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.42, 0.62, 0.72)),
            ));
        });

    commands.spawn((
        StatusText,
        Text::new("GRID // REACTOR DIAGNOSTICS"),
        TextFont {
            font_size: 19.0,
            ..default()
        },
        TextColor(Color::srgb(0.28, 0.92, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(22.0),
            top: Val::Px(18.0),
            ..default()
        },
    ));
}

fn tick_clock(time: Res<Time>, mut clock: ResMut<GameClock>, state: Res<State<GamePhase>>) {
    if matches!(
        state.get(),
        GamePhase::StableOperation | GamePhase::CriticalSurge
    ) {
        clock.elapsed += time.delta_secs();
    }
}

fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GamePhase>>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    if matches!(state.get(), GamePhase::AftermathA | GamePhase::AftermathB) {
        return;
    }
    let Ok(mut player) = player.single_mut() else {
        return;
    };
    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.z += 1.0;
    }
    let movement = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_secs();
    player.translation += movement;
    player.translation.x = player.translation.x.clamp(-10.0, 10.0);
    player.translation.z = player.translation.z.clamp(-8.4, 8.2);
    if direction.length_squared() > 0.0 {
        player.rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.z));
    }
}

fn follow_camera(
    player: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    mut camera: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let (Ok(player), Ok(mut camera)) = (player.single(), camera.single_mut()) else {
        return;
    };
    let target = player.translation + Vec3::new(0.0, 0.0, 0.75);
    let desired = target + Vec3::new(12.5, 17.2, 14.5);
    camera.translation = camera.translation.lerp(desired, 0.06);
    camera.look_at(target, Vec3::Y);
}

fn update_nearby_node(
    player: Query<&Transform, With<Player>>,
    nodes: Query<(Entity, &Transform), With<EnergyNode>>,
    mut nearby: ResMut<NearbyNode>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    nearby.0 = nodes
        .iter()
        .filter(|(_, node_transform)| {
            player.translation.distance(node_transform.translation) <= INTERACTION_RANGE
        })
        .min_by(|(_, left), (_, right)| {
            player
                .translation
                .distance_squared(left.translation)
                .total_cmp(&player.translation.distance_squared(right.translation))
        })
        .map(|(entity, _)| entity);
}

fn node_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    nearby: Res<NearbyNode>,
    state: Res<State<GamePhase>>,
    mut nodes: Query<(&mut EnergyNode, &mut Transform)>,
    mut assistant: ResMut<AssistantLog>,
) {
    if !matches!(state.get(), GamePhase::StableOperation) {
        return;
    }
    let Some(entity) = nearby.0 else {
        return;
    };
    let Ok((mut node, mut transform)) = nodes.get_mut(entity) else {
        return;
    };
    if keyboard.just_pressed(KeyCode::KeyE) {
        assistant.message = format!(
            "[MULTIMETER // NODE {:02}]\nVoltage: {:.1}V {}\nCapacitor State: {}\nTemperature: {:.1} C\nLinks: {}\n\n{}",
            node.id + 1,
            node.voltage,
            if node.voltage < 8.0 {
                "(LEAK)"
            } else {
                "(NOMINAL)"
            },
            if node.is_blown { "BLOWN" } else { "OK" },
            node.temperature,
            node.connections.len(),
            if node.is_blown {
                "Do not repair in place. Rotate a healthy relay with Q to divert the load around this node."
            } else {
                "Relay ready. Press Q to rotate its routing gate."
            }
        );
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        if node.is_blown {
            assistant.message = "[ROUTING DENIED] The damaged capacitor has no moving relay. Choose a neighbouring healthy node and press Q.".into();
        } else {
            node.routing = (node.routing + 1) % 4;
            transform.rotate_y(std::f32::consts::FRAC_PI_2);
            assistant.message = format!(
                "[RELAY {:02}] Gate orientation {} / 4. Thermal load is being recalculated.",
                node.id + 1,
                node.routing + 1
            );
        }
    }
}

fn route_power_system(mut nodes: Query<&mut EnergyNode>, state: Res<State<GamePhase>>) {
    if !matches!(state.get(), GamePhase::StableOperation) {
        return;
    }
    let bypass_open = nodes
        .iter()
        .any(|node| !node.is_blown && node.routing % 2 == 1);
    for mut node in &mut nodes {
        if node.is_blown {
            node.is_bypassed = bypass_open;
            if bypass_open {
                node.voltage = 0.4;
                node.temperature = node.temperature.min(94.0);
            }
        }
    }
}

fn thermal_dynamics_system(
    time: Res<Time>,
    state: Res<State<GamePhase>>,
    mut nodes: Query<&mut EnergyNode>,
    mut assistant: ResMut<AssistantLog>,
) {
    if !matches!(state.get(), GamePhase::StableOperation) {
        return;
    }
    for mut node in &mut nodes {
        if node.is_blown {
            node.temperature = (node.temperature
                + if node.is_bypassed { -0.7 } else { 1.45 } * time.delta_secs())
            .clamp(20.0, 170.0);
            continue;
        }
        let load = 0.58 + node.routing as f32 * 0.26;
        node.temperature = (node.temperature + load * time.delta_secs() - 0.23 * time.delta_secs())
            .clamp(20.0, 130.0);
        node.voltage = (12.4 - (node.temperature - 30.0).max(0.0) * 0.012).max(8.6);
        if node.temperature > node.capacity + 24.0 {
            node.is_blown = true;
            node.is_bypassed = false;
            node.voltage = 4.2;
            assistant.message = format!(
                "[THERMAL ALERT] NODE {:02} exceeded {:.0} C. Capacitor failure detected. Route around it immediately.",
                node.id + 1,
                node.temperature
            );
        }
    }
}

fn create_periodic_fault(
    mut clock: ResMut<GameClock>,
    state: Res<State<GamePhase>>,
    mut nodes: Query<&mut EnergyNode>,
    mut assistant: ResMut<AssistantLog>,
) {
    if !matches!(state.get(), GamePhase::StableOperation) || clock.elapsed < clock.next_fault_at {
        return;
    }
    if let Some(mut node) = nodes.iter_mut().find(|node| !node.is_blown) {
        node.is_blown = true;
        node.is_bypassed = false;
        node.voltage = 4.2;
        node.temperature = 108.0;
        assistant.message = format!(
            "[ALERT] New voltage leak in sector node {:02}. Scan it with E, then change a relay route with Q.",
            node.id + 1
        );
    }
    clock.next_fault_at += if clock.elapsed < 480.0 { 60.0 } else { 25.0 };
}

fn critical_surge_trigger(
    keyboard: Res<ButtonInput<KeyCode>>,
    clock: Res<GameClock>,
    state: Res<State<GamePhase>>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut assistant: ResMut<AssistantLog>,
) {
    if !matches!(state.get(), GamePhase::StableOperation)
        || (!keyboard.just_pressed(KeyCode::F9) && clock.elapsed < CRITICAL_TIME_SECONDS)
    {
        return;
    }
    assistant.glitch_level = 1.0;
    assistant.message = "0xE7 // SIGNAL OVERFLOW\n0x00 // CONTROL SURFACE UNLOCKED\n\nCATASTROPHIC REACTOR FAILURE. THERMAL COLLAPSE IS CERTAIN.\n\nChoose a discharge route.".into();
    next_state.set(GamePhase::CriticalSurge);
}

fn enter_critical_surge(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_control_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-1.7, 0.0, -0.7),
        Ending::ExternalDischarge,
    );
    spawn_control_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(1.7, 0.0, -0.7),
        Ending::ReactorMeltdown,
    );
}

fn spawn_control_platform(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    ending: Ending,
) {
    let (color, label) = match ending {
        Ending::ExternalDischarge => (Color::srgb(0.0, 0.72, 0.95), "A // EXTERNAL DISCHARGE"),
        Ending::ReactorMeltdown => (Color::srgb(0.95, 0.04, 0.015), "B // INTERNAL MELTDOWN"),
    };
    let platform_material = materials.add(StandardMaterial {
        base_color: color,
        emissive: if matches!(ending, Ending::ExternalDischarge) {
            LinearRgba::rgb(0.0, 4.5, 8.0)
        } else {
            LinearRgba::rgb(8.0, 0.0, 0.0)
        },
        metallic: 0.7,
        perceptual_roughness: 0.18,
        ..default()
    });
    let base_mesh = meshes.add(Cylinder::new(1.18, 0.24));
    let lever_mesh = meshes.add(Cuboid::new(0.14, 1.35, 0.14));
    let indicator_mesh = meshes.add(Sphere::new(0.22).mesh().uv(20, 12));
    commands
        .spawn((
            ControlPlatform { ending },
            Mesh3d(base_mesh),
            MeshMaterial3d(platform_material.clone()),
            Transform::from_translation(position + Vec3::Y * 0.12),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(lever_mesh),
                MeshMaterial3d(platform_material.clone()),
                Transform::from_xyz(0.0, 0.76, 0.0).with_rotation(Quat::from_rotation_z(-0.42)),
            ));
            parent.spawn((
                Mesh3d(indicator_mesh),
                MeshMaterial3d(platform_material),
                Transform::from_xyz(0.0, 1.42, 0.0),
            ));
        });
    commands.spawn((
        Text2d::new(label),
        TextFont {
            font_size: 19.0,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position + Vec3::new(0.0, 0.12, 1.1))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
}

fn platform_interaction(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GamePhase>>,
    player: Query<&Transform, With<Player>>,
    platforms: Query<(&Transform, &ControlPlatform)>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut assistant: ResMut<AssistantLog>,
) {
    if !matches!(state.get(), GamePhase::CriticalSurge) || !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };
    let Some((_, platform)) = platforms
        .iter()
        .filter(|(transform, _)| player.translation.distance(transform.translation) < 2.0)
        .min_by(|(left, _), (right, _)| {
            player
                .translation
                .distance_squared(left.translation)
                .total_cmp(&player.translation.distance_squared(right.translation))
        })
    else {
        assistant.message = "[CRITICAL] Two emergency platforms are active in the centre. Walk to A or B and press E to pull its lever.".into();
        return;
    };
    match platform.ending {
        Ending::ExternalDischarge => {
            assistant.message = "[COMMIT A] External discharge accepted. The reactor core survives; the outer grid receives the overload.".into();
            next_state.set(GamePhase::AftermathA);
        }
        Ending::ReactorMeltdown => {
            assistant.message = "[COMMIT B] Internal containment accepted. The reactor is sacrificed to preserve the external grid.".into();
            next_state.set(GamePhase::AftermathB);
        }
    }
}

fn update_energy_visuals(
    state: Res<State<GamePhase>>,
    nodes: Query<(&EnergyNode, &MeshMaterial3d<StandardMaterial>)>,
    paths: Query<(&EnergyPath, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lights: Query<&mut PointLight, With<EnergyLight>>,
) {
    let critical = matches!(
        state.get(),
        GamePhase::CriticalSurge | GamePhase::AftermathB
    );
    let node_status: Vec<(usize, bool)> = nodes
        .iter()
        .map(|(node, _)| (node.id, node.is_blown))
        .collect();
    for (node, material_handle) in &nodes {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            *material = if critical {
                StandardMaterial {
                    base_color: Color::srgb(0.30, 0.01, 0.01),
                    emissive: LinearRgba::rgb(5.2, 0.0, 0.0),
                    metallic: 0.86,
                    perceptual_roughness: 0.2,
                    ..default()
                }
            } else {
                node_material(node.is_blown, node.is_bypassed)
            };
        }
    }
    for (path, material_handle) in &paths {
        let faulted = node_status
            .iter()
            .any(|(id, blown)| *blown && (*id == path.from || *id == path.to));
        if let Some(material) = materials.get_mut(&material_handle.0) {
            *material = StandardMaterial {
                base_color: if critical || faulted {
                    Color::srgb(0.55, 0.02, 0.01)
                } else {
                    Color::srgb(0.0, 0.42, 0.52)
                },
                emissive: if critical || faulted {
                    LinearRgba::rgb(5.5, 0.0, 0.0)
                } else {
                    LinearRgba::rgb(0.0, 2.4, 4.2)
                },
                metallic: 0.45,
                perceptual_roughness: 0.18,
                ..default()
            };
        }
    }
    for mut light in &mut lights {
        if critical {
            light.color = Color::srgb(1.0, 0.01, 0.0);
            light.intensity = 190_000.0;
        }
    }
}

fn crisis_pulse(
    time: Res<Time>,
    state: Res<State<GamePhase>>,
    mut camera: Query<&mut Bloom, With<MainCamera>>,
    mut lights: Query<&mut PointLight, With<EnergyLight>>,
) {
    if !matches!(state.get(), GamePhase::CriticalSurge) {
        return;
    }
    let pulse = (time.elapsed_secs() * 9.0).sin().abs();
    if let Ok(mut bloom) = camera.single_mut() {
        bloom.intensity = 0.26 + pulse * 0.42;
    }
    for mut light in &mut lights {
        light.intensity = 100_000.0 + pulse * 460_000.0;
    }
}

fn update_assistant_ui(
    state: Res<State<GamePhase>>,
    clock: Res<GameClock>,
    nearby: Res<NearbyNode>,
    assistant: Res<AssistantLog>,
    nodes: Query<&EnergyNode>,
    mut assistant_text: Query<(&mut Text, &mut TextColor), With<AssistantText>>,
    mut status_text: Query<&mut Text, (With<StatusText>, Without<AssistantText>)>,
) {
    let phase = match state.get() {
        GamePhase::StableOperation => "STABLE OPERATION",
        GamePhase::CriticalSurge => "CRITICAL SURGE",
        GamePhase::AftermathA => "AFTERMATH A // CORE SAVED",
        GamePhase::AftermathB => "AFTERMATH B // GRID SAVED",
    };
    let minutes = (clock.elapsed / 60.0) as u32;
    let seconds = (clock.elapsed % 60.0) as u32;
    let nearest = nearby
        .0
        .and_then(|entity| nodes.get(entity).ok())
        .map(|node| format!("NODE {:02} IN RANGE", node.id + 1))
        .unwrap_or_else(|| "NO NODE IN RANGE".into());
    let glitch = if assistant.glitch_level > 0.0 {
        let code = ((clock.elapsed * 97.0) as u32) & 0xFFFF;
        format!("0x{code:04X} // DATA CORRUPTION\n")
    } else {
        String::new()
    };
    if let Ok((mut text, mut color)) = assistant_text.single_mut() {
        text.0 = format!(
            "STATUS: {phase}\nUPTIME: {minutes:02}:{seconds:02}\nPROXIMITY: {nearest}\n\n{glitch}{}",
            assistant.message
        );
        color.0 = if assistant.glitch_level > 0.0 {
            Color::srgb(1.0, 0.22, 0.13)
        } else {
            Color::srgb(0.66, 0.91, 0.95)
        };
    }
    if let Ok(mut text) = status_text.single_mut() {
        text.0 = format!("GRID // REACTOR DIAGNOSTICS     {phase}");
    }
}

fn resolve_external_discharge(
    mut assistant: ResMut<AssistantLog>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    assistant.message = "[AFTERMATH A] Discharge complete. The outer network is dark, but reactor containment is stable.\n\nEND STATE LOCKED: CORE SURVIVES / CITY GRID LOST.".into();
    ambient.brightness = 22.0;
}

fn resolve_reactor_meltdown(
    mut assistant: ResMut<AssistantLog>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    assistant.message = "[AFTERMATH B] Containment detonation complete. The reactor has been sacrificed.\n\nEND STATE LOCKED: EXTERNAL GRID SURVIVES / CORE LOST.".into();
    ambient.brightness = 8.0;
}
