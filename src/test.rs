extern crate bh_mhd;
extern crate glium;
extern crate time;

use glium::{glutin,DisplayBuild};

fn main() {
    let display = build_display();
    let sim = bh_mhd::Sim::new(&display);
    loop {
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

