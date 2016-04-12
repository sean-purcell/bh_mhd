extern crate bh_mhd;
extern crate glium;

use glium::{glutin,DisplayBuild};

fn main() {
    let display = build_display();
 //   let bh = bh_mhd::BhMhd::new(&display);
    let mut a = 0i32;
    loop {
   //     bh.test_render();
        display.finish();
        println!("rendered {}", a);
        a+=1;

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

