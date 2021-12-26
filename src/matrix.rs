
pub struct Mat4 {
    data: [f32; 16]
}

impl Mat4 {

    pub fn new(values: [f32; 16]) -> Mat4 {
        Mat4 {
            data: values
        }
    }

    pub fn perspective(
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32
    ) -> Mat4 {
        let rad_per_degree = std::f32::consts::PI / 180.0f32;
        let fov = fov * rad_per_degree;
        let frustum_scale = 1.0f32 / (fov / 2.0).tan();

        Mat4::new([
            frustum_scale / aspect, 0f32, 0f32, 0f32,          // 3
            0f32, frustum_scale, 0f32, 0f32,                   // 7
            0f32, 0f32, (far + near) / (near - far), -1.0f32,  // 11
            0f32, 0f32, (2.0f32*far*near) / (near - far), 0f32 // 15

        ])
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Mat4 {
        Mat4::new([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
              x,   y,   z, 1.0
        ])
    }

    pub fn identity() -> Mat4 {
        Mat4::new([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ])
    }

    pub fn data(&self) -> &[f32; 16] {
        &self.data
    }
}

#[cfg(test)]
mod tests {

    use super::Mat4;

    fn threshold_assert(left: f32, right: f32) {
        assert!((left - right).abs() < 0.000001f32, "{:?} and {:?} are not within threshold", left, right);
    }

    #[test]
    fn translation_test() {
        let mat = Mat4::translation(1.0f32, 2.0f32, 3.0f32);
        let data = mat.data();
        threshold_assert(1.0f32, data[0]);
        threshold_assert(0.0f32, data[1]);
        threshold_assert(0.0f32, data[2]);
        threshold_assert(0.0f32, data[3]);

        threshold_assert(0.0f32, data[4]);
        threshold_assert(1.0f32, data[5]);
        threshold_assert(0.0f32, data[6]);
        threshold_assert(0.0f32, data[7]);

        threshold_assert(0.0f32, data[8]);
        threshold_assert(0.0f32, data[9]);
        threshold_assert(1.0f32, data[10]);
        threshold_assert(0.0f32, data[11]);

        threshold_assert(1.0f32, data[12]);
        threshold_assert(2.0f32, data[13]);
        threshold_assert(3.0f32, data[14]);
        threshold_assert(1.0f32, data[15]);
    }

    #[test]
    fn perspective_test() {
        let mat = Mat4::perspective(45.0f32, 640.0f32 / 480.0f32, 0.1f32, 100.0f32);
        let data = mat.data();
        threshold_assert(1.8106601238250732f32, data[0]);
        threshold_assert(0.0f32, data[1]);
        threshold_assert(0.0f32, data[2]);
        threshold_assert(0.0f32, data[3]);

        threshold_assert(0.0f32, data[4]);
        threshold_assert(2.4142136573791504f32, data[5]);
        threshold_assert(0.0f32, data[6]);
        threshold_assert(0.0f32, data[7]);

        threshold_assert(0.0f32, data[8]);
        threshold_assert(0.0f32, data[9]);
        threshold_assert(-1.0020020008087158f32, data[10]);
        threshold_assert(-1.0f32, data[11]);

        threshold_assert(0.0f32, data[12]);
        threshold_assert(0.0f32, data[13]);
        threshold_assert(-0.20020020008087158f32, data[14]);
        threshold_assert(0.0f32, data[15]);
    }
}