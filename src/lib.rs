#![feature(iter_map_windows)]

use bevy::prelude::*;

pub mod bees;
pub mod levels;
pub mod loading;
pub mod main_menu;
pub mod map;
pub mod player;
pub mod portals;

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
		app.add_systems(OnEnter(GameState::Splash), show_splash);
	}
}

pub fn show_splash(mut next_state: ResMut<NextState<GameState>>) {
	// TODO: Show splash screen
	info!("Going to main menu");
	next_state.set(GameState::MainMenu);
}
