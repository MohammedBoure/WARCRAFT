use bevy::prelude::*;

#[derive(Component)]
pub struct FlowingWaterTag;

#[derive(Resource)]
pub struct WaterMaterialAssets {
    pub water_material: Handle<StandardMaterial>,
}

pub fn setup_flowing_water_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let water_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.12, 0.55, 0.88, 0.72), // ماء شفاف مبهج
        emissive: LinearRgba::rgb(0.05, 0.35, 0.6),
        perceptual_roughness: 0.1,
        metallic: 0.2,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.insert_resource(WaterMaterialAssets {
        water_material: water_mat,
    });
}

pub fn animate_flowing_water_system(
    time: Res<Time>,
    water_assets: Option<Res<WaterMaterialAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(water_assets) = water_assets else {
        return;
    };
    
    if let Some(mat) = materials.get_mut(&water_assets.water_material) {
        let pulse = (time.elapsed_secs() * 2.5).sin() * 0.15;
        mat.emissive = LinearRgba::rgb(0.05 + pulse * 0.03, 0.4 + pulse * 0.1, 0.7 + pulse * 0.15);
    }
}
