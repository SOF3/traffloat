//! Node rendering

use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::{mesh, texture};
use crate::render::util::{create_program, AttrLocation, UniformLocation};
use safety::Safety;
use traffloat::space::{Matrix, Vector};

/// Stores the setup data for node rendering.
pub struct Program {
    prog: WebGlProgram,
    cube: mesh::PreparedMesh,
    a_pos: AttrLocation,
    a_normal: AttrLocation,
    a_tex_pos: AttrLocation,
    u_proj: UniformLocation<Matrix>,
    u_sun: UniformLocation<Vector>,
    u_brightness: UniformLocation<f64>,
    u_inv_gain: UniformLocation<f32>,
    u_tex: UniformLocation<i32>,
}

impl Program {
    /// Initializes node canvas resources.
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let prog = create_program(
            gl,
            "node.vert",
            include_str!("node.min.vert"),
            "node.frag",
            include_str!("node.min.frag"),
        );
        let cube = mesh::CUBE.prepare(gl);

        let a_pos = AttrLocation::new(gl, &prog, "a_pos");
        let a_normal = AttrLocation::new(gl, &prog, "a_normal");
        let a_tex_pos = AttrLocation::new(gl, &prog, "a_tex_pos");
        let u_proj = UniformLocation::new(gl, &prog, "u_proj");
        let u_sun = UniformLocation::new(gl, &prog, "u_sun");
        let u_brightness = UniformLocation::new(gl, &prog, "u_brightness");
        let u_inv_gain = UniformLocation::new(gl, &prog, "u_inv_gain");
        let u_tex = UniformLocation::new_optional(gl, &prog, "u_tex");

        Self {
            prog,
            cube,
            a_pos,
            a_normal,
            a_tex_pos,
            u_proj,
            u_sun,
            u_brightness,
            u_inv_gain,
            u_tex,
        }
    }

    /// Draws a node on the canvas.
    ///
    /// The projection matrix transforms unit model coordinates to projection coordinates directly.
    pub fn draw(
        &self,
        gl: &WebGlRenderingContext,
        proj: Matrix,
        sun: Vector,
        brightness: f64,
        selected: bool,
        texture: &texture::PreparedTexture,
    ) {
        gl.use_program(Some(&self.prog));
        self.u_proj.assign(gl, proj);
        self.u_sun.assign(gl, sun);
        self.u_brightness.assign(gl, brightness.clamp(0.5, 1.));
        self.u_inv_gain
            .assign(gl, if selected { 0.5f32 } else { 1f32 });

        self.a_pos.assign(gl, self.cube.positions());
        self.a_normal.assign(gl, self.cube.normals());

        texture.apply(self.cube.tex_pos(), self.a_tex_pos, &self.u_tex, gl);

        gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MAG_FILTER,
            WebGlRenderingContext::NEAREST.homosign(),
        );
        gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MIN_FILTER,
            WebGlRenderingContext::NEAREST_MIPMAP_NEAREST.homosign(),
        );
        self.cube.draw(gl);
    }
}
