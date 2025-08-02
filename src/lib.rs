#![feature(iter_map_windows)]

use bevy::prelude::*;
use crate::levels::LevelList;

pub mod bees;
pub mod levels;
pub mod loading;
pub mod main_menu;
pub mod map;
pub mod player;
pub mod portals;
pub mod stats;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
	#[default]
	Splash,
	MainMenu,
	Loading,
	Playing,
	LevelEnd,
}

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::Splash), show_splash)
			.add_systems(PostUpdate, goto_main_menu
				.run_if(in_state(GameState::Splash))
				.run_if(resource_exists::<LevelList>));
	}
}

pub fn show_splash() {
	// TODO: Show splash screen
}

pub fn goto_main_menu(
	mut next_state: ResMut<NextState<GameState>>
) {
	info!("Going to main menu");
	next_state.set(GameState::MainMenu);
}