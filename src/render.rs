use beach_buggy::sim::*;
use macroquad::prelude::*;

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}
fn p3(p: V2, h: f32) -> Vec3 {
    vec3(p.x, h, p.y)
}

fn road_sample(track: &Track, sample: f32) -> (V2, f32, V2) {
    let index = sample.floor() as usize % SAMPLES;
    let t = sample.fract();
    let at = |offset: usize| track.points[(index + SAMPLES + offset - 1) % SAMPLES];
    let (a, b, c, d) = (at(0), at(1), at(2), at(3));
    let height = track.heights[index] * (1. - t) + track.heights[(index + 1) % SAMPLES] * t;
    if track.id == 0 {
        let point = b.lerp(c, t);
        let direction = (c - a).unit().lerp((d - b).unit(), t).unit();
        return (point, height, direction);
    }
    // Subdivide the existing route, retaining its original sample positions and elevations.
    let quadratic = a * 2. - b * 5. + c * 4. - d;
    let cubic = (b - c) * 3. + d - a;
    let point = (b * 2. + (c - a) * t + quadratic * (t * t) + cubic * (t * t * t)) * 0.5;
    let direction = ((c - a) + quadratic * (2. * t) + cubic * (3. * t * t)).unit();
    (point, height, direction)
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

fn triangle(mesh: &mut Mesh, points: [Vec3; 3], color: Color) {
    let base = mesh.vertices.len() as u16;
    mesh.vertices
        .extend(points.map(|p| Vertex::new2(p, Vec2::ZERO, color)));
    mesh.indices.extend([base, base + 1, base + 2]);
}

// Split by whole triangles: vertex windows can silently discard faces crossing a boundary.
fn batches(mesh: Mesh) -> Vec<Mesh> {
    let mut result = Vec::new();
    let mut batch = empty();
    for face in mesh.indices.chunks_exact(3) {
        if batch.vertices.len() + 3 > 3000 {
            result.push(batch);
            batch = empty();
        }
        let base = batch.vertices.len() as u16;
        batch
            .vertices
            .extend(face.iter().map(|&i| mesh.vertices[i as usize]));
        batch.indices.extend([base, base + 1, base + 2]);
    }
    if !batch.indices.is_empty() {
        result.push(batch);
    }
    result
}

fn rounded_rock(mesh: &mut Mesh, center: Vec3, size: Vec3, color: Color) {
    for row in 0..6 {
        for column in 0..12 {
            let point = |r: usize, c: usize| {
                let latitude = r as f32 * std::f32::consts::PI / 6.;
                let longitude = c as f32 * std::f32::consts::TAU / 12.;
                center
                    + vec3(
                        latitude.sin() * longitude.cos(),
                        latitude.cos(),
                        latitude.sin() * longitude.sin(),
                    ) * size
            };
            quad(
                mesh,
                [
                    point(row, column),
                    point(row + 1, column),
                    point(row + 1, column + 1),
                    point(row, column + 1),
                ],
                shade(
                    color,
                    0.78 + row as f32 * 0.025 + (column as f32 * 0.35).cos() * 0.08,
                ),
            );
        }
    }
}

// Clip against a world-space plane, retaining its positive half-space. Intersections
// share the exact plane, so adjacent volcano faces form a continuous road cutting.
fn clip_polygon(points: &[Vec3], normal: Vec3, offset: f32) -> Vec<Vec3> {
    let mut clipped = Vec::new();
    if points.is_empty() {
        return clipped;
    }
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        let da = normal.dot(a) + offset;
        let db = normal.dot(b) + offset;
        if da >= 0. {
            clipped.push(a);
        }
        if (da >= 0.) != (db >= 0.) {
            clipped.push(a.lerp(b, da / (da - db)));
        }
    }
    clipped
}

fn polygon(mesh: &mut Mesh, points: &[Vec3], color: Color) {
    for i in 1..points.len().saturating_sub(1) {
        // Keep six-index primitive groups for the camera cutaway and batch splitter.
        quad(
            mesh,
            [points[0], points[i], points[i + 1], points[i + 1]],
            color,
        );
    }
}

fn volcano_face(mesh: &mut Mesh, points: &[Vec3], color: Color, width: f32) {
    let upper = vec3(0.8, 0., 1.).normalize();
    let lower = vec3(-0.8, 0., 1.).normalize();
    let high = clip_polygon(points, Vec3::Y, -11.);
    let upper_pieces = [
        clip_polygon(points, -Vec3::Y, 11.),
        clip_polygon(&high, upper, -width),
        clip_polygon(&high, -upper, -width),
    ];
    for piece in upper_pieces {
        let low = clip_polygon(&piece, -Vec3::Y, 9.3);
        for part in [
            clip_polygon(&piece, Vec3::Y, -9.3),
            clip_polygon(&low, lower, -9.9),
            clip_polygon(&low, -lower, -9.9),
        ] {
            polygon(mesh, &part, color);
        }
    }
}

fn sightline_cutaway(points: &[Vec3], player: Vec3, sight: Vec3) -> Vec<Vec<Vec3>> {
    let mut visible = vec![clip_polygon(points, -Vec3::Y, player.y + 0.7)];
    let high = clip_polygon(points, Vec3::Y, -player.y - 0.7);
    visible.push(clip_polygon(&high, -sight, player.dot(sight)));
    let mut inside = clip_polygon(&high, sight, -player.dot(sight));
    let right = sight.cross(Vec3::Y).normalize_or_zero();
    let right = if right.length_squared() < 0.1 {
        Vec3::X
    } else {
        right
    };
    let up = right.cross(sight).normalize();
    // Subtract a convex twelve-sided visibility shaft. Plane intersections produce
    // smooth shared boundaries rather than jagged deleted triangles around the car.
    for side in 0..12 {
        let angle = side as f32 * std::f32::consts::TAU / 12.;
        let radial = right * angle.cos() + up * angle.sin();
        visible.push(clip_polygon(&inside, radial, -8. - radial.dot(player)));
        inside = clip_polygon(&inside, -radial, 8. + radial.dot(player));
        if inside.is_empty() {
            break;
        }
    }
    visible
}

fn draw_polygon(mesh: &mut Mesh, points: &[Vec3], template: Vertex) {
    for i in 1..points.len().saturating_sub(1) {
        if mesh.vertices.len() + 3 > 3000 {
            draw_mesh(mesh);
            mesh.vertices.clear();
            mesh.indices.clear();
        }
        let base = mesh.vertices.len() as u16;
        for point in [points[0], points[i], points[i + 1]] {
            let mut vertex = template;
            vertex.position = point;
            mesh.vertices.push(vertex);
        }
        mesh.indices.extend([base, base + 1, base + 2]);
    }
}

