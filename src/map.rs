use crate::GameState;
use crate::levels::Level;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AssetPath, LoadContext, ReflectAsset, ron};
use bevy::color::palettes::basic::{BLUE, GREEN, WHITE, YELLOW};
use bevy::input::common_conditions::input_toggle_active;
use bevy::math::cubic_splines::InsufficientDataError;
use bevy::prelude::*;
use bevy::reflect::TypeRegistryArc;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

pub struct MapPlugin;

impl Plugin for MapPlugin {
	fn build(&self, app: &mut App) {
		app.init_asset::<Map>()
			.register_asset_reflect::<Map>()
			.add_systems(PreUpdate, insert_loaded_map)
			.add_systems(
				Update,
				(
					tick_timeline_positions,
					move_timeline_items,
					dbg_draw_curve.run_if(input_toggle_active(false, KeyCode::KeyC)),
				)
					.run_if(in_state(GameState::Playing)),
			);
		let loader = MapLoader {
			registry: app.world().resource::<AppTypeRegistry>().0.clone(),
		};
		app.register_asset_loader(loader);
	}
}

#[derive(Resource, Asset, Debug, Clone, Deref, Reflect)]
#[reflect(Default, Resource, Asset)]
pub struct Map {
	#[deref]
	#[reflect(ignore)]
	curve: CubicCurve<Vec2>,
	spline: CubicBSpline<Vec2>,
	/// Also July. And sometimes, the time when nothing, never happens.
	pub tuesdays: Vec<Vec2>,
	pub background: AssetPath<'static>,
	#[reflect(ignore)]
	pub bg_handle: Handle<Image>,
	pub size: Vec2,
}

impl Map {
	pub fn new(
		control_points: impl IntoIterator<Item = Vec2>,
	) -> Result<Self, InsufficientDataError> {
		let spline = CubicBSpline::new(control_points);
		let curve = spline.to_curve_cyclic()?;
		let tuesdays = Vec::new();
		Ok(Self {
			curve,
			spline,
			tuesdays,
			background: "bg.png".into(),
			bg_handle: default(),
			size: Vec2::splat(8192.0),
		})
	}

	pub fn sync(&mut self) -> Result<(), InsufficientDataError> {
		self.curve = self.spline.to_curve_cyclic()?;
		Ok(())
	}

	pub fn draw(
		&self,
		gizmos: &mut Gizmos,
		resolution: usize,
		scale: f32,
		curve_color: Option<Color>,
		segment_color: Option<Color>,
		closest_segment_color: Color,
		handle_color: Option<Color>,
		hovered_handle_color: Color,
		cursor_pos: Option<Vec2>,
		grab_radius: f32,
	) {
		if let Some(segment_color) = segment_color {
			for [a, b] in self
				.spline
				.control_points
				.iter()
				.copied()
				.chain(std::iter::once(self.spline.control_points[0]))
				.map_windows(|&pair| pair)
			{
				gizmos.line_2d(a, b, segment_color);
			}
		}
		if let Some(closest_segment) = cursor_pos.and_then(|pos| self.closest_segment(pos)) {
			gizmos.line_2d(
				self.spline.control_points[closest_segment],
				self.spline.control_points
					[(closest_segment + 1) % self.spline.control_points.len()],
				closest_segment_color,
			);
		}
		let interactable = cursor_pos
			.map(|pos| self.interactable_handle(pos, grab_radius * scale))
			.unwrap_or(CurveHandle::None);
		for (i, p) in self.spline.control_points.iter().enumerate() {
			let color = if interactable == CurveHandle::CtrlPt(i) {
				hovered_handle_color
			} else if let Some(handle_color) = handle_color {
				handle_color
			} else {
				continue;
			};
			gizmos.circle_2d(*p, 4.0 * scale, color);
		}
		if let Some(curve_color) = curve_color {
			for [a, b] in self
				.curve
				.iter_positions(resolution * self.spline.control_points.len())
				.map_windows(|&pair| pair)
			{
				gizmos.line(
					Vec3::new(a.x, a.y, 0.0),
					Vec3::new(b.x, b.y, 0.0),
					curve_color,
				);
			}

			for (i, tue) in self.tuesdays.iter().enumerate() {
				let color = if interactable == CurveHandle::Tuesday(i) {
					hovered_handle_color
				} else {
					curve_color
				};
				gizmos.circle_2d(*tue, 16.0, color);
			}
		}
	}

