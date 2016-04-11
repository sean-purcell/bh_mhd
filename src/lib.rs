#[macro_use]
extern crate glium;
extern crate rand;

use glium::{Program,VertexBuffer,IndexBuffer, Display};
use glium::vertex::VertexBufferAny;
use glium::index::{PrimitiveType,IndexBufferAny};
use glium::backend::Facade;
use glium::texture::Texture2d;

pub struct BhMhd<'a> {
    facade: &'a Display,
    layers: Vec<Texture2d>,
    sim_prog: Program,
}

impl<'a> BhMhd<'a> {
    pub fn new(f: &'a Display) -> Self {
        BhMhd {
            facade: f,
            layers: BhMhd::init_layers(f),
            sim_prog: test_program(f),
        }
    }

    pub fn test_render(&self) {
        use glium::{Surface,Rect,BlitTarget,uniforms};
        let (vert_buf, ind_buf) = test_buffers(self.facade);

        let index = rand::random::<usize>() % self.layers.len();

        let tex = &self.layers[index];
        let mut fbo = tex.as_surface();
        fbo.draw(&vert_buf, &ind_buf, &self.sim_prog,
            &uniforms::EmptyUniforms, &Default::default()).unwrap();

        let target = self.facade.draw();

        let (s_width, s_height) = fbo.get_dimensions();
        let (t_width, t_height) = target.get_dimensions();
        target.blit_from_simple_framebuffer(&fbo,
            &Rect {left: 0, bottom: 0, width: s_width, height: s_height},
            &BlitTarget {left: 0, bottom: 0, width: t_width as i32, height: t_height as i32},
            uniforms::MagnifySamplerFilter::Nearest);
        target.finish().unwrap();
    }

    fn init_layers(f: &'a Display) -> Vec<Texture2d> {
        let ang_num = 100u32;
        let height_num = 100u32;
        let rad_num = 100u32;

        (0..height_num)
            .into_iter()
            .map(|_| { Texture2d::empty(f, ang_num, rad_num).unwrap() })
            .collect()
    }
}

fn test_program<F>(f: &F) -> Program where F: Facade {
    let vert_str = r#"
        #version 330
        in vec2 pos;
        out vec2 pos_o;
        void main() {
            pos_o = pos;
            gl_Position = vec4(pos, 0.0, 1.0);
        }"#;
    let frag_str = r#"
        #version 330
        in vec2 pos_o;
        out vec4 color;
        void main() {
            color = vec4((pos_o.x+1)/2, (pos_o.y+1)/2, 0, 1.0);
        }"#;

    Program::from_source(f, vert_str, frag_str, None).unwrap()
}

fn test_buffers<F>(f: &F) -> (VertexBufferAny, IndexBufferAny)
    where F: Facade {

    #[derive(Copy, Clone)]
    struct Vertex {
        pos: (f32, f32),
    }

    implement_vertex!(Vertex, pos);

    (
        VertexBuffer::new(f, &[
            Vertex { pos: (-1.0,  1.0) }, Vertex { pos: (1.0,  1.0) },
            Vertex { pos: (-1.0, -1.0) }, Vertex { pos: (1.0, -1.0) },
        ]).unwrap().into(),
        IndexBuffer::new(f, PrimitiveType::TriangleStrip, &[0u8, 1, 2, 3]).unwrap().into()
    )
}

