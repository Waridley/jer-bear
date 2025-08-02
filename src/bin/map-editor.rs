use bevy::asset::{AssetPath, UnapprovedPathMode, ron};
use bevy::color::palettes::basic::{BLACK, BLUE, GREEN, WHITE};
use bevy::color::palettes::css::YELLOW;
use bevy::input::ButtonState;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, reflect_inspector};
use egui_extras::{Column, TableBuilder};
use jeremy_bearimy::levels::Level;
use jeremy_bearimy::map::{Background, CurveHandle, MapPlugin};
use jeremy_bearimy::*;
use map::Map;
use std::path::PathBuf;

const HANDLE_GRAB_RADIUS: f32 = 12.0;
const KEY_PAN_SPEED: f32 = 400.0;

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins.set(AssetPlugin {
				// Just for the editor during development
				unapproved_path_mode: UnapprovedPathMode::Allow,
				..default()
			}),
			EguiPlugin::default(),
			DefaultInspectorConfigPlugin,
			MapPlugin,
		))
		.register_type::<DisplaySettings>()
		.add_systems(Startup, setup)
		.add_systems(Update, (draw_curve, input).run_if(resource_exists::<Map>))
		.add_systems(EguiPrimaryContextPass, draw_gui)
		.run();
}

pub fn setup(mut cmds: Commands, server: Res<AssetServer>) {
	cmds.spawn(Camera2d);
	cmds.insert_resource(ClearColor(BLACK.into()));
	cmds.insert_resource(State::default());
	cmds.insert_resource(DisplaySettings::default());
	cmds.insert_resource(SaveOptions::default());
	let mut level = Level::default();
	let map_handle = server.load(&level.map);
	level.map_handle = map_handle;
	cmds.insert_resource(level);
}

