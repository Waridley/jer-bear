use crate::GameState;
use crate::levels::{Level, LevelList};
use bevy::prelude::*;
use crate::stats::RunStats;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::MainMenu), show_main_menu);
	}
}

pub fn show_main_menu(
	mut cmds: Commands,
	level_list: Res<LevelList>,
) {
	info!("Showing main menu");
	// TODO: Show main menu
	info!("Loading default level");
	cmds.insert_resource(level_list[0].clone());
	cmds.insert_resource(RunStats::default());
}
