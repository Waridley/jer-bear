use crate::GameState;
use crate::loading::LoadingTasks;
use crate::map::{Background, Map};
use crate::player::PlayerSpeedParams;
use crate::portals::{PortalDescriptor, PortalSwirls};
use crate::save::SaveData;
use crate::stats::{GameResult, LevelStats, RunStats, end_level};
use bevy::asset::{AssetPath, ReflectAsset};
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_persistent::Persistent;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;

pub struct LevelsPlugin;

pub static LEVEL_LIST_HANDLE: OnceLock<Handle<LevelList>> = OnceLock::new();

impl Plugin for LevelsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(RonAssetPlugin::<LevelList>::new(&["ron"]))
			.init_asset::<LevelList>()
			.register_asset_reflect::<LevelList>()
			.add_systems(
				First,
				(
					load_level.run_if(resource_added::<Level>),
					insert_loaded_level_list.run_if(not(resource_exists::<LevelList>)),
				),
			)
			.add_systems(
				Update,
				(
					check_level_loading_progress.run_if(in_state(GameState::Loading)),
					(handle_main_menu_btn, handle_next_level_btn)
						.run_if(in_state(GameState::LevelEnd)),
				),
			)
			.add_systems(Last, check_goal.run_if(in_state(GameState::Playing)))
			.add_systems(OnEnter(GameState::Playing), start_wave)
			.add_systems(OnEnter(GameState::LevelEnd), show_level_end_screen);

		LEVEL_LIST_HANDLE
			.set(
				app.world()
					.resource::<AssetServer>()
					.load("levels/level_list.ron"),
			)
			.expect("no other calls to set");
	}
}

#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Default, Resource, Serialize, Deserialize)]
#[serde(default)]
pub struct Level {
	pub name: String,
	pub map: AssetPath<'static>,
	pub scene: AssetPath<'static>,
	#[reflect(ignore)]
	#[serde(skip)]
	pub map_handle: Handle<Map>,
	pub goal: Goal,
	pub duration: Duration,
	pub waves: Vec<Wave>,
	pub current_wave: usize,
	pub player_speed_params: PlayerSpeedParams,
}

