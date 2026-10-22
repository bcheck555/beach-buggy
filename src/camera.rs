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
        let mut road_min = Vec2::splat(f32::INFINITY);
        let mut road_max = Vec2::splat(f32::NEG_INFINITY);
        let mut min = road_min;
        let mut max = road_max;
        // Center the visible road itself. Fit its recovery area separately so
        // terrain elevation and jump clearance cannot bias the road's center.
        let radius = track.width * 0.5 + 22.;
        for (i, (point, ground)) in track.points.iter().zip(&track.heights).enumerate() {
            let count = track.points.len();
            let normal = (track.points[(i + 1) % count] - track.points[(i + count - 1) % count])
                .unit()
                .normal();
            for side in [-1., 1.] {
                let edge = *point + normal * (side * track.width * 0.5);
                let projected = vec2(edge.x, cos * ground - sin * edge.y);
                road_min = road_min.min(projected);
                road_max = road_max.max(projected);
            }
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
        let center = (road_min + road_max) * 0.5;
        let half_size = (center - min).max(max - center);
        let edge = 16_f32.min(width.min(height) * 0.04);
        let extent = (2. * half_size.x * height / (width - 2. * edge))
            .max(2. * half_size.y * height / (height - 2. * edge));
        Self {
            key: (track.id, width, height),
            target: vec3(center.x, 0., -center.y / sin),
            extent,
        }
    }

    pub fn update(&mut self, track: &Track, width: f32, height: f32) {
        if self.key != (track.id, width.max(1.), height.max(1.)) {
            *self = Self::new(track, width, height);
        }
    }

    pub fn camera(&self, zoom: f32) -> Camera3D {
        let (sin, cos) = ELEVATION.sin_cos();
        let zoom = if zoom.is_finite() {
            zoom.clamp(0.5, 2.)
        } else {
            1.
        };
        Camera3D {
            position: self.target + vec3(0., sin, cos) * 400.,
            target: self.target,
            up: Vec3::Y,
            fovy: self.extent / zoom,
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
    fn fitted_tracks_and_recovery_stay_visible_at_every_aspect() {
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
                let camera = framing.camera(1.);
                let half = framing.extent * 0.5;
                let right = half * width / height;
                let matrix = Mat4::orthographic_rh_gl(-right, right, -half, half, 0.1, 1500.)
                    * Mat4::look_at_rh(camera.position, camera.target, camera.up);
                let border = 16_f32.min(width.min(height) * 0.04);
                let radius = track.width * 0.5 + 22.;
                let mut frame_min = Vec2::splat(f32::INFINITY);
                let mut frame_max = Vec2::splat(f32::NEG_INFINITY);
                for (point, ground) in track.points.iter().zip(&track.heights) {
                    for x in [point.x - radius, point.x + radius] {
                        for z in [point.y - radius, point.y + radius] {
                            for y in [0., ground + 10.] {
                                let p = matrix.project_point3(vec3(x, y, z));
                                let pixel =
                                    vec2((p.x + 1.) * width * 0.5, (1. - p.y) * height * 0.5);
                                assert!(
                                    pixel.x >= border - 0.01 && pixel.x <= width - border + 0.01,
                                    "track {id}: {pixel:?}"
                                );
                                assert!(
                                    pixel.y >= border - 0.01 && pixel.y <= height - border + 0.01,
                                    "track {id}: {pixel:?}"
                                );
                                assert!(p.z > -1. && p.z < 1.);
                                frame_min = frame_min.min(pixel);
                                frame_max = frame_max.max(pixel);
                            }
                        }
                    }
                }
                // Fit must use all available space along at least one edge.
                let clearance = frame_min
                    .x
                    .min(frame_min.y)
                    .min(width - frame_max.x)
                    .min(height - frame_max.y);
                assert!((clearance - border).abs() < 0.02);
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
                let center = (road_min + road_max) * 0.5;
                assert!(
                    (center - vec2(width, height) * 0.5).length() < 0.02,
                    "track {id} at {width}x{height}: road center {center:?}"
                );
            }
        }
    }

    #[test]
    fn zoom_changes_scale_without_moving_the_center_and_clamps_limits() {
        for id in 0..3 {
            let track = Track::new(id);
            for (width, height) in [(1440., 900.), (600., 900.), (320., 568.)] {
                let framing = TrackCamera::new(&track, width, height);
                let fitted = framing.camera(1.);
                for zoom in [0.5, 0.9, 1., 1.5, 2.] {
                    let camera = framing.camera(zoom);
                    assert_eq!(camera.position, fitted.position);
                    assert_eq!(camera.target, fitted.target);
                    assert!((camera.fovy * zoom - fitted.fovy).abs() < 0.001);
                    let half = camera.fovy * 0.5;
                    let right = half * width / height;
                    let matrix = Mat4::orthographic_rh_gl(-right, right, -half, half, 0.1, 1500.)
                        * Mat4::look_at_rh(camera.position, camera.target, camera.up);
                    let center = matrix.project_point3(framing.target);
                    assert!(center.x.abs() < 0.00001 && center.y.abs() < 0.00001);
                }
                assert_eq!(framing.camera(0.).fovy, framing.camera(0.5).fovy);
                assert_eq!(framing.camera(3.).fovy, framing.camera(2.).fovy);
                assert_eq!(framing.camera(f32::NAN).fovy, fitted.fovy);
                assert_eq!(framing.camera(f32::INFINITY).fovy, fitted.fovy);
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
            assert_eq!(framing.camera(1.).position, original.camera(1.).position);
        }
        framing.update(&race.track, 600., 900.);
        assert_ne!(framing, original);
        let track = Track::new(2);
        framing.update(&track, 600., 900.);
        assert_eq!(framing, TrackCamera::new(&track, 600., 900.));
    }
}
