use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::WindowFocused;

use crate::gameplay::RunLifecycle;
use crate::state::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAction {
    Resume,
    Restart,
    MainMenu,
    Quit,
    ChooseEvacuate,
    ChooseStabilize,
    ToggleMotion,
    VolumeDown,
    VolumeUp,
}

#[derive(Component)]
pub struct SettingsSummary;

pub fn setup_hud(mut commands: Commands, font: Res<ArabicFont>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
            ZIndex(40),
            HudRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(18.0),
                    left: Val::Percent(50.0),
                    width: Val::Px(460.0),
                    max_width: Val::Percent(48.0),
                    height: Val::Px(68.0),
                    margin: UiRect::left(Val::Px(-230.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.015, 0.04, 0.055, 0.88)),
                BorderColor::all(Color::srgba(0.52, 0.96, 0.86, 0.62)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("استقرار العالم"),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.86, 0.96, 0.94)),
                    RiskText,
                ));
                panel
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(15.0),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.02, 0.08, 0.10, 0.96)),
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            Node {
                                width: Val::Percent(10.0),
                                height: Val::Percent(100.0),
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.22, 0.88, 0.70)),
                            RiskFill,
                        ));
                    });
            });

            root.spawn((
                Text::new(""),
                TextFont {
                    font: font.0.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.94, 0.98, 0.96)),
                TextLayout::new_with_justify(Justify::Right),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(24.0),
                    right: Val::Px(24.0),
                    width: Val::Px(340.0),
                    max_width: Val::Percent(34.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.015, 0.04, 0.055, 0.78)),
                ObjectiveText,
            ));

            root.spawn((
                Text::new(""),
                TextFont {
                    font: font.0.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.94, 0.92)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(24.0),
                    bottom: Val::Px(22.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.015, 0.04, 0.055, 0.78)),
                InventoryText,
            ));

            root.spawn((
                Text::new(""),
                TextFont {
                    font: font.0.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.82, 0.38)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Px(24.0),
                    margin: UiRect::left(Val::Px(-80.0)),
                    ..default()
                },
                TimerText,
            ));
        });
}

pub fn update_hud(
    state: Res<State<AppState>>,
    session: Res<GameSession>,
    mut hud: Query<&mut Visibility, With<HudRoot>>,
    mut risk_fill: Query<(&mut Node, &mut BackgroundColor), With<RiskFill>>,
    mut risk_text: Query<&mut Text, (With<RiskText>, Without<ObjectiveText>, Without<InventoryText>, Without<TimerText>)>,
    mut objective: Query<&mut Text, (With<ObjectiveText>, Without<RiskText>, Without<InventoryText>, Without<TimerText>)>,
    mut inventory: Query<&mut Text, (With<InventoryText>, Without<RiskText>, Without<ObjectiveText>, Without<TimerText>)>,
    mut timer: Query<&mut Text, (With<TimerText>, Without<RiskText>, Without<ObjectiveText>, Without<InventoryText>)>,
) {
    if let Ok(mut visibility) = hud.single_mut() {
        *visibility = if matches!(
            state.get(),
            AppState::Playing | AppState::Paused | AppState::Decision
        ) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok((mut node, mut color)) = risk_fill.single_mut() {
        node.width = Val::Percent(session.criticality);
        color.0 = match session.risk_band() {
            RiskBand::Calm => Color::srgb(0.20, 0.88, 0.68),
            RiskBand::Warning => Color::srgb(1.0, 0.63, 0.18),
            RiskBand::Critical => Color::srgb(0.92, 0.16, 0.24),
            RiskBand::Terminal => Color::srgb(1.0, 0.03, 0.08),
        };
    }
    if let Ok(mut text) = risk_text.single_mut() {
        text.0 = format!("الخطر البنيوي: {:.0}%", session.criticality);
    }
    if let Ok(mut text) = objective.single_mut() {
        text.0 = format!("الهدف\n{}", session.objective_hint);
    }
    if let Ok(mut text) = inventory.single_mut() {
        text.0 = format!(
            "الشظايا: {}/3   |   كتل الدعم: {}\nالفأرة اليسرى: حفر   |   اليمنى: دعم",
            session.crystals, session.supports
        );
    }
    if let Ok(mut text) = timer.single_mut() {
        text.0 = session.phase_time_remaining.map_or_else(String::new, |remaining| {
            let seconds = remaining.ceil() as u32;
            format!("الوقت المتبقي  {:02}:{:02}", seconds / 60, seconds % 60)
        });
    }
}

pub fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    match state.get() {
        AppState::Playing => next_state.set(AppState::Paused),
        AppState::Paused => next_state.set(AppState::Playing),
        _ => {}
    }
}

