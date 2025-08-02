use crate::GameState;
use crate::loading::LoadingTasks;
use crate::map::Map;
use crate::portals::{Portal, SpawnedItem};
use bevy::asset::AssetPath;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(First, load_level.run_if(resource_added::<Level>))
			.add_systems(
				Update,
				check_level_loading_progress.run_if(in_state(GameState::Loading)),
			)
			.add_systems(OnEnter(GameState::Playing), start_wave)
			.add_systems(OnEnter(GameState::LevelEnd), show_level_end_screen)
			.add_systems(OnExit(GameState::LevelEnd), on_exit_level_end);
	}
}

#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Level {
	pub name: String,
	pub map: AssetPath<'static>,
	#[reflect(ignore)]
	#[serde(skip)]
	pub map_handle: Handle<Map>,
	pub waves: Vec<Wave>,
	pub current_wave: usize,
}

impl Default for Level {
	fn default() -> Self {
		Self {
			name: "Level 1".to_string(),
			map: AssetPath::from("map.ron"),
			map_handle: Handle::default(),
			waves: vec![Wave {
				portals: vec![Portal {
					spawns: SpawnedItem::Beehive,
				}],
			}],
			current_wave: 0,
		}
	}
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Wave {
	pub portals: Vec<Portal>,
}

pub fn load_level(
	mut level: ResMut<Level>,
	server: Res<AssetServer>,
	mut next_state: ResMut<NextState<GameState>>,
	mut loading_tasks: ResMut<LoadingTasks>,
) {
	debug_assert!(
		level.is_added(),
		"load_level should be run on resource_added::<Level>"
	);
	info!("Loading level {}", level.name);
	next_state.set(GameState::Loading);
	let _ = loading_tasks.start("Map");
	level.map_handle = server.load(&level.map);
}

pub fn check_level_loading_progress(
	map: Option<Res<Map>>,
	mut loading_tasks: ResMut<LoadingTasks>,
) {
	if map.is_some() {
		let handle = loading_tasks.find("Map").unwrap();
		loading_tasks.finish(handle);
	}
}

pub fn start_wave(mut cmds: Commands, level: Res<Level>) {
	level.waves[level.current_wave]
		.portals
		.iter()
		.for_each(|portal| {
			cmds.spawn(portal.clone());
		});
}

pub fn show_level_end_screen(mut next_state: ResMut<NextState<GameState>>) {
	info!("Showing level end screen");
	// TODO: Show level end screen
	next_state.set(GameState::MainMenu);
}

pub fn on_exit_level_end(mut cmds: Commands) {
	cmds.remove_resource::<Level>();
	cmds.remove_resource::<Map>();
}
