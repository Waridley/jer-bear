use crate::GameState;
use crate::levels::Level;
use crate::player::spawn_player;
use crate::stats::LevelStats;
use bevy::color::palettes::basic::YELLOW;
use bevy::color::palettes::css::ORANGE;
use bevy::prelude::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			OnEnter(GameState::Playing),
			spawn_display.after(spawn_player),
		)
		.add_systems(
			Update,
			update_stats_display.run_if(in_state(GameState::Playing)),
		);
	}
}

pub fn spawn_display(mut cmds: Commands, server: Res<AssetServer>) {
	let font = server.load::<Font>("ShareTechMono-Regular.ttf");
	let font = TextFont {
		font,
		font_size: 24.0,
		..default()
	};
	let panel = (
		Node {
			min_width: Val::Px(180.0),
			height: Val::Px(40.0),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		},
		BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
		StateScoped::<GameState>(GameState::LevelEnd),
	);
	cmds.spawn(Node {
		width: Val::Percent(100.0),
		flex_direction: FlexDirection::Row,
		justify_content: JustifyContent::SpaceBetween,
		..default()
	})
	.with_children(|cmds| {
		cmds.spawn(panel.clone())
			.with_child((TimeDisplay, Text("0.00".into()), font.clone()));
		cmds.spawn(Node {
			flex_direction: FlexDirection::Column,
			..default()
		})
		.with_children(|cmds| {
			cmds.spawn(panel.clone()).with_child((
				KilledBeesDisplay,
				Text("Killed: 0".into()),
				font.clone(),
				TextColor(YELLOW.into()),
			));
			cmds.spawn(panel).with_child((
				MissedBeesDisplay,
				Text("Missed: 0".into()),
				font,
				TextColor(ORANGE.into()),
			));
		});
	});
}

pub fn update_stats_display(
	level: Res<Level>,
	stats: Res<LevelStats>,
	mut time_display: Single<&mut Text, TimeDisplayQueryFilter>,
	mut bee_count_display: Single<&mut Text, KilledBeesDisplayQueryFilter>,
	mut missed_bees_display: Single<&mut Text, MissedBeesDisplayQueryFilter>,
) {
	let rem = level.duration.checked_sub(stats.time).unwrap_or_default();
	time_display.0 = format!("{:.2}", rem.as_secs_f32());
	bee_count_display.0 = format!("Killed: {}", stats.killed_bees);
	missed_bees_display.0 = format!("Missed: {}", stats.missed_bees);
}

#[derive(Component, Debug, Copy, Clone)]
#[require(Text)]
pub struct TimeDisplay;
type TimeDisplayQueryFilter = (
	With<TimeDisplay>,
	Without<KilledBeesDisplay>,
	Without<MissedBeesDisplay>,
);

#[derive(Component, Debug, Copy, Clone)]
#[require(Text)]
pub struct KilledBeesDisplay;
type KilledBeesDisplayQueryFilter = (
	With<KilledBeesDisplay>,
	Without<TimeDisplay>,
	Without<MissedBeesDisplay>,
);

#[derive(Component, Debug, Copy, Clone)]
#[require(Text)]
pub struct MissedBeesDisplay;
type MissedBeesDisplayQueryFilter = (
	With<MissedBeesDisplay>,
	Without<TimeDisplay>,
	Without<KilledBeesDisplay>,
);
