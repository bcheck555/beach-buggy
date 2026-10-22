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
        // Recovery occurs at half-width + 18. Include shoulders, cars and jumps.
        let radius = track.width * 0.5 + 20.;
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
        let top = 210_f32.min(height * 0.30);
        let bottom = 160_f32.min(height * 0.24);
        let side = 24_f32.min(width * 0.04);
        let usable_width = width - 2. * side;
        let usable_height = height - top - bottom;
        let size = max - min;
        let extent = (size.y * height / usable_height).max(size.x * height / usable_width) * 1.04;
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
                (900., 500.),
            ] {
                let framing = TrackCamera::new(&track, width, height);
                let camera = framing.camera();
                let half = framing.extent * 0.5;
                let right = half * width / height;
                let matrix = Mat4::orthographic_rh_gl(-right, right, -half, half, 0.1, 1500.)
                    * Mat4::look_at_rh(camera.position, camera.target, camera.up);
                let side = 24_f32.min(width * 0.04);
                let top = 210_f32.min(height * 0.30);
                let bottom = 160_f32.min(height * 0.24);
                let radius = track.width * 0.5 + 20.;
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
