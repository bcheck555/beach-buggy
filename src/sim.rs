use std::f32::consts::{PI, TAU};

pub const DT: f32 = 1.0 / 60.0;
pub const SAMPLES: usize = 128;
pub const TRACK_NAMES: [&str; 3] = ["Beach Loop", "Rocky Coast", "Volcano Run"];
pub const PRESET_NAMES: [&str; 3] = ["Arcade", "Balanced", "Realistic"];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}
impl V2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn dot(self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y
    }
    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn unit(self) -> Self {
        self * (1.0 / self.len().max(0.0001))
    }
    pub fn normal(self) -> Self {
        Self::new(-self.y, self.x)
    }
    pub fn lerp(self, b: Self, t: f32) -> Self {
        self + (b - self) * t
    }
}
impl std::ops::Add for V2 {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y)
    }
}
impl std::ops::Sub for V2 {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y)
    }
}
impl std::ops::Mul<f32> for V2 {
    type Output = Self;
    fn mul(self, b: f32) -> Self {
        Self::new(self.x * b, self.y * b)
    }
}

pub fn angle_diff(a: f32, b: f32) -> f32 {
    (a - b + PI).rem_euclid(TAU) - PI
}

#[derive(Clone, Copy)]
pub struct CarSpec {
    pub name: &'static str,
    pub tagline: &'static str,
    pub mass: f32,
    pub accel: f32,
    pub top: f32,
    pub steering: f32,
    pub grip: f32,
    pub suspension: f32,
    pub color: [u8; 3],
}
pub const CARS: [CarSpec; 4] = [
    CarSpec {
        name: "Sunrunner",
        tagline: "THE ALL-ROUNDER",
        mass: 680.,
        accel: 11.,
        top: 33.,
        steering: 0.55,
        grip: 1.,
        suspension: 38.,
        color: [255, 171, 60],
    },
    CarSpec {
        name: "Sand Flea",
        tagline: "LIGHT / NIMBLE",
        mass: 490.,
        accel: 12.8,
        top: 30.,
        steering: 0.66,
        grip: 1.12,
        suspension: 46.,
        color: [64, 218, 189],
    },
    CarSpec {
        name: "Riptide",
        tagline: "POWER / DRIFT",
        mass: 740.,
        accel: 14.8,
        top: 39.,
        steering: 0.53,
        grip: 0.76,
        suspension: 33.,
        color: [246, 104, 131],
    },
    CarSpec {
        name: "Dune Hauler",
        tagline: "HEAVY / PLANTED",
        mass: 1150.,
        accel: 8.8,
        top: 31.,
        steering: 0.46,
        grip: 1.18,
        suspension: 30.,
        color: [157, 151, 246],
    },
];

#[derive(Clone, Copy)]
pub struct Preset {
    pub grip: f32,
    pub assist: f32,
    pub steering: f32,
    pub power: f32,
}
pub fn preset(id: usize) -> Preset {
    match id {
        0 => Preset {
            grip: 1.7,
            assist: 9.,
            steering: 0.8,
            power: 1.08,
        },
        1 => Preset {
            grip: 1.15,
            assist: 4.5,
            steering: 0.95,
            power: 1.,
        },
        _ => Preset {
            grip: 0.88,
            assist: 1.6,
            steering: 1.,
            power: 1.,
        },
    }
}

