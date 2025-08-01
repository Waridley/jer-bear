use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, ReflectAsset, ron};
use bevy::math::cubic_splines::InsufficientDataError;
use bevy::prelude::*;
use bevy::reflect::TypeRegistryArc;
use serde::de::DeserializeSeed;
use std::fmt::{Display, Formatter};

pub struct MapPlugin;

impl Plugin for MapPlugin {
	fn build(&self, app: &mut App) {
		app.init_asset::<Map>()
			.register_asset_reflect::<Map>()
			.add_systems(PreUpdate, insert_loaded_map);
		let loader = MapLoader {
			registry: app.world().resource::<AppTypeRegistry>().0.clone(),
		};
		app.register_asset_loader(loader);
	}
}

#[derive(Resource, Asset, Debug, Deref, Reflect)]
#[reflect(Default, Resource, Asset)]
pub struct Map {
	#[deref]
	#[reflect(ignore)]
	curve: CubicCurve<Vec2>,
	spline: CubicBSpline<Vec2>,
}

impl Map {
	pub fn new(
		control_points: impl IntoIterator<Item = Vec2>,
	) -> Result<Self, InsufficientDataError> {
		let spline = CubicBSpline::new(control_points);
		let curve = spline.to_curve_cyclic()?;
		Ok(Self { curve, spline })
	}

	pub fn sync(&mut self) -> Result<(), InsufficientDataError> {
		self.curve = self.spline.to_curve_cyclic()?;
		Ok(())
	}

	pub fn draw(
		&self,
		gizmos: &mut Gizmos,
		resolution: usize,
		curve_color: Color,
		segment_color: Color,
		closest_segment_color: Color,
		handle_color: Color,
		hovered_handle_color: Color,
		cursor_pos: Option<Vec2>,
		grab_radius: f32,
	) {
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
		if let Some(closest_segment) = cursor_pos.and_then(|pos| self.closest_segment(pos)) {
			gizmos.line_2d(
				self.spline.control_points[closest_segment],
				self.spline.control_points
					[(closest_segment + 1) % self.spline.control_points.len()],
				closest_segment_color,
			);
		}
		for p in &self.spline.control_points {
			let color = if let Some(pos) = cursor_pos
				&& pos.distance(*p) < grab_radius
			{
				hovered_handle_color
			} else {
				handle_color
			};
			gizmos.circle_2d(*p, 4.0, color);
		}
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
			return Err(self.spline.control_points.len());
		}
		let removed = self.spline.control_points.remove(index);
		self.curve = self
			.spline
			.to_curve_cyclic()
			.expect("already checked length");
		Ok(removed)
	}

	pub fn closest_control_point(&self, point: Vec2) -> Option<usize> {
		self.spline
			.control_points
			.iter()
			.copied()
			.enumerate()
			.min_by(|(_, a), (_, b)| a.distance(point).partial_cmp(&b.distance(point)).unwrap())
			.map(|(i, _)| i)
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
		_load_context: &mut LoadContext<'_>,
	) -> std::result::Result<Self::Asset, Self::Error> {
		let mut bytes = Vec::new();
		reader.read_to_end(&mut bytes).await?;
		let reg = self.registry.read();
		let reflect_deserializer = bevy::reflect::serde::TypedReflectDeserializer::of::<Map>(&reg);
		let mut deserializer = ron::Deserializer::from_bytes(&bytes)?;
		let map = reflect_deserializer.deserialize(&mut deserializer)?;
		let mut map: Map = Map::take_from_reflect(map)?;
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

#[derive(Resource)]
pub struct LoadingMapHandle(pub Handle<Map>);

pub fn insert_loaded_map(
	mut cmds: Commands,
	mut maps: ResMut<Assets<Map>>,
	mut events: EventReader<AssetEvent<Map>>,
	loading: Option<Res<LoadingMapHandle>>,
) {
	for ev in events.read() {
		if let AssetEvent::LoadedWithDependencies { id } = ev {
			if let Some(LoadingMapHandle(loading)) = loading.as_deref()
				&& loading.id() == *id
			{
				cmds.remove_resource::<LoadingMapHandle>();
			}
			if let Some(mut map) = maps.remove(*id) {
				if let Err(e) = map.sync() {
					error!("Failed to convert control points to curve: {e}");
					return;
				}
				cmds.insert_resource(map);
			}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AddPointError;

impl Display for AddPointError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Failed to add point to map")
	}
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
