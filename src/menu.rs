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

#[derive(Resource, Default)]
pub struct PendingRoute(pub Option<PlanetRoute>);

#[derive(Component)]
pub struct HelpPanel;
#[derive(Component)]
pub struct RouteSummary;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Start,
    LoadGame,
    Help,
    Quit,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum RouteAction {
    SelectHome,
    SelectAlien,
    Confirm,
    Back,
}

pub fn setup_loading_screen(
    mut commands: Commands,
    font: Res<ArabicFont>,
    mut timer: ResMut<LoadingTimer>,
) {
    timer.0.reset();
    commands
        .spawn(screen_root(Color::srgb(0.018, 0.035, 0.052), 90))
        .with_children(|root| {
            root.spawn(text(
                &font.0,
                "نقطة العبور",
                54.0,
                Color::srgb(0.62, 1.0, 0.92),
            ));
            root.spawn((
                text(
                    &font.0,
                    "تهيئة منظومة الدفاع الكوكبي...",
                    18.0,
                    Color::srgb(0.70, 0.82, 0.86),
                ),
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
    mut timer: ResMut<LoadingTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        next_state.set(AppState::MainMenu);
    }
}

pub fn setup_main_menu(mut commands: Commands, font: Res<ArabicFont>) {
    commands.spawn(screen_root(Color::srgba(0.01, 0.025, 0.04, 0.58), 80)).with_children(|root| {
        root.spawn(panel(640.0)).with_children(|panel| {
            panel.spawn(text(&font.0, "نقطة العبور", 50.0, Color::srgb(0.66, 1.0, 0.92)));
            panel.spawn(text(
                &font.0,
                "قرار واحد يفصل بين البقاء في الوطن والسفر إلى كوكب الحرب",
                19.0,
                Color::srgb(0.78, 0.88, 0.90),
            ));
            spawn_button(panel, &font.0, "ابدأ مهمة جديدة", MenuAction::Start, true);
            spawn_button(panel, &font.0, "📂 تحميل اللعبة المحفوظة", MenuAction::LoadGame, false);
            spawn_button(panel, &font.0, "طريقة اللعب", MenuAction::Help, false);
            spawn_button(panel, &font.0, "خروج", MenuAction::Quit, false);
            panel.spawn(text(
                &font.0,
                "Critical Point Game Jam 2026",
                12.0,
                Color::srgba(0.62, 0.74, 0.78, 0.72),
            ));
        });

        root.spawn((
            panel(650.0),
            Visibility::Hidden,
            HelpPanel,
            ZIndex(84),
        )).with_children(|help| {
            help.spawn(text(&font.0, "التحكم", 34.0, Color::srgb(0.66, 1.0, 0.92)));
            help.spawn(text(
                &font.0,
                "WASD حركة  |  Shift ركض  |  Space قفز\n1–3 الأسلحة  |  4 الحفار  |  5 البناء\nالفأرة اليسرى استخدام  |  اليمنى تثبيت التصويب\nR مطولاً صناعة أو تطوير  |  العجلة اختيار البلوك/تقريب\nE تفاعل  |  Esc إيقاف مؤقت",
                18.0,
                Color::srgb(0.88, 0.96, 0.94),
            ));
            spawn_button(help, &font.0, "فهمت", MenuAction::Help, true);
        });
    });
}

pub fn setup_route_choice(
    mut commands: Commands,
    font: Res<ArabicFont>,
    mut pending: ResMut<PendingRoute>,
) {
    pending.0 = None;
    commands.spawn(screen_root(Color::srgba(0.008, 0.018, 0.032, 0.86), 82)).with_children(|root| {
        root.spawn(panel(900.0)).with_children(|panel| {
            panel.spawn(text(&font.0, "هذه هي النقطة الحاسمة", 42.0, Color::srgb(0.72, 1.0, 0.94)));
            panel.spawn(text(
                &font.0,
                "اختر المسار. لا يمكن تغييره قبل نهاية الجولة.",
                18.0,
                Color::srgb(0.78, 0.86, 0.90),
            ));
            spawn_route_button(
                panel,
                &font.0,
                "البقاء في الكوكب الأصلي\nحرية بناء واستكشاف مع دوريات فضائية ضعيفة وغارات محدودة",
                RouteAction::SelectHome,
                Color::srgba(0.07, 0.34, 0.30, 0.98),
            );
            spawn_route_button(
                panel,
                &font.0,
                "السفر إلى كوكب الحرب\nحملة قتالية 12–18 دقيقة — أبراج غزو، أعداء أرضيون وجويون وزعيم",
                RouteAction::SelectAlien,
                Color::srgba(0.38, 0.10, 0.18, 0.98),
            );
            panel.spawn((
                text(&font.0, "اختر أحد المسارين لعرض التأكيد", 17.0, Color::srgb(0.72, 0.82, 0.86)),
                RouteSummary,
            ));
            spawn_route_button(panel, &font.0, "تأكيد القرار", RouteAction::Confirm, Color::srgba(0.08, 0.58, 0.50, 0.98));
            spawn_route_button(panel, &font.0, "رجوع", RouteAction::Back, Color::srgba(0.07, 0.15, 0.20, 0.96));
        });
    });
}

pub fn handle_menu_buttons(
    mut buttons: Query<
        (&Interaction, &MenuAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut help_panel: Query<&mut Visibility, With<HelpPanel>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut save_state: ResMut<crate::save_load::SaveLoadState>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, action, mut background) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                background.0 = Color::srgb(0.16, 0.84, 0.68);
                match action {
                    MenuAction::Start => next_state.set(AppState::RouteChoice),
                    MenuAction::LoadGame => {
                        save_state.load_requested = true;
                    }
                    MenuAction::Help => {
                        if let Ok(mut visibility) = help_panel.single_mut() {
                            *visibility = if *visibility == Visibility::Hidden {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };
                        }
                    }
                    MenuAction::Quit => {
                        exit.write(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => background.0 = Color::srgba(0.12, 0.48, 0.44, 0.98),
            Interaction::None => {
                background.0 = if *action == MenuAction::Start {
                    Color::srgba(0.08, 0.64, 0.54, 0.96)
                } else {
                    Color::srgba(0.08, 0.20, 0.25, 0.94)
                }
            }
        }
    }
}

pub fn handle_route_buttons(
    mut buttons: Query<
        (&Interaction, &RouteAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut pending: ResMut<PendingRoute>,
    mut summaries: Query<&mut Text, With<RouteSummary>>,
    mut session: ResMut<GameSession>,
    mut lifecycle: ResMut<RunLifecycle>,
    mut committed: MessageWriter<RouteCommitted>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, action, mut background) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                background.0 = Color::srgb(0.18, 0.82, 0.68);
                match action {
                    RouteAction::SelectHome => pending.0 = Some(PlanetRoute::HomeDefense),
                    RouteAction::SelectAlien => pending.0 = Some(PlanetRoute::InvadedPlanet),
                    RouteAction::Confirm => {
                        if let Some(route) = pending.0 {
                            session.route = route;
                            lifecycle.active = false;
                            committed.write(RouteCommitted(route));
                            next_state.set(AppState::Playing);
                        }
                    }
                    RouteAction::Back => next_state.set(AppState::MainMenu),
                }
                if let Ok(mut summary) = summaries.single_mut() {
                    summary.0 = match pending.0 {
                        Some(PlanetRoute::HomeDefense) => {
                            "القرار المحدد: البقاء في الكوكب الأصلي — خطر منخفض"
                        }
                        Some(PlanetRoute::InvadedPlanet) => {
                            "القرار المحدد: السفر إلى كوكب الحرب — خطر مرتفع"
                        }
                        _ => "اختر أحد المسارين لعرض التأكيد",
                    }
                    .into();
                }
            }
            Interaction::Hovered => background.0 = Color::srgba(0.15, 0.42, 0.40, 0.98),
            Interaction::None => {}
        }
    }
}

pub fn cleanup_screen(mut commands: Commands, screens: Query<Entity, With<ScreenRoot>>) {
    for entity in &screens {
        if let Ok(mut cmd) = commands.get_entity(entity) {
            cmd.despawn();
        }
    }
}

fn screen_root(color: Color, z: i32) -> (Node, BackgroundColor, ZIndex, ScreenRoot) {
    (
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(color),
        ZIndex(z),
        ScreenRoot,
    )
}

fn panel(width: f32) -> (Node, BackgroundColor, BorderColor) {
    (
        Node {
            width: Val::Px(width),
            max_width: Val::Percent(94.0),
            padding: UiRect::all(Val::Px(30.0)),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.018, 0.052, 0.072, 0.97)),
        BorderColor::all(Color::srgba(0.34, 0.95, 0.82, 0.66)),
    )
}

fn text(
    font: &Handle<Font>,
    label: &str,
    size: f32,
    color: Color,
) -> (Text, TextFont, TextColor, TextLayout) {
    (
        Text::new(label),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
        TextLayout::new_with_justify(Justify::Center),
    )
}

fn spawn_button(
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
            button.spawn(text(font, label, 20.0, Color::WHITE));
        });
}

fn spawn_route_button(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &'static str,
    action: RouteAction,
    color: Color,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(94.0),
                min_height: Val::Px(72.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(color),
            BorderColor::all(Color::srgba(0.48, 0.96, 0.86, 0.50)),
            action,
        ))
        .with_children(|button| {
            button.spawn(text(font, label, 18.0, Color::WHITE));
        });
}
