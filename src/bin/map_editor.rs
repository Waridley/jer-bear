use bevy::color::palettes::basic::{BLACK, BLUE, GRAY, GREEN, WHITE};
use bevy::color::palettes::css::{DARK_GRAY, YELLOW};
use bevy::input::ButtonState;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::picking::pointer::{PointerAction, PointerInput};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use egui_extras::{Column, TableBuilder};
use jeremy_bearimy::*;
use map::Map;

const HANDLE_GRAB_RADIUS: f32 = 8.0;

fn main() {
	App::new()
		.add_plugins((DefaultPlugins, EguiPlugin::default()))
		.add_systems(Startup, setup)
		.add_systems(Update, (draw_curve, input))
		.add_systems(EguiPrimaryContextPass, draw_editor)
		.run();
}

pub fn setup(mut cmds: Commands) {
	cmds.spawn(Camera2d);
	cmds.insert_resource(ClearColor(BLACK.into()));
	cmds.insert_resource(Map::default());
	cmds.insert_resource(DragState::default());
}

pub fn draw_editor(mut ctx: EguiContexts, mut map: ResMut<Map>, dragging: Res<DragState>) {
	let ctx = ctx.ctx_mut().unwrap();
	egui::Window::new("Hello").show(ctx, |ui| {
		ui.label("Click to add point in highlighted segment.");
		ui.label("Drag handles to reshape curve.");
		ui.label("Scroll to zoom.");
		ui.separator();
		ui.label("Control points:");
		ui.vertical(|ui| {
			TableBuilder::new(ui)
				.column(Column::auto())
				.column(Column::auto())
				.column(Column::auto())
				.header(20.0, |mut header| {
					header.col(|ui| {
						ui.heading("Index");
					});
					header.col(|ui| {
						ui.heading("X");
					});
					header.col(|ui| {
						ui.heading("Y");
					});
				})
				.body(|mut body| {
					for (i, p) in map.control_points().iter().enumerate() {
						body.row(16.0, |mut row| {
							row.set_selected(dragging.point() == Some(i));
							row.col(|ui| {
								ui.label(i.to_string());
							});
							row.col(|ui| {
								ui.label(format!("{:.2}", p.x));
							});
							row.col(|ui| {
								ui.label(format!("{:.2}", p.y));
							});
						});
					}
				})
		});
		ui.separator();
		if ui.button("Reset").clicked() {
			*map = Map::default();
		}
		// egui_plot::Plot::new("my_plot").show(ui, |ui| {
		// 	ui.line(egui_plot::Line::new(map.iter_positions(100).map(|p| [p.x, p.y])));
		// });
	});
}

pub fn draw_curve(
	window: Single<&Window, With<PrimaryWindow>>,
	cam: Single<(&Camera, &GlobalTransform)>,
	map: Res<Map>,
	mut gizmos: Gizmos,
) {
	let pos = window
		.cursor_position()
		.and_then(|pos| cam.0.viewport_to_world(cam.1, pos).ok())
		.map(|ray| ray.origin.xy());
	map.draw(
		&mut gizmos,
		100,
		YELLOW.into(),
		Color::srgb(0.2, 0.2, 0.2),
		BLUE.into(),
		WHITE.into(),
		GREEN.into(),
		pos,
		HANDLE_GRAB_RADIUS,
	);
}

pub fn input(
	motion: EventReader<MouseMotion>,
	mut clicks: EventReader<MouseButtonInput>,
	mut scrolls: EventReader<MouseWheel>,
	window: Single<&Window, With<PrimaryWindow>>,
	mut cam: Single<(&Camera, &GlobalTransform, &mut Projection)>,
	mut map: ResMut<Map>,
	mut dragging: ResMut<DragState>,
	mut egui_ctx: EguiContexts,
) {
	if egui_ctx.ctx_mut().unwrap().wants_pointer_input() {
		return;
	}
	let Some(pos) = window.cursor_position() else {
		return;
	};
	let Ok(ray) = cam.0.viewport_to_world(cam.1, pos) else {
		return;
	};
	let pos = ray.origin.xy();
	let closest_handle = map.closest_control_point(pos);
	for click in clicks.read() {
		if click.state == ButtonState::Released {
			dragging.clear();
		} else if click.state == ButtonState::Pressed {
			if let Some(closest_handle) = closest_handle
				&& pos.distance(map.control_points()[closest_handle]) < HANDLE_GRAB_RADIUS
			{
				match click.button {
					MouseButton::Left => {
						dragging.grab(closest_handle);
						info!("Now dragging {closest_handle}");
						continue;
					}
					MouseButton::Right => {
						map.remove_point(closest_handle)
							.map(|removed| {
								info!("Removed control point {closest_handle} at {removed}")
							})
							.map_err(|i| error!("Failed to remove control point {i}"))
							.ok();
					}
					_ => {}
				}
			} else {
				match click.button {
					MouseButton::Left => {
						**dragging = map
							.add_point(pos)
							.map_err(|e| error!("Failed to add point: {e}"))
							.ok();
						dragging.interaction = Interaction::Pressed;
					}
					MouseButton::Right => {
						// TODO: Context menu?
					}
					_ => {}
				}
			}
		}
	}
	if !motion.is_empty() {
		if dragging.is_grabbed() {
			map.move_point(dragging.point.unwrap(), pos);
		} else if let Some(closest_handle) = closest_handle {
			if pos.distance(map.control_points()[closest_handle]) < HANDLE_GRAB_RADIUS {
				dragging.hover(closest_handle);
			} else {
				dragging.clear();
			}
		} else {
			dragging.clear();
		}
	}

	for scroll in scrolls.read() {
		let Projection::Orthographic(projection) = &mut *cam.2 else {
			unreachable!();
		};

		if scroll.y > 0.0 {
			projection.scale /= 1.1;
		} else if scroll.y < 0.0 {
			projection.scale *= 1.1;
		}
	}
}

#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct DragState {
	#[deref]
	point: Option<usize>,
	interaction: Interaction,
}

impl DragState {
	pub fn clear(&mut self) {
		self.point = None;
		self.interaction = Interaction::None;
	}

	pub fn point(&self) -> Option<usize> {
		self.point
	}

	pub fn is_grabbed(&self) -> bool {
		self.interaction == Interaction::Pressed
	}

	pub fn grab(&mut self, point: usize) {
		self.point = Some(point);
		self.interaction = Interaction::Pressed;
	}

	pub fn hover(&mut self, point: usize) {
		self.point = Some(point);
		self.interaction = Interaction::Hovered;
	}
}
