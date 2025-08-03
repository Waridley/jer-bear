use bevy::prelude::*;
use bevy_enhanced_input::EnhancedInputPlugin;
pub use jeremy_bearimy::*;

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins.set(WindowPlugin {
				primary_window: Some(Window {
					title: "Jeremy Bearimy".into(),
					resolution: (960.0, 540.0).into(),
					..default()
				}),
				..default()
			}),
			EnhancedInputPlugin,
			bees::BeesPlugin,
			hud::HudPlugin,
			levels::LevelsPlugin,
			loading::LoadingPlugin,
			main_menu::MainMenuPlugin,
			map::MapPlugin,
			player::PlayerPlugin,
			portals::PortalsPlugin,
			save::SavePlugin,
			stats::StatsPlugin,
			SplashPlugin,
		))
		.init_state::<GameState>()
		.enable_state_scoped_entities::<GameState>()
		.run();
}
