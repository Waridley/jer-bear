use crate::stats::RunStats;
use bevy::prelude::*;
use bevy_persistent::{PersistenceError, Persistent, StorageFormat};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

pub struct SavePlugin;

impl Plugin for SavePlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<SaveDir>()
			.init_resource::<ConfigDir>()
			.add_systems(Startup, init_save_data);
	}
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveData {
	pub runs: BTreeMap<DateTime<Local>, RunStats>,
	pub unlocked_levels: HashSet<String>,
}

pub fn init_save_data(mut cmds: Commands, dir: Res<SaveDir>) {
	let path = dir.join("save.ron");
	let save = match init_persistence(&path) {
		Ok(save) => save,
		Err(PersistenceError::RonDeserialization(e)) => {
			error!("Failed to deserialize save data: {e}");
			#[cfg(not(target_arch = "wasm32"))]
			{
				let new_path = path.with_extension(format!("failed_{}", Local::now().timestamp()));
				std::fs::rename(&path, &new_path).unwrap();
				warn!("Moved failed save data to {}", new_path.display());
				match init_persistence(&path) {
					Ok(save) => save,
					Err(e) => {
						error!("Failed to init new save data: {e}");
						return;
					}
				}
			}
			#[cfg(target_arch = "wasm32")]
			{
				todo!("retry with new save data on wasm")
			}
		}
		Err(e) => {
			error!("Failed to init save data: {e}");
			return;
		}
	};
	cmds.insert_resource(save);
}

fn init_persistence(path: impl Into<PathBuf>) -> Result<Persistent<SaveData>, PersistenceError> {
	Persistent::<SaveData>::builder()
		.name("save_data")
		.path(path)
		.format(StorageFormat::Ron)
		.default(SaveData::default())
		.build()
}

#[derive(Resource, Clone, Debug, Deref, DerefMut)]
pub struct SaveDir(pub PathBuf);

impl Default for SaveDir {
	fn default() -> Self {
		Self(
			dirs::data_dir()
				.map(|dir| dir.join("waridley/jeremy-bearimy"))
				.unwrap_or_else(|| "./saves".into()),
		)
	}
}

#[derive(Resource, Clone, Debug, Deref, DerefMut)]
pub struct ConfigDir(pub PathBuf);

impl Default for ConfigDir {
	fn default() -> Self {
		Self(
			dirs::config_dir()
				.map(|dir| dir.join("waridley/jeremy-bearimy"))
				.unwrap_or_else(|| "./config".into()),
		)
	}
}
