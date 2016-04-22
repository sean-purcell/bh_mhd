#[macro_use]
extern crate glium;
extern crate rand;

mod simulate;

use glium::Display;
use glium::Program;
use glium::program::ProgramCreationError;

use simulate::{Data,Shaders};

pub struct Sim<'a> {
    facade: &'a Display,
    data: Data,
    shaders: Shaders,
}

impl<'a> Sim<'a> {
    pub fn new(f: &'a Display) -> Self {
        Sim {
            facade: f,
            data: Data::new(f),
            shaders: Shaders::new(f),
        }
    }

    pub fn test_render(&self) {
        use glium::{Surface,uniforms};

        let tex1 = &self.data.back_layer().v_p;
        let fbo = tex1.as_surface();

        let target = self.facade.draw();

        fbo.fill(&target, uniforms::MagnifySamplerFilter::Nearest);
        target.finish().unwrap();
    }

    pub fn initial(&self) {
        self.data.draw(&self.facade, &self.shaders.initialize,
            &self.shaders.buffers, 0.0f32);
    }

    pub fn iteration(&self, dt: f32, t: f32) {
        self.data.draw(&self.facade, &self.shaders.update,
            &self.shaders.buffers, dt);
    }
}

fn check_program(prog: Result<Program, ProgramCreationError>) -> Program {
    use glium::program::ProgramCreationError::*;
    match prog {
        Ok(p) => p,
        Err(e) => {
            match e {
                CompilationError(s) => println!("Failed to compile: {}", s),
                _ => println!("Failed to compile: {:?}", e),
            };
            panic!("Failed to compile")
        }
    }
}

