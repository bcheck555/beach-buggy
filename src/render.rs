use beach_buggy::sim::*;
use macroquad::prelude::*;

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}
fn p3(p: V2, h: f32) -> Vec3 {
    vec3(p.x, h, p.y)
}
fn shade(c: Color, v: f32) -> Color {
    Color::new(c.r * v, c.g * v, c.b * v, c.a)
}

// Face colors supply cheap directional lighting without textures or a custom shader.
fn box_mesh(center: Vec3, size: Vec3, rotation: Quat, color: Color) -> Mesh {
    let corners = [
        vec3(-1., -1., -1.),
        vec3(1., -1., -1.),
        vec3(1., 1., -1.),
        vec3(-1., 1., -1.),
        vec3(-1., -1., 1.),
        vec3(1., -1., 1.),
        vec3(1., 1., 1.),
        vec3(-1., 1., 1.),
    ];
    let faces = [
        ([0, 1, 2, 3], 0.72),
        ([4, 7, 6, 5], 0.85),
        ([0, 4, 5, 1], 0.55),
        ([3, 2, 6, 7], 1.0),
        ([0, 3, 7, 4], 0.67),
        ([1, 5, 6, 2], 0.9),
    ];
    let mut mesh = Mesh {
        vertices: vec![],
        indices: vec![],
        texture: None,
    };
    for (face, light) in faces {
        let base = mesh.vertices.len() as u16;
        for i in face {
            mesh.vertices.push(Vertex::new2(
                center + rotation * (corners[i] * size * 0.5),
                Vec2::ZERO,
                shade(color, light),
            ));
        }
        mesh.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    mesh
}
fn quad(mesh: &mut Mesh, points: [Vec3; 4], color: Color) {
    let base = mesh.vertices.len() as u16;
    for p in points {
        mesh.vertices.push(Vertex::new2(p, Vec2::ZERO, color));
    }
    mesh.indices
        .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}
fn append(to: &mut Mesh, from: Mesh) {
    let offset = to.vertices.len() as u16;
    to.vertices.extend(from.vertices);
    to.indices.extend(from.indices.iter().map(|i| i + offset));
}
fn empty() -> Mesh {
    Mesh {
        vertices: vec![],
        indices: vec![],
        texture: None,
    }
}

pub struct Scenery {
    pub meshes: Vec<Mesh>,
}
impl Scenery {
    pub fn new(track: &Track) -> Self {
        let mut land = empty();
        let mut road = empty();
        let mut props = empty();
        // Individual tiles create a faceted island silhouette and gentle sand variation.
        for x in -12i32..12 {
            for z in -12i32..12 {
                let p = V2::new(x as f32 * 10. + 5., z as f32 * 10. + 5.);
                let r = track.nearest(p);
                let island = (p.x / 110.).powi(2) + (p.y / 106.).powi(2) < 1.
                    || r.distance < track.width * 0.5 + 6.;
                let lagoon = track.id == 2
                    && ((r.distance > track.width * 0.5 + 3.
                        && (p.x / 43.).powi(2) + (p.y / 48.).powi(2) < 1.)
                        || ((20..38).contains(&r.index) && r.distance < track.width));
                if island && !lagoon {
                    let h = -0.12;
                    let c = if r.distance > 27. && p.len() < 65. {
                        rgb(139, 176 + (x * z).unsigned_abs() as u8 % 16, 112)
                    } else {
                        rgb(230 + (x + z).unsigned_abs() as u8 % 12, 207, 156)
                    };
                    let a = vec3(p.x - 5., h, p.y - 5.);
                    let b = vec3(p.x + 5., h, p.y - 5.);
                    let c1 = vec3(p.x + 5., h, p.y + 5.);
                    let d = vec3(p.x - 5., h, p.y + 5.);
                    quad(&mut land, [a, b, c1, d], c);
                }
            }
        }
        for i in 0..SAMPLES {
            let j = (i + 1) % SAMPLES;
            let tangent = |k: usize| {
                (track.points[(k + 1) % SAMPLES] - track.points[(k + SAMPLES - 1) % SAMPLES]).unit()
            };
            let normal = tangent(i).normal();
            let normal_j = tangent(j).normal();
            let a = track.points[i];
            let b = track.points[j];
            let h = track.heights[i];
            let hj = track.heights[j];
            let w = track.width * 0.5;
            let bridge = track.id == 2 && (20..38).contains(&i);
            let c = if bridge {
                rgb(155 + (i % 2) as u8 * 12, 117, 81)
            } else if track.id == 1 {
                rgb(167 + (i % 3) as u8 * 3, 156, 141)
            } else {
                rgb(211 + (i % 3) as u8 * 3, 179, 123)
            };
            quad(
                &mut road,
                [
                    p3(a - normal * w, h),
                    p3(a + normal * w, h),
                    p3(b + normal_j * w, hj),
                    p3(b - normal_j * w, hj),
                ],
                c,
            );
            // Sloping shoulders seal elevation gaps between road and island.
            for side in [-1., 1.] {
                if !bridge {
                    quad(
                        &mut road,
                        [
                            p3(a + normal * (w * side), h - 0.02),
                            p3(a + normal * ((w + 5.) * side), 0.),
                            p3(b + normal_j * ((w + 5.) * side), 0.),
                            p3(b + normal_j * (w * side), hj - 0.02),
                        ],
                        shade(c, 0.85),
                    );
                }
                let edge = 0.28;
                quad(
                    &mut road,
                    [
                        p3(a + normal * ((w - edge) * side), h + 0.04),
                        p3(a + normal * (w * side), h + 0.04),
                        p3(b + normal_j * (w * side), hj + 0.04),
                        p3(b + normal_j * ((w - edge) * side), hj + 0.04),
                    ],
                    if i % 4 < 2 {
                        rgb(251, 244, 218)
                    } else {
                        rgb(226, 109, 73)
                    },
                );
                if (track.id == 1 || bridge) && i % 2 == 0 {
                    let p = p3(a + normal * ((w + 1.) * side), h + 0.7);
                    append(
                        &mut props,
                        box_mesh(p, vec3(0.35, 1.4, 0.35), Quat::IDENTITY, rgb(116, 98, 78)),
                    );
                    let mid = (p3(a + normal * ((w + 1.) * side), h + 1.1)
                        + p3(b + normal_j * ((w + 1.) * side), hj + 1.1))
                        * 0.5;
                    append(
                        &mut props,
                        box_mesh(
                            mid,
                            vec3((b - a).len() + 0.4, 0.18, 0.18),
                            Quat::from_rotation_y(-tangent(i).y.atan2(tangent(i).x)),
                            rgb(239, 216, 166),
                        ),
                    );
                }
            }
            if i == 0 {
                for row in 0..2 {
                    for cell in 0..12 {
                        let f = (cell as f32 / 12. - 0.5) * track.width;
                        let start = a + normal * f + tangent(i) * (row as f32 * 0.8);
                        quad(
                            &mut road,
                            [
                                p3(start, h + 0.06),
                                p3(start + normal * (track.width / 12.), h + 0.06),
                                p3(
                                    start + normal * (track.width / 12.) + tangent(i) * 0.8,
                                    h + 0.06,
                                ),
                                p3(start + tangent(i) * 0.8, h + 0.06),
                            ],
                            if (cell + row) % 2 == 0 {
                                rgb(35, 53, 57)
                            } else {
                                rgb(255, 250, 229)
                            },
                        );
                    }
                }
            }
        }
        // Deterministic original palms, rocks, parasols, and beach huts.
        let mut seed = 19271u32;
        let mut random = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216.
        };
        for i in 0..145 {
            let x = random() * 218. - 109.;
            let z = random() * 202. - 101.;
            let p = V2::new(x, z);
            let r = track.nearest(p);
            if r.distance < track.width * 0.5 + 6.
                || (x / 105.).powi(2) + (z / 101.).powi(2) > 0.94
                || (track.id == 2 && (x / 48.).powi(2) + (z / 53.).powi(2) < 1.)
            {
                continue;
            }
            if i % 4 < 2 {
                append(
                    &mut props,
                    box_mesh(
                        vec3(x, 2.7, z),
                        vec3(0.5, 5.4, 0.5),
                        Quat::from_rotation_z(0.13),
                        rgb(137, 104, 70),
                    ),
                );
                for k in 0..5 {
                    let angle = k as f32 * std::f32::consts::TAU / 5.;
                    let rotation = Quat::from_rotation_y(angle) * Quat::from_rotation_z(0.2);
                    append(
                        &mut props,
                        box_mesh(
                            vec3(x + angle.cos() * 1.2, 5.1, z - angle.sin() * 1.2),
                            vec3(3.9, 0.18, 0.9),
                            rotation,
                            rgb(57, 130 + (i % 25) as u8, 98),
                        ),
                    );
                }
            } else if i % 4 == 2 {
                append(
                    &mut props,
                    box_mesh(
                        vec3(x, 0.8, z),
                        vec3(2.5, 1.6, 2.),
                        Quat::from_rotation_y(i as f32),
                        rgb(154, 157, 145),
                    ),
                );
            } else {
                append(
                    &mut props,
                    box_mesh(
                        vec3(x, 1., z),
                        vec3(0.15, 2., 0.15),
                        Quat::IDENTITY,
                        rgb(157, 119, 75),
                    ),
                );
                append(
                    &mut props,
                    box_mesh(
                        vec3(x, 2.1, z),
                        vec3(3., 0.2, 3.),
                        Quat::from_rotation_y(i as f32),
                        if i % 3 == 0 {
                            rgb(251, 155, 99)
                        } else {
                            rgb(245, 238, 211)
                        },
                    ),
                );
            }
        }
        // A finish gantry frames the grid.
        let (p, t) = track.checkpoint(0);
        let n = t.normal();
        let h = track.heights[0];
        for side in [-1., 1.] {
            append(
                &mut props,
                box_mesh(
                    p3(p + n * (track.width * 0.5 + 1.) * side, h + 3.),
                    vec3(0.45, 6., 0.45),
                    Quat::IDENTITY,
                    rgb(41, 72, 73),
                ),
            );
        }
        append(
            &mut props,
            box_mesh(
                p3(p, h + 5.8),
                vec3(track.width + 3., 1.25, 0.6),
                Quat::from_rotation_y(-n.y.atan2(n.x)),
                rgb(245, 176, 73),
            ),
        );
        // Split props so geometry stays below Macroquad's default batch capacity.
        let mut meshes = vec![land, road];
        for chunk in props.vertices.chunks(3600) {
            let offset = (meshes.len() - 2) * 3600;
            let end = offset + chunk.len();
            let indices: Vec<u16> = props
                .indices
                .chunks(3)
                .filter(|tri| {
                    tri.iter()
                        .all(|&i| (i as usize) >= offset && (i as usize) < end)
                })
                .flat_map(|tri| tri.iter().map(|&i| (i as usize - offset) as u16))
                .collect();
            meshes.push(Mesh {
                vertices: chunk.to_vec(),
                indices,
                texture: None,
            });
        }
        Self { meshes }
    }
    pub fn draw(&self, time: f32) {
        clear_background(rgb(121, 205, 205));
        draw_plane(
            vec3(0., -0.55, 0.),
            vec2(2000., 2000.),
            None,
            rgb(70, 173, 186),
        );
        for i in -16..16 {
            let z = i as f32 * 13. + time.sin() * 0.8;
            draw_line_3d(
                vec3(-230., -0.48, z),
                vec3(230., -0.48, z + 3.),
                Color::new(0.49, 0.82, 0.81, 0.25),
            );
        }
        for mesh in &self.meshes {
            draw_mesh(mesh);
        }
    }
}

