#[macro_use]
extern crate glium;
extern crate rand;

use std::boxed::Box;
use std::rc::Rc;
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
}

struct SimData {
    layer1: Vec<Layer>,
    layer2: Vec<Layer>,
}

struct Layer {
    v_p: Texture2d,
    b: Texture2d,
}

impl Layer {
    fn new(f: &Display, width: u32, height: u32) -> Self {
        let format = glium::texture::UncompressedFloatFormat::F32F32F32F32;
        let mipmaps = glium::texture::MipmapsOption::NoMipmap;

        let v_p = Texture2d::empty_with_format(f,
                format, mipmaps, width, height).unwrap();
        let b = Texture2d::empty_with_format(f,
                format, mipmaps, width, height).unwrap();


        let mut layer = Layer {
            v_p: Texture2d::empty_with_format(f,
                format, mipmaps, width, height).unwrap(),
            b: Texture2d::empty_with_format(f,
                format, mipmaps, width, height).unwrap(),
        };

        layer
    }
}

/*
impl<'a> Sim<'a> {
    pub fn new(f: &'a Display) -> Self {
        Sim {
            facade: f,
            data: Sim::init_layers(f),
        }
    }

    pub fn test_render(&self) {
        use glium::{Surface,Rect,BlitTarget,uniforms};
        let (vert_buf, ind_buf) = test_buffers(self.facade);

        let index = rand::random::<usize>() % self.data.len();

        let tex1 = &self.data[index].v_p;
        let tex2 = &self.data[index].b;
        let mut fbo1 = tex1.as_surface();
        let mut fbo2 = tex2.as_surface();
        fbo1.draw(&vert_buf, &ind_buf, &test_program1(self.facade),
            &uniforms::EmptyUniforms, &Default::default()).unwrap();
        fbo2.draw(&vert_buf, &ind_buf, &test_program2(self.facade),
            &uniform! {tex:
                tex1.sampled()
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                    .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)},
            &Default::default()).unwrap();


        let target = self.facade.draw();

        let (s_width, s_height) = fbo2.get_dimensions();
        let (t_width, t_height) = target.get_dimensions();
        target.blit_from_simple_framebuffer(&fbo2,
            &Rect {left: 0, bottom: 0, width: s_width, height: s_height},
            &BlitTarget {left: 0, bottom: 0, width: t_width as i32, height: t_height as i32},
            uniforms::MagnifySamplerFilter::Nearest);
        target.finish().unwrap();
    }

    fn init_layers(f: &'a Display) -> Vec<Layer> {
        let ang_num = 100u32;
        let height_num = 100u32;
        let rad_num = 100u32;


        (0..height_num)
            .into_iter()
            .map(|_| { Layer::new(f, ang_num, rad_num) })
            .collect()
    }
}
*/
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

fn test_program1<F>(f: &F) -> Program where F: Facade {
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
            color = vec4((pos_o.x+1)/2 + 1, (pos_o.y+1)/2, 0, 1.0);
        }"#;

    Program::from_source(f, vert_str, frag_str, None).unwrap()
}

fn test_program2<F>(f: &F) -> Program where F: Facade {
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
        uniform sampler2D tex;
        void main() {
            vec2 tex_coords = vec2((pos_o.x+1)/2,(pos_o.y+1)/2);
            vec4 tex_col = texture(tex, tex_coords);
            color = vec4(tex_col.r - 1, tex_col.gba);
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

