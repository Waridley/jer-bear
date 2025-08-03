use crate::GameState;
use crate::levels::Level;
use bevy::prelude::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, tick_stats_time.run_if(in_state(GameState::Playing)));
	}
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct LevelStats {
	pub time: Duration,
	pub killed_bees: u32,
	pub missed_bees: u32,
	pub result: Option<GameResult>,
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct RunStats {
	pub levels: IndexMap<String, LevelStats>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum GameResult {
	Win,
	OutOfBounds,
	TimedOut,
	MissedTooMany,
}

pub fn end_level(
	result: In<GameResult>,
	level: Res<Level>,
	stats: Res<LevelStats>,
	mut run_stats: ResMut<RunStats>,
	mut next_state: ResMut<NextState<GameState>>,
) {
	run_stats.levels.insert(
		level.name.clone(),
		LevelStats {
			result: Some(*result),
			..stats.clone()
		},
	);
	next_state.set(GameState::LevelEnd);
}

pub fn tick_stats_time(mut stats: ResMut<LevelStats>, t: Res<Time>) {
	stats.time += t.delta();
}
