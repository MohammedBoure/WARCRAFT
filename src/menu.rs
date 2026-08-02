use astra_voxel_world::prelude::*;
use bevy::prelude::*;
use crate::state::*;
use crate::world::*;

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct PresetOptionButton {
    pub preset: &'static str,
}

#[derive(Component)]
pub struct StartGameButton;

pub fn setup_main_menu_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font_handle: Handle<Font> = asset_server.load("fonts/arabic.ttf");

    commands
        .spawn((
            Name::new("Joyful Main Menu"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.08, 0.14, 0.92)),
            ZIndex(50),
            MainMenuRoot,
        ))
        .with_children(|root| {
            // بطاقة الواجهة الرئيسية المبهجة
            root.spawn((
                Node {
                    width: Val::Px(650.0),
                    padding: UiRect::all(Val::Px(28.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.16, 0.24, 0.95)),
                BorderColor::all(Color::srgba(0.35, 0.75, 0.85, 0.80)),
            ))
            .with_children(|panel| {
                // العنوان الرائع
                panel.spawn((
                    Text::new("🌟 حديقة الفوكسل السحرية 🌟"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.98, 1.0)),
                ));

                panel.spawn((
                    Text::new("اختر نوع العالم والبيئة قبل بداية المغامرة:"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.75, 0.88, 0.95)),
                ));

                // شبكة الخيارات والبيئات
                panel
                    .spawn(Node {
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
                        column_gap: Val::Px(12.0),
                        row_gap: Val::Px(12.0),
                        width: Val::Percent(100.0),
                        ..default()
                    })
                    .with_children(|grid| {
                        spawn_preset_button(grid, font_handle.clone(), "balanced", "🌿 حديقة خضراء متوازنة");
                        spawn_preset_button(grid, font_handle.clone(), "crystal", "💎 عالم البلور السحري");
                        spawn_preset_button(grid, font_handle.clone(), "volcanic", "🌋 جبال وصخور بركانية");
                        spawn_preset_button(grid, font_handle.clone(), "frozen", "❄️ عالم الجليد والثلج");
                    });

                // زر بدء اللعبة المبهج
                panel
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(90.0),
                            height: Val::Px(54.0),
                            margin: UiRect::top(Val::Px(10.0)),
                            border_radius: BorderRadius::all(Val::Px(12.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.65, 0.45, 0.95)),
                        StartGameButton,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("🚀 ابدأ المغامرة الآن"),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            });
        });
}

fn spawn_preset_button(
    parent: &mut ChildSpawnerCommands,
    font: Handle<Font>,
    preset: &'static str,
    label: &'static str,
) {
    parent
        .spawn((
            Button,
            Node {
                height: Val::Px(46.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.24, 0.35, 0.90)),
            BorderColor::all(Color::srgba(0.28, 0.55, 0.70, 0.50)),
            PresetOptionButton { preset },
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.90, 0.95, 1.0)),
            ));
        });
}

pub fn handle_main_menu_interactions(
    mut next_state: ResMut<NextState<AppState>>,
    mut world_res: ResMut<VoxelViewerWorld>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut commands: Commands,
    preset_buttons: Query<(&Interaction, &PresetOptionButton), Changed<Interaction>>,
    start_buttons: Query<&Interaction, (With<StartGameButton>, Changed<Interaction>)>,
) {
    for (interaction, preset_btn) in &preset_buttons {
        if *interaction == Interaction::Pressed {
            if let Some(preset) = VoxelWorldComposition::preset(preset_btn.preset) {
                world_res.settings.composition = preset;
                reload_loaded_chunks(&mut commands, &mut loaded);
            }
        }
    }

    for interaction in &start_buttons {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Playing);
        }
    }
}

pub fn cleanup_main_menu_ui(
    mut commands: Commands,
    menu_query: Query<Entity, With<MainMenuRoot>>,
) {
    for entity in &menu_query {
        commands.entity(entity).despawn();
    }
}
