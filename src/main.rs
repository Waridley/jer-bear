use bevy::prelude::*;

pub use jeremy_bearimy::*;

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins.set(WindowPlugin {
				primary_window: Some(Window {
					title: "Jeremy Bearimy".into(),
					..default()
				}),
				..default()
			}),
			bees::BeesPlugin,
			levels::LevelsPlugin,
			loading::LoadingPlugin,
			main_menu::MainMenuPlugin,
			map::MapPlugin,
			player::PlayerPlugin,
			portals::PortalsPlugin,
			SplashPlugin,
		))
		.init_state::<GameState>()
		.enable_state_scoped_entities::<GameState>()
		.run();
}
