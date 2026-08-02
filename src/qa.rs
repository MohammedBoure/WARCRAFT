use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use crate::state::AppState;

#[derive(Resource)]
pub struct QaScreenshot {
    path: Option<String>,
    delay: Timer,
    requested: bool,
}

impl Default for QaScreenshot {
    fn default() -> Self {
        Self {
            path: std::env::var("CRITICAL_POINT_QA_SCREENSHOT").ok(),
            delay: Timer::from_seconds(2.0, TimerMode::Once),
            requested: false,
        }
    }
}

pub fn capture_qa_screenshot(
    time: Res<Time>,
    state: Res<State<AppState>>,
    mut qa: ResMut<QaScreenshot>,
    mut commands: Commands,
) {
    if qa.requested || *state.get() != AppState::MainMenu {
        return;
    }
    let Some(path) = qa.path.clone() else {
        return;
    };
    if qa.delay.tick(time.delta()).just_finished() {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        qa.requested = true;
    }
}
