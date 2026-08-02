use astra_voxel_world::prelude::*;
use bevy::prelude::*;
use crate::config::*;
use crate::state::*;
use crate::world::*;

pub fn spawn_generation_dialog_button(
    parent: &mut ChildSpawnerCommands,
    action: VoxelGenerationDialogAction,
    label: &'static str,
    font: Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                min_width: Val::Px(104.0),
                height: Val::Px(32.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.18, 0.22, 0.92)),
            BorderColor::all(Color::srgba(0.36, 0.62, 0.70, 0.60)),
            VoxelGenerationDialogButton { action },
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.84, 0.96, 0.98)),
            ));
        });
}

pub fn spawn_generation_dialog(commands: &mut Commands, font: Handle<Font>) {
    commands
        .spawn((
            Name::new("Voxel Generation Input Dialog"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.46)),
            ZIndex(30),
            Visibility::Hidden,
            VoxelGenerationDialogRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(880.0),
                    max_width: Val::Percent(92.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(7.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.018, 0.030, 0.038, 0.96)),
                BorderColor::all(Color::srgba(0.36, 0.62, 0.70, 0.62)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("إعدادات العالم الفوكسلي"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.88, 0.98, 1.0)),
                ));
                panel.spawn((
                    Text::new(
                        "قم بتعديل قيم التوليد البرمجي ثم اضغط تطبيق (Apply). اضغط Esc للإلغاء.",
                    ),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.66, 0.78, 0.82)),
                ));
                panel
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(440.0),
                            max_height: Val::Px(440.0),
                            padding: UiRect::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(5.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.004, 0.010, 0.014, 0.96)),
                        BorderColor::all(Color::srgba(0.20, 0.44, 0.52, 0.62)),
                    ))
                    .with_children(|input| {
                        input.spawn((
                            Text::new(""),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.88, 0.92, 0.86)),
                            VoxelGenerationDialogInputText,
                        ));
                    });
                panel.spawn((
                    Text::new(""),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.78, 0.88, 0.86)),
                    VoxelGenerationDialogStatusText,
                ));
                panel
                    .spawn((Node {
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },))
                    .with_children(|row| {
                        spawn_generation_dialog_button(
                            row,
                            VoxelGenerationDialogAction::Apply,
                            "تطبيق (APPLY)",
                            font.clone(),
                        );
                        spawn_generation_dialog_button(
                            row,
                            VoxelGenerationDialogAction::Cancel,
                            "إلغاء (CANCEL)",
                            font.clone(),
                        );
                    });
            });
        });
}

pub fn handle_generation_dialog_buttons(
    mut dialog: ResMut<VoxelGenerationDialogState>,
    mut world: ResMut<VoxelViewerWorld>,
    mut controls: ResMut<VoxelViewerLiveControls>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut commands: Commands,
    changed_buttons: Query<(&Interaction, &VoxelGenerationDialogButton), Changed<Interaction>>,
) {
    for (interaction, button) in &changed_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match button.action {
            VoxelGenerationDialogAction::Open => {
                dialog.open = true;
                dialog.buffer = generation_arguments_text(world.settings, world.load_radius);
                dialog.status =
                    "اكتب القيم البرمجية ثم انقر فوق تطبيق.".to_string();
            }
            VoxelGenerationDialogAction::Apply => {
                apply_generation_dialog_buffer(
                    &mut dialog,
                    &mut world,
                    &mut controls,
                    &mut loaded,
                    &mut commands,
                );
            }
            VoxelGenerationDialogAction::Cancel => {
                dialog.open = false;
                dialog.status = "تم إلغاء النافذة.".to_string();
            }
        }
    }
}

pub fn handle_generation_dialog_keyboard_input(
    mut keyboard_input: MessageReader<bevy::input::keyboard::KeyboardInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dialog: ResMut<VoxelGenerationDialogState>,
    mut world: ResMut<VoxelViewerWorld>,
    mut controls: ResMut<VoxelViewerLiveControls>,
    mut loaded: ResMut<LoadedVoxelChunks>,
    mut commands: Commands,
) {
    if !dialog.open {
        keyboard_input.clear();
        return;
    }

    for event in keyboard_input.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }

        match event.key_code {
            KeyCode::Escape => {
                dialog.open = false;
                dialog.status = "تم إلغاء النافذة.".to_string();
            }
            KeyCode::Enter | KeyCode::NumpadEnter
                if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) =>
            {
                apply_generation_dialog_buffer(
                    &mut dialog,
                    &mut world,
                    &mut controls,
                    &mut loaded,
                    &mut commands,
                );
            }
            KeyCode::Enter | KeyCode::NumpadEnter => {
                dialog.buffer.push('\n');
            }
            KeyCode::Backspace => {
                dialog.buffer.pop();
            }
            KeyCode::Tab => {
                dialog.buffer.push(' ');
            }
            _ => {
                if let Some(text) = event.text.as_ref() {
                    for ch in text.chars().filter(|ch| !ch.is_control()) {
                        dialog.buffer.push(ch);
                    }
                }
            }
        }

        if dialog.buffer.len() > 4096 {
            dialog.buffer.truncate(4096);
        }
    }
}

