//! Stationary circuit framing. No vehicle or clock state enters this calculation.
use crate::sim::Track;
use macroquad::prelude::*;

const ELEVATION: f32 = 65. * std::f32::consts::PI / 180.;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackCamera {
    key: (usize, f32, f32),
    pub target: Vec3,
    pub extent: f32,
}

impl TrackCamera {
    pub fn new(track: &Track, width: f32, height: f32) -> Self {
        let width = width.max(1.);
        let height = height.max(1.);
        let (sin, cos) = ELEVATION.sin_cos();
        let mut min = vec2(f32::INFINITY, f32::INFINITY);
        let mut max = vec2(f32::NEG_INFINITY, f32::NEG_INFINITY);
        // Fit the circuit and immediate shoulders, rather than the full recovery
        // area. A small runoff margin keeps roadside cars and jumps readable.
        let radius = track.width * 0.5 + 6.;
        for (point, ground) in track.points.iter().zip(&track.heights) {
            for x in [point.x - radius, point.x + radius] {
                for z in [point.y - radius, point.y + radius] {
                    for y in [0., ground + 10.] {
                        let projected = vec2(x, cos * y - sin * z);
                        min = min.min(projected);
                        max = max.max(projected);
                    }
                }
            }
        }
        // Match the HUD's reserved bands in web/style.css (CSS/logical pixels).
        let top = 180_f32.min(height * 0.30);
        let bottom = 120_f32.min(height * 0.24);
        let side = 24_f32.min(width * 0.04);
        let usable_width = width - 2. * side;
        let usable_height = height - top - bottom;
        let size = max - min;
        let road_extent =
            (size.y * height / usable_height).max(size.x * height / usable_width) * 1.02;
        // Runoff can use the outer screen around the HUD-safe circuit. Keep cars
        // visible until recovery, including a body margin, without shrinking the
        // road to reserve HUD bands around that entire off-track area.
        // Expand +6 to +22: 18 units before recovery, plus car-body clearance.
        let recovery_size = size + vec2(32., 32. * sin);
        let edge = 4_f32.min(width.min(height) * 0.01);
        let extent = road_extent
            .max(recovery_size.x * height / (width - 2. * edge))
            .max(recovery_size.y * height / (height - (top - bottom).abs() - 2. * edge));
        let center = (min + max) * 0.5;
        let focus_y = center.y + (top - bottom) * 0.5 * extent / height;
        Self {
            key: (track.id, width, height),
            target: vec3(center.x, 0., -focus_y / sin),
            extent,
        }
    }

    pub fn update(&mut self, track: &Track, width: f32, height: f32) {
        if self.key != (track.id, width.max(1.), height.max(1.)) {
            *self = Self::new(track, width, height);
        }
    }