	pub fn add_point(&mut self, point: Vec2) -> Result<usize, AddPointError> {
		let closest_segment = self.closest_segment(point).ok_or(AddPointError)?;
		info!("Inserting point into segment {closest_segment} at {point:?}");
		let i = closest_segment + 1;
		self.spline.control_points.insert(i, point);
		self.curve = self
			.spline
			.to_curve_cyclic()
			.expect("spline already had at least 2 points");
		Ok(i)
	}

	pub fn move_point(&mut self, index: usize, new_pos: Vec2) {
		self.spline.control_points[index] = new_pos;
		self.curve = self
			.spline
			.to_curve_cyclic()
			.expect("spline already had at least 2 points");
	}

	pub fn remove_point(&mut self, index: usize) -> Result<Vec2, usize> {
		if self.spline.control_points.len() <= 2 {
			// TODO: Create a new error type for this
			// InsufficientDataError has private fields
			return Err(self.spline.control_points.len());
		}
		let removed = self.spline.control_points.remove(index);
		self.curve = self
			.spline
			.to_curve_cyclic()
			.expect("already checked length");
		Ok(removed)
	}

	pub fn rotate_points(&mut self, n: isize) {
		if n < 0 {
			self.spline.control_points.rotate_left(n.unsigned_abs());
			self.sync().expect("curve was already valid");
		} else if n > 0 {
			self.spline.control_points.rotate_right(n as usize);
			self.sync().expect("curve was already valid");
		}
	}

	pub fn translate(&mut self, delta: Vec2) {
		self.spline
			.control_points
			.iter_mut()
			.for_each(|p| *p += delta);
		self.tuesdays.iter_mut().for_each(|p| *p += delta);
		self.sync().expect("curve was already valid");
	}

	pub fn find_center(&self) -> Vec2 {
		self.spline
			.control_points
			.iter()
			.chain(self.tuesdays.iter())
			.sum::<Vec2>()
			/ self.spline.control_points.len() as f32
	}

	pub fn recenter(&mut self) {
		let center = self.bounding_rect().center();
		self.translate(-center);
	}

	pub fn bounding_rect(&self) -> Rect {
		let min = self
			.spline
			.control_points
			.iter()
			.chain(self.tuesdays.iter())
			.fold(Vec2::ZERO, |acc, p| acc.min(*p));
		let max = self
			.spline
			.control_points
			.iter()
			.chain(self.tuesdays.iter())
			.fold(Vec2::ZERO, |acc, p| acc.max(*p));
		Rect { min, max }
	}

	pub fn scale_curve_to(&mut self, size: Vec2) {
		let center = self.find_center();
		let scale = size / self.bounding_rect().size();
		self.translate(-center);
		self.spline
			.control_points
			.iter_mut()
			.for_each(|p| *p *= scale);
		self.tuesdays.iter_mut().for_each(|p| *p *= scale);
		self.translate(center);
		self.sync().expect("curve was already valid");
	}

	pub fn closest_control_point(&self, point: Vec2) -> Option<(usize, f32)> {
		self.spline
			.control_points
			.iter()
			.copied()
			.enumerate()
			.map(|(i, p)| (i, p.distance(point)))
			.min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
	}

	pub fn closest_tuesday(&self, point: Vec2) -> Option<(usize, f32)> {
		self.tuesdays
			.iter()
			.copied()
			.enumerate()
			.map(|(i, p)| (i, p.distance(point)))
			.min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
	}

