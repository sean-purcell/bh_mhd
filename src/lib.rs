#[macro_use]
extern crate glium;
extern crate rand;

use std::cell::RefCell;

use glium::{Program,VertexBuffer,IndexBuffer, Display};
use glium::vertex::VertexBufferAny;
use glium::index::{PrimitiveType,IndexBufferAny};
use glium::backend::Facade;
use glium::texture::Texture2d;
use glium::framebuffer::MultiOutputFrameBuffer;

pub struct Sim<'a> {
    facade: &'a Display,
    data: SimData,
    sim_prog: Program,
    cover_buffers: (VertexBufferAny,IndexBufferAny),
}

struct SimData {
    layers: (Vec<Layer>, Vec<Layer>),
    iter: RefCell<i32>,
}

struct Layer {
    v_p: Texture2d,
    b: Texture2d,
}

impl<'a> Sim<'a> {
    pub fn new(f: &'a Display) -> Self {
        Sim {
            facade: f,
            data: SimData::new(f),
            sim_prog: test_program(f),
            cover_buffers: cover_buffers(f),
        }
    }

    pub fn test_render(&self) {
        use glium::{Surface,Rect,BlitTarget,uniforms};
        let (vert_buf, ind_buf) = cover_buffers(self.facade);

        let layers = &self.data.layers;
        let index = rand::random::<usize>() % self.data.layers.0.len();

        let tex1 = &layers.0[index].v_p;
        let tex2 = &layers.0[index].b;
        let mut fbo = {
            let outputs = [("v_p", tex1), ("b", tex2)];
            MultiOutputFrameBuffer::new(self.facade, outputs.iter().cloned()).unwrap()
        };
        let mut fbo1 = tex1.as_surface();
        let mut fbo2 = tex2.as_surface();

        fbo.draw(&vert_buf, &ind_buf, &self.sim_prog,
            &uniforms::EmptyUniforms, &Default::default()).unwrap();

        let target = self.facade.draw();

        let (s_width, s_height) = fbo.get_dimensions();
        let (t_width, t_height) = target.get_dimensions();
        target.blit_from_simple_framebuffer(&fbo1,
            &Rect {left: 0, bottom: 0, width: s_width, height: s_height},
            &BlitTarget {left: 0, bottom: 0, width: t_width as i32, height: t_height as i32},
            uniforms::MagnifySamplerFilter::Nearest);
        target.blit_from_simple_framebuffer(&fbo1,
            &Rect {left: 0, bottom: 0, width: s_width, height: s_height},
            &BlitTarget {left: 0, bottom: 0, width: t_width as i32, height: t_height as i32},
            uniforms::MagnifySamplerFilter::Nearest);
        target.finish().unwrap();
    }

    pub fn iteration(&self, dt: f32) {
        use glium::{Surface,uniforms};
        let front = self.data.front_layer();
        let back = self.data.back_layer();

        for i in 0..front.len() {
            let layer = &front[i];
            let outputs = [("v_p", &layer.v_p), ("b", &layer.b)];
            let mut fbo = MultiOutputFrameBuffer::new(self.facade, outputs.iter().cloned()).unwrap();

            let bufs = &self.cover_buffers;
            fbo.draw(&bufs.0, &bufs.1, &self.sim_prog,
                &uniforms::EmptyUniforms, &Default::default()).unwrap();
        }

        self.data.next_iter();
    }
}

impl SimData {
    fn new(f: &Display) -> Self {
        let layers = {
            let ang_num = 100u32;
            let rad_num = 100u32;
            let lev_num = 100u32;

            ((0..lev_num)
                .map(|_| Layer::new(f, ang_num, rad_num))
                .collect(),
             (0..lev_num)
                .map(|_| Layer::new(f, ang_num, rad_num))
                .collect(),)
        };

        SimData { layers: layers, iter: RefCell::new(0) }
    }

    fn front_layer(&self) -> &Vec<Layer> {
        if *self.iter.borrow() % 2 == 0 {
            &self.layers.0
        } else {
            &self.layers.1
        }
    }

    fn back_layer(&self) -> &Vec<Layer> {
        if *self.iter.borrow() % 2 == 1 {
            &self.layers.0
        } else {
            &self.layers.1
        }
    }

    fn next_iter(&self) {
        *self.iter.borrow_mut() += 1;
    }
}

impl Layer {
    fn new(f: &Display, width: u32, height: u32) -> Self {
        let format = glium::texture::UncompressedFloatFormat::F32F32F32F32;
        let mipmaps = glium::texture::MipmapsOption::NoMipmap;

        Layer {
            v_p: Texture2d::empty_with_format(f,
                format, mipmaps, width, height).unwrap(),
            b: Texture2d::empty_with_format(f,
                format, mipmaps, width, height).unwrap(),
        }
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
        layout(location = 0) out vec4 v_p;
        layout(location = 1) out vec4 b;
        void main() {
            v_p = vec4((pos_o.x+1)/2, (pos_o.y+1)/2, 0, 1.0);
            b = v_p.grba;
        }"#;

    Program::from_source(f, vert_str, frag_str, None).unwrap()
}

fn cover_buffers<F>(f: &F) -> (VertexBufferAny, IndexBufferAny)
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