#[derive(Clone)]
pub struct Track {
    pub id: usize,
    pub points: Vec<V2>,
    pub heights: Vec<f32>,
    pub width: f32,
    sections: Vec<RoadSection>,
}
#[derive(Clone, Copy, Default)]
pub struct RoadSection {
    pub tunnel: bool,
    pub elevated: bool,
    pub barrier: bool,
    pub surface: usize,
}
#[derive(Clone, Copy)]
pub struct Road {
    pub pos: V2,
    pub tangent: V2,
    pub distance: f32,
    pub index: usize,
    pub height: f32,
    pub grip: f32,
}
impl Track {
    pub fn new(id: usize) -> Self {
        if id == 2 {
            return Self::volcano();
        }
        let controls: &[(f32, f32)] = match id {
            0 => &[
                (0., -76.),
                (57., -69.),
                (87., -24.),
                (68., 37.),
                (14., 72.),
                (-54., 66.),
                (-83., 14.),
                (-56., -50.),
            ],
            1 => &[
                (0., -83.),
                (49., -76.),
                (78., -35.),
                (41., -2.),
                (77., 44.),
                (27., 78.),
                (-36., 57.),
                (-67., 17.),
                (-44., -19.),
                (-66., -65.),
            ],
            _ => &[
                (0., -82.),
                (63., -60.),
                (70., -12.),
                (29., 9.),
                (69., 50.),
                (12., 77.),
                (-54., 58.),
                (-70., 5.),
                (-36., -25.),
                (-57., -67.),
            ],
        };
        let n = controls.len();
        let mut points = Vec::new();
        let mut heights = Vec::new();
        for i in 0..SAMPLES {
            let t = i as f32 / SAMPLES as f32 * n as f32;
            let k = t.floor() as usize;
            let u = t.fract();
            let p = |j: usize| V2::new(controls[j % n].0, controls[j % n].1);
            let (a, b, c, d) = (p((k + n - 1) % n), p(k), p(k + 1), p(k + 2));
            points.push(
                (b * 2.
                    + (c - a) * u
                    + (a * 2. - b * 5. + c * 4. - d) * (u * u)
                    + (b * 3. - a - c * 3. + d) * (u * u * u))
                    * 0.5,
            );
            let f = i as f32 / SAMPLES as f32;
            let base = match id {
                0 => 0.3,
                1 => 2.8 + (f * TAU * 2.).sin() * 2.3,
                _ => 0.8 + (f * TAU * 2.).sin().max(0.) * 1.2,
            };
            // A rising ramp with a sharp exit launches cars without a scripted jump.
            let ramp = if (42..48).contains(&i) {
                (i - 42) as f32 * 0.55
            } else {
                0.
            };
            heights.push(base + ramp);
        }
        Self {
            id,
            points,
            heights,
            width: if id == 0 { 18. } else { 15. },
            sections: vec![
                RoadSection {
                    barrier: id == 1,
                    surface: usize::from(id == 1),
                    ..RoadSection::default()
                };
                SAMPLES
            ],
        }
    }
    fn volcano() -> Self {
        // Two distinct lobes meet only at a height-separated central crossing.
        // Closely spaced controls at the western tips form the braking hairpins.
        let controls = [
            (0., -90., 0.8),
            (65., -83., 0.8),
            (96., -58., 1.),
            (85., -28., 3.),
            (55., -32., 9.),
            (25., -20., 12.),
            (-25., 20., 12.),
            (-60., 48., 10.),
            (-84., 42., 6.),
            (-103., 66., 3.),
            (-70., 95., 0.8),
            (0., 95., 0.8),
            (65., 85., 0.8),
            (96., 60., 0.8),
            (85., 28., 0.8),
            (60., 22., 0.8),
            (25., 20., 0.8),
            (-25., -20., 0.8),
            (-60., -48., 0.8),
            (-84., -42., 0.8),
            (-103., -66., 0.8),
            (-70., -95., 0.8),
        ];
        let n = controls.len();
        let mut dense = Vec::new();
        let mut cumulative = vec![0.];
        for sample in 0..=n * 64 {
            let parameter = sample as f32 / 64.;
            let k = parameter.floor() as usize % n;
            let u = parameter.fract();
            let value = |offset: usize, component: usize| {
                let p = controls[(k + offset) % n];
                match component {
                    0 => p.0,
                    1 => p.1,
                    _ => p.2,
                }
            };
            let interpolate = |component| {
                let (a, b, c, d) = (
                    value(n - 1, component),
                    value(0, component),
                    value(1, component),
                    value(2, component),
                );
                0.5 * (2. * b
                    + (c - a) * u
                    + (2. * a - 5. * b + 4. * c - d) * u * u
                    + (3. * b - a - 3. * c + d) * u * u * u)
            };
            let p = V2::new(interpolate(0), interpolate(1));
            if let Some((previous, _, _)) = dense.last() {
                cumulative.push(cumulative.last().unwrap() + (p - *previous).len());
            }
            dense.push((p, interpolate(2).max(0.8), parameter));
        }
        let length = *cumulative.last().unwrap();
        // Spread the bridge climb across the approach, easing onto the upper deck.
        // The old localized rise after the bend could launch cars over the crossing.
        let climb_start = cumulative[64];
        let climb_end = cumulative[5 * 64];
        let jump_crest = cumulative[11 * 64] - 5.;
        for (i, (_, height, parameter)) in dense.iter_mut().enumerate() {
            let distance = cumulative[i];
            if (1.0..5.0).contains(parameter) {
                let t = ((distance - climb_start) / (climb_end - climb_start)).clamp(0., 1.);
                *height = 0.8 + 11.2 * t * t * (3. - 2. * t);
            } else if (5.0..6.0).contains(parameter) {
                *height = 12.;
            }
            // A modest ramp on the outer straight leaves room to land before the bend.
            let offset = distance - jump_crest;
            let ramp = if (-25.0..0.0).contains(&offset) {
                (offset + 25.) / 25.
            } else if (0.0..2.0).contains(&offset) {
                1. - offset / 2.
            } else {
                0.
            };
            *height += 2.6 * ramp;
        }
        let mut track = Self {
            id: 2,
            points: Vec::new(),
            heights: Vec::new(),
            width: 15.,
            sections: Vec::new(),
        };
        for i in 0..SAMPLES {
            let distance = length * i as f32 / SAMPLES as f32;
            let j = cumulative
                .partition_point(|&d| d <= distance)
                .clamp(1, cumulative.len() - 1);
            let t = (distance - cumulative[j - 1]) / (cumulative[j] - cumulative[j - 1]);
            let (a, ah, ap) = dense[j - 1];
            let (b, bh, bp) = dense[j];
            let parameter = ap + (bp - ap) * t;
            let tunnel = (15.6..17.4).contains(&parameter);
            let elevated = (3.8..8.1).contains(&parameter);
            track.points.push(a.lerp(b, t));
            track.heights.push(ah + (bh - ah) * t);
            track.sections.push(RoadSection {
                tunnel,
                elevated,
                barrier: tunnel || elevated,
                surface: 1,
            });
        }
        track
    }
    pub fn section(&self, index: usize) -> RoadSection {
        self.sections[index % SAMPLES]
    }
    pub fn nearest(&self, pos: V2) -> Road {
        self.query(pos, None)
    }
    pub fn road_for(&self, pos: V2, height: f32, route_index: usize) -> Road {
        self.query(pos, Some((height, route_index)))
    }
    fn query(&self, pos: V2, route: Option<(f32, usize)>) -> Road {
        let mut best = Road {
            pos: self.points[0],
            tangent: V2::new(1., 0.),
            distance: f32::MAX,
            index: 0,
            height: 0.,
            grip: 1.,
        };
        let mut local = best;
        let route = route.filter(|_| self.id == 2);
        let mut best_score = f32::MAX;
        for i in 0..SAMPLES {
            let j = (i + 1) % SAMPLES;
            let delta = self.points[j] - self.points[i];
            let t = ((pos - self.points[i]).dot(delta) / delta.dot(delta)).clamp(0., 1.);
            let p = self.points[i] + delta * t;
            let distance = (pos - p).len();
            let height = self.heights[i] * (1. - t) + self.heights[j] * t;
            let candidate = || Road {
                pos: p,
                tangent: delta.unit(),
                distance,
                index: i,
                height,
                grip: if self.id == 1 { 0.92 } else { 1. },
            };
            let score = if let Some((car_height, index)) = route {
                let separation = i.abs_diff(index);
                let steps = separation.min(SAMPLES - separation);
                // Adjacent segments share a surface: altitude must not snap an
                // offset car uphill. First find the geometric nearest local road.
                if steps <= 10 {
                    if distance < local.distance {
                        local = candidate();
                    }
                    continue;
                }
                distance + (height - car_height).abs() * 2. + 14.
            } else {
                distance
            };
            if score < best_score {
                best_score = score;
                best = candidate();
            }
        }
        // Altitude still selects the right crossing level and recovers stale hints.
        if let Some((height, _)) = route {
            if local.distance + (local.height - height).abs() * 2. < best_score {
                best = local;
            }
        }
        best
    }
    pub fn ground(&self, road: Road) -> f32 {
        let shoulder = ((road.distance - self.width * 0.5) / 9.).clamp(0., 1.);
        road.height * (1. - shoulder)
    }
    pub fn checkpoint(&self, cp: usize) -> (V2, V2) {
        let i = cp * 16 % SAMPLES;
        (
            self.points[i],
            (self.points[(i + 1) % SAMPLES] - self.points[(i + SAMPLES - 1) % SAMPLES]).unit(),
        )
    }
}

