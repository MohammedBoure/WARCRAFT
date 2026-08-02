use bevy::app::AppExit;
use bevy::prelude::*;

use crate::gameplay::RunLifecycle;
use crate::state::*;

#[derive(Resource)]
pub struct LoadingTimer(pub Timer);

impl Default for LoadingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.55, TimerMode::Once))
    }
}

#[derive(Component)]
pub struct MenuRoot;
#[derive(Component)]
pub struct LoadingRoot;
#[derive(Component)]
pub struct HelpPanel;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Start,
    Help,
    Quit,
}

pub fn setup_loading_screen(
    mut commands: Commands,
    font: Res<ArabicFont>,
    mut timer: ResMut<LoadingTimer>,
) {
    timer.0.reset();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.025, 0.055, 0.075)),
            ZIndex(90),
            LoadingRoot,
            ScreenRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("نقطة الانهيار"),
                TextFont {
                    font: font.0.clone(),
                    font_size: 52.0,
                    ..default()
                },
                TextColor(Color::srgb(0.62, 1.0, 0.92)),
                TextLayout::new_with_justify(Justify::Center),
            ));
            root.spawn((
                Text::new("تهيئة العالم..."),
                TextFont {
                    font: font.0.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.70, 0.82, 0.86)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Percent(34.0),
                    ..default()
                },
            ));
        });
}

pub fn finish_loading(
    time: Res<Time>,
    loaded: Res<LoadedVoxelChunks>,
    mut timer: ResMut<LoadingTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() && loaded.chunks.len() >= 9 {
        next_state.set(AppState::MainMenu);
    }
}

pub fn setup_main_menu(mut commands: Commands, font: Res<ArabicFont>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.018, 0.045, 0.065, 0.52)),
            ZIndex(80),
            MenuRoot,
            ScreenRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(620.0),
                    max_width: Val::Percent(92.0),
                    padding: UiRect::all(Val::Px(34.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(22.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(18.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.025, 0.075, 0.10, 0.94)),
                BorderColor::all(Color::srgba(0.34, 0.95, 0.82, 0.76)),
                BoxShadow::new(
                    Color::srgba(0.0, 0.0, 0.0, 0.56),
                    Val::Px(0.0),
                    Val::Px(12.0),
                    Val::Px(32.0),
                    Val::Px(2.0),
                ),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("نقطة الانهيار"),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.66, 1.0, 0.92)),
                    TextLayout::new_with_justify(Justify::Center),
                ));
                panel.spawn((
                    Text::new("كل ضربة تقرّب العالم من لحظته الأخيرة"),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 19.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.78, 0.88, 0.90)),
                    TextLayout::new_with_justify(Justify::Center),
                ));
                spawn_menu_button(panel, &font.0, "ابدأ المهمة", MenuAction::Start, true);
                spawn_menu_button(panel, &font.0, "طريقة اللعب", MenuAction::Help, false);
                spawn_menu_button(panel, &font.0, "خروج", MenuAction::Quit, false);
                panel.spawn((
                    Text::new("Critical Point Game Jam 2026"),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.62, 0.74, 0.78, 0.72)),
                ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(560.0),
                    max_width: Val::Percent(88.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.025, 0.055, 0.075, 0.98)),
                BorderColor::all(Color::srgba(0.36, 0.86, 0.78, 0.72)),
                Visibility::Hidden,
                HelpPanel,
            ))
            .with_children(|help| {
                help.spawn((
                    Text::new("WASD للحركة  |  Shift للركض  |  Space للقفز\nالفأرة اليسرى للحفر  |  اليمنى لوضع دعم\nE للتفاعل  |  Esc للإيقاف\n\nاستخرج ثلاث شظايا، لكن تذكّر: كل حفر يزعزع العالم."),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.88, 0.96, 0.94)),
                    TextLayout::new_with_justify(Justify::Center),
                ));
                spawn_menu_button(help, &font.0, "فهمت", MenuAction::Help, true);
            });
        });
}

fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &'static str,
    action: MenuAction,
    primary: bool,
) {
    let color = if primary {
        Color::srgba(0.08, 0.64, 0.54, 0.96)
    } else {
        Color::srgba(0.08, 0.20, 0.25, 0.94)
    };
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(86.0),
                height: Val::Px(52.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(color),
            BorderColor::all(Color::srgba(0.48, 0.96, 0.86, 0.52)),
            action,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 21.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn handle_menu_buttons(
    mut buttons: Query<
        (&Interaction, &MenuAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut help_panel: Query<&mut Visibility, With<HelpPanel>>,
    mut lifecycle: ResMut<RunLifecycle>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, action, mut background) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                background.0 = Color::srgb(0.16, 0.84, 0.68);
                match action {
                    MenuAction::Start => {
                        lifecycle.active = false;
                        next_state.set(AppState::Playing);
                    }
                    MenuAction::Help => {
                        if let Ok(mut visibility) = help_panel.single_mut() {
                            *visibility = match *visibility {
                                Visibility::Hidden => Visibility::Visible,
                                _ => Visibility::Hidden,
                            };
                        }
                    }
                    MenuAction::Quit => {
                        exit.write(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => {
                background.0 = Color::srgba(0.12, 0.48, 0.44, 0.98);
            }
            Interaction::None => {
                background.0 = if *action == MenuAction::Start {
                    Color::srgba(0.08, 0.64, 0.54, 0.96)
                } else {
                    Color::srgba(0.08, 0.20, 0.25, 0.94)
                };
            }
        }
    }
}

pub fn cleanup_screen(mut commands: Commands, screens: Query<Entity, With<ScreenRoot>>) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}