pub fn draw_gui(
	mut cmds: Commands,
	mut ctx: EguiContexts,
	map: Option<ResMut<Map>>,
	mut background: Single<&mut Sprite, With<Background>>,
	mut state: ResMut<State>,
	server: Res<AssetServer>,
	mut display_settings: ResMut<DisplaySettings>,
	reg: Res<AppTypeRegistry>,
	mut save_opts: ResMut<SaveOptions>,
) {
	let ctx = ctx.ctx_mut().unwrap();
	egui::Window::new("Hello").show(ctx, |ui| {
		ui.label("Click to add point in highlighted segment.");
		ui.label("Drag handles to reshape curve.");
		ui.label("Scroll to zoom.");
		ui.label("Middle click and drag or use arrow keys to pan.");
		ui.separator();
		reflect_inspector::ui_for_value(&mut *display_settings, ui, &reg.read());
		ui.separator();
		if let Some(mut map) = map {
			ui.collapsing("Control points", |ui| {
				ui.horizontal(|ui| {
					ui.label("Rotate:")
						.on_hover_text("Rotate control points to change starting point");
					if ui
						.button("⬆")
						.on_hover_text("Rotate control points list up")
						.clicked()
					{
						map.rotate_points(-1);
					}
					if ui
						.button("⬇")
						.on_hover_text("Rotate control points list down")
						.clicked()
					{
						map.rotate_points(1);
					}
				});
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
								row.set_selected(state.selection == CurveHandle::CtrlPt(i));
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
			ui.collapsing("Tuesdays", |ui| {
				reflect_inspector::ui_for_value_readonly(&map.tuesdays, ui, &reg.read());
			});
			ui.add_enabled_ui(state.mode != Mode::AddingTuesday, |ui| {
				if ui.button("Add Tuesday").clicked() {
					state.mode = Mode::AddingTuesday;
				}
			});
			if ui.button("Re-center").clicked() {
				map.recenter();
			}
			ui.separator();
			ui.horizontal(|ui| {
				ui.label("Background size:");
				if reflect_inspector::ui_for_value(&mut map.size, ui, &reg.read()) {
					background.custom_size = Some(map.size);
				}
			});
			let mut size = map.bounding_rect().size();
			ui.horizontal(|ui| {
				ui.label("Curve dimensions:");
				if reflect_inspector::ui_for_value(&mut size, ui, &reg.read()) {
					map.scale_curve_to(size);
				}
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
				if let Some(path) = dialogue.save_file() {
					save_opts.path = path;
				}
			}
		});
		ui.horizontal(|ui| {
			if ui.button("Load").clicked() {
				let path = save_opts.path.to_string_lossy();
				let path = AssetPath::from(path.as_ref()).clone_owned();
				cmds.remove_resource::<Map>();
				cmds.insert_resource(Level {
					map_handle: server.load(&path),
					map: path,
					..default()
				});
			}
			if ui.button("Save").clicked() {
				cmds.run_system_cached(save_map);
			}
			ui.checkbox(&mut save_opts.pretty, "Pretty");
		});
		if ui.button("New").clicked() {
			cmds.insert_resource(Map::default());
			save_opts.path = save_opts.path.with_file_name("new_map.ron");
		}
	});
}

pub fn draw_curve(
	window: Single<&Window, With<PrimaryWindow>>,
	cam: Single<(&Camera, &GlobalTransform, &Projection)>,
	map: Res<Map>,
	display_settings: Res<DisplaySettings>,
	mut gizmos: Gizmos,
) {
	let scale = if let Projection::Orthographic(projection) = cam.2 {
		projection.scale
	} else {
		unreachable!();
	};
	let pos = window
		.cursor_position()
		.and_then(|pos| cam.0.viewport_to_world(cam.1, pos).ok())
		.map(|ray| ray.origin.xy());
	map.draw(
		&mut gizmos,
		100,
		scale,
		display_settings.curve.then_some(YELLOW.into()),
		display_settings
			.segments
			.then_some(Color::srgb(0.2, 0.2, 0.2)),
		BLUE.into(),
		display_settings.control_points.then_some(WHITE.into()),
		GREEN.into(),
		pos,
		HANDLE_GRAB_RADIUS,
	);
}

pub fn input(
	mut cmds: Commands,
	keys: Res<ButtonInput<KeyCode>>,
	mut motion: EventReader<MouseMotion>,
	buttons: Res<ButtonInput<MouseButton>>,
	mut clicks: EventReader<MouseButtonInput>,
	mut scrolls: EventReader<MouseWheel>,
	window: Single<&Window, With<PrimaryWindow>>,
	cam: Single<(&Camera, &mut Transform, &GlobalTransform, &mut Projection)>,
	mut map: ResMut<Map>,
	mut state: ResMut<State>,
	mut egui_ctx: EguiContexts,
	t: Res<Time>,
) {
	let (cam, ref mut cam_xform, cam_global, mut projection) = cam.into_inner();
	let dt = t.delta_secs();
	let scale = match &*projection {
		Projection::Orthographic(projection) => projection.scale,
		_ => unreachable!(),
	};

	if egui_ctx.ctx_mut().unwrap().wants_pointer_input() {
		return;
	}
	let Some(pos) = window.cursor_position() else {
		return;
	};
	let Ok(ray) = cam.viewport_to_world(cam_global, pos) else {
		return;
	};
	let pos = ray.origin.xy();
	let interactable = map.interactable_handle(pos, HANDLE_GRAB_RADIUS * scale);
	for click in clicks.read() {
		if click.state == ButtonState::Released {
			match (state.mode, click.button) {
				(Mode::Dragging, MouseButton::Left) => {
					state.mode = Mode::Default;
				}
				other => {
					debug!("unhandled mouse button release: {other:?}");
				}
			}
		} else if click.state == ButtonState::Pressed {
			match (state.mode, click.button, interactable) {
				(Mode::Dragging, MouseButton::Left, _) => {
					error!("should be unreachable")
				}
				(Mode::AddingTuesday, MouseButton::Left, _) => {
					let i = map.tuesdays.len();
					map.tuesdays.push(pos);
					state.selection = CurveHandle::Tuesday(i);
					state.mode = Mode::Dragging;
				}
				(Mode::Default, MouseButton::Left, CurveHandle::CtrlPt(handle)) => {
					state.selection = CurveHandle::CtrlPt(handle);
					state.mode = Mode::Dragging;
					info!("Now dragging {handle}");
					continue;
				}
				(Mode::Default, MouseButton::Left, CurveHandle::Tuesday(tue)) => {
					state.selection = CurveHandle::Tuesday(tue);
					state.mode = Mode::Dragging;
					info!("Now dragging {tue}");
					continue;
				}
				(Mode::Default, MouseButton::Left, CurveHandle::None) => match map.add_point(pos) {
					Ok(i) => {
						state.selection = CurveHandle::CtrlPt(i);
						state.mode = Mode::Dragging;
					}
					Err(e) => error!("Failed to add point: {e}"),
				},
				(Mode::Default, MouseButton::Right, CurveHandle::CtrlPt(handle)) => {
					if let Err(e) = map.remove_point(handle) {
						error!("Failed to remove point: {e}");
					}
				}
				(Mode::Default, MouseButton::Right, CurveHandle::Tuesday(tue)) => {
					map.tuesdays.remove(tue);
				}
				(_, MouseButton::Middle, _) => {
					// Could change cursor here for panning
				}
				_ => {}
			}
		}
	}

	if !motion.is_empty() {
		match state.mode {
			Mode::Dragging => match state.selection {
				CurveHandle::CtrlPt(i) => {
					map.move_point(i, pos);
				}
				CurveHandle::Tuesday(i) => {
					map.tuesdays[i] = pos;
				}
				CurveHandle::None => {
					error!("Not dragging anything?");
					state.mode = Mode::Default;
				}
			},
			Mode::AddingTuesday => {
				// If drawing is immediate-mode, this doesn't need to do anything,
				// but if switched to moving an entity, this is where the transform needs updated.
			}
			Mode::Default => {
				// Could change cursor here for hovering
			}
		}
	}

	{
		if buttons.pressed(MouseButton::Middle) {
			for motion in motion.read() {
				cam_xform.translation.x += -motion.delta.x * scale;
				cam_xform.translation.y += motion.delta.y * scale;
			}
		}

		let speed = scale * KEY_PAN_SPEED * dt;
		if keys.pressed(KeyCode::ArrowLeft) {
			cam_xform.translation.x -= speed;
		}
		if keys.pressed(KeyCode::ArrowRight) {
			cam_xform.translation.x += speed;
		}
		if keys.pressed(KeyCode::ArrowUp) {
			cam_xform.translation.y += speed;
		}
		if keys.pressed(KeyCode::ArrowDown) {
			cam_xform.translation.y -= speed;
		}
	}

	if keys.just_pressed(KeyCode::KeyS)
		&& (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight))
	{
		cmds.run_system_cached(save_map);
	}

	for scroll in scrolls.read() {
		let Projection::Orthographic(projection) = &mut *projection else {
			unreachable!();
		};

		if scroll.y > 0.0 {
			projection.scale /= 1.1;
		} else if scroll.y < 0.0 {
			projection.scale *= 1.1;
		}
	}
}

#[derive(Resource, Default, Debug)]
pub struct State {
	pub mode: Mode,
	pub selection: CurveHandle,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
	#[default]
	Default,
	Dragging,
	AddingTuesday,
}

#[derive(Resource, Debug, Clone)]
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
			path: dir.join("maps/map.ron"),
			pretty: false,
		}
	}
}

pub fn save_map(opts: Res<SaveOptions>, map: Res<Map>, reg: Res<AppTypeRegistry>) {
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

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource, Default)]
pub struct DisplaySettings {
	pub control_points: bool,
	pub curve: bool,
	pub segments: bool,
}

impl Default for DisplaySettings {
	fn default() -> Self {
		Self {
			control_points: false,
			curve: true,
			segments: true,
		}
	}
}