pub fn pause_when_unfocused(
    mut focused: MessageReader<WindowFocused>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for event in focused.read() {
        if !event.focused && *state.get() == AppState::Playing {
            next_state.set(AppState::Paused);
        }
    }
}

pub fn setup_pause_overlay(mut commands: Commands, font: Res<ArabicFont>, prefs: Res<GamePreferences>) {
    commands
        .spawn(screen_container(Color::srgba(0.01, 0.02, 0.035, 0.72)))
        .insert((ScreenRoot, ZIndex(75)))
        .with_children(|root| {
            root.spawn(panel_node(480.0)).with_children(|panel| {
                spawn_title(panel, &font.0, "توقف مؤقت", 38.0);
                spawn_action_button(panel, &font.0, "متابعة", OverlayAction::Resume, true);
                spawn_action_button(panel, &font.0, "إعادة المهمة", OverlayAction::Restart, false);
                spawn_action_button(panel, &font.0, "القائمة الرئيسية", OverlayAction::MainMenu, false);
                spawn_action_button(panel, &font.0, "خفض الصوت", OverlayAction::VolumeDown, false);
                spawn_action_button(panel, &font.0, "رفع الصوت", OverlayAction::VolumeUp, false);
                spawn_action_button(panel, &font.0, "تقليل الاهتزاز والوميض", OverlayAction::ToggleMotion, false);
                panel.spawn((
                    Text::new(settings_text(&prefs)),
                    TextFont { font: font.0.clone(), font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.68, 0.84, 0.82)),
                    SettingsSummary,
                ));
                spawn_action_button(panel, &font.0, "خروج", OverlayAction::Quit, false);
            });
        });
}

pub fn setup_decision_overlay(mut commands: Commands, font: Res<ArabicFont>) {
    commands
        .spawn(screen_container(Color::srgba(0.10, 0.015, 0.035, 0.76)))
        .insert((ScreenRoot, ZIndex(78)))
        .with_children(|root| {
            root.spawn(panel_node(760.0)).with_children(|panel| {
                spawn_title(panel, &font.0, "بلغ العالم النقطة الحرجة", 38.0);
                panel.spawn((
                    Text::new("الشظايا الثلاث تكفي لمسار واحد فقط. لا رجعة بعد اختيارك."),
                    TextFont { font: font.0.clone(), font_size: 19.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.84, 0.76)),
                    TextLayout::new_with_justify(Justify::Center),
                ));
                spawn_action_button(panel, &font.0, "إخلاء المستعمرة — 90 ثانية، العالم سيُفقد", OverlayAction::ChooseEvacuate, true);
                spawn_action_button(panel, &font.0, "تثبيت النواة — 120 ثانية، خطر مضاعف", OverlayAction::ChooseStabilize, false);
            });
        });
}

pub fn setup_ending_overlay(
    mut commands: Commands,
    font: Res<ArabicFont>,
    session: Res<GameSession>,
) {
    let (title, body, color) = match session.outcome {
        RunOutcome::PeopleSaved => (
            "نجا الناس",
            "أضاءت سفن الإخلاء السماء، بينما اختفى العالم خلفها.\nقرار آمن... لكنه لم يكن بلا ثمن.",
            Color::srgb(0.32, 0.86, 1.0),
        ),
        RunOutcome::WorldSaved => (
            "تغيّر المصير",
            "ثبتت الشظايا قلب الصدع قبل اللحظة الأخيرة.\nنجا الناس والعالم لأنك خاطرت بكل شيء.",
            Color::srgb(0.44, 1.0, 0.68),
        ),
        _ => (
            "انهيار كامل",
            "وصل الخطر إلى مئة بالمئة. ابتلع الصدع كل الاحتمالات.\nربما يكون القرار أسرع في المحاولة القادمة.",
            Color::srgb(1.0, 0.24, 0.28),
        ),
    };
    commands
        .spawn(screen_container(Color::srgba(0.01, 0.02, 0.035, 0.82)))
        .insert((ScreenRoot, ZIndex(82)))
        .with_children(|root| {
            root.spawn(panel_node(660.0)).with_children(|panel| {
                panel.spawn((
                    Text::new(title),
                    TextFont { font: font.0.clone(), font_size: 46.0, ..default() },
                    TextColor(color),
                    TextLayout::new_with_justify(Justify::Center),
                ));
                panel.spawn((
                    Text::new(body),
                    TextFont { font: font.0.clone(), font_size: 20.0, ..default() },
                    TextColor(Color::srgb(0.88, 0.94, 0.92)),
                    TextLayout::new_with_justify(Justify::Center),
                ));
                panel.spawn((
                    Text::new(format!("الوقت: {}:{:02}   |   الخطر النهائي: {:.0}%", (session.elapsed as u32) / 60, (session.elapsed as u32) % 60, session.criticality)),
                    TextFont { font: font.0.clone(), font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.70, 0.82, 0.82)),
                ));
                spawn_action_button(panel, &font.0, "إعادة اللعب", OverlayAction::Restart, true);
                spawn_action_button(panel, &font.0, "القائمة الرئيسية", OverlayAction::MainMenu, false);
                spawn_action_button(panel, &font.0, "خروج", OverlayAction::Quit, false);
            });
        });
}