pub fn draw_car(car: &Car, alpha: f32, player: bool, time: f32) {
    let spec = CARS[car.spec];
    let pos = car.previous.lerp(car.pos, alpha);
    let h = car.previous_height * (1. - alpha) + car.height * alpha;
    let heading = car.previous_heading + angle_diff(car.heading, car.previous_heading) * alpha;
    let yaw = Quat::from_rotation_y(-heading);
    let rot = yaw * Quat::from_rotation_z(car.pitch) * Quat::from_rotation_x(car.roll);
    let base = p3(pos, h + 0.75);
    let tint = rgb(spec.color[0], spec.color[1], spec.color[2]);
    draw_cube(
        p3(pos, car.height - car.height.min(0.) + 0.03),
        vec3(3.4, 0.025, 2.2),
        None,
        Color::new(0.12, 0.20, 0.18, 0.27),
    );
    let mut mesh = empty();
    let mut part = |offset: Vec3, size: Vec3, rotation: Quat, color: Color| {
        append(
            &mut mesh,
            box_mesh(base + rot * offset, size, rot * rotation, color),
        )
    };
    part(
        vec3(0., 0., 0.),
        vec3(3.5, 0.65, 1.65),
        Quat::IDENTITY,
        tint,
    );
    part(
        vec3(1.05, 0.35, 0.),
        vec3(1.1, 0.3, 1.55),
        Quat::from_rotation_z(-0.12),
        tint,
    );
    part(
        vec3(-0.35, 0.55, 0.),
        vec3(1.4, 0.65, 1.3),
        Quat::IDENTITY,
        rgb(39, 63, 67),
    );
    part(
        vec3(-0.35, 0.99, 0.),
        vec3(1.65, 0.12, 1.5),
        Quat::IDENTITY,
        tint,
    );
    for x in [-1.12, 1.12] {
        for z in [-1., 1.] {
            let bump = if car.grounded {
                (time * car.vel.len() * 1.6 + x + z).sin() * 0.035
            } else {
                -0.18
            };
            let wheel_rot = Quat::from_rotation_y(if x > 0. { car.steer * -0.42 } else { 0. });
            part(
                vec3(x, -0.3 + bump, z),
                vec3(0.95, 0.9, 0.52),
                wheel_rot,
                rgb(36, 48, 50),
            );
            part(
                vec3(x, -0.28 + bump, z * 1.27),
                vec3(0.47, 0.45, 0.08),
                wheel_rot,
                rgb(217, 216, 189),
            );
            part(
                vec3(x, -0.28 + bump, z * 1.32),
                vec3(0.42, 0.07, 0.05),
                wheel_rot * Quat::from_rotation_z(time * car.vel.len() * 2.),
                rgb(67, 83, 79),
            );
            part(
                vec3(x, -0.05, z * 0.75),
                vec3(0.22, 0.5, 0.18),
                Quat::IDENTITY,
                rgb(227, 129, 61),
            );
        }
    }
    if car.spec == 3 {
        part(
            vec3(-1.1, 0.5, 0.),
            vec3(1.05, 0.55, 1.65),
            Quat::IDENTITY,
            tint,
        );
        part(
            vec3(-1.1, 0.8, 0.),
            vec3(0.8, 0.06, 1.3),
            Quat::IDENTITY,
            rgb(53, 73, 73),
        );
    } else if car.spec == 2 {
        part(
            vec3(-1.5, 0.85, 0.),
            vec3(0.35, 0.15, 2.1),
            Quat::IDENTITY,
            tint,
        );
    }
    for z in [-0.5, 0.5] {
        part(
            vec3(1.79, 0.1, z),
            vec3(0.08, 0.21, 0.32),
            Quat::IDENTITY,
            rgb(255, 241, 175),
        );
    }
    draw_mesh(&mesh);
    if player {
        let marker = base + vec3(0., 3.3 + (time * 3.).sin() * 0.15, 0.);
        draw_line_3d(marker, marker + vec3(0., 1., 0.), rgb(255, 225, 129));
        draw_cube(
            marker + vec3(0., 1., 0.),
            vec3(0.5, 0.5, 0.5),
            None,
            rgb(255, 225, 129),
        );
    }
}

