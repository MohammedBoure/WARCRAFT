use astra_voxel_world::prelude::BlockKind;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::WindowFocused;

use crate::combat::Enemy;
use crate::gameplay::RunLifecycle;
use crate::state::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAction {
    Resume,
    SaveGame,
    LoadGame,
    Restart,
    ChooseRoute,
    MainMenu,
    Quit,
    Extract,
    AssaultGate,
    ToggleMotion,
    VolumeDown,
    VolumeUp,
}
#[derive(Component)]
pub struct SettingsSummary;

pub fn setup_hud(mut commands: Commands, font: Res<ArabicFont>) {
    commands.spawn((
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
    )).with_children(|root| {
        root.spawn((
            hud_panel(310.0),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(310.0),
                padding: UiRect::all(Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(7.0),
                ..default()
            },
        )).with_children(|panel| {
            panel.spawn((label(&font.0, "الصحة: 100 / 100", 14.0, Color::srgb(0.86, 0.94, 0.94)), PlayerHealthText));
            spawn_bar(panel, HealthFill, Color::srgb(0.92, 0.20, 0.22), 100.0);
            panel.spawn((label(&font.0, "الدرع: 50 / 50", 14.0, Color::srgb(0.74, 0.90, 1.0)), PlayerShieldText));
            spawn_bar(panel, ShieldFill, Color::srgb(0.10, 0.66, 1.0), 100.0);
            panel.spawn((label(&font.0, "", 14.0, Color::srgb(0.76, 0.88, 0.90)), BaseHealthText));
        });

        // Targeted Enemy HP Panel (Top-Center)
        root.spawn((
            Name::new("Targeted Enemy HP Panel"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(68.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-200.0)),
                width: Val::Px(400.0),
                padding: UiRect::all(Val::Px(9.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.012, 0.032, 0.048, 0.88)),
            Visibility::Hidden,
            TargetEnemyPanelRoot,
        )).with_children(|panel| {
            panel.spawn((
                label(&font.0, "", 16.0, Color::srgb(1.0, 0.35, 0.35)),
                TextLayout::new_with_justify(Justify::Center),
                TargetEnemyText,
            ));
        });

        root.spawn((
            label(&font.0, "", 19.0, Color::WHITE),
            TextLayout::new_with_justify(Justify::Right),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                width: Val::Px(390.0),
                max_width: Val::Percent(38.0),
                padding: UiRect::all(Val::Px(13.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.012, 0.032, 0.048, 0.84)),
            ObjectiveText,
        ));

        root.spawn((
            label(&font.0, "", 16.0, Color::srgb(0.80, 0.92, 0.92)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                bottom: Val::Px(20.0),
                width: Val::Px(340.0),
                padding: UiRect::all(Val::Px(11.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.012, 0.032, 0.048, 0.84)),
            ResourceText,
        ));

        root.spawn((
            label(&font.0, "", 17.0, Color::WHITE),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                bottom: Val::Px(20.0),
                width: Val::Px(720.0),
                max_width: Val::Percent(58.0),
                margin: UiRect::left(Val::Px(-360.0)),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.012, 0.032, 0.048, 0.90)),
            HotbarText,
        ));

        root.spawn((
            label(&font.0, "", 22.0, Color::srgb(1.0, 0.78, 0.28)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(24.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-70.0)),
                ..default()
            },
            TimerText,
        ));
    });
}

fn spawn_bar(parent: &mut ChildSpawnerCommands, marker: impl Component, color: Color, width: f32) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(11.0),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.01, 0.06, 0.08, 0.96)),
    )).with_children(|bar| {
        bar.spawn((
            Node {
                width: Val::Percent(width),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(color),
            marker,
        ));
    });
}