#[derive(Clone, Copy, Default)]
pub struct Input {
    pub throttle: f32,
    pub brake: f32,
    pub steer: f32,
    pub handbrake: bool,
}
#[derive(Clone)]
pub struct Car {
    pub road_index: usize,
    pub spec: usize,
    pub pos: V2,
    pub previous: V2,
    pub vel: V2,
    pub heading: f32,
    pub previous_heading: f32,
    pub yaw: f32,
    pub height: f32,
    pub ground_height: f32,
    pub previous_height: f32,
    pub vertical: f32,
    pub pitch: f32,
    pub roll: f32,
    pub steer: f32,
    pub slip: f32,
    pub impact: f32,
    pub throttle: f32,
    pub load: f32,
    pub landing: f32,
    pub surface: usize,
    pub grounded: bool,
    pub next_cp: usize,
    pub laps: u32,
    pub lap_time: f32,
    pub best: f32,
    pub finished: bool,
    pub finish_time: f32,
    pub stuck: f32,
}
impl Car {
    pub fn spawn(track: &Track, spec: usize, grid: usize) -> Self {
        let (p, t) = track.checkpoint(0);
        let pos = p
            + t * (5. - (grid / 2) as f32 * 4.)
            + t.normal() * if grid.is_multiple_of(2) { -2.4 } else { 2.4 };
        let h = track.ground(track.nearest(pos));
        Self {
            road_index: track.nearest(pos).index,
            spec,
            pos,
            previous: pos,
            vel: V2::default(),
            heading: t.y.atan2(t.x),
            previous_heading: t.y.atan2(t.x),
            yaw: 0.,
            height: h,
            ground_height: h,
            previous_height: h,
            vertical: 0.,
            pitch: 0.,
            roll: 0.,
            steer: 0.,
            slip: 0.,
            impact: 0.,
            throttle: 0.,
            load: 0.,
            landing: 0.,
            surface: surface_at(track, track.nearest(pos)),
            grounded: true,
            next_cp: 1,
            laps: 0,
            lap_time: 0.,
            best: 0.,
            finished: false,
            finish_time: 0.,
            stuck: 0.,
        }
    }
    pub fn reset(&mut self, track: &Track) {
        let last = (self.next_cp + 7) % 8;
        let (p, t) = track.checkpoint(last);
        self.pos = p + t * 2.;
        self.previous = self.pos;
        self.heading = t.y.atan2(t.x);
        self.previous_heading = self.heading;
        self.vel = V2::default();
        self.yaw = 0.;
        self.vertical = 0.;
        self.road_index = last * 16;
        self.height =
            track.ground(track.road_for(self.pos, track.heights[self.road_index], self.road_index));
        self.ground_height = self.height;
        self.previous_height = self.height;
        self.grounded = true;
        self.stuck = 0.;
        self.pitch = 0.;
        self.roll = 0.;
        self.steer = 0.;
        self.slip = 0.;
        self.impact = 0.;
        self.throttle = 0.;
        self.load = 0.;
        self.landing = 0.;
        self.surface = surface_at(
            track,
            track.road_for(self.pos, self.height, self.road_index),
        );
    }
    pub fn forward(&self) -> V2 {
        V2::new(self.heading.cos(), self.heading.sin())
    }
    pub fn step(&mut self, input: Input, track: &Track, physics: usize) {
        self.previous = self.pos;
        self.previous_height = self.height;
        self.previous_heading = self.heading;
        self.impact = (self.impact - DT * 4.).max(0.);
        self.landing = (self.landing - DT * 3.).max(0.);
        self.throttle = input.throttle.clamp(0., 1.);
        let spec = CARS[self.spec];
        let tune = preset(physics);
        let road = track.road_for(self.pos, self.height, self.road_index);
        let offroad = road.distance > track.width * 0.5;
        let traction = spec.grip
            * tune.grip
            * road.grip
            * if offroad {
                if self.spec == 3 {
                    0.85
                } else {
                    0.6
                }
            } else {
                1.
            };
        let f = self.forward();
        let side = f.normal();
        let longitudinal = self.vel.dot(f);
        let lateral = self.vel.dot(side);
        self.steer += (input.steer.clamp(-1., 1.) - self.steer) * DT * 8.;
        let speed = longitudinal.abs();
        let effort = self.throttle * (1. - (longitudinal / spec.top).max(0.).powi(2)).max(0.);
        self.load += (effort - self.load) * DT * 8.;
        let ground_before = track.ground(road);
        if self.grounded {
            let drive = input.throttle.clamp(0., 1.)
                * spec.accel
                * tune.power
                * (1. - (longitudinal / spec.top).max(0.).powi(2)).max(0.);
            let brake = if longitudinal > 0.8 {
                -input.brake * 20.
            } else {
                -input.brake * spec.accel * 0.55
            };
            let drag =
                longitudinal * (if offroad { 0.8 } else { 0.14 }) + longitudinal * speed * 0.002;
            let grip = traction * if input.handbrake { 0.2 } else { 1. };
            let lateral_accel = (-lateral * (tune.assist + 4.)).clamp(-grip * 13., grip * 13.);
            self.vel = self.vel + (f * (drive + brake - drag) + side * lateral_accel) * DT;
            let max_steer = spec.steering * tune.steering / (1. + speed * 0.018);
            let desired = (longitudinal / 2.7) * (self.steer * max_steer).tan();
            let yaw_limit = (traction * 14. / speed.max(5.)).max(0.45);
            self.yaw += (desired.clamp(-yaw_limit, yaw_limit) - self.yaw) * DT * (tune.assist + 2.);
            if input.handbrake {
                self.yaw += self.steer * DT * 0.8;
            }
            self.heading += self.yaw * DT;
            self.slip = lateral.abs() / speed.max(4.);
            // Springs follow rising terrain; downhill edges release the vehicle.
            let ahead =
                track.ground(track.road_for(self.pos + self.vel * DT, self.height, road.index));
            let ground_velocity = (ahead - ground_before) / DT;
            self.vertical += (ground_velocity - self.vertical) * (spec.suspension * DT).min(1.);
            let transfer = -(drive + brake - drag) * spec.mass / 680. * 0.004;
            self.pitch += (((ground_velocity / speed.max(3.)).atan() + transfer)
                .clamp(-0.45, 0.45)
                - self.pitch)
                * DT
                * 7.;
            self.roll += ((-self.yaw * speed * spec.mass / 680. * 0.005).clamp(-0.3, 0.3)
                - self.roll)
                * DT
                * 6.;
        } else {
            self.vertical -= 14. * DT;
            self.slip *= 0.98;
            self.pitch -= DT * 0.16;
        }
        self.pos = self.pos + self.vel * DT;
        let next = track.road_for(self.pos, self.height, road.index);
        self.road_index = next.index;
        let ground = track.ground(next);
        self.ground_height = ground;
        self.height += self.vertical * DT;
        if self.height <= ground + 0.02 {
            if !self.grounded {
                self.impact = (-self.vertical / 13.).clamp(0., 1.);
                self.landing = self.landing.max(self.impact);
            }
            self.height = ground;
            self.grounded = true;
        } else {
            self.grounded = false;
        }
        self.surface = surface_at(track, next);
        // Railings constrain rocky sections and bridges, shoulders remain drivable elsewhere.
        let rails = track.section(next.index).barrier;
        if rails && next.distance > track.width * 0.5 + 1. {
            let normal = (self.pos - next.pos).unit();
            let outward = self.vel.dot(normal);
            if outward > 0. {
                self.vel = self.vel - normal * (outward * 1.35);
                self.impact = (outward / 12.).min(1.);
            }
            self.pos = next.pos + normal * (track.width * 0.5 + 1.);
        }
        if next.distance > track.width * 0.5 + 18. || !self.pos.x.is_finite() {
            self.reset(track);
        }
    }
    pub fn checkpoint_step(&mut self, track: &Track, elapsed: f32) {
        if self.finished {
            return;
        }
        self.lap_time += DT;
        let (p, t) = track.checkpoint(self.next_cp);
        let before = (self.previous - p).dot(t);
        let after = (self.pos - p).dot(t);
        if before < 0. && after >= 0. {
            let fraction = (-before / (after - before)).clamp(0., 1.);
            let crossing = self.previous.lerp(self.pos, fraction);
            let crossing_height = self.previous_height * (1. - fraction) + self.height * fraction;
            if (crossing - p).dot(t.normal()).abs() < track.width * 0.5 + 2.
                && (track.id != 2
                    || (crossing_height - track.heights[self.next_cp * 16]).abs() < 3.)
            {
                if self.next_cp == 0 {
                    self.laps += 1;
                    if self.best == 0. || self.lap_time < self.best {
                        self.best = self.lap_time;
                    }
                    self.lap_time = 0.;
                    if self.laps >= 3 {
                        self.finished = true;
                        self.finish_time = elapsed;
                    }
                }
                self.next_cp = (self.next_cp + 1) % 8;
            }
        }
    }
    pub fn progress(&self, track: &Track) -> f32 {
        let last = (self.next_cp + 7) % 8;
        if track.id == 2 {
            let road = track.road_for(self.pos, self.height, self.road_index);
            let delta = track.points[(road.index + 1) % SAMPLES] - track.points[road.index];
            let fraction =
                ((self.pos - track.points[road.index]).dot(delta) / delta.dot(delta)).clamp(0., 1.);
            let delta = road.index as f32 + fraction - (last * 16) as f32;
            let signed = if delta < -(SAMPLES as f32) * 0.5 {
                delta + SAMPLES as f32
            } else if delta > SAMPLES as f32 * 0.5 {
                delta - SAMPLES as f32
            } else {
                delta
            };
            let relative = (signed / 16.).clamp(0., 0.99);
            return self.laps as f32 * 8. + last as f32 + relative;
        }
        let (a, _) = track.checkpoint(last);
        let (b, _) = track.checkpoint(self.next_cp);
        let segment = ((self.pos - a).dot((b - a).unit()) / (b - a).len()).clamp(0., 0.99);
        self.laps as f32 * 8. + last as f32 + segment
    }
}

