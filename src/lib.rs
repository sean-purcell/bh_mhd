#[macro_use]
extern crate glium;
extern crate rand;

mod simulate;

use glium::Display;
use glium::framebuffer::MultiOutputFrameBuffer;

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
        use glium::{Surface,Rect,BlitTarget,uniforms};

        let layers = &self.data.layers;

        let tex1 = &layers.0.v_p;
        let fbo = tex1.as_surface();

        let target = self.facade.draw();

        let (s_width, s_height) = fbo.get_dimensions();
        let (t_width, t_height) = target.get_dimensions();
        target.blit_from_simple_framebuffer(&fbo,
            &Rect {left: 0, bottom: 0, width: s_width, height: s_height},
            &BlitTarget {left: 0, bottom: 0, width: t_width as i32, height: t_height as i32},
            uniforms::MagnifySamplerFilter::Nearest);
        target.finish().unwrap();
    }

    pub fn iteration(&self, dt: f32) {
        use glium::Surface;
        use glium::uniforms::{MinifySamplerFilter,MagnifySamplerFilter};

        let data = &self.data;
        let front = data.front_layer();
        let back = data.back_layer();

        let outputs = [("v_p", &front.v_p), ("b", &front.b)];
        let mut fbo = MultiOutputFrameBuffer::new(self.facade,
            outputs.iter().cloned()).unwrap();

        let uniforms = uniform! {
            rads: data.dimensions.rads,
            angs: data.dimensions.angs,
            levs: data.dimensions.levs,
            tex_v_p: back.v_p.sampled()
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest),
            tex_b: back.b.sampled()
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest),
        };/* here because returning Uniforms is very hard */

        let bufs = &self.shaders.buffers;
        fbo.draw(&bufs.0, &bufs.1, &self.shaders.initialize,
            &uniforms, &Default::default()).unwrap();

        self.data.next_iter();
    }
}