impl Default for Level {
	fn default() -> Self {
		Self {
			name: "Level 1".to_string(),
			map: AssetPath::from("maps/map.ron"),
			scene: AssetPath::from("levels/empty.scn.ron"),
			map_handle: Handle::default(),
			goal: Goal::Bees(500),
			duration: Duration::from_secs(120),
			waves: vec![Wave {
				portals: vec![PortalDescriptor::default()],
			}],
			current_wave: 0,
			player_speed_params: default(),
		}
	}
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct Wave {
	pub portals: Vec<PortalDescriptor>,
}

pub fn load_level(
	mut cmds: Commands,
	mut level: ResMut<Level>,
	server: Res<AssetServer>,
	mut next_state: ResMut<NextState<GameState>>,
	mut loading_tasks: ResMut<LoadingTasks>,
	save: Option<ResMut<Persistent<SaveData>>>,
) {
	if let Some(mut save) = save
		&& let Err(e) = save.update(|save| {
			save.unlocked_levels.insert(level.name.clone());
		}) {
		error!("Failed to update save data: {e}");
	}
	cmds.insert_resource(LevelStats::default());
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

pub fn start_wave(mut cmds: Commands, level: Res<Level>, server: Res<AssetServer>) {
	level.waves[level.current_wave]
		.portals
		.iter()
		.for_each(|portal| {
			cmds.spawn((
				portal.bundle(),
				Sprite {
					// TODO: Load this in loading state
					image: server.load("portal.png"),
					..default()
				},
			))
			.with_child((
				PortalSwirls,
				Sprite {
					// TODO: Load this in loading state
					image: server.load("portal_swirls.png"),
					..default()
				},
			));
		});
}

pub fn show_level_end_screen(
	mut cmds: Commands,
	level_list: Res<LevelList>,
	stats: Res<LevelStats>,
	run_stats: Res<RunStats>,
	mut next_state: ResMut<NextState<GameState>>,
	server: Res<AssetServer>,
) {
	info!("{stats:#?}");
	info!("{run_stats:#?}");
	cmds.spawn((Camera2d, StateScoped::<GameState>(GameState::LevelEnd)));
	let font = server.load::<Font>("ShareTechMono-Regular.ttf");

	// Result display
	cmds.spawn((
		Node {
			position_type: PositionType::Absolute,
			align_self: AlignSelf::Center,
			justify_self: JustifySelf::Center,
			padding: UiRect::all(Val::Px(20.0)),
			..default()
		},
		BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
		StateScoped::<GameState>(GameState::LevelEnd),
	))
	.with_children(|cmds| {
		let font = TextFont {
			font: font.clone(),
			font_size: 48.0,
			..default()
		};
		let fail_color = TextColor(Color::srgb(0.8, 0.2, 0.2));
		match stats.result {
			Some(GameResult::Win) => {
				cmds.spawn((
					Text("Success!".into()),
					font.clone(),
					TextColor(Color::srgb(0.0, 0.8, 0.0)),
				));
			}
			Some(GameResult::OutOfBounds) => {
				cmds.spawn((
					Text("Oops! You fell out of the arena...".into()),
					font.clone(),
					fail_color,
				));
			}
			Some(GameResult::TimedOut) => {
				cmds.spawn((Text("Time's up!".into()), font.clone(), fail_color));
			}
			Some(GameResult::MissedTooMany) => {
				cmds.spawn((Text("You missed too many bees!".into()), font, fail_color));
			}
			None => {
				error!("Result should exist");
				next_state.set(GameState::MainMenu);
			}
		}
	});

	let font = TextFont {
		font: font.clone(),
		font_size: 32.0,
		..default()
	};

	// Buttons
	cmds.spawn((
		Button,
		Node {
			position_type: PositionType::Absolute,
			bottom: Val::Px(60.0),
			justify_self: JustifySelf::Center,
			..default()
		},
		StateScoped::<GameState>(GameState::LevelEnd),
	))
	.with_children(|cmds| {
		cmds.spawn((
			MainMenuButton,
			Button,
			Node {
				margin: UiRect::all(Val::Px(10.0)),
				padding: UiRect::all(Val::Px(10.0)),
				..default()
			},
			BackgroundColor(Color::srgb(0.4, 0.2, 0.2)),
		))
		.with_child((Text("Main Menu".into()), font.clone()));

		if level_list.1 < level_list.0.len() - 1 {
			if let Some(GameResult::Win) = stats.result {
				cmds.spawn((
					ContinueButton,
					Button,
					Node {
						margin: UiRect::all(Val::Px(10.0)),
						padding: UiRect::all(Val::Px(10.0)),
						..default()
					},
					BackgroundColor(Color::srgb(0.0, 0.3, 0.4)),
				))
				.with_child((Text("Next Level".into()), font.clone()));
			} else {
				cmds.spawn((
					ContinueButton,
					Button,
					Node {
						margin: UiRect::all(Val::Px(10.0)),
						padding: UiRect::all(Val::Px(10.0)),
						..default()
					},
					BackgroundColor(Color::srgb(0.0, 0.3, 0.4)),
				))
				.with_child((Text("Try Again".into()), font));
			}
		}
	});
}

#[derive(Component, Debug, Copy, Clone)]
pub struct MainMenuButton;

pub fn handle_main_menu_btn(
	mut cmds: Commands,
	interaction: Single<&Interaction, With<MainMenuButton>>,
	background: Single<Entity, With<Background>>,
	mut next_state: ResMut<NextState<GameState>>,
) {
	if **interaction == Interaction::Pressed {
		cmds.entity(*background).despawn();
		cmds.remove_resource::<Level>();
		cmds.remove_resource::<Map>();
		next_state.set(GameState::MainMenu);
	}
}

#[derive(Component, Debug, Copy, Clone)]
pub struct ContinueButton;

pub fn handle_next_level_btn(
	mut cmds: Commands,
	mut level_list: ResMut<LevelList>,
	stats: Res<LevelStats>,
	interaction: Single<&Interaction, With<ContinueButton>>,
	mut next_state: ResMut<NextState<GameState>>,
) {
	if **interaction == Interaction::Pressed {
		if stats.result == Some(GameResult::Win) {
			if level_list.1 >= level_list.0.len() - 1 {
				error!("No more levels (i = {}) how did the next level button show up?", level_list.1);
			} else {
				level_list.1 += 1;
			}
			cmds.remove_resource::<Level>();
			cmds.insert_resource(level_list[level_list.1].clone());
		} else {
			next_state.set(GameState::Playing);
		}
		cmds.remove_resource::<Map>();
		cmds.insert_resource(LevelStats::default());
	}
}

#[derive(
	Resource, Asset, Debug, Default, Clone, Deref, DerefMut, Reflect, Serialize, Deserialize,
)]
#[reflect(Resource, Asset, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LevelList(
	#[deref] pub Vec<Level>,
	/// Tracks which level is currently being played during a run.
	#[serde(skip)]
	pub usize,
);

pub fn insert_loaded_level_list(mut cmds: Commands, assets: Res<Assets<LevelList>>) {
	let handle = LEVEL_LIST_HANDLE.get().unwrap().clone();
	let Some(level_list) = assets.get(handle.id()) else {
		return;
	};
	info!("{level_list:#?}");
	cmds.insert_resource(level_list.clone());
}

#[derive(Reflect, Debug, Clone, Copy, Serialize, Deserialize)]
#[reflect(Debug, Serialize, Deserialize)]
pub enum Goal {
	/// Win by avoiding going out of bounds for the duration of the level.
	Time,
	/// Kill this many bees.
	Bees(u32),
	/// Don't miss this many bees.
	MaxMissed(u32),
}

pub fn check_goal(mut cmds: Commands, level: Res<Level>, stats: Res<LevelStats>) {
	match level.goal {
		Goal::Time => {
			if stats.time >= level.duration {
				cmds.run_system_cached_with(end_level, GameResult::Win);
			}
		}
		Goal::Bees(n) => {
			if stats.killed_bees >= n {
				cmds.run_system_cached_with(end_level, GameResult::Win);
			} else if stats.time >= level.duration {
				cmds.run_system_cached_with(end_level, GameResult::TimedOut);
			}
		}
		Goal::MaxMissed(n) => {
			if stats.missed_bees >= n {
				cmds.run_system_cached_with(end_level, GameResult::MissedTooMany);
			} else if stats.time >= level.duration {
				cmds.run_system_cached_with(end_level, GameResult::Win);
			}
		}
	}
}