	pub fn closest_handle(&self, point: Vec2) -> (CurveHandle, f32) {
		let pt = self.closest_control_point(point);
		let tue = self.closest_tuesday(point);
		match (pt, tue) {
			(Some((h, hdist)), Some((t, tdist))) => {
				if hdist < tdist {
					(CurveHandle::CtrlPt(h), hdist)
				} else {
					(CurveHandle::Tuesday(t), tdist)
				}
			}
			(Some((h, dist)), None) => (CurveHandle::CtrlPt(h), dist),
			(None, Some((t, dist))) => (CurveHandle::Tuesday(t), dist),
			(None, None) => (CurveHandle::None, f32::INFINITY),
		}
	}

	pub fn interactable_handle(&self, point: Vec2, grab_radius: f32) -> CurveHandle {
		let (handle, dist) = self.closest_handle(point);
		if dist < grab_radius {
			handle
		} else {
			CurveHandle::None
		}
	}

	pub fn closest_segment(&self, point: Vec2) -> Option<usize> {
		self.spline
			.control_points
			.iter()
			.copied()
			.chain(std::iter::once(self.spline.control_points[0]))
			.map_windows(|&[a, b]| [a, b])
			.enumerate()
			.min_by(|(_, pair0), (_, pair1)| {
				let dist0 = dist_squared_point_to_line_segment(pair0[0], pair0[1], point);
				let dist1 = dist_squared_point_to_line_segment(pair1[0], pair1[1], point);
				dist0.partial_cmp(&dist1).unwrap()
			})
			.map(|(i, _)| i)
	}

	pub fn control_points(&self) -> &[Vec2] {
		&self.spline.control_points
	}

	pub fn curve(&self) -> &CubicCurve<Vec2> {
		&self.curve
	}
}

impl Default for Map {
	fn default() -> Self {
		Self::new([
			Vec2::new(-100.0, 100.0),
			Vec2::new(100.0, 100.0),
			Vec2::new(100.0, -100.0),
			Vec2::new(-100.0, -100.0),
		])
		.unwrap()
	}
}

pub struct MapLoader {
	pub registry: TypeRegistryArc,
}

impl AssetLoader for MapLoader {
	type Asset = Map;
	type Settings = ();
	type Error = LoadMapError;

	async fn load(
		&self,
		reader: &mut dyn Reader,
		_settings: &Self::Settings,
		load_context: &mut LoadContext<'_>,
	) -> std::result::Result<Self::Asset, Self::Error> {
		let mut bytes = Vec::new();
		reader.read_to_end(&mut bytes).await?;
		let reg = self.registry.read();
		let reflect_deserializer = bevy::reflect::serde::TypedReflectDeserializer::of::<Map>(&reg);
		let mut deserializer = ron::Deserializer::from_bytes(&bytes)?;
		let map = reflect_deserializer.deserialize(&mut deserializer)?;
		let mut map: Map = Map::take_from_reflect(map)?;
		let bg = load_context.load(&map.background);
		map.bg_handle = bg;
		map.sync()?;
		Ok(map)
	}
}

#[derive(Debug)]
pub enum LoadMapError {
	Io(std::io::Error),
	Ron(ron::de::Error),
	Spanned(ron::de::SpannedError),
	Reflect(Box<dyn PartialReflect>),
	Invalid(InsufficientDataError),
}

impl Display for LoadMapError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Io(err) => write!(f, "IO error: {err}"),
			Self::Ron(err) => write!(f, "RON error: {err}"),
			Self::Spanned(err) => write!(f, "RON error: {err}"),
			Self::Reflect(got) => write!(f, "Reflect error: {got:?}"),
			Self::Invalid(err) => write!(f, "Invalid map: {err}"),
		}
	}
}

impl From<std::io::Error> for LoadMapError {
	fn from(value: std::io::Error) -> Self {
		Self::Io(value)
	}
}

impl From<ron::de::Error> for LoadMapError {
	fn from(value: ron::de::Error) -> Self {
		Self::Ron(value)
	}
}

