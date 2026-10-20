mod bridge;
mod render;
use beach_buggy::sim::*;
use macroquad::prelude::*;

fn config() -> Conf {
    Conf {
        window_title: "Beach Buggy • Coast Club".into(),
        window_width: 1440,
        window_height: 900,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}
fn keyboard() -> Input {
    Input {
        throttle: if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            1.
        } else {
            0.
        },
        brake: if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            1.
        } else {
            0.
        },
        steer: f32::from(is_key_down(KeyCode::D) || is_key_down(KeyCode::Right))
            - f32::from(is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)),
        handbrake: is_key_down(KeyCode::Space),
    }
}
#[macroquad::main(config)]
async fn main() {
    let mut race = Race::new(0, 0, 0, false);
    let mut scenery = render::Scenery::new(&race.track);
    let mut effects = render::Effects::new();
    let mut state = 0; // 0 menu, 1 race, 2 pause, 3 results
    let mut camera = vec3(0., 0., 0.);
    let mut previous_pause = false;
    let mut previous_reset = false;
    let mut previous_count = 4;
    let mut previous_laps = 0;
    let mut saved_best = 0.;
    let mut frame = 0u64;
    loop {
        let dt = get_frame_time().min(0.1);
        let command = bridge::command();
        let pad_pause = bridge::pad(4) > 0.5;
        let pad_reset = bridge::pad(5) > 0.5;
        let pause =
            is_key_pressed(KeyCode::Escape) || (pad_pause && !previous_pause) || command == 5;
        previous_pause = pad_pause;
        if command == 1 || command == 2 {
            let track = bridge::option(0) as usize;
            let car = bridge::option(1) as usize;
            let physics = bridge::option(2) as usize;
            race = Race::new(
                track.min(2),
                car.min(3),
                physics.min(2),
                bridge::option(3) > 0.5,
            );
            scenery = render::Scenery::new(&race.track);
            effects = render::Effects::new();
            state = 1;
            camera = vec3(race.cars[0].pos.x, 0., race.cars[0].pos.y);
            saved_best =
                bridge::read(record_key(race.track.id, race.cars[0].spec, race.physics)).max(0.);
            previous_count = 4;
            previous_laps = 0;
        }
        if command == 3 {
            state = 0;
        }
        if pause {
            if state == 1 {
                state = 2;
            } else if state == 2 {
                state = 1;
            }
        }
        if state == 1
            && (is_key_pressed(KeyCode::R) || command == 4 || (pad_reset && !previous_reset))
        {
            race.cars[0].reset(&race.track);
        }
        previous_reset = pad_reset;
        if state == 0 {
            if bridge::option(0) as usize != race.track.id {
                race = Race::new((bridge::option(0) as usize).min(2), 0, 0, false);
                scenery = render::Scenery::new(&race.track);
            }
            race.countdown = 0.;
            let input = ai_input(&race.cars[0], &race.track, race.physics);
            race.advance(dt, input);
        } else if state == 1 {
            let mut input = keyboard();
            input.throttle = input.throttle.max(bridge::pad(0));
            input.brake = input.brake.max(bridge::pad(1));
            let stick = bridge::pad(2);
            if stick.abs() > 0.05 {
                input.steer = stick;
            }
            input.handbrake |= bridge::pad(3) > 0.5;
            if bridge::option(5) > 0.5 {
                // Explicit browser-test/demo mode; never writes personal records.
                for _ in 0..8 {
                    let control = ai_input(&race.cars[0], &race.track, race.physics);
                    race.advance(dt, control);
                }
            } else {
                race.advance(dt, input);
            }
            let count = race.countdown.ceil().max(0.) as i32;
            if count != previous_count {
                bridge::event(if count == 0 { 1 } else { 0 }, 1.);
                previous_count = count;
            }
            let car = &race.cars[0];
            if car.laps > previous_laps {
                previous_laps = car.laps;
                if bridge::option(5) < 0.5
                    && car.best > 0.
                    && (saved_best == 0. || car.best < saved_best)
                {
                    saved_best = car.best;
                    bridge::store(
                        record_key(race.track.id, car.spec, race.physics),
                        saved_best,
                    );
                }
                bridge::event(2, 0.7);
            }
            if car.finished {
                state = 3;
                bridge::event(3, 1.);
            }
            effects.update(&race, dt);
        }
        let player = &race.cars[0];
        let alpha = (race.accumulator / DT).clamp(0., 1.);
        let car_pos = player.previous.lerp(player.pos, alpha);
        let target = if state == 0 {
            vec3(5., 0., -5.)
        } else {
            vec3(
                car_pos.x + player.vel.x * 0.3,
                player.height,
                car_pos.y + player.vel.y * 0.3,
            )
        };
        camera = camera.lerp(target, 1. - (-dt * 5.).exp());
        let overview = state == 0;
        set_camera(&Camera3D {
            position: camera
                + if overview {
                    vec3(80., 155., 160.)
                } else {
                    vec3(0., 65., 46.)
                },
            target: camera,
            up: vec3(0., 1., 0.),
            fovy: if overview {
                235.
            } else {
                70. + player.vel.len() * 0.35
            },
            projection: Projection::Orthographics,
            z_near: 0.1,
            z_far: 1500.,
            ..Default::default()
        });
        scenery.draw(get_time() as f32);
        effects.draw();
        for (i, car) in race.cars.iter().enumerate() {
            render::draw_car(car, alpha, i == 0 && !overview, get_time() as f32);
        }
        set_default_camera();
        if !overview {
            render::minimap(&race);
        }
        bridge::telemetry(bridge::Telemetry {
            state,
            speed: player.vel.len() * 3.6,
            lap: (player.laps + 1).min(3) as i32,
            position: race.position() as i32,
            elapsed: if race.trial {
                player.lap_time
            } else {
                race.elapsed
            },
            best: if saved_best > 0. {
                saved_best
            } else {
                player.best
            },
            countdown: race.countdown,
            finish: player.finish_time,
            slip: player.slip,
            impact: player.impact,
            grounded: player.grounded,
        });
        frame += 1;
        if frame == 1 {
            bridge::event(9, 1.);
        }
        next_frame().await;
    }
}
