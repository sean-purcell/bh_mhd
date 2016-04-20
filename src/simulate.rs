extern crate glium;

use std::cell::RefCell;
use std::fmt;

use glium::{Program,VertexBuffer,IndexBuffer,Display};
use glium::framebuffer::MultiOutputFrameBuffer;
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

    pub fn draw(&self, f: &Display, program: &Program,
        buffers: &(VertexBufferAny, IndexBufferAny)) {
        use glium::Surface;
        use glium::uniforms::{MinifySamplerFilter,MagnifySamplerFilter};

        let front = self.front_layer();
        let back = self.back_layer();

        let outputs = [("v_p", &front.v_p), ("b", &front.b)];
        let mut fbo = MultiOutputFrameBuffer::new(f,
            outputs.iter().cloned()).unwrap();

        let uniforms = uniform! {
            rads: self.dimensions.rads,
            angs: self.dimensions.angs,
            levs: self.dimensions.levs,
            tex_v_p: back.v_p.sampled()
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest),
            tex_b: back.b.sampled()
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest),
        };/* here because returning Uniforms is very hard :( */

        fbo.draw(&buffers.0, &buffers.1, program,
            &uniforms, &Default::default()).unwrap();
        self.next_iter();
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

impl fmt::Debug for Layer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Layer: ({:?}, {:?})", self.v_p, self.b)
    }
}

pub struct Shaders {
    pub initialize: Program,
    pub update: Program,
    pub buffers: (VertexBufferAny, IndexBufferAny),
}

impl Shaders {
    pub fn new(f: &Display) -> Self {
        Shaders {
            initialize: sim_code::initial(f),
            update: sim_code::update(f),
            buffers: cover_buffers(f),
        }
    }
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
        IndexBuffer::new(f, PrimitiveType::TriangleStrip,
            &[0u8, 1, 2, 3]).unwrap().into()
    )
}

mod sim_code {
    use glium::{Display,Program};
    pub fn initial(f: &Display) -> Program {
        Program::from_source(f,
            VERT_SHADER,
            &initial_frag_shader(),
            None).unwrap()
    }

    pub fn update(f: &Display) -> Program {
        Program::from_source(f,
            VERT_SHADER,
            &update_frag_shader(),
            None).unwrap()
    }

    fn initial_frag_shader() -> String {
        gen_frag_shader(INIT)
    }

    fn update_frag_shader() -> String {
        gen_frag_shader(UPDATE)
    }

    const INIT: &'static str = r#"
        float ang = (rind / float(rads) +
                     lind / float(rads) +
                     aind/float(angs)) * 2 * M_PI;
        float s = sin(ang);
        float c = cos(ang);
        v_p = vec4(s, s, s, 1.0);
        b = vec4(c, c, c, 1.0);
    "#;

    const UPDATE: &'static str = r#"
        /* take v_p as value, b as derivative */

        v_p = lookup(tex_v_p, wrap(rind-1, rads), lind, aind);
        b = lookup(tex_b, wrap(rind-1, rads), lind, aind);
    "#;

    const VERT_SHADER: &'static str = r#"
        #version 330
        in vec2 pos;
        out vec2 uv;

        void main() {
            gl_Position = vec4(pos, 0.0, 1.0);
            uv = vec2((pos.x + 1) / 2.0, (pos.y + 1) / 2.0);
        }
    "#;

    const FRAG_PREAMBLE: &'static str = r#"
        #version 330
        #define M_PI 3.1415926535897932384626433832795
        uniform uint rads; /* texture height */
        uniform uint angs; /* texture width per section */
        uniform uint levs; /* texture section count */

        in vec2 uv;

        uniform sampler2D tex_v_p;
        uniform sampler2D tex_b;

        layout(location = 0) out vec4 v_p;
        layout(location = 1) out vec4 b;

        vec4 lookup(sampler2D tex, int rind, int lind, int aind) {
            return texelFetch(tex, ivec2(lind * int(levs) + aind, rind), 0);
        }

        int wrap(int ind, uint max) {
            return (ind + int(max)) % int(max);
        }
    "#;

    fn gen_frag_shader(update_func: &'static str) -> String {
        format!(r#"

            {preamble}

            void main() {{
                int rind = int(uv.y * rads);
                int lind = int(uv.x * levs);
                int aind = int(fract(uv.x * levs) * angs);

                {update}
            }}"#,
            preamble = FRAG_PREAMBLE,
            update = update_func)
    }
}

