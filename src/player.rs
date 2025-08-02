use crate::GameState;
use crate::loading::{LoadingTaskHandle, LoadingTasks};
use crate::map::Map;
use bevy::prelude::*;
use bevy::render::camera;

pub const PLAYER_ACCEL: f32 = 500.0;
pub const PLAYER_VELOCITY_DECAY: f32 = 0.5;
pub const PLAYER_MAX_VELOCITY: f32 = 2000.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::Loading), PlayerAssets::load)
			.add_systems(
				Update,
				(
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
	avatar: Handle<Image>,
}

impl PlayerAssets {
	pub fn load(
		mut cmds: Commands,
		mut loading_tasks: ResMut<LoadingTasks>,
		server: Res<AssetServer>,
	) {
		cmds.insert_resource(Self {
			loading_task_handle: loading_tasks.start("Player Assets"),
			avatar: server.load("avatar.png"),
		});
	}

	pub fn check_progress(
		assets: Res<PlayerAssets>,
		server: Res<AssetServer>,
		mut loading_tasks: ResMut<LoadingTasks>,
	) {
		for handle in &[assets.avatar.clone().untyped()] {
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
		Sprite {
			image: assets.avatar.clone(),
			..default()
		},
	))
	.with_child((
		Camera2d,
		Projection::Orthographic(OrthographicProjection {
			scaling_mode: camera::ScalingMode::Fixed {
				width: 1920.0,
				height: 1080.0,
			},
			..OrthographicProjection::default_2d()
		}),
	));
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[require(Sprite, Velocity, StateScoped::<GameState>(GameState::Playing))]
pub struct Avatar;

pub fn player_movement(
	mut query: Query<(&mut Transform, &mut Velocity), With<Avatar>>,
	map: Res<Map>,
	keys: Res<ButtonInput<KeyCode>>,
	mut next_state: ResMut<NextState<GameState>>,
	t: Res<Time>,
) {
	for (mut xform, mut vel) in &mut query {
		let mut delta = Vec2::ZERO;
		if keys.pressed(KeyCode::KeyA) {
			delta.x -= 1.0;
		}
		if keys.pressed(KeyCode::KeyD) {
			delta.x += 1.0;
		}
		if keys.pressed(KeyCode::KeyW) {
			delta.y += 1.0;
		}
		if keys.pressed(KeyCode::KeyS) {
			delta.y -= 1.0;
		}
		vel.0 *= 1.0 - t.delta_secs() * PLAYER_VELOCITY_DECAY;
		vel.0 += delta * t.delta_secs() * PLAYER_ACCEL;
		vel.0 = vel.0.clamp_length_max(PLAYER_MAX_VELOCITY);
		xform.translation += vel.0.extend(0.0) * t.delta_secs();
		let abs_pos = xform.translation.xy().abs();
		if abs_pos.x > map.size.x * 0.5 || abs_pos.y > map.size.y * 0.5 {
			next_state.set(GameState::LevelEnd);
		}
	}
}

#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
pub struct Velocity(pub Vec2);
