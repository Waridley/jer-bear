use bevy::prelude::*;
use crate::GameState;

pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::LevelSelect), show_level_select);
	}
}

pub fn show_level_select(
	mut cmds: Commands,
) {
	info!("Showing level select screen");
	cmds.spawn((Camera2d, StateScoped::<GameState>(GameState::LevelSelect)));
	
}
