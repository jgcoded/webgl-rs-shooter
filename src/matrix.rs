use std::ops::Mul;

pub struct Mat4 {
    data: [f32; 16],
}

impl Mat4 {
    pub fn new(values: [f32; 16]) -> Mat4 {
        Mat4 { data: values }
    }

    pub fn identity() -> Mat4 {
        Mat4::new([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let rad_per_degree = std::f32::consts::PI / 180.0f32;
        let fov = fov * rad_per_degree;
        let frustum_scale = 1.0f32 / (fov / 2.0).tan();

        Mat4::new([
            frustum_scale / aspect,
            0f32,
            0f32,
            0f32, // 3
            0f32,
            frustum_scale,
            0f32,
            0f32, // 7
            0f32,
            0f32,
            (far + near) / (near - far),
            -1.0f32, // 11
            0f32,
            0f32,
            (2.0f32 * far * near) / (near - far),
            0f32, // 15
        ])
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Mat4 {
        Mat4::new([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
        ])
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Mat4 {
        Mat4::new([
            x, 0.0, 0.0, 0.0, 0.0, y, 0.0, 0.0, 0.0, 0.0, z, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn rotate(x: f32, y: f32, z: f32) -> Mat4 {

        // TODO optimize by using assignments instead of matrix multiplications

        let mut rot_x = Mat4::identity();
        let mut rot_y = Mat4::identity();
        let mut rot_z = Mat4::identity();

        let rad_per_degree = 2.0f32*std::f32::consts::PI / 360.0f32;

        let rad_x = x*rad_per_degree;
        let rad_y = y*rad_per_degree;
        let rad_z = z*rad_per_degree;

        rot_x.data[5] = rad_x.cos();
        rot_x.data[6] = rad_x.sin();
        rot_x.data[9] = -rad_x.sin();
        rot_x.data[10] = rad_x.cos();

        rot_y.data[0] = rad_y.cos();
        rot_y.data[2] = -rad_y.sin();
        rot_y.data[8] = rad_y.sin();
        rot_y.data[10] = rad_y.cos();

        rot_z.data[0] = rad_z.cos();
        rot_z.data[1] = rad_z.sin();
        rot_z.data[4] = -rad_z.sin();
        rot_z.data[5] = rad_z.cos();

        rot_x * rot_y * rot_z
    }

    pub fn data(&self) -> &[f32; 16] {
        &self.data
    }
}

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, right: Self) -> Self {
        let mut result = Mat4::new([0.0; 16]);

        // first row
        result.data[0] = self.data[0] * right.data[0]
            + self.data[1] * right.data[4]
            + self.data[2] * right.data[8]
            + self.data[3] * right.data[12];

        result.data[1] = self.data[0] * right.data[1]
            + self.data[1] * right.data[5]
            + self.data[2] * right.data[9]
            + self.data[3] * right.data[13];

        result.data[2] = self.data[0] * right.data[2]
            + self.data[1] * right.data[6]
            + self.data[2] * right.data[10]
            + self.data[3] * right.data[14];

        result.data[3] = self.data[0] * right.data[3]
            + self.data[1] * right.data[7]
            + self.data[2] * right.data[11]
            + self.data[3] * right.data[15];

        // second row

        result.data[4] = self.data[4] * right.data[0]
            + self.data[5] * right.data[4]
            + self.data[6] * right.data[8]
            + self.data[7] * right.data[12];

        result.data[5] = self.data[4] * right.data[1]
            + self.data[5] * right.data[5]
            + self.data[6] * right.data[9]
            + self.data[7] * right.data[13];

        result.data[6] = self.data[4] * right.data[2]
            + self.data[5] * right.data[6]
            + self.data[6] * right.data[10]
            + self.data[7] * right.data[14];

        result.data[7] = self.data[4] * right.data[3]
            + self.data[5] * right.data[7]
            + self.data[6] * right.data[11]
            + self.data[7] * right.data[15];

        // third row
        result.data[8] = self.data[8] * right.data[0]
            + self.data[9] * right.data[4]
            + self.data[10] * right.data[8]
            + self.data[11] * right.data[12];

        result.data[9] = self.data[8] * right.data[1]
            + self.data[9] * right.data[5]
            + self.data[10] * right.data[9]
            + self.data[11] * right.data[13];

        result.data[10] = self.data[8] * right.data[2]
            + self.data[9] * right.data[6]
            + self.data[10] * right.data[10]
            + self.data[11] * right.data[14];

        result.data[11] = self.data[8] * right.data[3]
            + self.data[9] * right.data[7]
            + self.data[10] * right.data[11]
            + self.data[11] * right.data[15];

        // fourth row
        result.data[12] = self.data[12] * right.data[0]
            + self.data[13] * right.data[4]
            + self.data[14] * right.data[8]
            + self.data[15] * right.data[12];

        result.data[13] = self.data[12] * right.data[1]
            + self.data[13] * right.data[5]
            + self.data[14] * right.data[9]
            + self.data[15] * right.data[13];

        result.data[14] = self.data[12] * right.data[2]
            + self.data[13] * right.data[6]
            + self.data[14] * right.data[10]
            + self.data[15] * right.data[14];

        result.data[15] = self.data[12] * right.data[3]
            + self.data[13] * right.data[7]
            + self.data[14] * right.data[11]
            + self.data[15] * right.data[15];

        result
    }
}

#[cfg(test)]
mod tests {

    use super::Mat4;

    fn threshold_assert(left: f32, right: f32) {
        assert!(
            (left - right).abs() < 0.000001f32,
            "{:?} and {:?} are not within threshold",
            left,
            right
        );
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
    fn scale_test() {
        let mat = Mat4::scale(2.0f32, 3.0f32, 4.0f32);
        let data = mat.data();
        threshold_assert(2.0f32, data[0]);
        threshold_assert(0.0f32, data[1]);
        threshold_assert(0.0f32, data[2]);
        threshold_assert(0.0f32, data[3]);

        threshold_assert(0.0f32, data[4]);
        threshold_assert(3.0f32, data[5]);
        threshold_assert(0.0f32, data[6]);
        threshold_assert(0.0f32, data[7]);

        threshold_assert(0.0f32, data[8]);
        threshold_assert(0.0f32, data[9]);
        threshold_assert(4.0f32, data[10]);
        threshold_assert(0.0f32, data[11]);

        threshold_assert(0.0f32, data[12]);
        threshold_assert(0.0f32, data[13]);
        threshold_assert(0.0f32, data[14]);
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

    #[test]
    fn matmul_test() {
        let left = Mat4::identity();
        let right = Mat4::identity();

        println!("left:\t {:?}", left.data());
        println!("right:\t {:?}", right.data());

        let result = left * right;
        let data = result.data();


        println!("result:\t {:?}", data);

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

        threshold_assert(0.0f32, data[12]);
        threshold_assert(0.0f32, data[13]);
        threshold_assert(0.0f32, data[14]);
        threshold_assert(1.0f32, data[15]);
    }

    #[test]
    fn identity_test() {
        let mat = Mat4::identity();
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

        threshold_assert(0.0f32, data[12]);
        threshold_assert(0.0f32, data[13]);
        threshold_assert(0.0f32, data[14]);
        threshold_assert(1.0f32, data[15]);
    }
}
