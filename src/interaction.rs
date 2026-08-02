use astra_voxel_world::prelude::*;
use bevy::prelude::*;
use crate::player::*;
use crate::state::*;
use crate::world::*;

#[derive(Component)]
pub struct TargetBlockHighlightTag;

#[derive(Resource)]
pub struct VoxelWorldEdits {
    pub edits: Vec<VoxelTerrainEdit>,
    pub active_block_type: BlockKind,
    pub targeted_position: Option<VoxelBlockPosition>,
}

impl Default for VoxelWorldEdits {
    fn default() -> Self {
        Self {
            edits: Vec::new(),
            active_block_type: BlockKind::Dirt,
            targeted_position: None,
        }
    }
}

impl VoxelWorldEdits {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn setup_target_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wire_mesh = meshes.add(Cuboid::new(BLOCK_SIZE * 1.05, HEIGHT_SCALE * 1.05, BLOCK_SIZE * 1.05));
    let wire_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.85, 0.2, 0.35),
        emissive: LinearRgba::rgb(1.2, 1.0, 0.3),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        Name::new("Target Block Highlight"),
        Mesh3d(wire_mesh),
        MeshMaterial3d(wire_mat),
        Transform::from_xyz(0.0, -500.0, 0.0),
        Visibility::Hidden,
        TargetBlockHighlightTag,
    ));
}

pub fn update_target_block_highlight(
    mut edits_res: ResMut<VoxelWorldEdits>,
    player_query: Query<&Transform, (With<PlayerTag>, Without<TargetBlockHighlightTag>)>,
    mut highlight_query: Query<(&mut Transform, &mut Visibility), (With<TargetBlockHighlightTag>, Without<PlayerTag>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok((mut highlight_transform, mut visibility)) = highlight_query.single_mut() else {
        return;
    };

    // إيجاد موضع الفوكسل المستهدف أمام البطل مباشرة
    let forward = player_transform.forward();
    let target_vec = player_transform.translation + forward * 2.8 + Vec3::NEG_Y * 0.4;

    let target_pos = VoxelBlockPosition::new(
        (target_vec.x / BLOCK_SIZE).round() as i64,
        (target_vec.y / HEIGHT_SCALE).round() as i32,
        (target_vec.z / BLOCK_SIZE).round() as i64,
    );

    edits_res.targeted_position = Some(target_pos);

    // نقل المربع المظلل لموقع الكتلة المستهدفة
    highlight_transform.translation = Vec3::new(
        target_pos.x as f32 * BLOCK_SIZE,
        target_pos.y as f32 * HEIGHT_SCALE,
        target_pos.z as f32 * BLOCK_SIZE,
    );
    *visibility = Visibility::Visible;
}

pub fn handle_voxel_digging_and_building(
    mouse: Res<ButtonInput<MouseButton>>,
    mut edits_res: ResMut<VoxelWorldEdits>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut commands: Commands,
) {
    // الزر الأيسر: التعدين والهدم
    let dig_requested = mouse.just_pressed(MouseButton::Left);
    // الزر الأيمن: البناء والوضع
    let build_requested = mouse.just_pressed(MouseButton::Right);

    if !dig_requested && !build_requested {
        return;
    }

    let Some(voxel_pos) = edits_res.targeted_position else {
        return;
    };

    if dig_requested {
        // هدم الكتلة المستهدفة بالزر الأيسر
        edits_res.edits.push(VoxelTerrainEdit::DigSphere {
            center: voxel_pos,
            radius: 2,
        });
        reload_loaded_chunks(&mut commands, &mut loaded);
    } else if build_requested {
        // بناء كتلة بالزر الأيمن
        let active_block = edits_res.active_block_type;
        edits_res.edits.push(VoxelTerrainEdit::FillSphere {
            center: voxel_pos,
            radius: 1,
            block: active_block,
        });
        reload_loaded_chunks(&mut commands, &mut loaded);
    }
}

pub fn cycle_build_block_kind(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut edits_res: ResMut<VoxelWorldEdits>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        edits_res.active_block_type = BlockKind::Dirt;
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        edits_res.active_block_type = BlockKind::Stone;
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        edits_res.active_block_type = BlockKind::CrystalOre;
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        edits_res.active_block_type = BlockKind::GoldOre;
    }
}
