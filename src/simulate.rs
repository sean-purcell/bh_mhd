extern crate glium;

use glium::Program;

const PREAMBLE: &'static str = r#"
    #version 330
    uniform uint rads; /* texture height */
    uniform uint angs; /* texture width per section */
    uniform uint levs; /* texture sections

    in vec2 pos;
"#;

pub struct Shaders {
    pub initialize: Program,
}

impl Shaders {
    pub fn new() {
        format!(r#"
        {preamble}
        void main() {

        }"#);
    }
}

