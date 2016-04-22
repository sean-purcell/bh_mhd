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
        buffers: &(VertexBufferAny, IndexBufferAny), dt: f32) {
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
            dt: dt,
            tex_v_p: back.v_p.sampled()
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest),
            tex_b: back.b.sampled()
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest),
        };

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
        ::check_program(Program::from_source(f,
            VERT_SHADER,
            &initial_frag_shader(),
            None))
    }

    pub fn update(f: &Display) -> Program {
        ::check_program(Program::from_source(f,
            VERT_SHADER,
            &update_frag_shader(),
            None))
    }

    fn initial_frag_shader() -> String {
        gen_frag_shader(INIT)
    }

    fn update_frag_shader() -> String {
        gen_frag_shader(UPDATE)
    }

    const INIT: &'static str = r#"
        float val;
        if(abs(rind - int(rads)/2) <= 2 && abs(lind - int(levs)/2) <= 2 && abs(aind - int(angs)/2) <= 2) {
            val = 10;
        } else {
            val = 0;
        }

        v_p = vec4(val, val, val, 1.0);
        b = vec4(0, 0, 0, 0);
    "#;

    const UPDATE: &'static str = r#"
        /* take v_p as value, b as derivative */

        #define VAL(r, l, a) (lookup(tex_v_p, r, l, a).g)

        float div2 = (
            (-VAL(wrap(rind-2, rads), lind, aind) + VAL(wrap(rind-1, rads), lind, aind)*16
             -VAL(wrap(rind+2, rads), lind, aind) + VAL(wrap(rind+1, rads), lind, aind)*16) +
            (-VAL(rind, wrap(lind-2, levs), aind) + VAL(rind, wrap(lind-1, levs), aind)*16
             -VAL(rind, wrap(lind+2, levs), aind) + VAL(rind, wrap(lind+1, levs), aind)*16) +
            (-VAL(rind, lind, wrap(aind-2, angs)) + VAL(rind, lind, wrap(aind-1, angs))*16
             -VAL(rind, lind, wrap(aind+2, angs)) + VAL(rind, lind, wrap(aind+1, angs))*16) +
            -VAL(rind, lind, aind) * 30) / 12;

        #undef VAL

        float x = lookup(tex_v_p, rind, lind, aind).r;
        float v = lookup(tex_b, rind, lind, aind).r;
        float nx = x + v * dt;
        float nv = v + div2 * dt * 343;

        v_p = vec4(nx, nx, nx, 1.0);
        b = vec4(nv, nv, nv, 1.0);
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

        uniform float dt;

        uniform sampler2D tex_v_p;
        uniform sampler2D tex_b;

        layout(location = 0) out vec4 v_p;
        layout(location = 1) out vec4 b;

        vec4 lookup(sampler2D tex, int rind, int lind, int aind) {
            return texelFetch(tex, ivec2(lind * int(levs) + aind, rind), 0);
        }

        vec3 v_f(vec4 v_p) {
            return vec3(v_p);
        }
        float p_f(vec4 v_p) {
            return v_p.w;
        }
        vec3 b_f(vec4 b) {
            return vec3(b);
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