pub fn collide(a: &mut Car, b: &mut Car) {
    let delta = b.pos - a.pos;
    let distance = delta.len();
    if distance < 2.8 && (a.height - b.height).abs() < 1.6 {
        let normal = if distance < 0.001 {
            V2::new(1., 0.)
        } else {
            delta.unit()
        };
        let inv_a = 1. / CARS[a.spec].mass;
        let inv_b = 1. / CARS[b.spec].mass;
        let total = inv_a + inv_b;
        let separation = 2.8 - distance;
        a.pos = a.pos - normal * (separation * inv_a / total);
        b.pos = b.pos + normal * (separation * inv_b / total);
        let approach = (b.vel - a.vel).dot(normal);
        if approach < 0. {
            let impulse = -approach * 1.3 / total;
            a.vel = a.vel - normal * (impulse * inv_a);
            b.vel = b.vel + normal * (impulse * inv_b);
            a.impact = (-approach / 15.).min(1.);
            b.impact = a.impact;
        }
    }
}

fn surface_at(track: &Track, road: Road) -> usize {
    if road.distance > track.width * 0.5 {
        0
    } else {
        track.section(road.index).surface
    }
}

pub fn ai_input(car: &Car, track: &Track, physics: usize) -> Input {
    ai_drive(car, track, physics, 1, 0., f32::INFINITY)
}

fn ai_drive(
    car: &Car,
    track: &Track,
    physics: usize,
    difficulty: usize,
    lane: f32,
    speed_cap: f32,
) -> Input {
    let road = track.road_for(car.pos, car.height, car.road_index);
    let speed = car.vel.len();
    let look = if track.id == 2 {
        2 + (speed * 0.04) as usize
    } else {
        3 + (speed * 0.08) as usize
    };
    let target_index = (road.index + look) % SAMPLES;
    let target_tangent = (track.points[(target_index + 1) % SAMPLES]
        - track.points[(target_index + SAMPLES - 1) % SAMPLES])
        .unit();
    let target = track.points[target_index] + target_tangent.normal() * lane;
    let direction = target - car.pos;
    let error = angle_diff(direction.y.atan2(direction.x), car.heading);
    let tangent_a = (track.points[(road.index + 3) % SAMPLES] - road.pos).unit();
    let tangent_b = (track.points[(road.index + 8) % SAMPLES]
        - track.points[(road.index + 3) % SAMPLES])
        .unit();
    let bend = angle_diff(
        tangent_b.y.atan2(tangent_b.x),
        tangent_a.y.atan2(tangent_a.x),
    )
    .abs();
    let grip = (CARS[car.spec].grip * preset(physics).grip).sqrt().min(1.2);
    // Difficulty changes pace; handling still determines safe corner speed.
    let pace = [0.84, 1., 1.12][difficulty.min(2)];
    let desired = if track.id == 2 {
        ((CARS[car.spec].top * 0.68 * grip / (1. + bend * 3.2)).max(6.5) * pace).min(speed_cap)
    } else {
        ((CARS[car.spec].top * 0.62 * grip / (1. + bend * 2.8)).max(7.) * pace).min(speed_cap)
    };
    let error_speed = desired - speed;
    Input {
        throttle: (error_speed * 0.6).clamp(0., 0.9),
        // Brake becomes reverse near rest, so reduce the final pulse to avoid backing into traffic.
        brake: if error_speed < -0.15 && car.vel.dot(car.forward()) > 0.05 {
            ((-error_speed * 0.22).clamp(0., 1.)).min(speed / (20. * DT))
        } else {
            0.
        },
        steer: (error * 2.1 - car.yaw * 0.18).clamp(-1., 1.),
        handbrake: false,
    }
}

#[derive(Clone, Default)]
struct AiState {
    lane: f32,
    target_lane: f32,
    commitment: f32,
    waiting: bool,
}

fn traffic_input(
    index: usize,
    cars: &[Car],
    track: &Track,
    physics: usize,
    difficulty: usize,
    state: &mut AiState,
) -> Input {
    let car = &cars[index];
    let road = track.road_for(car.pos, car.height, car.road_index);
    let forward = road.tangent;
    let side = forward.normal();
    let lateral = (car.pos - road.pos).dot(side);
    let speed = car.vel.dot(forward).max(0.);
    let margin = track.width * 0.5 - 1.8;
    state.commitment = (state.commitment - DT).max(0.);
    state.waiting = false;
    let mut lead: Option<(f32, f32, f32)> = None;
    for (other_index, other) in cars.iter().enumerate() {
        if other_index == index || (other.height - car.height).abs() > 1.6 {
            continue;
        }
        let delta = other.pos - car.pos;
        let ahead = delta.dot(forward);
        let across = delta.dot(side);
        if ahead > 0.
            && ahead < 10. + speed * 1.4
            && across.abs() < 3.5
            && lead.is_none_or(|(gap, _, _)| ahead < gap)
        {
            lead = Some((ahead, lateral + across, other.vel.dot(forward).max(0.)));
        }
    }
    if let Some((gap, lead_lane, lead_speed)) = lead {
        if state.commitment <= 0. && (lead_speed < speed + 1. || gap < 7.) {
            // Choose an open adjacent corridor. Check the entire lateral move as well as
            // its destination, so a passing car never cuts through a car alongside it.
            let mut candidates = [lead_lane - 3.9, lead_lane + 3.9];
            candidates.sort_by(|a, b| (a - lateral).abs().total_cmp(&(b - lateral).abs()));
            for candidate in candidates {
                if candidate.abs() > margin {
                    continue;
                }
                let clear = cars.iter().enumerate().all(|(other_index, other)| {
                    if other_index == index || (other.height - car.height).abs() > 1.6 {
                        return true;
                    }
                    let delta = other.pos - car.pos;
                    let ahead = delta.dot(forward);
                    let other_lane = lateral + delta.dot(side);
                    let in_corridor = (other_lane - candidate).abs() < 3.4;
                    let crosses_move = other_lane > lateral.min(candidate) - 2.8
                        && other_lane < lateral.max(candidate) + 2.8;
                    !(ahead > -7. && ahead < 15. && in_corridor
                        || ahead.abs() < 3.5 && crosses_move)
                });
                if clear {
                    state.target_lane = candidate;
                    state.commitment = 1.4;
                    break;
                }
            }
        }
    } else if state.commitment <= 0. {
        // Hold position while alongside another car; otherwise gently return to a racing line.
        let alongside = cars.iter().enumerate().any(|(other_index, other)| {
            other_index != index
                && (other.height - car.height).abs() < 1.6
                && (other.pos - car.pos).dot(forward).abs() < 6.
                && (other.pos - car.pos).dot(side).abs() < 6.
        });
        if !alongside {
            state.target_lane = if index.is_multiple_of(2) { -1.8 } else { 1.8 };
        }
    }
    state.lane +=
        (state.target_lane.clamp(-margin, margin) - state.lane).clamp(-DT * 2.5, DT * 2.5);
    let mut speed_cap = f32::INFINITY;
    if let Some((gap, _, lead_speed)) = lead {
        // Leave room to stop even during a lane change. No pace adjustment depends on
        // the player's standing or lap time: this is local collision avoidance only.
        speed_cap = (lead_speed * lead_speed + 16. * (gap - 4.2).max(0.)).sqrt();
        if gap < 4.2 {
            speed_cap = (lead_speed - (4.2 - gap) * 3.).max(0.);
        }
        state.waiting = lead_speed < 1. && gap < 6.;
    }
    ai_drive(car, track, physics, difficulty, state.lane, speed_cap)
}