impl From<ron::de::SpannedError> for LoadMapError {
	fn from(value: ron::de::SpannedError) -> Self {
		Self::Spanned(value)
	}
}

impl From<Box<dyn PartialReflect>> for LoadMapError {
	fn from(value: Box<dyn PartialReflect>) -> Self {
		Self::Reflect(value)
	}
}

impl From<InsufficientDataError> for LoadMapError {
	fn from(value: InsufficientDataError) -> Self {
		Self::Invalid(value)
	}
}

impl std::error::Error for LoadMapError {}

pub fn insert_loaded_map(
	mut cmds: Commands,
	maps: ResMut<Assets<Map>>,
	server: Res<AssetServer>,
	level: Option<Res<Level>>,
	existing_map: Option<Res<Map>>,
	existing_background: Option<Single<Entity, With<Background>>>,
) {
	if existing_map.is_some() {
		return;
	}
	if let Some(level) = level.as_deref()
		&& server.is_loaded_with_dependencies(level.map_handle.id())
	{
		info!("Loaded map");
		if let Some(mut map) = maps.get(level.map_handle.id()).cloned() {
			if let Err(e) = map.sync() {
				error!("Failed to convert control points to curve: {e}");
				return;
			}
			if let Some(existing) = existing_background.as_deref() {
				cmds.entity(*existing).despawn();
			}
			cmds.spawn((
				Background,
				Sprite {
					image: map.bg_handle.clone(),
					image_mode: SpriteImageMode::Tiled {
						tile_x: true,
						tile_y: true,
						stretch_value: 1.0,
					},
					custom_size: Some(map.size),
					..default()
				},
			));
			cmds.insert_resource(map);
		}
	}
}

#[derive(Component, Debug)]
#[require(Sprite, StateScoped::<GameState>(GameState::LevelEnd))]
pub struct Background;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveHandle {
	#[default]
	None,
	CtrlPt(usize),
	Tuesday(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AddPointError;

impl Display for AddPointError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Failed to add point to map")
	}
}

#[derive(Component, Debug, Default, Copy, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct TimelinePosition {
	/// The "time" value used to sample the map curve for position.
	pub t: f32,
}

pub fn tick_timeline_positions(
	mut query: Query<&mut TimelinePosition>,
	map: Res<Map>,
	t: Res<Time>,
) {
	let domain = map.curve.domain();
	debug_assert_eq!(domain.start(), 0.0);

	let dt = t.delta_secs() * 0.5;
	for mut pos in &mut query {
		pos.t = (pos.t + dt) % domain.end();
	}
}

pub fn move_timeline_items(mut query: Query<(&mut Transform, &TimelinePosition)>, map: Res<Map>) {
	for (mut xform, pos) in &mut query {
		let Some(Vec2 { x, y, .. }) = map.sample(pos.t) else {
			error!("Failed to sample map at t={}", pos.t);
			continue;
		};
		xform.translation = Vec3::new(x, y, xform.translation.z);
	}
}

pub fn dbg_draw_curve(map: Res<Map>, mut gizmos: Gizmos) {
	map.draw(
		&mut gizmos,
		100,
		1.0,
		Some(YELLOW.into()),
		Some(Color::srgb(0.2, 0.2, 0.2)),
		BLUE.into(),
		Some(WHITE.into()),
		GREEN.into(),
		None,
		10.0,
	);
}

fn dist_squared_point_to_line_segment(a: Vec2, b: Vec2, p: Vec2) -> f32 {
	let ab = b - a;
	let ab_norm = ab.normalize();
	let ba = a - b;
	let ba_norm = ba.normalize();
	let ap = p - a;
	let bp = p - b;
	let dot_a = ap.dot(ab_norm);
	let dot_b = bp.dot(ba_norm);
	if dot_a < 0.0 {
		ap.length_squared()
	} else if dot_b < 0.0 {
		bp.length_squared()
	} else {
		let proj = a + (ab_norm * dot_a);
		(p - proj).length_squared()
	}
}