pub fn handle_overlay_buttons(
    mut buttons: Query<
        (&Interaction, &OverlayAction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut session: ResMut<GameSession>,
    balance: Res<BalanceConfig>,
    mut prefs: ResMut<GamePreferences>,
    mut lifecycle: ResMut<RunLifecycle>,
    mut next_state: ResMut<NextState<AppState>>,
    mut choices: MessageWriter<ChoiceCommitted>,
    mut exit: MessageWriter<AppExit>,
    mut settings: Query<&mut Text, With<SettingsSummary>>,
) {
    for (interaction, action, mut background) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                background.0 = Color::srgb(0.18, 0.82, 0.68);
                match action {
                    OverlayAction::Resume => next_state.set(AppState::Playing),
                    OverlayAction::Restart => {
                        lifecycle.active = false;
                        next_state.set(AppState::Playing);
                    }
                    OverlayAction::MainMenu => {
                        lifecycle.active = false;
                        next_state.set(AppState::MainMenu);
                    }
                    OverlayAction::Quit => {
                        exit.write(AppExit::Success);
                    }
                    OverlayAction::ChooseEvacuate => {
                        session.choose(CriticalChoice::Evacuate);
                        session.phase_time_remaining = Some(balance.evacuation_seconds);
                        choices.write(ChoiceCommitted(CriticalChoice::Evacuate));
                        next_state.set(AppState::Playing);
                    }
                    OverlayAction::ChooseStabilize => {
                        session.choose(CriticalChoice::Stabilize);
                        session.phase_time_remaining = Some(balance.stabilization_seconds);
                        choices.write(ChoiceCommitted(CriticalChoice::Stabilize));
                        next_state.set(AppState::Playing);
                    }
                    OverlayAction::ToggleMotion => prefs.reduced_motion = !prefs.reduced_motion,
                    OverlayAction::VolumeDown => prefs.master_volume = (prefs.master_volume - 0.1).max(0.0),
                    OverlayAction::VolumeUp => prefs.master_volume = (prefs.master_volume + 0.1).min(1.0),
                }
                if let Ok(mut text) = settings.single_mut() {
                    text.0 = settings_text(&prefs);
                }
            }
            Interaction::Hovered => background.0 = Color::srgba(0.14, 0.46, 0.42, 0.98),
            Interaction::None => background.0 = Color::srgba(0.07, 0.20, 0.25, 0.96),
        }
    }
}

fn screen_container(color: Color) -> (Node, BackgroundColor) {
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
    )
}

fn panel_node(width: f32) -> (Node, BackgroundColor, BorderColor) {
    (
        Node {
            width: Val::Px(width),
            max_width: Val::Percent(92.0),
            padding: UiRect::all(Val::Px(28.0)),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(18.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(14.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.025, 0.06, 0.08, 0.97)),
        BorderColor::all(Color::srgba(0.42, 0.92, 0.82, 0.66)),
    )
}

fn spawn_title(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, label: &'static str, size: f32) {
    parent.spawn((
        Text::new(label),
        TextFont { font: font.clone(), font_size: size, ..default() },
        TextColor(Color::srgb(0.68, 1.0, 0.92)),
        TextLayout::new_with_justify(Justify::Center),
    ));
}

fn spawn_action_button(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &'static str,
    action: OverlayAction,
    primary: bool,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(92.0),
                min_height: Val::Px(48.0),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(11.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(if primary {
                Color::srgba(0.08, 0.62, 0.52, 0.97)
            } else {
                Color::srgba(0.07, 0.20, 0.25, 0.96)
            }),
            BorderColor::all(Color::srgba(0.42, 0.92, 0.82, 0.54)),
            action,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont { font: font.clone(), font_size: 18.0, ..default() },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

fn settings_text(prefs: &GamePreferences) -> String {
    format!(
        "الصوت: {:.0}%   |   تقليل الحركة: {}",
        prefs.master_volume * 100.0,
        if prefs.reduced_motion { "مفعّل" } else { "متوقف" }
    )
}