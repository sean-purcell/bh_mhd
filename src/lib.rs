extern crate glium;

pub struct BhMhd<'a> {
    facade: &'a glium::backend::Facade,
    layers: Vec<glium::texture::Texture2d>,
    sim_prog: glium::Program,
}

impl<'a> BhMhd<'a> {
    pub fn new<F>(f: &'a F) -> Self where F: glium::backend::Facade {
        BhMhd {
            facade: f,
            layers: BhMhd::init_layers(f),
            sim_prog: glium::Program::from_source(f, "", "", None).unwrap(),
        }
    }

    fn init_layers<F>(f: &'a F) -> Vec<glium::texture::Texture2d>
        where F: glium::backend::Facade {
        let ang_num = 100u32;
        let height_num = 100u32;
        let rad_num = 100u32;

        (0..height_num)
            .into_iter()
            .map(|x| { glium::texture::Texture2d::empty(f, ang_num, rad_num).unwrap() })
            .collect()
    }
}

pub fn print_hello() {
    println!("hello");
}