fn volcano(track: &Track, walls: &mut Mesh, cover: &mut Mesh, tunnel_roof: &mut Vec<Mesh>) {
    // Rounded concentric slopes surround an inset glowing crater. Road corridors remain open.
    let rings = [
        (18., 25.),
        (21., 28.),
        (25., 24.),
        (29., 15.),
        (35., 5.),
        (39., -0.1),
    ];
    for pair in rings.windows(2) {
        for sector in 0..72 {
            let point = |ring: (f32, f32), k: usize| {
                let a = k as f32 * std::f32::consts::TAU / 72.;
                vec3(a.cos() * ring.0, ring.1, a.sin() * ring.0)
            };
            let points = [
                point(pair[0], sector),
                point(pair[1], sector),
                point(pair[1], sector + 1),
                point(pair[0], sector + 1),
            ];
            volcano_face(
                cover,
                &points,
                shade(
                    rgb(112, 119, 111),
                    0.84 + (sector as f32 * std::f32::consts::TAU / 72. - 0.5).cos() * 0.16,
                ),
                track.width * 0.5 + 3.,
            );
        }
    }
    for sector in 0..48 {
        let a = sector as f32 * std::f32::consts::TAU / 48.;
        let b = (sector + 1) as f32 * std::f32::consts::TAU / 48.;
        for radial in 0..4 {
            let inner = radial as f32 * 4.5;
            let outer = inner + 4.5;
            let points = [
                vec3(a.cos() * inner, 24.8, a.sin() * inner),
                vec3(a.cos() * outer, 24.8, a.sin() * outer),
                vec3(b.cos() * outer, 24.8, b.sin() * outer),
                vec3(b.cos() * inner, 24.8, b.sin() * inner),
            ];
            volcano_face(cover, &points, rgb(245, 117, 58), track.width * 0.5 + 3.);
        }
    }
    for i in 0..SAMPLES {
        if !track.section(i).tunnel {
            continue;
        }
        let j = (i + 1) % SAMPLES;
        let normal = |k: usize| {
            (track.points[(k + 1) % SAMPLES] - track.points[(k + SAMPLES - 1) % SAMPLES])
                .unit()
                .normal()
        };
        let (a, b, n, m) = (track.points[i], track.points[j], normal(i), normal(j));
        let w = track.width * 0.5 + 0.9;
        let mut roof = empty();
        // Elliptical arch has full vehicle clearance throughout the drivable width.
        for arc in 0..16 {
            let vertex = |p: V2, n: V2, h: f32, k: usize, outer: bool| {
                let angle = k as f32 * std::f32::consts::PI / 16.;
                p3(
                    p + n * (angle.cos() * (w + if outer { 1.5 } else { 0. })),
                    h + 2.5 + angle.sin() * (6. + if outer { 1.5 } else { 0. }),
                )
            };
            for outer in [false, true] {
                quad(
                    &mut roof,
                    [
                        vertex(a, n, track.heights[i], arc, outer),
                        vertex(b, m, track.heights[j], arc, outer),
                        vertex(b, m, track.heights[j], arc + 1, outer),
                        vertex(a, n, track.heights[i], arc + 1, outer),
                    ],
                    if outer {
                        rgb(125, 130, 119)
                    } else {
                        rgb(66, 89, 91)
                    },
                );
            }
            if !track.section((i + SAMPLES - 1) % SAMPLES).tunnel || !track.section(j).tunnel {
                let (p, n, h) = if !track.section(j).tunnel {
                    (b, m, track.heights[j])
                } else {
                    (a, n, track.heights[i])
                };
                quad(
                    &mut roof,
                    [
                        vertex(p, n, h, arc, false),
                        vertex(p, n, h, arc, true),
                        vertex(p, n, h, arc + 1, true),
                        vertex(p, n, h, arc + 1, false),
                    ],
                    rgb(177, 166, 141),
                );
            }
        }
        tunnel_roof.push(roof);
        for side in [-1., 1.] {
            quad(
                walls,
                [
                    p3(a + n * w * side, track.heights[i]),
                    p3(b + m * w * side, track.heights[j]),
                    p3(b + m * w * side, track.heights[j] + 2.5),
                    p3(a + n * w * side, track.heights[i] + 2.5),
                ],
                rgb(73, 93, 91),
            );
            if i % 3 == 0 {
                append(
                    walls,
                    box_mesh(
                        p3(a + n * (w - 0.12) * side, track.heights[i] + 1.8),
                        vec3(0.18, 0.35, 0.6),
                        Quat::IDENTITY,
                        rgb(255, 220, 140),
                    ),
                );
                ring(
                    walls,
                    p3(a + n * (w - 1.) * side, track.heights[i] + 0.055),
                    0.25,
                    0.16,
                    rgb(242, 198, 101),
                );
            }
        }
    }
}

fn skill_markers(track: &Track, props: &mut Mesh, road: &mut Mesh) {
    let tangent = |i: usize| {
        (track.points[(i + 1) % SAMPLES] - track.points[(i + SAMPLES - 1) % SAMPLES]).unit()
    };
    for apex in [V2::new(-84., 42.), V2::new(-84., -42.)] {
        let bend = track.nearest(apex).index;
        let before = tangent((bend + SAMPLES - 1) % SAMPLES);
        let after = tangent((bend + 1) % SAMPLES);
        let direction = angle_diff(after.y.atan2(after.x), before.y.atan2(before.x)).signum();
        for (marker, offset) in [8, 5, 2].into_iter().enumerate() {
            let i = (bend + SAMPLES - offset) % SAMPLES;
            let (t, n) = (tangent(i), tangent(i).normal());
            let p = track.points[i];
            let h = track.heights[i];
            // Three bright slowing bars lead into each hairpin; fewer bars near the apex.
            for stripe in 0..3 - marker {
                let center = p - t * (stripe as f32 * 1.1);
                quad(
                    road,
                    [
                        p3(center - n * (track.width * 0.42), h + 0.065),
                        p3(center + n * (track.width * 0.42), h + 0.065),
                        p3(center + n * (track.width * 0.42) + t * 0.3, h + 0.065),
                        p3(center - n * (track.width * 0.42) + t * 0.3, h + 0.065),
                    ],
                    rgb(255, 224, 151),
                );
            }
        }
        for offset in [0, 2, 4] {
            let i = (bend + offset) % SAMPLES;
            let t = tangent(i);
            let n = t.normal();
            let center = p3(
                track.points[i] - n * direction * (track.width * 0.5 + 2.5),
                track.heights[i] + 2.5,
            );
            let across = p3(t, 0.);
            append(
                props,
                box_mesh(
                    center - Vec3::Y * 1.1,
                    vec3(0.22, 3.2, 0.22),
                    Quat::IDENTITY,
                    rgb(99, 111, 99),
                ),
            );
            quad(
                props,
                [
                    center - across * 1.8 - Vec3::Y,
                    center + across * 1.8 - Vec3::Y,
                    center + across * 1.8 + Vec3::Y,
                    center - across * 1.8 + Vec3::Y,
                ],
                rgb(41, 72, 73),
            );
            for shift in [-0.8, 0.8] {
                let c = center + across * shift - p3(n * direction, 0.) * 0.04;
                triangle(
                    props,
                    [
                        c - across * 0.5 - Vec3::Y * 0.7,
                        c + across * 0.5,
                        c - across * 0.5 + Vec3::Y * 0.7,
                    ],
                    rgb(255, 224, 151),
                );
            }
        }
    }
}

