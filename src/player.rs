use crate::GameState;
use crate::levels::Level;
use crate::loading::{LoadingTaskHandle, LoadingTasks};
use crate::map::Map;
use crate::stats::{GameResult, end_level};
use bevy::prelude::*;
use bevy::render::camera;
use bevy_enhanced_input::prelude::*;
use serde::{Deserialize, Serialize};

pub const BASE_PLAYER_MAX_VELOCITY: f32 = 2000.0;
pub const BASE_PLAYER_ACCEL: f32 = 500.0;
pub const BASE_PLAYER_VELOCITY_DECAY: f32 = 0.5;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_input_context::<Avatar>()
			.add_systems(OnEnter(GameState::Loading), PlayerAssets::load)
			.add_systems(
				Update,
				(
					spin_blades,
					player_movement.run_if(in_state(GameState::Playing)),
					PlayerAssets::check_progress.run_if(in_state(GameState::Loading)),
				),
			)
			.add_systems(OnEnter(GameState::Playing), spawn_player);
	}
}

#[derive(Resource, Debug, Clone)]
pub struct PlayerAssets {
	loading_task_handle: LoadingTaskHandle,
	blades: Handle<Image>,
}

impl PlayerAssets {
	pub fn load(
		mut cmds: Commands,
		mut loading_tasks: ResMut<LoadingTasks>,
		server: Res<AssetServer>,
	) {
		cmds.insert_resource(Self {
			loading_task_handle: loading_tasks.start("Player Assets"),
			blades: server.load("blades.png"),
		});
	}

	pub fn check_progress(
		assets: Res<PlayerAssets>,
		server: Res<AssetServer>,
		mut loading_tasks: ResMut<LoadingTasks>,
	) {
		for handle in &[assets.blades.clone().untyped()] {
			if !server.is_loaded_with_dependencies(handle.id()) {
				return;
			}
		}
		loading_tasks.finish(assets.loading_task_handle);
	}
}

pub fn spawn_player(mut cmds: Commands, assets: Res<PlayerAssets>) {
	cmds.spawn((
		Avatar,
		actions!(Avatar[
			(
				Action::<Move>::new(),
				DeadZone::default(),
				SmoothNudge::default(),
				Bindings::spawn((
					Cardinal::wasd_keys(),
					Cardinal::arrow_keys(),
					Axial::left_stick(),
					Axial::right_stick(),
				),
			))
		])
	)).with_children(|cmds| {
		cmds.spawn((
			Blades {
				radius: 48.0,
				spin_speed: -24.0,
			},
			Sprite {
				image: assets.blades.clone(),
				..default()
			},
			Transform::from_xyz(0.0, 0.0, 100.0),
		));
		cmds.spawn((
			Camera2d,
			Projection::Orthographic(OrthographicProjection {
				scaling_mode: camera::ScalingMode::Fixed {
					width: 1920.0,
					height: 1080.0,
				},
				..OrthographicProjection::default_2d()
			}),
		));
	});
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[require(Transform, Visibility, Velocity, StateScoped::<GameState>(GameState::Playing))]
pub struct Avatar;

pub fn player_movement(
	mut cmds: Commands,
	mut query: Query<(&mut Transform, &mut Velocity), With<Avatar>>,
	move_action: Single<&Action<Move>>,
	level: Res<Level>,
	map: Res<Map>,
	keys: Res<ButtonInput<KeyCode>>,
	mut next_state: ResMut<NextState<GameState>>,
	t: Res<Time>,
) {
	for (mut xform, mut vel) in &mut query {
		let delta = ***move_action;
		let PlayerSpeedParams {
			max_velocity,
			accel,
			velocity_decay,
		} = level.player_speed_params;

		vel.0 *= 1.0 - t.delta_secs() * velocity_decay;
		vel.0 += delta * t.delta_secs() * accel;
		vel.0 = vel.0.clamp_length_max(max_velocity);
		xform.translation += vel.0.extend(0.0) * t.delta_secs();
		let abs_pos = xform.translation.xy().abs();
		if abs_pos.x > map.size.x * 0.5 || abs_pos.y > map.size.y * 0.5 {
			cmds.run_system_cached_with(end_level, GameResult::OutOfBounds);
			next_state.set(GameState::LevelEnd);
		}
	}
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
pub struct Velocity(pub Vec2);

#[derive(Reflect, Debug, Clone, Copy, Serialize, Deserialize)]
#[reflect(Default, Serialize, Deserialize)]
pub struct PlayerSpeedParams {
	pub max_velocity: f32,
	pub accel: f32,
	pub velocity_decay: f32,
}

impl Default for PlayerSpeedParams {
	fn default() -> Self {
		Self {
			max_velocity: BASE_PLAYER_MAX_VELOCITY,
			accel: BASE_PLAYER_ACCEL,
			velocity_decay: BASE_PLAYER_VELOCITY_DECAY,
		}
	}
}

pub fn spin_blades(mut query: Query<(&mut Transform, &Blades)>, t: Res<Time>) {
	for (mut xform, blades) in &mut query {
		xform.rotate_z(t.delta_secs() * blades.spin_speed);
	}
}

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct Blades {
	pub radius: f32,
	pub spin_speed: f32,
}

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Move;
