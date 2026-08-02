use astra_voxel_world::prelude::*;
use bevy::prelude::*;
use crate::player::*;
use crate::state::*;
use crate::world::*;

#[derive(Resource)]
pub struct VoxelWorldEdits {
    pub edits: Vec<VoxelTerrainEdit>,
    pub active_block_type: BlockKind,
}

impl Default for VoxelWorldEdits {
    fn default() -> Self {
        Self {
            edits: Vec::new(),
            active_block_type: BlockKind::Dirt,
        }
    }
}

impl VoxelWorldEdits {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn handle_voxel_digging_and_building(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut edits_res: ResMut<VoxelWorldEdits>,
    _world_res: Res<VoxelViewerWorld>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut commands: Commands,
    player_query: Query<&Transform, With<PlayerTag>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let dig_requested = mouse.just_pressed(MouseButton::Left) || keyboard.just_pressed(KeyCode::KeyF);
    let build_requested = mouse.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::KeyR);

    if !dig_requested && !build_requested {
        return;
    }

    // تحديد مكان التفاعل أمام البطل أو تحته
    let forward = player_transform.forward();
    let target_pos = player_transform.translation + forward * 2.8 + Vec3::NEG_Y * 0.5;

    let voxel_pos = VoxelBlockPosition::new(
        (target_pos.x / BLOCK_SIZE).round() as i64,
        (target_pos.y / HEIGHT_SCALE).round() as i32,
        (target_pos.z / BLOCK_SIZE).round() as i64,
    );

    if dig_requested {
        // حفر وإزالة المكعبات بحجم نصف قطر 2
        edits_res.edits.push(VoxelTerrainEdit::DigSphere {
            center: voxel_pos,
            radius: 2,
        });
        reload_loaded_chunks(&mut commands, &mut loaded);
    } else if build_requested {
        // بناء وإضافة مكعبات نوع الحجر/التراب/العشب
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