fn ring(mesh: &mut Mesh, center: Vec3, radius: f32, width: f32, color: Color) {
    for i in 0..32 {
        let a = i as f32 * std::f32::consts::TAU / 32.;
        let b = (i + 1) as f32 * std::f32::consts::TAU / 32.;
        let point = |angle: f32, r: f32| center + vec3(angle.cos() * r, 0., angle.sin() * r);
        quad(
            mesh,
            [
                point(a, radius),
                point(a, radius + width),
                point(b, radius + width),
                point(b, radius),
            ],
            color,
        );
    }
}

fn palm(mesh: &mut Mesh, p: V2, scale: f32, angle: f32) {
    append(
        mesh,
        box_mesh(
            p3(p, 2.8 * scale),
            vec3(0.5, 5.6, 0.5) * scale,
            Quat::from_rotation_y(angle) * Quat::from_rotation_z(0.12),
            rgb(140, 105, 73),
        ),
    );
    for k in 0..6 {
        let a = angle + k as f32 * std::f32::consts::TAU / 6.;
        let along = vec3(a.cos(), 0., a.sin());
        let across = vec3(-a.sin(), 0., a.cos());
        let crown = p3(p, 5.5 * scale);
        let mid = crown + along * (2.1 * scale) + Vec3::Y * (0.45 * scale);
        let tip = crown + along * (4.4 * scale) - Vec3::Y * (0.85 * scale);
        triangle(
            mesh,
            [crown, mid + across * (0.8 * scale), tip],
            rgb(48, 126, 95),
        );
        triangle(
            mesh,
            [crown, tip, mid - across * (0.8 * scale)],
            rgb(82, 157, 108),
        );
    }
}

fn coast_point(i: usize) -> V2 {
    let angle = i as f32 * std::f32::consts::TAU / 96.;
    let ripple = 1. + (angle * 3. + 0.6).sin() * 0.025 + (angle * 7.).sin() * 0.018;
    V2::new(angle.cos() * 111., angle.sin() * 104.) * ripple
}

fn coastal_island(land: &mut Mesh, shallows: &mut Mesh, foam: &mut Mesh, scale: f32) {
    for i in 0..96 {
        let a = coast_point(i) * scale;
        let b = coast_point((i + 1) % 96) * scale;
        triangle(
            land,
            [
                vec3(0., -0.13, 0.),
                p3(a * 0.55, -0.13),
                p3(b * 0.55, -0.13),
            ],
            rgb(164, 190, 129),
        );
        for (inner, outer, color) in [
            (0.55, 0.68, rgb(205, 204, 145)),
            (0.68, 0.94, rgb(238, 218, 171)),
            (0.94, 1., rgb(241, 225, 184)),
        ] {
            quad(
                land,
                [
                    p3(a * inner, -0.12),
                    p3(a * outer, -0.12),
                    p3(b * outer, -0.12),
                    p3(b * inner, -0.12),
                ],
                shade(color, 0.985 + (i % 3) as f32 * 0.008),
            );
        }
        for (inner, outer, height, color) in [
            (1., 1.055, -0.28, rgb(137, 205, 187)),
            (1.055, 1.12, -0.35, rgb(92, 192, 184)),
            (1.12, 1.20, -0.41, rgb(62, 176, 179)),
        ] {
            quad(
                shallows,
                [
                    p3(a * inner, height),
                    p3(a * outer, height),
                    p3(b * outer, height),
                    p3(b * inner, height),
                ],
                color,
            );
        }
        if i % 7 != 0 {
            quad(
                foam,
                [
                    p3(a * 1.008, -0.20),
                    p3(a * 1.015, -0.20),
                    p3(b * 1.015, -0.20),
                    p3(b * 1.008, -0.20),
                ],
                rgb(214, 241, 221),
            );
        }
    }
}

fn showcase_landmarks(track: &Track, props: &mut Mesh, road: &mut Mesh) {
    let point = |i: usize, side: f32, distance: f32| {
        let tangent =
            (track.points[(i + 1) % SAMPLES] - track.points[(i + SAMPLES - 1) % SAMPLES]).unit();
        (
            track.points[i] + tangent.normal() * (side * distance),
            tangent,
        )
    };
    // The surf club sits inside the grid: a recognizable landmark on every lap.
    let (club, tangent) = point(3, 1., 26.);
    let yaw = Quat::from_rotation_y(-tangent.y.atan2(tangent.x));
    let mut part = |offset: Vec3, size: Vec3, color: Color| {
        append(
            props,
            box_mesh(p3(club, 0.) + yaw * offset, size, yaw, color),
        );
    };
    part(vec3(0., 0.35, 0.), vec3(11., 0.7, 7.5), rgb(172, 134, 91));
    part(vec3(0., 2.3, 0.), vec3(8.8, 3.9, 5.7), rgb(47, 123, 118));
    part(vec3(0., 4.45, 0.), vec3(10.4, 0.5, 7.5), rgb(239, 148, 103));
    part(vec3(0., 4.85, 0.), vec3(8.4, 0.3, 5.5), rgb(250, 180, 125));
    for x in [-2.7, 2.7] {
        part(vec3(x, 2.6, -2.9), vec3(2., 1.6, 0.08), rgb(238, 231, 193));
        part(vec3(x, 2.6, -2.96), vec3(1.5, 1.1, 0.06), rgb(37, 76, 79));
    }
    part(
        vec3(0., 1.8, -2.92),
        vec3(1.6, 2.8, 0.08),
        rgb(236, 184, 109),
    );
    for offset in [V2::new(-10., 1.), V2::new(9., 4.), V2::new(13., 7.)] {
        palm(props, club + offset, 1.1, offset.x * 0.2);
    }
    // Landmark rock gardens use clustered sizes, rather than a uniform prop scatter.
    for index in [28, 72, 108] {
        let (p, _) = point(index, 1., 22.);
        for (j, offset) in [V2::new(0., 0.), V2::new(3., 1.5), V2::new(-2., 2.)]
            .into_iter()
            .enumerate()
        {
            let scale = 1. - j as f32 * 0.22;
            append(
                props,
                box_mesh(
                    p3(p + offset, scale),
                    vec3(4.8, 2.7, 3.2) * scale,
                    Quat::from_rotation_y(index as f32) * Quat::from_rotation_z(0.15),
                    rgb(157, 168, 148),
                ),
            );
        }
        palm(props, p + V2::new(4., -4.), 0.95, index as f32);
    }
    // Cream chevrons on dark boards make the three main bends readable at speed.
    for index in [22, 60, 92, 116] {
        let (p, tangent) = point(index, -1., track.width * 0.5 + 2.4);
        let cross = p3(tangent.normal(), 0.);
        let forward = p3(tangent, 0.);
        let center = p3(p, track.heights[index] + 2.2);
        append(
            props,
            box_mesh(
                center - Vec3::Y * 0.7,
                vec3(0.22, 3., 0.22),
                Quat::IDENTITY,
                rgb(101, 100, 76),
            ),
        );
        quad(
            props,
            [
                center - cross * 1.9 - Vec3::Y,
                center + cross * 1.9 - Vec3::Y,
                center + cross * 1.9 + Vec3::Y,
                center - cross * 1.9 + Vec3::Y,
            ],
            rgb(31, 84, 83),
        );
        for offset in [-0.95, 0.75] {
            let c = center + cross * offset - forward * 0.02;
            triangle(
                props,
                [
                    c - cross * 0.55 - Vec3::Y * 0.72,
                    c + cross * 0.45,
                    c - cross * 0.55 + Vec3::Y * 0.72,
                ],
                rgb(255, 224, 151),
            );
        }
    }
    // Flags and paint refer to the existing physical ramp; they add no collision.
    for index in [39, 47, 53] {
        for side in [-1., 1.] {
            let (p, tangent) = point(index, side, track.width * 0.5 + 1.8);
            let top = p3(p, track.heights[index] + 4.8);
            append(
                props,
                box_mesh(
                    top - Vec3::Y * 2.4,
                    vec3(0.16, 4.8, 0.16),
                    Quat::IDENTITY,
                    rgb(247, 231, 192),
                ),
            );
            let direction = p3(tangent, 0.);
            triangle(
                props,
                [
                    top,
                    top + direction * 2.4 - Vec3::Y * 0.7,
                    top - Vec3::Y * 1.4,
                ],
                if index == 53 {
                    rgb(53, 150, 133)
                } else {
                    rgb(244, 150, 81)
                },
            );
        }
    }
    for index in [39, 41, 53] {
        let (p, tangent) = point(index, 1., 0.);
        let n = tangent.normal();
        let h = track.heights[index] + 0.075;
        for lane in [-1., 0., 1.] {
            let center = p + n * (lane * 4.);
            triangle(
                road,
                [
                    p3(center - tangent * 0.8 - n, h),
                    p3(center + tangent * 0.8, h),
                    p3(center - tangent * 0.8 + n, h),
                ],
                rgb(252, 234, 181),
            );
        }
    }
}

