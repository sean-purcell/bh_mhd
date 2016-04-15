extern crate bh_mhd;
extern crate glium;
extern crate time;

use glium::{glutin,DisplayBuild};

fn main() {
    let display = build_display();
    let sim = bh_mhd::Sim::new(&display);
    let mut prev_time = time::precise_time_ns();
    loop {
        let dt = {
            let cur = time::precise_time_ns();
            let delta = cur - prev_time;
            prev_time = cur;
            delta as f32 / (1000000000.0f32)
        };
        println!("fps: {}, dt: {}", (1.0f32/dt) as i32, dt);
        sim.iteration(dt);
        sim.test_render();
        display.finish();

        for ev in display.poll_events() {
            use glium::glutin::Event::*;
            match ev {
                Closed => return,
                _ => (),
            }
        }
    }
}

fn build_display() -> glium::Display {
    glutin::WindowBuilder::new()
        .with_visibility(true)
        .build_glium()
        .unwrap()
}