pub fn update_hud(
    state: Res<State<AppState>>,
    session: Res<GameSession>,
    aim: Res<AimSolution>,
    enemies: Query<&Enemy>,
    mut hud: Query<&mut Visibility, (With<HudRoot>, Without<TargetEnemyPanelRoot>)>,
    mut health: Query<&mut Node, (With<HealthFill>, Without<ShieldFill>)>,
    mut shield: Query<&mut Node, (With<ShieldFill>, Without<HealthFill>)>,
    mut player_hp_text: Query<&mut Text, (With<PlayerHealthText>, Without<PlayerShieldText>, Without<TargetEnemyText>, Without<ObjectiveText>, Without<ResourceText>, Without<HotbarText>, Without<TimerText>, Without<BaseHealthText>)>,
    mut player_sh_text: Query<&mut Text, (With<PlayerShieldText>, Without<PlayerHealthText>, Without<TargetEnemyText>, Without<ObjectiveText>, Without<ResourceText>, Without<HotbarText>, Without<TimerText>, Without<BaseHealthText>)>,
    mut target_panel: Query<&mut Visibility, (With<TargetEnemyPanelRoot>, Without<HudRoot>)>,
    mut target_text: Query<&mut Text, (With<TargetEnemyText>, Without<PlayerHealthText>, Without<PlayerShieldText>, Without<ObjectiveText>, Without<ResourceText>, Without<HotbarText>, Without<TimerText>, Without<BaseHealthText>)>,
    mut objective: Query<&mut Text, (With<ObjectiveText>, Without<ResourceText>, Without<HotbarText>, Without<TimerText>, Without<BaseHealthText>, Without<PlayerHealthText>, Without<PlayerShieldText>, Without<TargetEnemyText>)>,
    mut resources: Query<&mut Text, (With<ResourceText>, Without<ObjectiveText>, Without<HotbarText>, Without<TimerText>, Without<BaseHealthText>, Without<PlayerHealthText>, Without<PlayerShieldText>, Without<TargetEnemyText>)>,
    mut hotbar: Query<&mut Text, (With<HotbarText>, Without<ObjectiveText>, Without<ResourceText>, Without<TimerText>, Without<BaseHealthText>, Without<PlayerHealthText>, Without<PlayerShieldText>, Without<TargetEnemyText>)>,
    mut timer: Query<&mut Text, (With<TimerText>, Without<ObjectiveText>, Without<ResourceText>, Without<HotbarText>, Without<BaseHealthText>, Without<PlayerHealthText>, Without<PlayerShieldText>, Without<TargetEnemyText>)>,
    mut base: Query<&mut Text, (With<BaseHealthText>, Without<ObjectiveText>, Without<ResourceText>, Without<HotbarText>, Without<TimerText>, Without<PlayerHealthText>, Without<PlayerShieldText>, Without<TargetEnemyText>)>,
) {
    if let Ok(mut visibility) = hud.single_mut() {
        *visibility = if matches!(state.get(), AppState::Playing | AppState::Paused | AppState::FinalDecision) {
            Visibility::Visible
        } else { Visibility::Hidden };
    }
    if let Ok(mut node) = health.single_mut() { node.width = Val::Percent(session.loadout.health.clamp(0.0, 100.0)); }
    if let Ok(mut node) = shield.single_mut() { node.width = Val::Percent((session.loadout.shield * 2.0).clamp(0.0, 100.0)); }
    if let Ok(mut text) = player_hp_text.single_mut() {
        text.0 = format!("الصحة: {:.0} / 100", session.loadout.health.clamp(0.0, 100.0));
    }
    if let Ok(mut text) = player_sh_text.single_mut() {
        text.0 = format!("الدرع: {:.0} / 50", session.loadout.shield.clamp(0.0, 50.0));
    }
    let mut show_target = false;
    if let Some(target_entity) = aim.enemy {
        if let Ok(enemy) = enemies.get(target_entity) {
            show_target = true;
            if let Ok(mut text) = target_text.single_mut() {
                let name = enemy.kind.arabic_name();
                if enemy.shield > 0.0 {
                    text.0 = format!(
                        "🎯 {}\nالدرع: {:.0} / {:.0}   |   الصحة: {:.0} / {:.0}",
                        name, enemy.shield, enemy.max_shield, enemy.health, enemy.max_health
                    );
                } else {
                    text.0 = format!(
                        "🎯 {}\nالصحة: {:.0} / {:.0}",
                        name, enemy.health, enemy.max_health
                    );
                }
            }
        }
    }
    if let Ok(mut vis) = target_panel.single_mut() {
        *vis = if show_target { Visibility::Visible } else { Visibility::Hidden };
    }
    if let Ok(mut text) = objective.single_mut() {
        text.0 = format!(
            "الهدف\n{}\nالموجة: {}   |   الأعداء: {}   |   الأبراج: {}/3",
            session.objective_hint, session.wave, session.active_enemies, session.relays_destroyed
        );
    }
    if let Ok(mut text) = resources.single_mut() {
        let line = ResourceKind::ALL.into_iter().map(|kind| {
            format!("{}: {}", kind.arabic_name(), session.loadout.resource_count(kind))
        }).collect::<Vec<_>>().join("  |  ");
        text.0 = format!(
            "{}\nبلوك البناء: {} × {}",
            line,
            block_name(session.loadout.selected_block),
            session.loadout.block_count(session.loadout.selected_block)
        );
    }
    if let Ok(mut text) = hotbar.single_mut() {
        let selected = session.loadout.selected_tool;
        let slot = |number: u8, tool: ToolSlot, name: &str| {
            if selected == tool { format!("[{number}: {name}]") } else { format!("{number}: {name}") }
        };
        let weapon = match selected {
            ToolSlot::Weapon(kind) => {
                let level = session.loadout.weapon_level(kind);
                if level == 0 { format!("مقفل — اضغط R مطولاً للصناعة: {}", kind.arabic_name()) }
                else if level < 3 { format!("{} مستوى {} — R مطولاً للتطوير", kind.arabic_name(), level) }
                else { format!("{} — المستوى الأقصى", kind.arabic_name()) }
            }
            ToolSlot::MiningLaser => "استمر بالنقر للحفر وجمع البلوك أو الخام".into(),
            ToolSlot::Builder => "العجلة تغيّر نوع البلوك — النقر يضع البلوك".into(),
        };
        text.0 = format!(
            "{}   {}   {}   {}   {}   {}   {}\n{}   |   الحرارة {:.0}%",
            slot(1, ToolSlot::Weapon(WeaponKind::PulseRifle), "نبض"),
            slot(2, ToolSlot::Weapon(WeaponKind::PlasmaMortar), "بلازما"),
            slot(3, ToolSlot::Weapon(WeaponKind::IonLance), "أيون"),
            slot(4, ToolSlot::Weapon(WeaponKind::QuantumTesla), "تسلا"),
            slot(5, ToolSlot::Weapon(WeaponKind::NukeMortar), "نووي"),
            slot(6, ToolSlot::MiningLaser, "حفر"),
            slot(7, ToolSlot::Builder, "بناء"),
            weapon,
            session.loadout.heat * 100.0
        );
    }
    if let Ok(mut text) = timer.single_mut() {
        text.0 = session.phase_time_remaining.map_or_else(String::new, |remaining| {
            let seconds = remaining.ceil() as u32;
            format!("{:02}:{:02}", seconds / 60, seconds % 60)
        });
    }
    if let Ok(mut text) = base.single_mut() {
        text.0 = if matches!(session.phase, MissionPhase::HomePreparation | MissionPhase::HomeDefense | MissionPhase::Extraction) {
            format!("سلامة الهدف: {:.0} / 500", session.base_health)
        } else { String::new() };
    }
}

