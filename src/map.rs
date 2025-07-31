use bevy::color::palettes::basic::GREEN;
use bevy::math::cubic_splines::InsufficientDataError;
use bevy::prelude::*;
use std::fmt::Display;

pub struct MapPlugin;

impl Plugin for MapPlugin {
	fn build(&self, app: &mut App) {}
}

#[derive(Resource, Debug, Deref)]
pub struct Map {
	#[deref]
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
			.min_by(|(i, pair0), (j, pair1)| {
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
