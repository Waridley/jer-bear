use crate::GameState;
use crate::levels::Level;
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::MainMenu), show_main_menu);
	}
}

pub fn show_main_menu(mut cmds: Commands) {
	info!("Showing main menu");
	// TODO: Show main menu
	info!("Loading default level");
	cmds.insert_resource(Level::default());
}