    pub fn camera(&self) -> Camera3D {
        let (sin, cos) = ELEVATION.sin_cos();
        Camera3D {
            position: self.target + vec3(0., sin, cos) * 400.,
            target: self.target,
            up: Vec3::Y,
            fovy: self.extent,
            aspect: Some(self.key.1 / self.key.2),
            projection: Projection::Orthographics,
            z_near: 0.1,
            z_far: 1500.,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{Input, Race, V2};

    #[test]
    fn padded_tracks_fit_hud_safe_area_at_every_aspect() {
        for id in 0..3 {
            let track = Track::new(id);
            for (width, height) in [
                (1920., 1080.),
                (1440., 900.),
                (900., 900.),
                (600., 900.),
                (375., 812.),
                (320., 568.),
                (900., 500.),
            ] {
                let framing = TrackCamera::new(&track, width, height);
                let camera = framing.camera();
                let half = framing.extent * 0.5;
                let right = half * width / height;
                let matrix = Mat4::orthographic_rh_gl(-right, right, -half, half, 0.1, 1500.)
                    * Mat4::look_at_rh(camera.position, camera.target, camera.up);
                let side = 24_f32.min(width * 0.04);
                let top = 180_f32.min(height * 0.30);
                let bottom = 120_f32.min(height * 0.24);
                let radius = track.width * 0.5 + 6.;
                for (point, ground) in track.points.iter().zip(&track.heights) {
                    for x in [point.x - radius, point.x + radius] {
                        for z in [point.y - radius, point.y + radius] {
                            for y in [0., ground + 10.] {
                                let p = matrix.project_point3(vec3(x, y, z));
                                let pixel =
                                    vec2((p.x + 1.) * width * 0.5, (1. - p.y) * height * 0.5);
                                assert!(
                                    pixel.x >= side && pixel.x <= width - side,
                                    "track {id}: {pixel:?}"
                                );
                                assert!(
                                    pixel.y >= top && pixel.y <= height - bottom,
                                    "track {id}: {pixel:?}"
                                );
                                assert!(p.z > -1. && p.z < 1.);
                            }
                        }
                    }
                }
                // The visible road should fill at least one available dimension,
                // rather than fitting a large, mostly empty off-track envelope.
                let mut road_min = Vec2::splat(f32::INFINITY);
                let mut road_max = Vec2::splat(f32::NEG_INFINITY);
                for (i, point) in track.points.iter().enumerate() {
                    let count = track.points.len();
                    let normal = (track.points[(i + 1) % count]
                        - track.points[(i + count - 1) % count])
                        .unit()
                        .normal();
                    for side in [-1., 1.] {
                        let edge = *point + normal * (side * track.width * 0.5);
                        let p = matrix.project_point3(vec3(edge.x, track.heights[i], edge.y));
                        let pixel = vec2((p.x + 1.) * width * 0.5, (1. - p.y) * height * 0.5);
                        road_min = road_min.min(pixel);
                        road_max = road_max.max(pixel);
                        let recovery_edge = *point + normal * (side * (track.width * 0.5 + 17.9));
                        for dx in [-3., 3.] {
                            for dz in [-3., 3.] {
                                let p = matrix.project_point3(vec3(
                                    recovery_edge.x + dx,
                                    track.heights[i] + 0.8,
                                    recovery_edge.y + dz,
                                ));
                                assert!(p.x.abs() < 1. && p.y.abs() < 1.,
                                    "track {id} at {width}x{height}: player clipped before recovery: {p:?}");
                            }
                        }
                    }
                }
                let road_size = road_max - road_min;
                let fill =
                    (road_size.x / (width - 2. * side)).max(road_size.y / (height - top - bottom));
                // Portrait can be limited by keeping the full recovery area on
                // screen; landscape should fill the tighter HUD-safe height.
                let minimum_fill = if width < height { 0.83 } else { 0.85 };
                assert!(
                    fill >= minimum_fill,
                    "track {id} at {width}x{height}: road fills {:.1}%",
                    fill * 100.
                );
            }
        }
    }

    #[test]
    fn vehicle_motion_and_resets_do_not_change_framing() {
        let mut race = Race::new(0, 0, 0, false);
        let mut framing = TrackCamera::new(&race.track, 1440., 900.);
        let original = framing;
        for i in 0..180 {
            race.advance(
                1. / 60.,
                Input {
                    throttle: 1.,
                    ..Default::default()
                },
            );
            race.cars[0].pos = V2::new(i as f32, -i as f32);
            race.cars[0].height = i as f32;
            race.cars[0].impact = 1.;
            race.cars[0].landing = 1.;
            if i % 30 == 0 {
                race.cars[0].reset(&race.track);
            }
            framing.update(&race.track, 1440., 900.);
            assert_eq!(framing, original);
            assert_eq!(framing.camera().position, original.camera().position);
        }
        framing.update(&race.track, 600., 900.);
        assert_ne!(framing, original);
        let track = Track::new(2);
        framing.update(&track, 600., 900.);
        assert_eq!(framing, TrackCamera::new(&track, 600., 900.));
    }
}
