use crate::GameState;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use crate::levels::{Level, LevelList};
use crate::save::SaveData;

pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::LevelSelect), show_level_select)
			.add_systems(
				Update,
				handle_level_selection_btn.run_if(in_state(GameState::LevelSelect)),
			);
	}
}

pub fn show_level_select(
	mut cmds: Commands,
	level_list: Res<LevelList>,
	save: Res<Persistent<SaveData>>,
	server: Res<AssetServer>
) {
	info!("Showing level select screen");
	cmds.spawn((Camera2d, StateScoped::<GameState>(GameState::LevelSelect)));
	let font = TextFont {
		font: server.load::<Font>("ShareTechMono-Regular.ttf"),
		font_size: 24.0,
		..default()
	};
	
	cmds.spawn((
		Node {
			flex_direction: FlexDirection::Column,
			align_self: AlignSelf::Center,
			justify_self: JustifySelf::Center,
			justify_content: JustifyContent::SpaceAround,
			align_items: AlignItems::Center,
			..default()
		},
		BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
		StateScoped::<GameState>(GameState::LevelSelect),
	)).with_children(|cmds| {
		for (i, level) in level_list.iter().enumerate().filter(|(_, level)| save.unlocked_levels.contains(&level.name)) {
			cmds.spawn((
				LevelSelectionButton(i),
				Button,
				Node {
					margin: UiRect::all(Val::Px(10.0)),
					padding: UiRect::all(Val::Px(10.0)),
					..default()
				},
				BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
			)).with_child((Text(level.name.clone()), font.clone()));
		}
	});
	
}

#[derive(Component, Debug, Copy, Clone, Deref, DerefMut)]
pub struct LevelSelectionButton(pub usize);

pub fn handle_level_selection_btn(
	mut cmds: Commands,
	btns: Query<(&Interaction, &LevelSelectionButton)>,
	level_list: Res<LevelList>,
) {
	for btn in &btns {
		if *btn.0 == Interaction::Pressed {
			info!("Loading level {}", level_list.0[**btn.1].name);
			cmds.remove_resource::<Level>();
			cmds.insert_resource(level_list.0[**btn.1].clone());
		}
	}
}