pub fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) { return; }
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
    commands.spawn(screen(Color::srgba(0.005, 0.012, 0.022, 0.76), 76)).with_children(|root| {
        root.spawn(panel(500.0)).with_children(|panel| {
            panel.spawn(label(&font.0, "توقف مؤقت", 38.0, Color::srgb(0.68, 1.0, 0.92)));
            spawn_action(panel, &font.0, "متابعة", OverlayAction::Resume, true);
            spawn_action(panel, &font.0, "💾 حفظ اللعبة", OverlayAction::SaveGame, false);
            spawn_action(panel, &font.0, "📂 تحميل اللعبة", OverlayAction::LoadGame, false);
            spawn_action(panel, &font.0, "إعادة الجولة", OverlayAction::Restart, false);
            spawn_action(panel, &font.0, "اختيار مسار آخر", OverlayAction::ChooseRoute, false);
            panel.spawn((label(&font.0, &settings_text(&prefs), 15.0, Color::srgb(0.72, 0.84, 0.86)), SettingsSummary));
            spawn_action(panel, &font.0, "خفض الصوت", OverlayAction::VolumeDown, false);
            spawn_action(panel, &font.0, "رفع الصوت", OverlayAction::VolumeUp, false);
            spawn_action(panel, &font.0, "تقليل الاهتزاز", OverlayAction::ToggleMotion, false);
            spawn_action(panel, &font.0, "القائمة الرئيسية", OverlayAction::MainMenu, false);
        });
    });
}

