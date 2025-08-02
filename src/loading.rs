use crate::GameState;
use bevy::ecs::intern::{Interned, Interner};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

static LOADING_TASK_INTERNER: Interner<str> = Interner::new();

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<LoadingTasks>()
			.add_systems(OnEnter(GameState::Loading), show_loading_screen)
			.add_systems(
				Update,
				check_loading_progress.run_if(in_state(GameState::Loading)),
			)
			.add_systems(OnExit(GameState::Loading), clear_loading_tasks);
	}
}

#[derive(Resource, Default, Debug)]
pub struct LoadingTasks(HashMap<LoadingTaskHandle, LoadingStatus>);

impl LoadingTasks {
	pub fn start(&mut self, name: impl AsRef<str>) -> LoadingTaskHandle {
		let name = LOADING_TASK_INTERNER.intern(name.as_ref());
		info!("Loading {}", &*name);
		let handle = LoadingTaskHandle(name);
		self.0.insert(handle, LoadingStatus::Loading);
		handle
	}

	pub fn finish(&mut self, handle: LoadingTaskHandle) {
		let Some(status) = self.0.get_mut(&handle) else {
			error!("No loading task {:?}", &*handle.0);
			return;
		};
		*status = LoadingStatus::Done;
	}

	/// Find a task by name. This is a lazy way to get a handle to a task, which doesn't
	/// scale to larger projects, as it is effectively a "stringly-typed" API.
	pub fn find(&self, name: impl AsRef<str>) -> Option<LoadingTaskHandle> {
		let name = LOADING_TASK_INTERNER.intern(name.as_ref());
		self.0.iter().find_map(|(handle, _)| {
			if handle.0 == name {
				Some(*handle)
			} else {
				None
			}
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingStatus {
	Loading,
	Done,
}

impl LoadingStatus {
	pub fn done(&self) -> bool {
		matches!(self, LoadingStatus::Done)
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[must_use]
pub struct LoadingTaskHandle(Interned<str>);

pub fn show_loading_screen() {
	// TODO: Show loading screen
	info!("Showing loading screen");
}

pub fn check_loading_progress(
	tasks: Res<LoadingTasks>,
	mut next_state: ResMut<NextState<GameState>>,
) {
	if tasks.0.is_empty() {
		return;
	}
	for task in tasks.0.values() {
		if !task.done() {
			return;
		}
	}
	info!("Loading complete");
	next_state.set(GameState::Playing);
}

pub fn clear_loading_tasks(mut tasks: ResMut<LoadingTasks>) {
	tasks.0.clear();
}
