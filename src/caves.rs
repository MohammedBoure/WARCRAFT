use bevy::prelude::*;
use crate::player::*;

pub fn update_cave_transparency_system(
    player_state: Res<PlayerState>,
    mut chunk_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !player_state.is_underground {
        // عند البقاء على السطح، كل الكتل صلبة وطبيعية
        for mat_handle in &mut chunk_materials {
            if let Some(mat) = materials.get_mut(&mat_handle.0) {
                mat.alpha_mode = AlphaMode::Opaque;
            }
        }
        return;
    }

    // عند دخول البطل في حفرة أو كهف سفلي، السطح يصبح شفافاً لمنع حجب الرؤية!
    for mat_handle in &mut chunk_materials {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.alpha_mode = AlphaMode::Blend;
            let color = mat.base_color.to_srgba();
            mat.base_color = Color::srgba(color.red, color.green, color.blue, 0.65);
        }
    }
}
