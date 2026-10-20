use std::f32::consts::{PI, TAU};

pub const DT: f32 = 1.0 / 60.0;
pub const SAMPLES: usize = 128;
pub const TRACK_NAMES: [&str; 3] = ["Beach Loop", "Rocky Coast", "Lagoon Run"];
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
        }
    }
    pub fn nearest(&self, pos: V2) -> Road {
        let mut best = Road {
            pos: self.points[0],
            tangent: V2::new(1., 0.),
            distance: f32::MAX,
            index: 0,
            height: 0.,
            grip: 1.,
        };
        for i in 0..SAMPLES {
            let j = (i + 1) % SAMPLES;
            let delta = self.points[j] - self.points[i];
            let t = ((pos - self.points[i]).dot(delta) / delta.dot(delta)).clamp(0., 1.);
            let p = self.points[i] + delta * t;
            let distance = (pos - p).len();
            if distance < best.distance {
                best = Road {
                    pos: p,
                    tangent: delta.unit(),
                    distance,
                    index: i,
                    height: self.heights[i] * (1. - t) + self.heights[j] * t,
                    grip: if self.id == 1 {
                        0.92
                    } else if self.id == 2 && (20..38).contains(&i) {
                        1.25
                    } else {
                        1.
                    },
                };
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
    pub spec: usize,
    pub pos: V2,
    pub previous: V2,
    pub vel: V2,
    pub heading: f32,
    pub previous_heading: f32,
    pub yaw: f32,
    pub height: f32,
    pub previous_height: f32,
    pub vertical: f32,
    pub pitch: f32,
    pub roll: f32,
    pub steer: f32,
    pub slip: f32,
    pub impact: f32,
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
            spec,
            pos,
            previous: pos,
            vel: V2::default(),
            heading: t.y.atan2(t.x),
            previous_heading: t.y.atan2(t.x),
            yaw: 0.,
            height: h,
            previous_height: h,
            vertical: 0.,
            pitch: 0.,
            roll: 0.,
            steer: 0.,
            slip: 0.,
            impact: 0.,
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
        self.height = track.ground(track.nearest(self.pos));
        self.previous_height = self.height;
        self.grounded = true;
        self.stuck = 0.;
        self.pitch = 0.;
        self.roll = 0.;
    }
    pub fn forward(&self) -> V2 {
        V2::new(self.heading.cos(), self.heading.sin())
    }
    pub fn step(&mut self, input: Input, track: &Track, physics: usize) {
        self.previous = self.pos;
        self.previous_height = self.height;
        self.previous_heading = self.heading;
        self.impact = (self.impact - DT * 4.).max(0.);
        let spec = CARS[self.spec];
        let tune = preset(physics);
        let road = track.nearest(self.pos);
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
            let ahead = track.ground(track.nearest(self.pos + self.vel * DT));
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
        let next = track.nearest(self.pos);
        let ground = track.ground(next);
        self.height += self.vertical * DT;
        if self.height <= ground + 0.02 {
            if !self.grounded {
                self.impact = (-self.vertical / 13.).clamp(0., 1.);
            }
            self.height = ground;
            self.grounded = true;
        } else {
            self.grounded = false;
        }
        // Railings constrain rocky sections and bridges, shoulders remain drivable elsewhere.
        let rails = track.id == 1 || (track.id == 2 && (20..38).contains(&next.index));
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
            if (crossing - p).dot(t.normal()).abs() < track.width * 0.5 + 2. {
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

pub fn ai_input(car: &Car, track: &Track, physics: usize) -> Input {
    let road = track.nearest(car.pos);
    let speed = car.vel.len();
    let look = 3 + (speed * 0.08) as usize;
    let target = track.points[(road.index + look) % SAMPLES];
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
    let desired =
        (CARS[car.spec].top * (0.65 - physics as f32 * 0.045) * grip / (1. + bend * 2.8)).max(7.);
    Input {
        throttle: if speed < desired { 0.9 } else { 0.12 },
        brake: if speed > desired + 2. { 0.5 } else { 0. },
        steer: (error * 2.1 - car.yaw * 0.18).clamp(-1., 1.),
        handbrake: false,
    }
}

pub struct Race {
    pub track: Track,
    pub cars: Vec<Car>,
    pub physics: usize,
    pub trial: bool,
    pub countdown: f32,
    pub elapsed: f32,
    pub accumulator: f32,
}
impl Race {
    pub fn new(track: usize, car: usize, physics: usize, trial: bool) -> Self {
        let track = Track::new(track);
        let cars = (0..if trial { 1 } else { 6 })
            .map(|i| Car::spawn(&track, if i == 0 { car } else { (i - 1) % 4 }, i))
            .collect();
        Self {
            track,
            cars,
            physics,
            trial,
            countdown: 3.,
            elapsed: 0.,
            accumulator: 0.,
        }
    }
    pub fn tick(&mut self, input: Input) {
        if self.countdown > 0. {
            self.countdown -= DT;
            return;
        }
        self.elapsed += DT;
        for i in 0..self.cars.len() {
            let control = if i == 0 && !self.cars[0].finished {
                input
            } else {
                ai_input(&self.cars[i], &self.track, self.physics)
            };
            self.cars[i].step(control, &self.track, self.physics);
            if i > 0 {
                if self.cars[i].vel.len() < 2. {
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
    (100 + track * 12 + car * 3 + physics) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
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
                    for _ in 0..60 * 240 {
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
                for _ in 0..60 * 300 {
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
}