pub struct Race {
    pub track: Track,
    pub cars: Vec<Car>,
    pub physics: usize,
    pub difficulty: usize,
    pub trial: bool,
    pub countdown: f32,
    pub elapsed: f32,
    pub accumulator: f32,
    ai: Vec<AiState>,
}
impl Race {
    pub fn new(track: usize, car: usize, physics: usize, trial: bool) -> Self {
        Self::with_difficulty(track, car, physics, trial, 1)
    }
    pub fn with_difficulty(
        track: usize,
        car: usize,
        physics: usize,
        trial: bool,
        difficulty: usize,
    ) -> Self {
        let track = Track::new(track);
        let cars: Vec<_> = (0..if trial { 1 } else { 6 })
            .map(|i| Car::spawn(&track, if i == 0 { car } else { (i - 1) % 4 }, i))
            .collect();
        let ai = cars
            .iter()
            .map(|car| {
                let road = track.nearest(car.pos);
                let lane = (car.pos - road.pos).dot(road.tangent.normal());
                AiState {
                    lane,
                    target_lane: lane,
                    ..AiState::default()
                }
            })
            .collect();
        Self {
            track,
            cars,
            physics,
            difficulty: difficulty.min(2),
            trial,
            countdown: 3.,
            elapsed: 0.,
            accumulator: 0.,
            ai,
        }
    }
    pub fn tick(&mut self, input: Input) {
        if self.countdown > 0. {
            self.countdown -= DT;
            return;
        }
        self.elapsed += DT;
        // Every controller observes the same instant, independent of update order.
        let snapshot = self.cars.clone();
        let controls: Vec<_> = (0..snapshot.len())
            .map(|i| {
                if i == 0 && !snapshot[0].finished {
                    input
                } else {
                    traffic_input(
                        i,
                        &snapshot,
                        &self.track,
                        self.physics,
                        self.difficulty,
                        &mut self.ai[i],
                    )
                }
            })
            .collect();
        for (i, control) in controls.into_iter().enumerate() {
            self.cars[i].step(control, &self.track, self.physics);
            if i > 0 {
                if self.cars[i].vel.len() < 2. && !self.ai[i].waiting {
                    self.cars[i].stuck += DT;
                } else {
                    self.cars[i].stuck = 0.;
                }
                if self.cars[i].stuck > 4. {
                    self.cars[i].reset(&self.track);
                }
            }
        }
        for i in 0..self.cars.len() {
            for j in i + 1..self.cars.len() {
                let (left, right) = self.cars.split_at_mut(j);
                collide(&mut left[i], &mut right[0]);
            }
        }
        for car in &mut self.cars {
            car.checkpoint_step(&self.track, self.elapsed);
        }
    }
    pub fn advance(&mut self, seconds: f32, input: Input) {
        self.accumulator += seconds.clamp(0., 0.2);
        while self.accumulator >= DT {
            self.tick(input);
            self.accumulator -= DT;
        }
    }
    pub fn position(&self) -> usize {
        let me = &self.cars[0];
        1 + self
            .cars
            .iter()
            .skip(1)
            .filter(|c| {
                if me.finished {
                    c.finished && c.finish_time < me.finish_time
                } else {
                    c.finished || c.progress(&self.track) > me.progress(&self.track)
                }
            })
            .count()
    }
}