pub fn apply_generation_dialog_buffer(
    dialog: &mut VoxelGenerationDialogState,
    world: &mut VoxelViewerWorld,
    controls: &mut VoxelViewerLiveControls,
    loaded: &mut LoadedVoxelChunks,
    commands: &mut Commands,
) {
    match parse_generation_arguments(&dialog.buffer, world.settings, world.load_radius) {
        Ok(update) => {
            world.settings = update.settings;
            world.load_radius = update.load_radius;
            *controls = VoxelViewerLiveControls::from_composition(world.settings.composition);
            controls.last_change = "dialog apply".to_string();
            reload_loaded_chunks(commands, loaded);
            dialog.open = false;
            dialog.status = "تم تطبيق إعدادات العالم بنجاح.".to_string();
        }
        Err(error) => {
            dialog.status = format!("خطأ: {error}");
        }
    }
}

pub fn update_generation_dialog_ui(
    dialog: Res<VoxelGenerationDialogState>,
    mut roots: Query<&mut Visibility, With<VoxelGenerationDialogRoot>>,
    mut dialog_text: ParamSet<(
        Query<&mut Text, With<VoxelGenerationDialogInputText>>,
        Query<&mut Text, With<VoxelGenerationDialogStatusText>>,
    )>,
    mut button_styles: Query<(
        &Interaction,
        &mut BackgroundColor,
        &VoxelGenerationDialogButton,
    )>,
) {
    if let Ok(mut visibility) = roots.single_mut() {
        *visibility = if dialog.open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut text) = dialog_text.p0().single_mut() {
        text.0 = if dialog.open {
            format!("{}_", dialog.buffer)
        } else {
            String::new()
        };
    }
    if let Ok(mut text) = dialog_text.p1().single_mut() {
        text.0 = dialog.status.clone();
    }

    for (interaction, mut background, button) in &mut button_styles {
        let base = match button.action {
            VoxelGenerationDialogAction::Open => Color::srgba(0.08, 0.18, 0.22, 0.92),
            VoxelGenerationDialogAction::Apply => Color::srgba(0.09, 0.30, 0.24, 0.94),
            VoxelGenerationDialogAction::Cancel => Color::srgba(0.22, 0.12, 0.11, 0.94),
        };
        let hover = match button.action {
            VoxelGenerationDialogAction::Open => Color::srgba(0.12, 0.28, 0.34, 0.96),
            VoxelGenerationDialogAction::Apply => Color::srgba(0.12, 0.42, 0.32, 0.96),
            VoxelGenerationDialogAction::Cancel => Color::srgba(0.34, 0.16, 0.14, 0.96),
        };
        *background = match *interaction {
            Interaction::Pressed => BackgroundColor(Color::srgba(0.18, 0.48, 0.52, 0.98)),
            Interaction::Hovered => BackgroundColor(hover),
            Interaction::None => BackgroundColor(base),
        };
    }
}

pub fn update_generation_hud(
    world: Res<VoxelViewerWorld>,
    controls: Res<VoxelViewerLiveControls>,
    weather_state: Res<VoxelViewerWeatherState>,
    loaded: Res<LoadedVoxelChunks>,
    mut text: Query<&mut Text, With<VoxelViewerHudText>>,
) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    text.0 = generation_hud_text(&world, &controls, &weather_state, loaded.chunks.len());
}

pub fn generation_hud_text(
    _world: &VoxelViewerWorld,
    _controls: &VoxelViewerLiveControls,
    weather_state: &VoxelViewerWeatherState,
    active_chunks: usize,
) -> String {
    let local_biome = match weather_state.biome {
        VoxelBiome::Plains => "المراعي الخضراء 🌿",
        VoxelBiome::Forest => "الغابة السحرية 🌲",
        VoxelBiome::Desert => "الصحراء الذهبية 🏜️",
        VoxelBiome::Tundra => "السهول الثلجية ❄️",
        VoxelBiome::Mountains => "الجبال المرتفعة 🏔️",
        VoxelBiome::Wetlands => "المستنقعات 🌾",
        VoxelBiome::Badlands => "الأراضي الصخرية 🏜️",
        VoxelBiome::CraterField => "فوهة النيزك 🌌",
        VoxelBiome::Volcanic => "الجبال البركانية 🌋",
        VoxelBiome::CrystalFields => "أبراج البلور 💎",
    };
    let local_weather = match weather_state.weather {
        VoxelWeather::Clear => "صافي ☀️",
        VoxelWeather::Cloudy => "غائم ☁️",
        VoxelWeather::Rain => "مطير 🌧️",
        VoxelWeather::Storm => "عاصفة 🌩️",
        VoxelWeather::Snow => "ثلجي ❄️",
        VoxelWeather::DustStorm => "عاصفة رملية 🌪️",
        VoxelWeather::Ashfall => "أمطار بركانية 🌋",
        VoxelWeather::IonStorm => "عاصفة أيونية ⚡",
    };

    format!(
        "🌟 حديقة الفوكسل السحرية\nالبيئة الحالية: {local_biome}\nالطقس: {local_weather}\nالأجزاء المكتملة: {active_chunks}\n\n🎮 تعليمات التحكم:\nWASD / الأسهم: تحريك البطل\nزر الماوس الأيسر / F: حفر وتعدين الكتل ⛏️\nزر الماوس الأيمن / R: بناء كتل جديدة 🧱\nالأرقام 1 - 4: اختيار الكتل (1: تراب | 2: حجر | 3: بلور | 4: ذهب)",
    )
}