pub fn setup_final_decision_overlay(mut commands: Commands, font: Res<ArabicFont>) {
    commands.spawn(screen(Color::srgba(0.015, 0.005, 0.025, 0.84), 82)).with_children(|root| {
        root.spawn(panel(720.0)).with_children(|panel| {
            panel.spawn(label(&font.0, "القرار الأخير", 44.0, Color::srgb(1.0, 0.42, 0.62)));
            panel.spawn(label(
                &font.0,
                "انهارت أبراج الغزو. أمامك لحظات قبل أن تستعيد البوابة طاقتها.",
                19.0,
                Color::srgb(0.88, 0.92, 0.94),
            ));
            spawn_action(panel, &font.0, "استخراج الموارد — دافع عن السفينة 90 ثانية", OverlayAction::Extract, true);
            spawn_action(panel, &font.0, "اقتحام البوابة — الزعيم و180 ثانية، الموت نهائي", OverlayAction::AssaultGate, false);
        });
    });
}

pub fn setup_ending_overlay(mut commands: Commands, font: Res<ArabicFont>, session: Res<GameSession>) {
    let (title, body, color) = match session.outcome {
        RunOutcome::HomeDefended => ("صمد الوطن", "نجت المستعمرة وانسحب آخر الكشافة الفضائيين.", Color::srgb(0.38, 1.0, 0.72)),
        RunOutcome::Extracted => ("استخراج ناجح", "عدت بالموارد والمعرفة، لكن بوابة الغزو ما زالت مفتوحة.", Color::srgb(0.30, 0.82, 1.0)),
        RunOutcome::GateDestroyed => ("تغيّر مسار الحرب", "دُمّرت حاملة البوابة وتوقف الغزو عند نقطته الحاسمة.", Color::srgb(0.72, 0.46, 1.0)),
        _ => ("فشلت المهمة", "سقط خط الدفاع. استخدم الموارد والبناء بحذر أكبر في المحاولة التالية.", Color::srgb(1.0, 0.24, 0.28)),
    };
    commands.spawn(screen(Color::srgba(0.005, 0.012, 0.022, 0.84), 84)).with_children(|root| {
        root.spawn(panel(680.0)).with_children(|panel| {
            panel.spawn(label(&font.0, title, 44.0, color));
            panel.spawn(label(&font.0, body, 19.0, Color::srgb(0.88, 0.94, 0.94)));
            panel.spawn(label(
                &font.0,
                &format!(
                    "الوقت {:02}:{:02}  |  الأعداء {}  |  البلوكات المبنية {}  |  الأبراج {}/3",
                    session.elapsed as u32 / 60,
                    session.elapsed as u32 % 60,
                    session.loadout.kills,
                    session.loadout.blocks_placed,
                    session.relays_destroyed
                ),
                16.0,
                Color::srgb(0.70, 0.82, 0.84),
            ));
            spawn_action(panel, &font.0, "إعادة نفس المسار", OverlayAction::Restart, true);
            spawn_action(panel, &font.0, "رؤية المسار الآخر", OverlayAction::ChooseRoute, false);
            spawn_action(panel, &font.0, "القائمة الرئيسية", OverlayAction::MainMenu, false);
            spawn_action(panel, &font.0, "خروج", OverlayAction::Quit, false);
        });
    });
}

