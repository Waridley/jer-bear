use bevy::color::palettes::basic::{BLACK, BLUE, GRAY, GREEN, WHITE};
use bevy::color::palettes::css::{DARK_GRAY, YELLOW};
use bevy::input::ButtonState;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::picking::pointer::{PointerAction, PointerInput};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPlugin, egui, EguiPrimaryContextPass};

use jeremy_bearimy::*;
use map::Map;

const HANDLE_GRAB_RADIUS: f32 = 8.0;

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins,
			EguiPlugin::default(),
		))
		.add_systems(Startup, setup)
		.add_systems(Update, (draw_curve, input))
		.add_systems(EguiPrimaryContextPass, draw_editor)
		.run();
}

pub fn setup(mut cmds: Commands) {
	cmds.spawn(Camera2d);
	cmds.insert_resource(ClearColor(BLACK.into()));
	cmds.insert_resource(Map::default());
}

pub fn draw_editor(
	mut ctx: EguiContexts,
	mut map: ResMut<Map>,
) {
	let ctx = ctx.ctx_mut().unwrap();
	egui::Window::new("Hello").show(ctx, |ui| {
		ui.label("Click to add point in highlighted segment.");
		ui.label("Drag handles to reshape curve.");
		ui.label("Scroll to zoom.");
		
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
	let pos = window.cursor_position()
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
	mut clicks: EventReader<MouseButtonInput>,
	mut scrolls: EventReader<MouseWheel>,
	window: Single<&Window, With<PrimaryWindow>>,
	mut cam: Single<(&Camera, &GlobalTransform, &mut Projection)>,
	mut map: ResMut<Map>,
) {
	let Some(pos) = window.cursor_position() else {
		return;
	};
	let Ok(ray) = cam.0.viewport_to_world(cam.1, pos) else {
		return;
	};
	let pos = ray.origin.xy();
	for click in clicks.read() {
		if click.button == MouseButton::Left && click.state == ButtonState::Pressed {
			for (i, ctrl_pt) in map.control_points().iter().enumerate() {
				if pos.distance(*ctrl_pt) < HANDLE_GRAB_RADIUS {
					info!("Clicked on control point {i}");
					return;
				}
			}
			map.add_point(pos)
				.unwrap_or_else(|e| error!("Failed to add point: {e}"));
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