pub struct Particle {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub life: f32,
    pub total: f32,
    pub skid: bool,
}
pub struct Effects {
    pub particles: Vec<Particle>,
    pub timer: f32,
}
impl Effects {
    pub fn new() -> Self {
        Self {
            particles: vec![],
            timer: 0.,
        }
    }
    pub fn update(&mut self, race: &Race, dt: f32) {
        self.timer += dt;
        for p in &mut self.particles {
            p.life -= dt;
            p.pos += p.velocity * dt;
        }
        self.particles.retain(|p| p.life > 0.);
        if self.timer > 0.06 {
            self.timer = 0.;
            for car in &race.cars {
                if car.grounded && car.vel.len() > 5. {
                    let road = race.track.nearest(car.pos);
                    for side in [-1., 1.] {
                        let p = car.pos - car.forward() * 1.2 + car.forward().normal() * side;
                        if car.slip > 0.15 {
                            self.particles.push(Particle {
                                pos: p3(p, car.height + 0.07),
                                velocity: Vec3::ZERO,
                                life: 5.,
                                total: 5.,
                                skid: true,
                            });
                        }
                        if road.grip < 1.2 {
                            self.particles.push(Particle {
                                pos: p3(p, car.height + 0.3),
                                velocity: vec3(-car.vel.x * 0.07, 1., -car.vel.y * 0.07),
                                life: 0.8,
                                total: 0.8,
                                skid: false,
                            });
                        }
                    }
                }
            }
        }
        if self.particles.len() > 2400 {
            self.particles.drain(0..200);
        }
    }
    pub fn draw(&self) {
        for p in &self.particles {
            if p.skid {
                draw_cube(
                    p.pos,
                    vec3(0.35, 0.02, 0.35),
                    None,
                    Color::new(0.28, 0.23, 0.18, p.life / p.total * 0.4),
                );
            } else {
                let size = 0.25 + (1. - p.life / p.total) * 0.7;
                draw_cube(
                    p.pos,
                    Vec3::splat(size),
                    None,
                    Color::new(0.85, 0.75, 0.55, p.life / p.total * 0.26),
                );
            }
        }
    }
}

pub fn minimap(race: &Race) {
    let size = 150.;
    let x = screen_width() - size - 36.;
    let y = screen_height() - size - 46.;
    draw_rectangle(
        x - 14.,
        y - 14.,
        size + 28.,
        size + 28.,
        Color::new(0.05, 0.14, 0.16, 0.8),
    );
    let map = |p: V2| vec2(x + size * 0.5 + p.x * 0.66, y + size * 0.5 + p.y * 0.66);
    for i in 0..SAMPLES {
        let a = map(race.track.points[i]);
        let b = map(race.track.points[(i + 1) % SAMPLES]);
        draw_line(a.x, a.y, b.x, b.y, 4., rgb(169, 188, 169));
    }
    for (i, car) in race.cars.iter().enumerate().rev() {
        let p = map(car.pos);
        draw_circle(
            p.x,
            p.y,
            if i == 0 { 4.5 } else { 2.8 },
            if i == 0 {
                rgb(255, 194, 92)
            } else {
                rgb(250, 244, 224)
            },
        );
    }
    draw_text("ISLAND MAP", x, y - 23., 14., rgb(223, 233, 211));
}