pub fn handle_overlay_buttons(
    mut buttons: Query<(&Interaction, &OverlayAction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut session: ResMut<GameSession>,
    balance: Res<BalanceConfig>,
    mut prefs: ResMut<GamePreferences>,
    mut lifecycle: ResMut<RunLifecycle>,
    mut save_state: ResMut<crate::save_load::SaveLoadState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut choices: MessageWriter<FinalChoiceCommitted>,
    mut exit: MessageWriter<AppExit>,
    mut settings: Query<&mut Text, With<SettingsSummary>>,
) {
    for (interaction, action, mut color) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                color.0 = Color::srgb(0.18, 0.82, 0.68);
                match action {
                    OverlayAction::Resume => next_state.set(AppState::Playing),
                    OverlayAction::SaveGame => {
                        save_state.save_requested = true;
                    }
                    OverlayAction::LoadGame => {
                        save_state.load_requested = true;
                    }
                    OverlayAction::Restart => {
                        lifecycle.active = false;
                        next_state.set(AppState::Playing);
                    }
                    OverlayAction::ChooseRoute => {
                        lifecycle.active = false;
                        next_state.set(AppState::RouteChoice);
                    }
                    OverlayAction::MainMenu => {
                        lifecycle.active = false;
                        next_state.set(AppState::MainMenu);
                    }
                    OverlayAction::Quit => { exit.write(AppExit::Success); }
                    OverlayAction::Extract => {
                        session.final_choice = FinalChoice::Extract;
                        session.phase = MissionPhase::Extraction;
                        session.phase_time_remaining = Some(balance.extraction_seconds);
                        session.objective_hint = "عد إلى السفينة ودافع عنها حتى الاستخراج".into();
                        choices.write(FinalChoiceCommitted(FinalChoice::Extract));
                        next_state.set(AppState::Playing);
                    }
                    OverlayAction::AssaultGate => {
                        session.final_choice = FinalChoice::AssaultGate;
                        session.phase = MissionPhase::GateAssault;
                        session.phase_time_remaining = Some(balance.gate_assault_seconds);
                        session.objective_hint = "اكسر الدرع برمح الأيون ثم دمّر نواة الحاملة".into();
                        choices.write(FinalChoiceCommitted(FinalChoice::AssaultGate));
                        next_state.set(AppState::Playing);
                    }
                    OverlayAction::ToggleMotion => prefs.reduced_motion = !prefs.reduced_motion,
                    OverlayAction::VolumeDown => prefs.master_volume = (prefs.master_volume - 0.1).max(0.0),
                    OverlayAction::VolumeUp => prefs.master_volume = (prefs.master_volume + 0.1).min(1.0),
                }
                if let Ok(mut text) = settings.single_mut() { text.0 = settings_text(&prefs); }
            }
            Interaction::Hovered => color.0 = Color::srgba(0.14, 0.46, 0.42, 0.98),
            Interaction::None => color.0 = Color::srgba(0.06, 0.18, 0.23, 0.97),
        }
    }
}

