use crate::GameState;
use crate::bees::Bee;
use crate::map::TimelinePosition;
use bevy::color::palettes::basic::BLUE;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct PortalsPlugin;

impl Plugin for PortalsPlugin {
	fn build(&self, app: &mut App) {
		if cfg!(feature = "dev_tools") {
			app.add_systems(
				Update,
				(
					(spin_portals, spawn_items),
					dbg_draw_portals.run_if(input_toggle_active(false, KeyCode::KeyP)),
				),
			);
		}
	}
}

#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Default, Serialize, Deserialize)]
#[serde(default)]
#[require(TimelinePosition, Transform, StateScoped::<GameState>(GameState::LevelEnd))]
pub struct Portal {
	pub spawn_timer: Timer,
	pub spawns: SpawnedItem,
}

impl Default for Portal {
	fn default() -> Self {
		Self {
			spawn_timer: Timer::new(Duration::from_secs_f32(0.05), TimerMode::Repeating),
			spawns: default(),
		}
	}
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Sprite, Transform::from_translation(Vec3::NEG_Z))]
pub struct PortalSwirls;

/// For level descriptions, since TimelinePosition is a separate component but may often need to be set.
#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PortalDescriptor {
	pub spawn_interval: Duration,
	pub spawns: SpawnedItem,
	pub t_start: f32,
	pub speed: f32,
}

impl Default for PortalDescriptor {
	fn default() -> Self {
		let tpos = TimelinePosition::default();
		Self {
			spawn_interval: Duration::from_secs_f32(0.05),
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
				spawn_timer: Timer::new(self.spawn_interval, TimerMode::Repeating),
				spawns: self.spawns,
			},
			TimelinePosition {
				t: self.t_start,
				speed: self.speed,
			},
		)
	}
}

#[derive(Reflect, Debug, Clone, Copy, Serialize, Deserialize)]
#[reflect(Default, Serialize, Deserialize)]
pub enum SpawnedItem {
	Bees { speed: f32 },
}

impl Default for SpawnedItem {
	fn default() -> Self {
		Self::Bees { speed: 400.0 }
	}
}

pub fn spawn_items(
	mut cmds: Commands,
	mut portals: Query<(&mut Portal, &GlobalTransform)>,
	server: Res<AssetServer>,
	t: Res<Time>,
) {
	for (mut portal, xform) in &mut portals {
		portal.spawn_timer.tick(t.delta());
		let xform = xform.compute_transform();
		for _ in 0..portal.spawn_timer.times_finished_this_tick() {
			match portal.spawns {
				SpawnedItem::Bees { speed } => {
					let rot = rand::random::<f32>() * std::f32::consts::TAU;
					let dir = Vec2::from_angle(rot).rotate(Vec2::Y);
					let pos = xform.translation.xy() + dir * 32.0;
					cmds.spawn((
						Bee { speed },
						Sprite {
							// TODO: Load this in loading state
							image: server.load("bee.png"),
							..default()
						},
						Transform {
							translation: pos.extend(0.0),
							rotation: Quat::from_rotation_z(rot),
							..default()
						},
					));
				}
			}
		}
	}
}

pub fn spin_portals(
	mut portals: Query<&mut Transform, (With<Portal>, Without<PortalSwirls>)>,
	mut swirls: Query<&mut Transform, (With<PortalSwirls>, Without<Portal>)>,
	t: Res<Time>,
) {
	for mut xform in &mut portals {
		xform.rotate_z(t.delta_secs() * 4.0);
	}
	for mut xform in &mut swirls {
		xform.rotate_z(t.delta_secs() * -std::f32::consts::TAU);
	}
}

pub fn dbg_draw_portals(portals: Query<&GlobalTransform, With<Portal>>, mut gizmos: Gizmos) {
	for xform in &portals {
		gizmos.circle_2d(xform.translation().xy(), 20.0, BLUE);
	}
}
