use crate::GameState;
use crate::player::Blades;
use crate::stats::LevelStats;
use bevy::prelude::*;

pub struct BeesPlugin;

impl Plugin for BeesPlugin {
	fn build(&self, _app: &mut App) {
		_app.add_systems(
			Update,
			(move_bees, despawn_bees, kill_bees).run_if(in_state(GameState::Playing)),
		);
	}
}

#[derive(Component, Debug, Clone, Reflect)]
#[require(DespawnTimer, Sprite, StateScoped::<GameState>(GameState::LevelEnd))]
pub struct Bee {
	pub speed: f32,
}

pub fn move_bees(mut query: Query<(&mut Transform, &Bee)>, t: Res<Time>) {
	for (mut xform, bee) in &mut query {
		let dir = xform.rotation * Vec3::Y;
		xform.translation += dir * t.delta_secs() * bee.speed;
	}
}

#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct DespawnTimer(Timer);

impl Default for DespawnTimer {
	fn default() -> Self {
		Self(Timer::from_seconds(0.5, TimerMode::Once))
	}
}

pub fn despawn_bees(
	mut cmds: Commands,
	mut query: Query<(Entity, &mut DespawnTimer, &mut Sprite)>,
	mut stats: ResMut<LevelStats>,
	t: Res<Time>,
) {
	for (id, mut timer, mut sprite) in &mut query {
		timer.tick(t.delta());
		let percent = timer.elapsed_secs() / timer.duration().as_secs_f32();
		sprite.color.set_alpha(1.0 - (percent * percent));
		if timer.finished() {
			cmds.entity(id).despawn();
			stats.missed_bees += 1;
		}
	}
}

pub fn kill_bees(
	mut cmds: Commands,
	bees: Query<(Entity, &GlobalTransform), With<Bee>>,
	blades: Query<(&GlobalTransform, &Blades)>,
	mut stats: ResMut<LevelStats>,
) {
	for (xform, blades) in blades {
		for (id, bee_xform) in &bees {
			if xform
				.translation()
				.xy()
				.distance(bee_xform.translation().xy())
				< blades.radius
			{
				cmds.entity(id).despawn();
				stats.killed_bees += 1;
			}
		}
	}
}
