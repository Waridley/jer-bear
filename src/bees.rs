use bevy::prelude::*;

pub struct BeesPlugin;

impl Plugin for BeesPlugin {
	fn build(&self, _app: &mut App) {}
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Hive {
	pub spawn_rate: f32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Bee {
	pub speed: f32,
}