pub struct Scenery {
    pub meshes: Vec<Mesh>,
    shallows: Mesh,
    foam: Mesh,
    showcase: bool,
    cover: Vec<Mesh>,
    crossing: Vec<Mesh>,
    reveal: Mesh,
    tunnel_roof: Vec<Mesh>,
    coast_scale: f32,
}
impl Scenery {
    pub fn new(track: &Track) -> Self {
        let mut land = empty();
        let mut road = empty();
        let mut props = empty();
        let mut shallows = empty();
        let mut foam = empty();
        let mut cover = empty();
        let mut crossing = empty();
        let mut tunnel_roof = Vec::new();
        let coast_scale = if track.id == 2 { 1.35 } else { 1. };
        coastal_island(&mut land, &mut shallows, &mut foam, coast_scale);
        let subdivisions = if track.id == 0 { 1 } else { 6 };
        for sample in 0..SAMPLES * subdivisions {
            let i = sample / subdivisions;
            let (a, h, t) = road_sample(track, sample as f32 / subdivisions as f32);
            let (b, hj, tj) = road_sample(track, (sample + 1) as f32 / subdivisions as f32);
            let tangent = |k: usize| {
                (track.points[(k + 1) % SAMPLES] - track.points[(k + SAMPLES - 1) % SAMPLES]).unit()
            };
            let normal = t.normal();
            let normal_j = tj.normal();
            let w = track.width * 0.5;
            let section = track.section(i);
            let bridge = section.elevated;
            let c = if bridge {
                rgb(154, 150, 138)
            } else if section.tunnel {
                rgb(119, 139, 133)
            } else if track.id == 1 {
                rgb(167 + (i % 3) as u8 * 3, 156, 141)
            } else {
                rgb(211 + (i % 3) as u8 * 3, 179, 123)
            };
            let deck = if bridge { &mut crossing } else { &mut road };
            quad(
                deck,
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
                if !bridge && !section.tunnel {
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
                let edge = if track.id == 0 { 0.52 } else { 0.28 };
                quad(
                    if bridge { &mut crossing } else { &mut road },
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
                if section.barrier && !section.tunnel {
                    let p = p3(a + normal * ((w + 1.) * side), h + 0.7);
                    if sample % subdivisions == 0 {
                        append(
                            if bridge { &mut crossing } else { &mut props },
                            box_mesh(p, vec3(0.35, 1.4, 0.35), Quat::IDENTITY, rgb(116, 98, 78)),
                        );
                    }
                    let mid = (p3(a + normal * ((w + 1.) * side), h + 1.1)
                        + p3(b + normal_j * ((w + 1.) * side), hj + 1.1))
                        * 0.5;
                    append(
                        if bridge { &mut crossing } else { &mut props },
                        box_mesh(
                            mid,
                            vec3((b - a).len() + 0.4, 0.18, 0.18),
                            Quat::from_rotation_y(-(b - a).y.atan2((b - a).x)),
                            rgb(239, 216, 166),
                        ),
                    );
                }
            }
            if bridge {
                quad(
                    &mut crossing,
                    [
                        p3(a - normal * w, h - 0.65),
                        p3(b - normal_j * w, hj - 0.65),
                        p3(b + normal_j * w, hj - 0.65),
                        p3(a + normal * w, h - 0.65),
                    ],
                    rgb(93, 105, 101),
                );
                for side in [-1., 1.] {
                    quad(
                        &mut crossing,
                        [
                            p3(a + normal * w * side, h),
                            p3(b + normal_j * w * side, hj),
                            p3(b + normal_j * w * side, hj - 0.65),
                            p3(a + normal * w * side, h - 0.65),
                        ],
                        rgb(129, 135, 124),
                    );
                    if i % 5 == 0 && sample % subdivisions == 0 {
                        rounded_rock(
                            &mut props,
                            p3(a + normal * (w + 1.7) * side, h * 0.5 - 0.4),
                            vec3(1.1, h * 0.5, 1.1),
                            rgb(129, 135, 124),
                        );
                    }
                }
            }
            if sample == 0 {
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
            if i % 4 < 2 && track.id == 0 {
                palm(&mut props, p, 0.8 + random() * 0.3, i as f32);
            } else if i % 4 < 2 {
                palm(&mut props, p, 0.9 + random() * 0.3, i as f32);
            } else if i % 4 == 2 {
                if track.id == 0 {
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
                    rounded_rock(
                        &mut props,
                        vec3(x, 0.8, z),
                        vec3(2.5, 1.6, 2.),
                        rgb(154, 157, 145),
                    );
                }
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
        if track.id == 0 {
            showcase_landmarks(track, &mut props, &mut road);
        } else if track.id == 2 {
            volcano(track, &mut props, &mut cover, &mut tunnel_roof);
            skill_markers(track, &mut props, &mut road);
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
        let mut meshes = batches(land);
        meshes.extend(batches(road));
        meshes.extend(batches(props));
        Self {
            meshes,
            shallows,
            foam,
            showcase: true,
            cover: batches(cover),
            crossing: batches(crossing),
            reveal: empty(),
            tunnel_roof,
            coast_scale,
        }
    }
    pub fn draw(&mut self, time: f32, high_quality: bool, reveal: Option<(Vec3, Vec3)>) {
        clear_background(rgb(121, 205, 205));
        draw_plane(
            vec3(0., -0.55, 0.),
            vec2(2000., 2000.),
            None,
            rgb(70, 173, 186),
        );
        for i in -12..12 {
            let z = i as f32 * 13. + time.sin() * 0.8;
            draw_line_3d(
                vec3(-230., -0.48, z),
                vec3(230., -0.48, z + 3.),
                Color::new(0.49, 0.82, 0.81, 0.25),
            );
        }
        if self.showcase {
            draw_mesh(&self.shallows);
            draw_mesh(&self.foam);
            if high_quality {
                for i in (0..96).step_by(4) {
                    let wave = 1.035 + (time * 0.6 + i as f32 * 0.12).sin() * 0.006;
                    draw_line_3d(
                        p3(coast_point(i) * wave * self.coast_scale, -0.23),
                        p3(coast_point(i + 1) * wave * self.coast_scale, -0.23),
                        Color::new(0.85, 0.97, 0.89, 0.38),
                    );
                }
            }
        }
        for mesh in &self.meshes {
            draw_mesh(mesh);
        }
        for mesh in self.cover.iter().chain(self.crossing.iter()) {
            if let Some((player, eye)) = reveal {
                // Open only the sightline to the driver, preserving the distant mountain,
                // crater and bridge. Works for orbiting overview and follow cameras alike.
                let sight = (eye - player).normalize();
                self.reveal.vertices.clear();
                self.reveal.indices.clear();
                for face in mesh.indices.chunks_exact(6) {
                    let points: [Vec3; 6] =
                        std::array::from_fn(|i| mesh.vertices[face[i] as usize].position);
                    let center = points.iter().copied().sum::<Vec3>() / 6.;
                    let radius = points.iter().map(|p| p.distance(center)).fold(0., f32::max);
                    let offset = center - player;
                    let along = offset.dot(sight);
                    if along > -radius
                        && (offset - sight * along).length() < 8. + radius
                        && points.iter().any(|p| p.y > player.y + 0.7)
                    {
                        let polygon = [points[0], points[1], points[2], points[5]];
                        for visible in sightline_cutaway(&polygon, player, sight) {
                            draw_polygon(
                                &mut self.reveal,
                                &visible,
                                mesh.vertices[face[0] as usize],
                            );
                        }
                        continue;
                    }
                    if self.reveal.vertices.len() + face.len() > 3000 {
                        draw_mesh(&self.reveal);
                        self.reveal.vertices.clear();
                        self.reveal.indices.clear();
                    }
                    let base = self.reveal.vertices.len() as u16;
                    self.reveal
                        .vertices
                        .extend(face.iter().map(|&index| mesh.vertices[index as usize]));
                    self.reveal
                        .indices
                        .extend((0..face.len()).map(|i| base + i as u16));
                }
                draw_mesh(&self.reveal);
            } else {
                draw_mesh(mesh);
            }
        }
        for roof in &self.tunnel_roof {
            let hidden = reveal.is_some_and(|(player, eye)| {
                let center = roof.vertices.iter().map(|v| v.position).sum::<Vec3>()
                    / roof.vertices.len().max(1) as f32;
                let radius = roof
                    .vertices
                    .iter()
                    .map(|v| v.position.distance(center))
                    .fold(0., f32::max);
                let sight = (eye - player).normalize();
                let offset = center - player;
                let along = offset.dot(sight);
                along > -radius && (offset - sight * along).length() < 8. + radius
            });
            if !hidden {
                draw_mesh(roof);
            }
        }
    }
}

fn car_body(spec_index: usize) -> Mesh {
    let spec = CARS[spec_index];
    let tint = rgb(spec.color[0], spec.color[1], spec.color[2]);
    let mut mesh = empty();
    let mut part = |offset: Vec3, size: Vec3, rotation: Quat, color: Color| {
        append(&mut mesh, box_mesh(offset, size, rotation, color))
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
    // Cream racing stripe and visible roll bars keep the toy silhouette legible.
    part(
        vec3(1.06, 0.515, 0.),
        vec3(1.1, 0.025, 0.32),
        Quat::from_rotation_z(-0.12),
        rgb(255, 237, 185),
    );
    for z in [-0.65, 0.65] {
        part(
            vec3(-1.04, 0.6, z),
            vec3(0.13, 0.9, 0.13),
            Quat::IDENTITY,
            rgb(240, 225, 186),
        );
        part(
            vec3(0.35, 0.6, z),
            vec3(0.13, 0.9, 0.13),
            Quat::from_rotation_z(0.17),
            rgb(240, 225, 186),
        );
    }
    for x in [-1.12, 1.12] {
        for z in [-1., 1.] {
            part(
                vec3(x, -0.05, z * 0.75),
                vec3(0.22, 0.5, 0.18),
                Quat::IDENTITY,
                rgb(227, 129, 61),
            );
        }
    }
    if spec_index == 3 {
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
    } else if spec_index == 2 {
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
    mesh
}

fn wheel_mesh() -> Mesh {
    let mut mesh = empty();
    for i in 0..10 {
        let a = i as f32 * std::f32::consts::TAU / 10.;
        let b = (i + 1) as f32 * std::f32::consts::TAU / 10.;
        let p =
            |angle: f32, z: f32, radius: f32| vec3(angle.cos() * radius, angle.sin() * radius, z);
        quad(
            &mut mesh,
            [
                p(a, -0.27, 0.49),
                p(a, 0.27, 0.49),
                p(b, 0.27, 0.49),
                p(b, -0.27, 0.49),
            ],
            if i % 2 == 0 {
                rgb(39, 54, 54)
            } else {
                rgb(49, 64, 61)
            },
        );
        for side in [-1., 1.] {
            triangle(
                &mut mesh,
                [
                    vec3(0., 0., side * 0.275),
                    p(a, side * 0.275, 0.49),
                    p(b, side * 0.275, 0.49),
                ],
                rgb(31, 47, 48),
            );
            triangle(
                &mut mesh,
                [
                    vec3(0., 0., side * 0.285),
                    p(a, side * 0.285, 0.24),
                    p(b, side * 0.285, 0.24),
                ],
                rgb(235, 222, 184),
            );
        }
    }
    mesh
}

fn transform_into(to: &mut Mesh, from: &Mesh, position: Vec3, rotation: Quat, scale: Vec3) {
    let offset = to.vertices.len() as u16;
    to.vertices.extend(from.vertices.iter().map(|vertex| {
        let mut v = *vertex;
        v.position = position + rotation * (v.position * scale);
        v
    }));
    to.indices.extend(from.indices.iter().map(|i| i + offset));
}

// The same triangles as the visible road and shoulders, restricted to the car's neighborhood.
fn ground_patch(
    track: &Track,
    pos: V2,
    height: f32,
    route_index: usize,
    triangles: &mut Vec<[Vec3; 3]>,
) -> Option<f32> {
    triangles.clear();
    let road = track.road_for(pos, height, route_index);
    let center = road.index;
    if road.distance + 4.6 < track.width * 0.5
        && (0..10).all(|offset| {
            (track.heights[(center + SAMPLES + offset - 4) % SAMPLES] - road.height).abs() < 0.0001
        })
    {
        return Some(road.height);
    }
    let mut add = |p: [Vec3; 4]| {
        triangles.extend([[p[0], p[1], p[2]], [p[0], p[2], p[3]]]);
    };
    let subdivisions = if track.id == 0 { 1 } else { 6 };
    for offset in 0..9 * subdivisions {
        let sample = (center + SAMPLES - 4) as f32 + offset as f32 / subdivisions as f32;
        let i = sample.floor() as usize % SAMPLES;
        let (a, h, t) = road_sample(track, sample);
        let (b, hj, u) = road_sample(track, sample + 1. / subdivisions as f32);
        let (n, m, w) = (t.normal(), u.normal(), track.width * 0.5);
        add([
            p3(a - n * w, h),
            p3(a + n * w, h),
            p3(b + m * w, hj),
            p3(b - m * w, hj),
        ]);
        if !track.section(i).elevated && !track.section(i).tunnel {
            for side in [-1., 1.] {
                add([
                    p3(a + n * (w * side), h - 0.02),
                    p3(a + n * ((w + 5.) * side), 0.),
                    p3(b + m * ((w + 5.) * side), 0.),
                    p3(b + m * (w * side), hj - 0.02),
                ]);
            }
        }
    }
    if track.id != 0 {
        triangles.retain(|triangle| {
            let min_x = triangle.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
            let max_x = triangle
                .iter()
                .map(|p| p.x)
                .fold(f32::NEG_INFINITY, f32::max);
            let min_z = triangle.iter().map(|p| p.z).fold(f32::INFINITY, f32::min);
            let max_z = triangle
                .iter()
                .map(|p| p.z)
                .fold(f32::NEG_INFINITY, f32::max);
            max_x >= pos.x - 5. && min_x <= pos.x + 5. && max_z >= pos.y - 5. && min_z <= pos.y + 5.
        });
    }
    None
}

fn patch_height(p: Vec3, triangles: &[[Vec3; 3]]) -> f32 {
    for [a, b, c] in triangles {
        let ab = *b - *a;
        let ac = *c - *a;
        let ap = p - *a;
        let determinant = ab.x * ac.z - ab.z * ac.x;
        if determinant.abs() < 0.0001 {
            continue;
        }
        let u = (ap.x * ac.z - ap.z * ac.x) / determinant;
        let v = (ab.x * ap.z - ab.z * ap.x) / determinant;
        if u >= -0.0001 && v >= -0.0001 && u + v <= 1.0001 {
            return a.y + u * ab.y + v * ac.y;
        }
    }
    -0.12
}

fn projected_triangle(
    to: &mut Mesh,
    vertices: [Vertex; 3],
    patch: &[[Vec3; 3]],
    clearance: f32,
    depth: u8,
) {
    let project = |mut v: Vertex| {
        v.position.y += patch_height(v.position, patch) + clearance;
        v
    };
    let projected = vertices.map(project);
    let midpoints: [Vertex; 3] = std::array::from_fn(|i| {
        let mut v = vertices[i];
        v.position = (v.position + vertices[(i + 1) % 3].position) * 0.5;
        v
    });
    // Subdivide only where a decal crosses a crest or road/shoulder boundary.
    let bends = depth < 3
        && midpoints.iter().enumerate().any(|(i, v)| {
            (project(*v).position.y
                - (projected[i].position.y + projected[(i + 1) % 3].position.y) * 0.5)
                .abs()
                > 0.025
        });
    if bends {
        let [a, b, c] = vertices;
        let [ab, bc, ca] = midpoints;
        for child in [[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]] {
            projected_triangle(to, child, patch, clearance, depth + 1);
        }
    } else {
        if to.vertices.len() > 2800 || to.indices.len() > 4200 {
            draw_mesh(to);
            to.vertices.clear();
            to.indices.clear();
        }
        let base = to.vertices.len() as u16;
        to.vertices.extend(projected);
        to.indices.extend([base, base + 1, base + 2]);
    }
}

fn project_into(
    to: &mut Mesh,
    from: &Mesh,
    position: V2,
    yaw: Quat,
    patch: &[[Vec3; 3]],
    clearance: f32,
) {
    for indices in from.indices.chunks_exact(3) {
        let vertices = std::array::from_fn(|i| {
            let mut vertex = from.vertices[indices[i] as usize];
            vertex.position = p3(position, 0.) + yaw * vertex.position;
            vertex
        });
        projected_triangle(to, vertices, patch, clearance, 0);
    }
}

pub struct CarRenderer {
    bodies: [Mesh; 4],
    wheel: Mesh,
    spoke: Mesh,
    halo: Mesh,
    shadow: Mesh,
    scratch: Mesh,
    ground: Vec<[Vec3; 3]>,
}

impl CarRenderer {
    pub fn new() -> Self {
        let mut halo = empty();
        ring(&mut halo, Vec3::ZERO, 2.45, 0.48, rgb(32, 79, 77));
        ring(&mut halo, Vec3::Y * 0.005, 2.55, 0.24, rgb(255, 226, 139));
        triangle(
            &mut halo,
            [
                vec3(4.5, 0.01, 0.),
                vec3(3.05, 0.01, -0.95),
                vec3(3.05, 0.01, 0.95),
            ],
            rgb(28, 78, 76),
        );
        triangle(
            &mut halo,
            [
                vec3(4.15, 0.02, 0.),
                vec3(3.2, 0.02, -0.55),
                vec3(3.2, 0.02, 0.55),
            ],
            rgb(255, 229, 155),
        );
        let mut shadow = empty();
        for i in 0..24 {
            let angle = |n: usize| n as f32 * std::f32::consts::TAU / 24.;
            let p = |a: f32| vec3(a.cos() * 2.1, 0., a.sin() * 1.4);
            triangle(
                &mut shadow,
                [Vec3::ZERO, p(angle(i)), p(angle(i + 1))],
                Color::new(0.10, 0.20, 0.18, 0.24),
            );
        }
        let mut scratch = empty();
        scratch.vertices.reserve(2200);
        scratch.indices.reserve(4000);
        Self {
            bodies: std::array::from_fn(car_body),
            wheel: wheel_mesh(),
            spoke: box_mesh(
                Vec3::ZERO,
                vec3(0.40, 0.065, 0.59),
                Quat::IDENTITY,
                rgb(60, 83, 78),
            ),
            halo,
            shadow,
            scratch,
            ground: Vec::with_capacity(54),
        }
    }

    pub fn draw(
        &mut self,
        car: &Car,
        track: &Track,
        alpha: f32,
        player: bool,
        time: f32,
        reduced_motion: bool,
    ) {
        let pos = car.previous.lerp(car.pos, alpha);
        let h = car.previous_height * (1. - alpha) + car.height * alpha;
        let heading = car.previous_heading + angle_diff(car.heading, car.previous_heading) * alpha;
        let yaw = Quat::from_rotation_y(-heading);
        let landing = if reduced_motion {
            0.
        } else {
            car.landing.clamp(0., 1.)
        };
        let rot = yaw
            * Quat::from_rotation_z(car.pitch + landing * 0.035)
            * Quat::from_rotation_x(car.roll);
        let base = p3(pos, h + 0.80 - landing * 0.16);
        self.scratch.vertices.clear();
        self.scratch.indices.clear();
        let flat_ground = ground_patch(track, pos, h, car.road_index, &mut self.ground);
        for (mesh, clearance) in [(&self.shadow, 0.025), (&self.halo, 0.085)]
            .into_iter()
            .take(if player { 2 } else { 1 })
        {
            if let Some(height) = flat_ground {
                transform_into(
                    &mut self.scratch,
                    mesh,
                    p3(pos, height + clearance),
                    yaw,
                    Vec3::ONE,
                );
            } else {
                project_into(&mut self.scratch, mesh, pos, yaw, &self.ground, clearance);
            }
        }
        // Keep room for the cached body and four wheels after decal subdivision.
        if self.scratch.vertices.len() > 1500 || self.scratch.indices.len() > 2500 {
            draw_mesh(&self.scratch);
            self.scratch.vertices.clear();
            self.scratch.indices.clear();
        }
        transform_into(
            &mut self.scratch,
            &self.bodies[car.spec],
            base,
            rot,
            vec3(1., 1. - landing * 0.075, 1.),
        );
        for x in [-1.12, 1.12] {
            for z in [-1., 1.] {
                let bump = if !car.grounded {
                    -0.15
                } else if reduced_motion {
                    0.
                } else {
                    (time * car.vel.len() * 1.6 + x + z).sin() * 0.025 + landing * 0.09
                };
                let wheel_rotation =
                    rot * Quat::from_rotation_y(if x > 0. { car.steer * -0.42 } else { 0. });
                let wheel_position = base + rot * vec3(x, -0.30 + bump, z);
                transform_into(
                    &mut self.scratch,
                    &self.wheel,
                    wheel_position,
                    wheel_rotation,
                    Vec3::ONE,
                );
                transform_into(
                    &mut self.scratch,
                    &self.spoke,
                    wheel_position,
                    wheel_rotation * Quat::from_rotation_z(time * car.vel.len() * 2.),
                    Vec3::ONE,
                );
            }
        }
        draw_mesh(&self.scratch);
    }
}

pub struct Particle {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub life: f32,
    pub total: f32,
    pub skid: bool,
    pub direction: Vec3,
    pub color: Color,
    pub size: f32,
}
pub struct Effects {
    pub particles: std::collections::VecDeque<Particle>,
    pub timer: f32,
    previous_tires: Vec<Option<[Vec3; 2]>>,
    previous_landing: Vec<f32>,
    emission_index: u32,
    scratch: Mesh,
}
impl Effects {
    pub fn new() -> Self {
        Self {
            particles: std::collections::VecDeque::with_capacity(384),
            timer: 0.,
            previous_tires: vec![],
            previous_landing: vec![],
            emission_index: 0,
            scratch: empty(),
        }
    }
    fn emit(&mut self, mut particle: Particle, limit: usize) {
        while self.particles.len() >= limit {
            self.particles.pop_front();
        }
        if !particle.skid {
            self.emission_index = self.emission_index.wrapping_add(1);
            let phase = self.emission_index as f32 * 2.399_963;
            let (sine, cosine) = phase.sin_cos();
            particle.size *= 0.8 + sine * 0.24;
            particle.pos += vec3(cosine * 0.36, sine.abs() * 0.16, sine * 0.36);
            particle.velocity += vec3(sine * 0.6, cosine * 0.18, cosine * 0.6);
            particle.direction = vec3(cosine, sine, 0.);
        }
        self.particles.push_back(particle);
    }
    pub fn update(&mut self, race: &Race, dt: f32, high_quality: bool) {
        let limit = if high_quality { 384 } else { 160 };
        while self.particles.len() > limit {
            self.particles.pop_front();
        }
        self.previous_tires.resize(race.cars.len(), None);
        self.previous_landing.resize(race.cars.len(), 0.);
        self.timer += dt;
        for p in &mut self.particles {
            p.life -= dt;
            p.pos += p.velocity * dt;
        }
        self.particles.retain(|p| p.life > 0.);
        if self.timer > if high_quality { 0.055 } else { 0.09 } {
            self.timer = 0.;
            for (index, car) in race.cars.iter().enumerate() {
                let dust = match car.surface {
                    1 => rgb(176, 179, 166),
                    2 => rgb(213, 197, 157),
                    _ => rgb(234, 214, 170),
                };
                let landing = car.landing > 0.15 && self.previous_landing[index] <= 0.15;
                self.previous_landing[index] = car.landing;
                if landing {
                    for j in 0..if high_quality { 9 } else { 5 } {
                        let a =
                            j as f32 * std::f32::consts::TAU / if high_quality { 9. } else { 5. };
                        self.emit(
                            Particle {
                                pos: p3(car.pos, car.ground_height + 0.25),
                                velocity: vec3(a.cos() * 3.3, 0.7, a.sin() * 3.3),
                                life: 0.6,
                                total: 0.6,
                                skid: false,
                                direction: Vec3::ZERO,
                                color: dust,
                                size: 0.6 + car.landing * 0.4,
                            },
                            limit,
                        );
                    }
                }
                if car.grounded && car.vel.len() > 5. {
                    let mut tires = [Vec3::ZERO; 2];
                    for (wheel, side) in [-1., 1.].into_iter().enumerate() {
                        let p = car.pos - car.forward() * 1.2 + car.forward().normal() * side;
                        let tire_road = race.track.road_for(p, car.height, car.road_index);
                        tires[wheel] = p3(p, race.track.ground(tire_road) + 0.065);
                        if let Some(previous) =
                            self.previous_tires[index].filter(|_| car.slip > 0.15)
                        {
                            let delta = tires[wheel] - previous[wheel];
                            // Reset/teleport never produces a stripe across the island.
                            if delta.length() < 8. && delta.length() > 0.04 {
                                self.emit(
                                    Particle {
                                        pos: (tires[wheel] + previous[wheel]) * 0.5,
                                        velocity: Vec3::ZERO,
                                        life: 2.6,
                                        total: 2.6,
                                        skid: true,
                                        direction: delta,
                                        color: if car.surface == 1 {
                                            rgb(68, 78, 76)
                                        } else {
                                            rgb(146, 114, 71)
                                        },
                                        size: 0.24,
                                    },
                                    limit,
                                );
                            }
                        }
                        if car.surface == 0 || car.slip > 0.25 {
                            self.emit(
                                Particle {
                                    pos: p3(p, car.ground_height + 0.3),
                                    velocity: vec3(-car.vel.x * 0.07, 1., -car.vel.y * 0.07),
                                    life: 0.65,
                                    total: 0.65,
                                    skid: false,
                                    direction: Vec3::ZERO,
                                    color: dust,
                                    size: if car.surface == 0 { 0.7 } else { 0.45 },
                                },
                                limit,
                            );
                        }
                    }
                    self.previous_tires[index] = Some(tires);
                } else {
                    self.previous_tires[index] = None;
                }
            }
        }
    }
    pub fn draw(&mut self) {
        self.scratch.vertices.clear();
        self.scratch.indices.clear();
        for p in &self.particles {
            if self.scratch.vertices.len() > 3000 {
                draw_mesh(&self.scratch);
                self.scratch.vertices.clear();
                self.scratch.indices.clear();
            }
            let fade = (p.life / p.total).clamp(0., 1.);
            let mut color = p.color;
            if p.skid {
                color.a = fade * 0.45;
                let side = vec3(-p.direction.z, 0., p.direction.x).normalize_or_zero() * p.size;
                let a = p.pos - p.direction * 0.5;
                let b = p.pos + p.direction * 0.5;
                quad(
                    &mut self.scratch,
                    [a - side, a + side, b + side, b - side],
                    color,
                );
            } else {
                // A feathered octagonal puff reads as sand spray, not collectible gems.
                color.a = fade * 0.19;
                let size = p.size * (1.6 - fade * 0.85);
                let camera_up = vec3(0., 0.70, -0.714);
                let right = Vec3::X * p.direction.x + camera_up * p.direction.y;
                let up = camera_up * p.direction.x - Vec3::X * p.direction.y;
                let base = self.scratch.vertices.len() as u16;
                self.scratch
                    .vertices
                    .push(Vertex::new2(p.pos, Vec2::ZERO, color));
                color.a = 0.;
                for i in 0..8 {
                    let angle = i as f32 * std::f32::consts::TAU / 8.;
                    let edge =
                        p.pos + right * (angle.cos() * size) + up * (angle.sin() * size * 0.76);
                    self.scratch
                        .vertices
                        .push(Vertex::new2(edge, Vec2::ZERO, color));
                    self.scratch
                        .indices
                        .extend([base, base + 1 + i, base + 1 + (i + 1) % 8]);
                }
            }
        }
        if !self.scratch.vertices.is_empty() {
            draw_mesh(&self.scratch);
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

#[cfg(test)]
mod scenery_tests {
    use super::*;

    #[test]
    fn batching_preserves_triangles_across_old_vertex_window() {
        let mut source = empty();
        for i in 0..3603 {
            source
                .vertices
                .push(Vertex::new2(vec3(i as f32, 0., 0.), Vec2::ZERO, WHITE));
        }
        source.indices.extend([3599, 3600, 3601, 0, 1, 3602]);
        let split = batches(source);
        assert_eq!(split.iter().map(|m| m.indices.len()).sum::<usize>(), 6);
        assert_eq!(split[0].vertices[0].position.x, 3599.);
        assert_eq!(split[0].vertices[5].position.x, 3602.);
    }

    #[test]
    fn every_course_mesh_fits_renderer_batches() {
        for id in 0..3 {
            let scenery = Scenery::new(&Track::new(id));
            for mesh in scenery
                .meshes
                .iter()
                .chain(scenery.cover.iter())
                .chain(scenery.crossing.iter())
                .chain(scenery.tunnel_roof.iter())
                .chain([&scenery.shallows, &scenery.foam])
            {
                assert!(
                    mesh.vertices.len() <= 3000,
                    "track {id}: {} vertices",
                    mesh.vertices.len()
                );
                assert!(
                    mesh.indices.len() <= 4500,
                    "track {id}: {} indices",
                    mesh.indices.len()
                );
                assert_eq!(mesh.indices.len() % 3, 0);
                assert!(mesh
                    .indices
                    .iter()
                    .all(|&i| (i as usize) < mesh.vertices.len()));
            }
            if id == 2 {
                assert!(!scenery.cover.is_empty());
                assert!(!scenery.crossing.is_empty());
            }
        }
    }

    #[test]
    fn volcano_runoff_stays_on_continuous_island() {
        let track = Track::new(2);
        for sample in 0..SAMPLES * 6 {
            let (point, _, tangent) = road_sample(&track, sample as f32 / 6.);
            for side in [-1., 1.] {
                let shoulder = point + tangent.normal() * (track.width * 0.5 + 5.) * side;
                let angle = (shoulder.y / 104.).atan2(shoulder.x / 111.);
                let ripple = 1. + (angle * 3. + 0.6).sin() * 0.025 + (angle * 7.).sin() * 0.018;
                let relative = V2::new(
                    shoulder.x / (111. * 1.35 * ripple),
                    shoulder.y / (104. * 1.35 * ripple),
                );
                assert!(
                    relative.len() < 1.,
                    "sample {sample}: shoulder outside coast"
                );
            }
        }
    }

    #[test]
    fn volcano_shell_and_lava_leave_exact_upper_road_clearance() {
        let track = Track::new(2);
        let mut walls = empty();
        let mut cover = empty();
        let mut roof = Vec::new();
        volcano(&track, &mut walls, &mut cover, &mut roof);
        let normal = vec3(0.8, 0., 1.).normalize();
        let width = track.width * 0.5 + 3.;
        let lava_area: f32 = cover
            .indices
            .chunks_exact(3)
            .filter(|face| {
                let color = cover.vertices[face[0] as usize].color;
                color[0] == 245 && color[1] == 117
            })
            .map(|face| {
                let [a, b, c]: [Vec3; 3] =
                    std::array::from_fn(|i| cover.vertices[face[i] as usize].position);
                (b - a).cross(c - a).length() * 0.5
            })
            .sum();
        assert!(lava_area > 250., "crater stays visible beside road cutting");
        assert!(
            cover
                .vertices
                .iter()
                .filter(|v| v.color[0] == 245 && v.color[1] == 117)
                .count()
                > 12,
            "lava remains in crater beside cutting"
        );
        assert!(
            cover.vertices.iter().filter(|v| v.position.y > 26.).count() > 12,
            "summit remains beside cutting"
        );
        for vertex in &cover.vertices {
            if vertex.position.y > 11.001 {
                assert!(
                    normal.dot(vertex.position).abs() >= width - 0.001,
                    "shell/lava over upper road: {:?}",
                    vertex.position
                );
            }
        }
        for face in cover.indices.chunks_exact(3) {
            let vertices: [Vec3; 3] =
                std::array::from_fn(|i| cover.vertices[face[i] as usize].position);
            let high = clip_polygon(&vertices, Vec3::Y, -11.001);
            if !high.is_empty() {
                assert!(
                    high.iter().all(|p| normal.dot(*p) >= width - 0.01)
                        || high.iter().all(|p| normal.dot(*p) <= -width + 0.01)
                );
            }
        }
    }

    #[test]
    fn clipped_visibility_boundaries_stay_convex_and_outside_shaft() {
        let player = Vec3::ZERO;
        let sight = Vec3::Y;
        let points = [
            vec3(-20., 12., -20.),
            vec3(20., 12., -20.),
            vec3(20., 12., 20.),
            vec3(-20., 12., 20.),
        ];
        let pieces = sightline_cutaway(&points, player, sight);
        assert!(pieces.iter().any(|p| p.len() >= 3));
        for piece in pieces.iter().filter(|p| p.len() >= 3) {
            for point in piece {
                assert!(vec2(point.x, point.z).length() >= 7.99);
            }
            let mut orientation = 0_f32;
            for i in 0..piece.len() {
                let a = piece[(i + 1) % piece.len()] - piece[i];
                let b = piece[(i + 2) % piece.len()] - piece[(i + 1) % piece.len()];
                let turn = a.cross(b).dot(Vec3::Y);
                if turn.abs() > 0.001 {
                    if orientation != 0. {
                        assert_eq!(orientation, turn.signum());
                    }
                    orientation = turn.signum();
                }
            }
        }
    }
}
