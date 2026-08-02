use astra_voxel_world::prelude::*;
use bevy::prelude::*;
use crate::state::*;

#[derive(Component)]
pub struct PlayerTag;

#[derive(Component)]
pub struct PlayerSpotlightTag;

#[derive(Resource)]
pub struct PlayerState {
    pub speed: f32,
    pub is_underground: bool,
    pub current_y: f32,
    pub surface_y: f32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            speed: 14.0,
            is_underground: false,
            current_y: 75.0,
            surface_y: 75.0,
        }
    }
}

pub fn spawn_player_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let body_mesh = meshes.add(Capsule3d::new(0.45, 0.9));
    let head_mesh = meshes.add(Sphere::new(0.4).mesh().uv(24, 16));
    let visor_mesh = meshes.add(Sphere::new(0.18).mesh().uv(16, 12));

    let body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.65, 0.95), // أزرق البطل
        metallic: 0.5,
        perceptual_roughness: 0.3,
        ..default()
    });

    let visor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.9, 0.2), // قناع مضيء
        emissive: LinearRgba::rgb(2.5, 2.0, 0.3),
        ..default()
    });

    let _player_id = commands
        .spawn((
            Name::new("Explorer Player"),
            PlayerTag,
            Mesh3d(body_mesh),
            MeshMaterial3d(body_mat.clone()),
            Transform::from_xyz(0.0, 80.0, 0.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(head_mesh),
                MeshMaterial3d(body_mat),
                Transform::from_xyz(0.0, 0.85, 0.0),
            ));
            parent.spawn((
                Mesh3d(visor_mesh),
                MeshMaterial3d(visor_mat),
                Transform::from_xyz(0.0, 0.95, 0.32),
            ));
        })
        .id();

    // كشاف ضوئي محمول للمغارات والكهوف
    commands.spawn((
        Name::new("Player Headlamp"),
        SpotLight {
            color: Color::srgb(1.0, 0.95, 0.8),
            intensity: 350_000.0,
            range: 25.0,
            radius: 0.3,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 81.5, 0.0).looking_at(Vec3::new(0.0, 75.0, 5.0), Vec3::Y),
        PlayerSpotlightTag,
    ));
}

pub fn update_player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    world_res: Res<VoxelViewerWorld>,
    camera_state: Res<VoxelViewerCamera>,
    mut player_state: ResMut<PlayerState>,
    mut transforms: ParamSet<(
        Query<&mut Transform, With<PlayerTag>>,
        Query<&mut Transform, With<PlayerSpotlightTag>>,
    )>,
) {
    let mut player_pos = Vec3::ZERO;
    let mut player_forward = Vec3::Z;

    {
        let mut p0 = transforms.p0();
        let Ok(mut player_transform) = p0.single_mut() else {
            return;
        };

        // حساب الاتجاهات نسبياً لزاوية رؤية الكاميرا الحالية (Camera-Relative Movement)
        let forward = Vec3::new(-camera_state.yaw.sin(), 0.0, -camera_state.yaw.cos());
        let right = Vec3::new(camera_state.yaw.cos(), 0.0, -camera_state.yaw.sin());

        let mut movement = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            movement += forward;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            movement -= forward;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            movement += right;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            movement -= right;
        }

        let speed = if keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            player_state.speed * 1.8
        } else {
            player_state.speed
        };

        let mut next_pos = player_transform.translation;
        if movement.length_squared() > 0.0 {
            let move_dir = movement.normalize();
            next_pos += move_dir * speed * time.delta_secs();
            player_transform.rotation = Quat::from_rotation_y((-move_dir.x).atan2(-move_dir.z));
        }

        // نظام الفيزياء والتصادم السطحي والجداري
        let world_x = (next_pos.x / BLOCK_SIZE).round() as i64;
        let world_z = (next_pos.z / BLOCK_SIZE).round() as i64;
        let column = sample_voxel_column(world_res.settings, world_x, world_z);
        let ground_y = column.height as f32 * HEIGHT_SCALE;

        let current_y = player_transform.translation.y;
        let y_diff = ground_y - current_y;

        // إذا كانت المنطقة الأمامية جداراً مرتفعاً جداً (أكثر من 1.8 وحدة)، نمنع الاختراق
        if y_diff > 1.8 {
            // تصادم جداري: نمنع الحركة الأفقية باتجاه الجدار
            next_pos.x = player_transform.translation.x;
            next_pos.z = player_transform.translation.z;
        } else {
            // مطابقة ارتفاع القدمين مع السطح بسلاسة
            next_pos.y = ground_y + 0.45;
        }

        player_transform.translation = next_pos;

        player_state.surface_y = ground_y;
        player_state.current_y = player_transform.translation.y;
        player_state.is_underground = player_transform.translation.y < (ground_y - 2.5);

        player_pos = player_transform.translation;
        player_forward = *player_transform.forward();
    }

    // ربط الضوء المحمول بموقع البطل
    let mut p1 = transforms.p1();
    if let Ok(mut light_transform) = p1.single_mut() {
        light_transform.translation = player_pos + Vec3::Y * 1.2;
        light_transform.look_at(player_pos + player_forward * 6.0 + Vec3::NEG_Y * 2.0, Vec3::Y);
    }
}
