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

#[derive(Component, Reflect, Default, Debug, Clone, Serialize, Deserialize)]
#[reflect(Default, Serialize, Deserialize)]
#[serde(default)]
#[require(TimelinePosition, Transform, StateScoped::<GameState>(GameState::LevelEnd))]
pub struct Portal {
	pub spawns: SpawnedItem,
}

/// For level descriptions, since TimelinePosition is a separate component but may often need to be set.
#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PortalDescriptor {
	pub spawns: SpawnedItem,
	pub t_start: f32,
	pub speed: f32,
}

impl Default for PortalDescriptor {
	fn default() -> Self {
		let tpos = TimelinePosition::default();
		Self {
			spawns: Portal::default().spawns,
			t_start: tpos.t,
			speed: tpos.speed,
		}
	}
}

impl PortalDescriptor {
	pub fn bundle(&self) -> (Portal, TimelinePosition) {
		(
			Portal {
				spawns: self.spawns,
			},
			TimelinePosition {
				t: self.t_start,
				speed: self.speed,
			},
		)
	}
}

#[derive(Reflect, Default, Debug, Clone, Copy, Serialize, Deserialize)]
#[reflect(Default, Serialize, Deserialize)]
pub enum SpawnedItem {
	#[default]
	Bees,
}

pub fn dbg_draw_portals(portals: Query<&GlobalTransform, With<Portal>>, mut gizmos: Gizmos) {
	for xform in &portals {
		gizmos.circle_2d(xform.translation().xy(), 20.0, BLUE);
	}
}
