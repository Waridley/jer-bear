use bevy::asset::{AssetPath, UnapprovedPathMode, ron};
use bevy::color::palettes::basic::{BLACK, BLUE, GREEN, WHITE};
use bevy::color::palettes::css::YELLOW;
use bevy::input::ButtonState;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use egui_extras::{Column, TableBuilder};
use jeremy_bearimy::map::{LoadingMapHandle, MapPlugin};
use jeremy_bearimy::*;
use map::Map;
use std::path::PathBuf;

const HANDLE_GRAB_RADIUS: f32 = 8.0;

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins.set(AssetPlugin {
				// Just for the editor during development
				unapproved_path_mode: UnapprovedPathMode::Allow,
				..default()
			}),
			EguiPlugin::default(),
			MapPlugin,
		))
		.add_systems(Startup, setup)
		.add_systems(Update, (draw_curve, input).run_if(resource_exists::<Map>))
		.add_systems(EguiPrimaryContextPass, draw_gui)
		.run();
}

pub fn setup(mut cmds: Commands, server: Res<AssetServer>) {
	cmds.spawn(Camera2d);
	cmds.insert_resource(ClearColor(BLACK.into()));
	cmds.insert_resource(DragState::default());

	cmds.insert_resource(LoadingMapHandle(server.load::<Map>("map.ron")));
}

pub fn draw_gui(
	mut cmds: Commands,
	mut ctx: EguiContexts,
	map: Option<Res<Map>>,
	dragging: Res<DragState>,
	server: Res<AssetServer>,
	loading: Option<Res<LoadingMapHandle>>,
	mut save_opts: Local<SaveOptions>,
) {
	let ctx = ctx.ctx_mut().unwrap();
	egui::Window::new("Hello").show(ctx, |ui| {
		ui.label("Click to add point in highlighted segment.");
		ui.label("Drag handles to reshape curve.");
		ui.label("Scroll to zoom.");
		ui.separator();
		ui.label("Control points:");
		if let Some(map) = map {
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
		}
		ui.separator();
		ui.horizontal(|ui| {
			ui.label("File:");
			if ui.button(format!("{}", save_opts.path.display())).clicked() {
				let dialogue = rfd::FileDialog::new();
				let dialogue = if let Some(dir) = save_opts.path.parent() {
					info!("starting in {}", dir.display());
					dialogue.set_directory(dir)
				} else if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR")
					.map(PathBuf::from)
					.or_else(|_| std::env::current_dir())
				{
					let dir = dir.join("assets");
					info!("starting in {}", dir.display());
					dialogue.set_directory(dir.join("assets"))
				} else {
					warn!("Couldn't set directory");
					dialogue
				};
				if let Some(path) = dialogue.pick_file() {
					save_opts.path = path;
				}
			}
		});
		ui.horizontal(|ui| {
			if ui.button("Load").clicked() {
				cmds.insert_resource(LoadingMapHandle(
					server.load::<Map>(save_opts.path.to_str().unwrap()),
				));
			}
			if ui.button("Save").clicked() {
				cmds.run_system_cached_with(save_map, save_opts.clone());
			}
			ui.checkbox(&mut save_opts.pretty, "Pretty");
		});
		if ui.button("Load Default").clicked() {
			cmds.insert_resource(Map::default());
		}
		if let Some(loading) = loading {
			ui.label(format!(
				"Loading {}",
				server
					.get_path(&loading.0)
					.as_ref()
					.map(AssetPath::to_string)
					.unwrap_or("[unknown path]".into())
			));
		}
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

#[derive(Clone)]
pub struct SaveOptions {
	path: PathBuf,
	pretty: bool,
}

impl Default for SaveOptions {
	fn default() -> Self {
		let dir = std::env::var("CARGO_MANIFEST_DIR")
			.map(PathBuf::from)
			.unwrap_or_else(|_| PathBuf::from("."))
			.join("assets");
		Self {
			path: dir.join("map.ron"),
			pretty: false,
		}
	}
}

pub fn save_map(opts: In<SaveOptions>, map: Res<Map>, reg: Res<AppTypeRegistry>) {
	let reg = reg.read();
	let serializer = bevy::reflect::serde::TypedReflectSerializer::new(&*map, &reg);

	let Ok(mut file) =
		std::fs::File::create(&opts.path).map_err(|e| error!("Failed to create file: {e}"))
	else {
		return;
	};

	let Ok(()) = if opts.pretty {
		ron::ser::to_writer_pretty(&mut file, &serializer, ron::ser::PrettyConfig::default())
	} else {
		ron::ser::to_writer(&mut file, &serializer)
	}
	.map_err(|e| error!("Failed to write to file: {e}")) else {
		return;
	};

	info!("Saved map to {}", opts.path.display());
}