pub fn record_key(track: usize, car: usize, physics: usize) -> i32 {
    (if track == 2 { 200 } else { 100 + track * 12 } + car * 3 + physics) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    fn place(track: &Track, index: usize, spec: usize) -> Car {
        let mut car = Car::spawn(track, spec, 0);
        car.pos = track.points[index];
        car.previous = car.pos;
        car.road_index = index;
        car.height = track.heights[index];
        car.previous_height = car.height;
        car.ground_height = car.height;
        let tangent = (track.points[(index + 1) % SAMPLES] - track.points[index]).unit();
        car.heading = tangent.y.atan2(tangent.x);
        car
    }
    #[test]
    fn volcano_jump_approaches_are_controlled() {
        let track = Track::new(2);
        for (name, start, end) in [
            ("climb", V2::new(55., -32.), V2::new(-25., 20.)),
            ("jump", V2::new(-40., 95.), V2::new(45., 90.)),
        ] {
            let start = track.nearest(start).index;
            let end = track.nearest(end).index;
            for (spec, profile) in CARS.iter().enumerate() {
                for physics in 0..3 {
                    let mut full_speed_peak = 0_f32;
                    // Full speed is the equilibrium of drive and drag, not the nominal top.
                    let mut low = 0.;
                    let mut high = profile.top;
                    for _ in 0..40 {
                        let speed = (low + high) * 0.5;
                        let drive = profile.accel
                            * preset(physics).power
                            * (1. - (speed / profile.top).powi(2));
                        if drive > speed * 0.14 + speed * speed * 0.002 {
                            low = speed;
                        } else {
                            high = speed;
                        }
                    }
                    for fraction in [0.0, 0.7, 1.0] {
                        let speed = if fraction == 0. { 8. } else { low * fraction };
                        let throttle = (speed * 0.14 + speed * speed * 0.002)
                            / (profile.accel
                                * preset(physics).power
                                * (1. - (speed / profile.top).powi(2)));
                        for offset in [-6., -3., 0., 3., 6.] {
                            // Shift crest arrival within a physics step; suspension is phase-sensitive.
                            for phase_step in 0..8 {
                                let phase = speed * DT * phase_step as f32 / 8.;
                                let mut car = place(&track, start, spec);
                                car.pos = car.pos
                                    + car.forward().normal() * offset
                                    + car.forward() * phase;
                                car.previous = car.pos;
                                car.vel = car.forward() * speed;
                                let mut peak = 0_f32;
                                let mut airborne = 0_f32;
                                let mut longest = 0_f32;
                                let mut distance = 0_f32;
                                let mut exited = false;
                                for _ in 0..1800 {
                                    let mut input = ai_input(&car, &track, physics);
                                    input.throttle = throttle;
                                    if name == "jump" {
                                        input.brake = 0.;
                                    }
                                    let previous = car.pos;
                                    car.step(input, &track, physics);
                                    let road = if name == "jump" {
                                        track.nearest(car.pos)
                                    } else {
                                        track.road_for(car.pos, car.height, car.road_index)
                                    };
                                    distance = distance.max(road.distance);
                                    peak = peak.max(car.height - track.ground(road));
                                    if car.grounded {
                                        airborne = 0.;
                                    } else {
                                        airborne += DT;
                                    }
                                    longest = longest.max(airborne);
                                    assert!((car.pos - previous).len() < 2., "reset on {name}");
                                    if car.road_index >= end {
                                        exited = true;
                                        break;
                                    }
                                }
                                assert!(exited && car.grounded, "{name} car{spec} physics{physics} speed{speed} offset{offset} failed to land/exit");
                                let limit = if name == "climb" {
                                    1.
                                } else if fraction == 0. {
                                    0.2
                                } else {
                                    3.
                                };
                                assert!(peak <= limit && longest <= 1.25,
                                "{name} car{spec} physics{physics} fraction{fraction} offset{offset} phase{phase}: peak={peak} air={longest}");
                                if name == "jump" {
                                    let runoff = if offset.abs() > 3. { 9. } else { 0. };
                                    assert!(distance < track.width * 0.5 + runoff, "jump left the road/runoff car{spec} physics{physics} offset{offset} phase{phase}");
                                    if fraction == 1. {
                                        full_speed_peak = full_speed_peak.max(peak);
                                    }
                                }
                            }
                        }
                    }
                    if name == "jump" {
                        assert!(
                            full_speed_peak > 0.5,
                            "car{spec} physics{physics} lost its jump"
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn volcano_ramp_query_keeps_local_surface_and_recovers_crossing_level() {
        let track = Track::new(2);
        // This shoulder approach previously selected a higher segment behind the
        // car, turning a downhill ramp into an enormous upward suspension impulse.
        let pos = V2::new(-1.492, 102.308);
        let nearest = track.nearest(pos);
        for height in [1.5, 2.5, 4.] {
            let road = track.road_for(pos, height, nearest.index);
            assert_eq!(road.index, nearest.index);
            assert!((road.height - nearest.height).abs() < 0.001);
        }
        // Stale route hints must still recover by altitude at the stacked crossing.
        assert!(track.road_for(V2::default(), 12., 100).height > 10.);
        assert!(track.road_for(V2::default(), 0.8, 35).height < 2.);
    }
    #[test]
    fn volcano_first_bend_has_a_gentle_bridge_approach() {
        let track = Track::new(2);
        let start = track.nearest(V2::new(65., -83.)).index;
        let end = track.nearest(V2::new(-25., 20.)).index;
        for spec in 0..CARS.len() {
            for physics in 0..3 {
                for offset in [-3., 0., 3.] {
                    let mut car = place(&track, start, spec);
                    car.pos = car.pos + car.forward().normal() * offset;
                    let mut exited = false;
                    for _ in 0..1800 {
                        let previous = car.pos;
                        car.step(ai_input(&car, &track, physics), &track, physics);
                        let road = track.road_for(car.pos, car.height, car.road_index);
                        assert!(
                            car.height - track.ground(road) < 0.5,
                            "bridge approach launched car{spec} physics{physics}"
                        );
                        assert!((car.pos - previous).len() < 2., "bridge approach reset");
                        if car.road_index >= end {
                            exited = true;
                            break;
                        }
                    }
                    assert!(exited, "bridge approach failed car{spec} physics{physics}");
                }
            }
        }
    }
    #[test]
    fn volcano_crossing_keeps_levels_and_collisions_separate() {
        let track = Track::new(2);
        let upper = track.road_for(V2::default(), 12., 35);
        let lower = track.road_for(V2::default(), 0.8, 100);
        assert!(track.section(upper.index).elevated);
        assert!(track.section(lower.index).tunnel);
        assert!(upper.height - lower.height > 10.);
        let mut a = place(&track, upper.index, 0);
        let mut b = place(&track, lower.index, 1);
        a.pos = V2::default();
        b.pos = a.pos;
        let before = (a.pos, b.pos);
        collide(&mut a, &mut b);
        assert_eq!((a.pos, b.pos), before);
        for _ in 0..30 {
            a.step(Input::default(), &track, 0);
            b.step(Input::default(), &track, 0);
            assert!(a.height - b.height > 10.);
            assert!(track.section(a.road_index).elevated);
            assert!(track.section(b.road_index).tunnel);
        }
    }
    #[test]
    fn volcano_gates_reset_and_reverse_preserve_route() {
        let track = Track::new(2);
        for cp in 0..8 {
            let mut car = place(&track, cp * 16, 0);
            car.next_cp = cp;
            car.height = track.heights[cp * 16] + 10.;
            car.previous_height = car.height;
            cross(&mut car, &track, cp, true);
            assert_eq!(car.next_cp, cp, "wrong-level checkpoint {cp}");
            car.next_cp = (cp + 1) % 8;
            car.height = 100.;
            car.road_index = (cp * 16 + 64) % SAMPLES;
            car.reset(&track);
            assert!((car.height - track.heights[cp * 16]).abs() < 1.);
            assert!(car.road_index.abs_diff(cp * 16) <= 1);
            car = place(&track, (cp * 16 + SAMPLES - 1) % SAMPLES, 0);
            car.next_cp = (cp + 1) % 8;
            assert!(
                (car.progress(&track) - cp as f32).abs() < 0.01,
                "reversing behind checkpoint {cp}"
            );
        }
    }
    #[test]
    fn volcano_tunnel_walls_match_drivable_boundary() {
        let track = Track::new(2);
        let index = (0..SAMPLES).find(|&i| track.section(i).tunnel).unwrap();
        let mut car = place(&track, index, 0);
        let road = track.road_for(car.pos, car.height, index);
        car.pos = car.pos + road.tangent.normal() * 12.;
        car.vel = road.tangent.normal() * 8.;
        car.step(Input::default(), &track, 1);
        let road = track.road_for(car.pos, car.height, car.road_index);
        assert!(road.distance <= track.width * 0.5 + 1.01);
        assert!(car.impact > 0.);
    }
    #[test]
    fn volcano_hairpins_reward_braking_for_every_car_and_preset() {
        let track = Track::new(2);
        for apex in [V2::new(-84., 42.), V2::new(-84., -42.)] {
            let index = track.nearest(apex).index;
            for (spec, profile) in CARS.iter().enumerate() {
                for physics in 0..3 {
                    let mut braked = place(&track, (index + SAMPLES - 6) % SAMPLES, spec);
                    // Solve steady-state full-throttle drive against the actual drag law.
                    let mut low = 0.;
                    let mut high = profile.top;
                    for _ in 0..40 {
                        let speed = (low + high) * 0.5;
                        let drive = profile.accel
                            * preset(physics).power
                            * (1. - (speed / profile.top).powi(2));
                        if drive > speed * 0.14 + speed * speed * 0.002 {
                            low = speed;
                        } else {
                            high = speed;
                        }
                    }
                    braked.vel = braked.forward() * ((low + high) * 0.5);
                    let mut flat = braked.clone();
                    let mut brake_error = 0_f32;
                    let mut flat_error = 0_f32;
                    let mut exited = false;
                    for _ in 0..1200 {
                        let input = ai_input(&braked, &track, physics);
                        braked.step(input, &track, physics);
                        let mut input = ai_input(&flat, &track, physics);
                        input.throttle = 1.;
                        input.brake = 0.;
                        flat.step(input, &track, physics);
                        brake_error = brake_error.max(
                            track
                                .road_for(braked.pos, braked.height, braked.road_index)
                                .distance,
                        );
                        flat_error = flat_error.max(
                            track
                                .road_for(flat.pos, flat.height, flat.road_index)
                                .distance,
                        );
                        if braked.road_index >= index + 3 {
                            exited = true;
                            break;
                        }
                    }
                    assert!(
                        exited,
                        "braking failed to finish hairpin car{spec} physics{physics}"
                    );
                    assert!(brake_error<track.width*0.5 && flat_error>track.width*0.5,"apex {apex:?} car{spec} physics{physics}: brake={brake_error} throttle={flat_error}");
                }
            }
        }
    }
    #[test]
    fn volcano_road_ribbon_has_room_for_smooth_shoulders() {
        let track = Track::new(2);
        for i in 0..SAMPLES {
            let a = track.points[(i + SAMPLES - 1) % SAMPLES];
            let b = track.points[i];
            let c = track.points[(i + 1) % SAMPLES];
            let ab = b - a;
            let bc = c - b;
            let area = (ab.x * bc.y - ab.y * bc.x).abs();
            let radius = ab.len() * bc.len() * (c - a).len() / (2. * area.max(0.0001));
            assert!(radius > 10., "sample{i} radius{radius}");
        }
    }
    #[test]
    fn car_profiles_and_presets_differ() {
        let track = Track::new(0);
        let speeds: Vec<_> = (0..4)
            .map(|id| {
                let mut c = Car::spawn(&track, id, 0);
                for _ in 0..90 {
                    c.step(
                        Input {
                            throttle: 1.,
                            ..Input::default()
                        },
                        &track,
                        1,
                    );
                }
                c.vel.len()
            })
            .collect();
        assert!(speeds[2] > speeds[0] && speeds[0] > speeds[3]);
        assert!(CARS[1].steering > CARS[3].steering);
        assert!(preset(0).grip > preset(1).grip && preset(1).grip > preset(2).grip);
    }
    #[test]
    fn braking_and_lateral_grip() {
        let t = Track::new(0);
        let mut c = Car::spawn(&t, 0, 0);
        c.vel = c.forward() * 18.;
        for _ in 0..30 {
            c.step(
                Input {
                    brake: 1.,
                    ..Input::default()
                },
                &t,
                1,
            );
        }
        assert!(c.vel.len() < 12.);
        let mut a = Car::spawn(&t, 0, 0);
        a.vel = a.forward() * 12. + a.forward().normal() * 6.;
        let mut b = a.clone();
        for _ in 0..12 {
            a.step(Input::default(), &t, 0);
            b.step(Input::default(), &t, 2);
        }
        assert!(a.vel.dot(a.forward().normal()).abs() < b.vel.dot(b.forward().normal()).abs());
    }
    #[test]
    fn falls_lands_and_resets() {
        let t = Track::new(0);
        let mut c = Car::spawn(&t, 0, 0);
        c.height = 8.;
        c.grounded = false;
        for _ in 0..180 {
            c.step(Input::default(), &t, 1);
        }
        assert!(c.grounded && (c.height - t.ground(t.nearest(c.pos))).abs() < 0.1);
        c.pos = V2::new(1000., 1000.);
        c.step(Input::default(), &t, 1);
        assert!(t.nearest(c.pos).distance < 1.);
    }
    #[test]
    fn collisions_separate_and_conserve_momentum() {
        let t = Track::new(0);
        let mut a = Car::spawn(&t, 0, 0);
        let mut b = a.clone();
        b.spec = 3;
        b.pos = a.pos + V2::new(2., 0.);
        a.vel = V2::new(10., 0.);
        let momentum = a.vel.x * CARS[a.spec].mass;
        collide(&mut a, &mut b);
        assert!((b.pos - a.pos).len() >= 2.799);
        assert!(
            (a.vel.x * CARS[a.spec].mass + b.vel.x * CARS[b.spec].mass - momentum).abs() < 0.01
        );
    }
    fn cross(car: &mut Car, t: &Track, cp: usize, forward: bool) {
        let (p, d) = t.checkpoint(cp);
        car.previous = p - d * if forward { 1. } else { -1. };
        car.pos = p + d * if forward { 1. } else { -1. };
        car.checkpoint_step(t, 100.);
    }
    #[test]
    fn ordered_directional_laps_and_finish() {
        let t = Track::new(0);
        let mut c = Car::spawn(&t, 0, 0);
        cross(&mut c, &t, 0, true);
        assert_eq!(c.laps, 0);
        cross(&mut c, &t, 1, false);
        assert_eq!(c.next_cp, 1);
        for _ in 0..3 {
            for cp in [1, 2, 3, 4, 5, 6, 7, 0] {
                cross(&mut c, &t, cp, true);
            }
        }
        assert_eq!(c.laps, 3);
        assert!(c.finished);
        assert!(c.best > 0.);
        cross(&mut c, &t, 1, true);
        assert_eq!(c.laps, 3);
    }
    #[test]
    fn checkpoint_outside_gate_rejected() {
        let t = Track::new(0);
        let mut c = Car::spawn(&t, 0, 0);
        let (p, d) = t.checkpoint(1);
        c.previous = p + d.normal() * 30. - d;
        c.pos = p + d.normal() * 30. + d;
        c.checkpoint_step(&t, 1.);
        assert_eq!(c.next_cp, 1);
    }
    #[test]
    fn fixed_step_is_render_rate_independent() {
        let mut a = Race::new(0, 0, 1, true);
        let mut b = Race::new(0, 0, 1, true);
        let input = Input {
            throttle: 1.,
            steer: 0.1,
            ..Input::default()
        };
        for _ in 0..600 {
            a.advance(1. / 60., input);
        }
        for _ in 0..300 {
            b.advance(1. / 30., input);
        }
        assert!((a.cars[0].pos - b.cars[0].pos).len() < 0.01);
    }
    #[test]
    fn records_are_separate() {
        let mut keys = std::collections::HashSet::new();
        for t in 0..3 {
            for c in 0..4 {
                for p in 0..3 {
                    assert!(keys.insert(record_key(t, c, p)));
                    if t == 2 {
                        assert!(record_key(t, c, p) >= 200);
                        assert_ne!(record_key(t, c, p), (124 + c * 3 + p) as i32);
                    } else {
                        assert_eq!(record_key(t, c, p), (100 + t * 12 + c * 3 + p) as i32);
                    }
                }
            }
        }
    }
    #[test]
    fn ai_completes_all_track_car_preset_combinations() {
        for track in 0..3 {
            for car in 0..4 {
                for physics in 0..3 {
                    let mut race = Race::new(track, car, physics, true);
                    race.countdown = 0.;
                    for _ in 0..60 * if track == 2 { 360 } else { 240 } {
                        let input = ai_input(&race.cars[0], &race.track, physics);
                        race.tick(input);
                        if race.cars[0].finished {
                            break;
                        }
                    }
                    assert!(
                        race.cars[0].finished,
                        "track {track} car {car} physics {physics}: cp {} lap {} at {:?}",
                        race.cars[0].next_cp, race.cars[0].laps, race.cars[0].pos
                    );
                }
            }
        }
    }
    #[test]
    fn race_position_respects_finish_order() {
        let mut r = Race::new(0, 0, 1, false);
        r.cars[0].finished = true;
        r.cars[0].finish_time = 90.;
        r.cars[1].finished = true;
        r.cars[1].finish_time = 89.;
        assert_eq!(r.position(), 2);
    }

    #[test]
    fn six_car_ai_grids_finish_each_track_and_preset() {
        for track in 0..3 {
            for physics in 0..3 {
                let mut race = Race::new(track, 0, physics, false);
                race.countdown = 0.;
                for _ in 0..60 * if track == 2 { 480 } else { 300 } {
                    let input = ai_input(&race.cars[0], &race.track, physics);
                    race.tick(input);
                    if race.cars.iter().all(|c| c.finished) {
                        break;
                    }
                }
                assert!(
                    race.cars.iter().all(|c| c.finished),
                    "grid track {track} physics {physics}: {:?}",
                    race.cars
                        .iter()
                        .map(|c| (c.spec, c.laps, c.next_cp))
                        .collect::<Vec<_>>()
                );
            }
        }
    }

    #[test]
    fn ramp_launches_and_lands_without_scripted_jump() {
        let mut race = Race::new(0, 0, 1, true);
        race.countdown = 0.;
        let mut airborne = false;
        let mut landed = false;
        for _ in 0..60 * 50 {
            let input = ai_input(&race.cars[0], &race.track, 1);
            race.tick(input);
            let car = &race.cars[0];
            if !car.grounded && car.height - race.track.ground(race.track.nearest(car.pos)) > 0.2 {
                airborne = true;
            }
            if airborne && car.grounded {
                landed = true;
                break;
            }
        }
        assert!(airborne && landed);
    }

    #[test]
    fn handbrake_changes_slip_and_reverse_works() {
        let track = Track::new(0);
        let mut a = Car::spawn(&track, 0, 0);
        a.vel = a.forward() * 16. + a.forward().normal() * 5.;
        let mut b = a.clone();
        for _ in 0..15 {
            a.step(Input::default(), &track, 1);
            b.step(
                Input {
                    handbrake: true,
                    ..Input::default()
                },
                &track,
                1,
            );
        }
        assert!(b.slip > a.slip);
        let mut reverse = Car::spawn(&track, 0, 0);
        for _ in 0..30 {
            reverse.step(
                Input {
                    brake: 1.,
                    ..Input::default()
                },
                &track,
                1,
            );
        }
        assert!(reverse.vel.dot(reverse.forward()) < -1.);
    }

    #[test]
    fn idle_grid_player_is_not_pushed_by_rivals() {
        for track in 0..3 {
            for physics in 0..3 {
                for difficulty in 0..3 {
                    let mut race = Race::with_difficulty(track, 0, physics, false, difficulty);
                    let start = race.cars[0].pos;
                    for _ in 0..60 * 12 {
                        race.tick(Input::default());
                    }
                    assert!(
                        (race.cars[0].pos - start).len() < 0.1,
                        "idle player pushed: track={track} physics={physics} difficulty={difficulty}"
                    );
                }
            }
        }
    }

    #[test]
    fn blocked_pass_brakes_and_open_pass_commits() {
        let track = Track::new(0);
        let (p, tangent) = track.checkpoint(0);
        let mut follower = Car::spawn(&track, 0, 0);
        follower.pos = p;
        follower.vel = tangent * 10.;
        let mut leader = follower.clone();
        leader.pos = p + tangent * 5.;
        leader.vel = tangent * 3.;
        let mut left = leader.clone();
        left.pos = p + tangent * 2. + tangent.normal() * -3.9;
        let mut right = left.clone();
        right.pos = p + tangent * 2. + tangent.normal() * 3.9;
        let mut cars = vec![follower, leader, left, right];
        let mut state = AiState::default();
        let control = traffic_input(0, &cars, &track, 0, 1, &mut state);
        assert_eq!(control.throttle, 0.);
        assert!(control.brake > 0.);
        assert_eq!(state.target_lane, 0.);
        assert_eq!(state.commitment, 0.);

        cars.truncate(2);
        cars[1].pos = p + tangent * 12.;
        traffic_input(0, &cars, &track, 0, 1, &mut state);
        assert!(state.target_lane.abs() > 3.);
        assert!(state.commitment > 1.);
        let chosen_lane = state.target_lane;
        // A small change in the leader's line must not flip passing side every tick.
        cars[1].pos = cars[1].pos + tangent.normal() * chosen_lane.signum() * 0.4;
        traffic_input(0, &cars, &track, 0, 1, &mut state);
        assert_eq!(state.target_lane, chosen_lane);
    }

    #[test]
    fn side_by_side_rivals_keep_separate_corridors() {
        for physics in 0..3 {
            let track = Track::new(0);
            let mut cars = vec![Car::spawn(&track, 0, 0), Car::spawn(&track, 0, 1)];
            for car in &mut cars {
                car.vel = car.forward() * 10.;
            }
            let mut states: Vec<_> = cars
                .iter()
                .map(|car| {
                    let road = track.nearest(car.pos);
                    let lane = (car.pos - road.pos).dot(road.tangent.normal());
                    AiState {
                        lane,
                        target_lane: lane,
                        ..AiState::default()
                    }
                })
                .collect();
            for _ in 0..120 {
                let snapshot = cars.clone();
                for i in 0..cars.len() {
                    let control = traffic_input(i, &snapshot, &track, physics, 1, &mut states[i]);
                    cars[i].step(control, &track, physics);
                }
                assert!((cars[0].pos - cars[1].pos).len() >= 2.8);
            }
        }
    }

    #[test]
    fn difficulty_does_not_change_player_handling() {
        let mut easy = Race::with_difficulty(0, 2, 1, true, 0);
        let mut hard = Race::with_difficulty(0, 2, 1, true, 2);
        assert_eq!(Race::new(0, 0, 0, false).difficulty, 1);
        assert_eq!(Race::with_difficulty(0, 0, 0, false, 99).difficulty, 2);
        for _ in 0..600 {
            let input = ai_input(&easy.cars[0], &easy.track, easy.physics);
            easy.tick(input);
            hard.tick(input);
        }
        assert!((easy.cars[0].pos - hard.cars[0].pos).len() < 0.0001);
        assert!((easy.cars[0].vel - hard.cars[0].vel).len() < 0.0001);
    }

    #[test]
    fn feedback_reports_drive_landing_surface_and_reset() {
        let track = Track::new(0);
        let mut car = Car::spawn(&track, 0, 0);
        car.step(
            Input {
                throttle: 1.,
                ..Input::default()
            },
            &track,
            0,
        );
        assert_eq!(car.throttle, 1.);
        assert!(car.load > 0. && car.load <= 1.);
        assert_eq!(car.surface, 0);
        car.height += 4.;
        car.vertical = -8.;
        car.grounded = false;
        let mut peak = 0_f32;
        for _ in 0..60 {
            car.step(Input::default(), &track, 0);
            peak = peak.max(car.landing);
        }
        assert!(peak > 0.5 && peak <= 1.);
        assert_eq!(car.landing, 0.);
        car.reset(&track);
        assert_eq!(
            car.throttle + car.load + car.landing + car.slip + car.impact,
            0.
        );
        assert_eq!(Car::spawn(&Track::new(1), 0, 0).surface, 1);
        let volcano = Track::new(2);
        car.pos = volcano.points[24];
        car.road_index = 24;
        car.height = volcano.heights[24];
        car.step(Input::default(), &volcano, 0);
        assert_eq!(car.surface, 1);
    }

    #[test]
    fn all_grid_configurations_finish_at_each_difficulty() {
        for track in 0..3 {
            for car in 0..4 {
                for physics in 0..3 {
                    for difficulty in 0..3 {
                        let mut race =
                            Race::with_difficulty(track, car, physics, false, difficulty);
                        for _ in 0..60 * if track == 2 { 480 } else { 360 } {
                            let input = ai_input(&race.cars[0], &race.track, physics);
                            race.tick(input);
                            if race.cars.iter().all(|car| car.finished) {
                                break;
                            }
                        }
                        assert!(
                            race.cars.iter().all(|car| car.finished),
                            "grid unfinished: track={track} car={car} physics={physics} difficulty={difficulty}: {:?}",race.cars.iter().map(|c|(c.spec,c.laps,c.next_cp,c.pos,c.vel.len(),c.road_index)).collect::<Vec<_>>()
                        );
                    }
                }
            }
        }
    }
}
