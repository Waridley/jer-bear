use bevy::color::palettes::tailwind::{GRAY_400, GRAY_500, GRAY_600, GRAY_800, GRAY_900};
use crate::GameState;
use crate::levels::LevelList;
use crate::stats::RunStats;
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::MainMenu), show_main_menu)
			.add_systems(Update, (
				handle_btn_colors,
				handle_play_btn,
				handle_level_select_btn,
			).run_if(in_state(GameState::MainMenu)));
	}
}

pub fn show_main_menu(mut cmds: Commands, server: Res<AssetServer>) {
	info!("Showing main menu");
	cmds.spawn((Camera2d, StateScoped::<GameState>(GameState::MainMenu)));
	let font = TextFont {
		font: server.load::<Font>("ShareTechMono-Regular.ttf"),
		font_size: 24.0,
		..default()
	};
	let btn_node = Node {
		width: Val::Percent(100.0),
		align_content: AlignContent::Center,
		justify_content: JustifyContent::Center,
		margin: UiRect::all(Val::Px(5.0)),
		padding: UiRect::all(Val::Px(5.0)),
		..default()
	};
	let btn_bg = BackgroundColor(GRAY_800.into());
	cmds.spawn((
		Node {
			flex_direction: FlexDirection::Column,
			align_self: AlignSelf::Center,
			justify_self: JustifySelf::Center,
			justify_content: JustifyContent::SpaceAround,
			align_items: AlignItems::Center,
			..default()
		},
		BackgroundColor(GRAY_400.into()),
	)).with_children(|cmds| {
		cmds.spawn((
			PlayButton,
			Button,
			Node {
				..btn_node.clone()
			},
			btn_bg,
		)).with_child((
			Text("Play".into()),
			font.clone()
		));
		
		cmds.spawn((
			LevelSelectButton,
			Button,
			Node {
				..btn_node.clone()
			},
			btn_bg,
			// Disabled until levels are unlocked
			Disabled,
		)).with_child((
			Text("Level Select".into()),
			font,
		));
	});
	
}

#[derive(Component, Debug, Copy, Clone)]
#[require(Button, StateScoped::<GameState>(GameState::MainMenu))]
pub struct PlayButton;

#[derive(Component, Debug, Copy, Clone)]
#[require(Button, StateScoped::<GameState>(GameState::MainMenu))]
pub struct LevelSelectButton;

pub fn handle_btn_colors(
	mut q: Query<(&Interaction, &mut BackgroundColor, Has<Disabled>), With<Button>>,
) {
	for (interaction, mut bg, disabled) in &mut q {
		if disabled {
			bg.0 = GRAY_500.into();
			continue;
		}
		match *interaction {
			Interaction::Pressed => {
				bg.0 = GRAY_900.into();
			}
			Interaction::Hovered => {
				bg.0 = GRAY_600.into();
			}
			Interaction::None => {
				bg.0 = GRAY_800.into();
			}
		}
	}
}

pub fn handle_play_btn(
	mut cmds: Commands,
	interaction: Single<&Interaction, With<PlayButton>>,
	level_list: Res<LevelList>,
) {
	if **interaction == Interaction::Pressed {
    info!("Loading first level");
    cmds.insert_resource(level_list[0].clone());
    cmds.insert_resource(RunStats::default());
  }
}

pub fn handle_level_select_btn(
	interaction: Single<&Interaction, (With<LevelSelectButton>, Without<Disabled>)>,
) {
	if **interaction == Interaction::Pressed {
		info!("Showing level select screen");
		// TODO: Implement level select screen
	}
}

/// Disables a button, preventing interactions from changing its color
#[derive(Component, Debug, Copy, Clone)]
pub struct Disabled;