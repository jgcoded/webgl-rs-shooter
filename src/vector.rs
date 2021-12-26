
pub struct Vec3 {
    data: [f32; 3]
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 {
            data: [x, y, z]
        }
    }

    pub fn x(&self) -> f32 {
        self.data[0]
    }

    pub fn y(&self) -> f32 {
        self.data[1]
    }

    pub fn z(&self) -> f32 {
        self.data[2]
    }

    pub fn data(&self) -> &[f32; 3] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    
    use super::Vec3;

    #[test]
    fn vec_new() {

        let vec = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(vec.x(), 1.0);
        assert_eq!(vec.y(), 2.0);
        assert_eq!(vec.z(), 3.0);
    }
}