fn hud_panel(_width: f32) -> BackgroundColor {
    BackgroundColor(Color::srgba(0.012, 0.032, 0.048, 0.86))
}
fn label(font: &Handle<Font>, value: &str, size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (Text::new(value), TextFont { font: font.clone(), font_size: size, ..default() }, TextColor(color))
}
fn screen(color: Color, z: i32) -> (Node, BackgroundColor, ZIndex, ScreenRoot) {
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
            padding: UiRect::all(Val::Px(28.0)),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(18.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(14.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.018, 0.052, 0.072, 0.98)),
        BorderColor::all(Color::srgba(0.42, 0.92, 0.82, 0.62)),
    )
}
fn spawn_action(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, text: &'static str, action: OverlayAction, primary: bool) {
    parent.spawn((
        Button,
        Node {
            width: Val::Percent(92.0),
            min_height: Val::Px(48.0),
            padding: UiRect::all(Val::Px(9.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(11.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(if primary { Color::srgba(0.08, 0.58, 0.50, 0.98) } else { Color::srgba(0.06, 0.18, 0.23, 0.97) }),
        BorderColor::all(Color::srgba(0.42, 0.92, 0.82, 0.52)),
        action,
    )).with_children(|button| {
        button.spawn((label(font, text, 18.0, Color::WHITE), TextLayout::new_with_justify(Justify::Center)));
    });
}
fn settings_text(prefs: &GamePreferences) -> String {
    format!(
        "الصوت {:.0}%  |  تقليل الاهتزاز: {}",
        prefs.master_volume * 100.0,
        if prefs.reduced_motion { "مفعّل" } else { "متوقف" }
    )
}
fn block_name(block: BlockKind) -> &'static str {
    match block {
        BlockKind::Stone => "حجر",
        BlockKind::Dirt => "تراب",
        BlockKind::Grass => "عشب",
        BlockKind::Sand => "رمل",
        BlockKind::Snow => "ثلج",
        BlockKind::Wood => "خشب",
        BlockKind::Leaves => "أوراق",
        BlockKind::Mud => "طين",
        BlockKind::Basalt => "بازلت",
        BlockKind::Ice => "جليد",
        BlockKind::VolcanicAsh => "رماد",
        _ => "بلوك",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Perk Selection Modal UI
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct PerkModalRoot;

#[derive(Component)]
pub struct PerkCardButton(pub usize);

pub fn update_perk_modal(
    mut commands: Commands,
    mut session: ResMut<GameSession>,
    font: Res<ArabicFont>,
    modal_query: Query<Entity, With<PerkModalRoot>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut buttons: Query<(&Interaction, &PerkCardButton), (Changed<Interaction>, With<Button>)>,
) {
    let Some(choices) = session.pending_perk_choices else {
        for entity in &modal_query {
            if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
        }
        return;
    };

    let mut chosen_index: Option<usize> = None;

    if keys.just_pressed(KeyCode::Digit1) || keys.just_pressed(KeyCode::Numpad1) {
        chosen_index = Some(0);
    } else if keys.just_pressed(KeyCode::Digit2) || keys.just_pressed(KeyCode::Numpad2) {
        chosen_index = Some(1);
    } else if keys.just_pressed(KeyCode::Digit3) || keys.just_pressed(KeyCode::Numpad3) {
        chosen_index = Some(2);
    }

    for (interaction, button) in &mut buttons {
        if *interaction == Interaction::Pressed {
            chosen_index = Some(button.0);
            break;
        }
    }

    if let Some(index) = chosen_index {
        if let Some(perk) = choices.get(index) {
            session.loadout.apply_perk(*perk);
        }
        session.pending_perk_choices = None;
        for entity in &modal_query {
            if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); }
        }
        return;
    }

    if !modal_query.is_empty() {
        return;
    }

    let level = session.loadout.level;
    commands
        .spawn((
            Name::new("Perk Selection Overlay"),
            PerkModalRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.05, 0.12, 0.88)),
            ZIndex(100),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text(format!("ارتفاع المستوى! اختر التطوير الخارق (المستوى {})", level)),
                TextFont {
                    font: font.0.clone(),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.84, 0.28)),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|card_parent| {
                    for (i, perk) in choices.iter().enumerate() {
                        card_parent
                            .spawn((
                                Button,
                                PerkCardButton(i),
                                Node {
                                    width: Val::Px(270.0),
                                    height: Val::Px(210.0),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceBetween,
                                    padding: UiRect::all(Val::Px(16.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.08, 0.18, 0.32, 0.95)),
                                BorderColor::all(Color::srgb(0.28, 0.75, 1.0)),
                            ))
                            .with_children(|card| {
                                card.spawn((
                                    Text(format!("[{}] {}", i + 1, perk.title())),
                                    TextFont {
                                        font: font.0.clone(),
                                        font_size: 19.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(1.0, 0.92, 0.42)),
                                ));
                                card.spawn((
                                    Text(perk.description().to_string()),
                                    TextFont {
                                        font: font.0.clone(),
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.85, 0.92, 1.0)),
                                ));
                                card.spawn((
                                    Text(format!("اضغط [{}] للاختيار", i + 1)),
                                    TextFont {
                                        font: font.0.clone(),
                                        font_size: 13.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.35, 0.85, 0.45)),
                                ));
                            });
                    }
                });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
// Save / Load status notification UI
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct SaveStatusText;

pub fn setup_save_status_ui(mut commands: Commands, font: Res<ArabicFont>) {
    commands.spawn((
        Text("".to_string()),
        TextFont {
            font: font.0.clone(),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        TextLayout::new_with_justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(8.0),
            left: Val::Percent(50.0),
            width: Val::Px(400.0),
            margin: UiRect::left(Val::Px(-200.0)),
            ..default()
        },
        Pickable::IGNORE,
        ZIndex(60),
        SaveStatusText,
    ));
}

pub fn update_save_status_ui(
    save_state: Res<crate::save_load::SaveLoadState>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<SaveStatusText>>,
) {
    let Ok((mut text, mut color)) = text_query.single_mut() else {
        return;
    };
    if let Some((ref msg, ttl)) = save_state.status_message {
        let alpha = (ttl * 1.4).clamp(0.0, 1.0);
        text.0 = msg.clone();
        let (r, g, b) = if msg.starts_with("✅") || msg.starts_with("💾") || msg.starts_with("📂") {
            (0.2, 1.0, 0.5)
        } else {
            (1.0, 0.35, 0.35)
        };
        *color = TextColor(Color::srgba(r, g, b, alpha));
    } else {
        text.0 = String::new();
        *color = TextColor(Color::NONE);
    }
}
