use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlUniformLocation};

pub trait WebglExt {
    fn canvas(&self) -> &WebGlRenderingContext;

    fn set_uniform(&self, program: &WebGlProgram, name: &str, uniform: impl Uniform) {
        let gl = self.canvas();
        let location = gl.get_uniform_location(program, name);
        uniform.apply(location.as_ref(), gl);
    }
}

impl WebglExt for WebGlRenderingContext {
    fn canvas(&self) -> &Self {
        self
    }
}

pub trait Uniform {
    fn apply(&self, location: Option<&WebGlUniformLocation>, gl: &WebGlRenderingContext);
}

macro_rules! impl_uniform {
    ($unif:ident, $vec:ident, {$($extra:tt)*}) => {
        impl Uniform for nalgebra::$vec<f32> {
            fn apply(&self, location: Option<&WebGlUniformLocation>, gl: &WebGlRenderingContext) {
                gl.$unif(location, $($extra)* self.as_slice());
            }
        }
    }
}

impl_uniform!(uniform2fv_with_f32_array, Vector2, {});
impl_uniform!(uniform3fv_with_f32_array, Vector3, {});
impl_uniform!(uniform4fv_with_f32_array, Vector4, {});

impl_uniform!(uniform_matrix2fv_with_f32_array, Matrix2, {false, });
impl_uniform!(uniform_matrix3fv_with_f32_array, Matrix3, {false, });
impl_uniform!(uniform_matrix4fv_with_f32_array, Matrix4, {false, });

impl Uniform for f32 {
    fn apply(&self, location: Option<&WebGlUniformLocation>, gl: &WebGlRenderingContext) {
        gl.uniform1f(location, *self);
    }
}
