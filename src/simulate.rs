extern crate glium;

use std::cell::RefCell;

use glium::{Program,VertexBuffer,IndexBuffer,Display};
use glium::backend::Facade;
use glium::vertex::VertexBufferAny;
use glium::index::{PrimitiveType,IndexBufferAny};
use glium::texture::Texture2d;

pub struct Data {
    pub layers: (Layer, Layer),
    pub dimensions: SimDimensions,
    pub iter: RefCell<i32>,
}

pub struct SimDimensions {
    pub angs: u32,
    pub rads: u32,
    pub levs: u32,
}

impl Data {
    pub fn new(f: &Display) -> Self {
        let dims = {
            let ang_num = 100u32;
            let rad_num = 100u32;
            let lev_num = 100u32;

            SimDimensions {
                angs: ang_num,
                rads: rad_num,
                levs: lev_num,
            }
        };

        let layers = {
            (Layer::new(f, dims.angs, dims.rads, dims.levs),
            Layer::new(f, dims.angs, dims.rads, dims.levs))
        };

        Data { layers: layers,
            dimensions: dims,
            iter: RefCell::new(0) }
    }

    pub fn front_layer(&self) -> &Layer {
        if *self.iter.borrow() % 2 == 0 {
            &self.layers.0
        } else {
            &self.layers.1
        }
    }

    pub fn back_layer(&self) -> &Layer {
        if *self.iter.borrow() % 2 == 1 {
            &self.layers.0
        } else {
            &self.layers.1
        }
    }

    pub fn next_iter(&self) {
        *self.iter.borrow_mut() += 1;
    }
}

pub struct Layer {
    pub v_p: Texture2d,
    pub b: Texture2d,
}

impl Layer {
    fn new(f: &Display, width: u32, height: u32, depth: u32) -> Self {
        let format = glium::texture::UncompressedFloatFormat::F32F32F32F32;
        let mipmaps = glium::texture::MipmapsOption::NoMipmap;

        Layer {
            v_p: Texture2d::empty_with_format(f,
                format, mipmaps, width * depth, height).unwrap(),
            b: Texture2d::empty_with_format(f,
                format, mipmaps, width * depth, height).unwrap(),
        }
    }
}

pub struct Shaders {
    pub initialize: Program,
    pub buffers: (VertexBufferAny, IndexBufferAny),
}

impl Shaders {
    pub fn new(f: &Display) -> Self {
        Shaders {
            initialize: initial_conditions::program(f),
            buffers: cover_buffers(f),
        }
    }
}

fn cover_buffers<F>(f: &F) -> (VertexBufferAny, IndexBufferAny)
    where F: Facade {

    #[derive(Copy, Clone)]
    struct Vertex {
        uv: (f32, f32),
    }

    implement_vertex!(Vertex, uv);

    (
        VertexBuffer::new(f, &[
            Vertex { uv: (-1.0,  1.0) }, Vertex { uv: (1.0,  1.0) },
            Vertex { uv: (-1.0, -1.0) }, Vertex { uv: (1.0, -1.0) },
        ]).unwrap().into(),
        IndexBuffer::new(f, PrimitiveType::TriangleStrip,
            &[0u8, 1, 2, 3]).unwrap().into()
    )
}

mod initial_conditions {
    use glium::{Display,Program};
    pub fn program(f: &Display) -> Program {
        Program::from_source(f,
            ::simulate::VERT_SHADER,
            &frag_shader(),
            None).unwrap()
    }

    fn frag_shader() -> String {
        format!(r#"

        {preamble}

        void main() {{
            uint rind = uint(pos.y * rads + 0.5);
            uint lind = uint(pos.x * levs + 0.5);
            uint aind = uint(fract(pos.x * levs) * angs + 0.5);


            /* setup the initial conditions */
            {init}
        }}"#,
        preamble = ::simulate::FRAG_PREAMBLE,
        init = INIT)
    }

    const INIT: &'static str = r#"
        float waves = 5;
        float ang = (rind / float(rads) +
                    lind / float(levs) +
                    aind / float(angs)) * waves * 2 * M_PI;
        float s = sin(ang);
        float c = cos(ang);
        v_p = vec4(s, s, s, 1.0);
        b = vec4(c, c, c, 1.0);
    "#;
}

const VERT_SHADER: &'static str = r#"
    #version 330
    in vec2 uv;
    out vec2 pos;

    void main() {
        gl_Position = vec4(uv, 0.0, 1.0);
        pos = uv;
    }
"#;

const FRAG_PREAMBLE: &'static str = r#"
    #version 330
    #define M_PI 3.1415926535897932384626433832795
    uniform uint rads; /* texture height */
    uniform uint angs; /* texture width per section */
    uniform uint levs; /* texture section count */

    in vec2 pos;

    uniform sampler2D tex_v_p;
    uniform sampler2D tex_b;

    layout(location = 0) out vec4 v_p;
    layout(location = 1) out vec4 b;

    vec4 lookup(sampler2D tex, uint rind, uint lind, uint aind) {
        return texelFetch(tex, ivec2(lind * levs + aind, rind), 0);
    }
"#;

