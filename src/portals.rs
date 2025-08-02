use crate::GameState;
use crate::map::TimelinePosition;
use bevy::color::palettes::basic::BLUE;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct PortalsPlugin;

impl Plugin for PortalsPlugin {
	fn build(&self, app: &mut App) {
		if cfg!(feature = "dev_tools") {
			app.add_systems(
				Update,
				dbg_draw_portals.run_if(input_toggle_active(false, KeyCode::KeyP)),
			);
		}
	}
}

#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[require(TimelinePosition, Transform, StateScoped::<GameState>(GameState::LevelEnd))]
pub struct Portal {
	pub spawns: SpawnedItem,
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub enum SpawnedItem {
	Beehive,
}

pub fn dbg_draw_portals(portals: Query<&GlobalTransform, With<Portal>>, mut gizmos: Gizmos) {
	for xform in &portals {
		gizmos.circle_2d(xform.translation().xy(), 20.0, BLUE);
	}
}
