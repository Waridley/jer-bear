use bevy::prelude::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use crate::GameState;
use crate::levels::Level;

#[derive(Resource, Default, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct LevelStats {
	pub time: f32,
	pub bee_count: u32,
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
}

pub fn end_level(
	result: In<GameResult>,
	level: Res<Level>,
	stats: Res<LevelStats>,
	mut run_stats: ResMut<RunStats>,
	mut next_state: ResMut<NextState<GameState>>
) {
	run_stats.levels.insert(level.name.clone(), LevelStats {
		result: Some(*result),
		..stats.clone()
	});
	next_state.set(GameState::LevelEnd);
}
