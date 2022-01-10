use std::ops::Sub;

#[derive(Debug)]
pub struct Vec3 {
    data: [f32; 3],
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { data: [x, y, z] }
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

    pub fn magnitude(&self) -> f32 {
        (self.data[0] * self.data[0] + self.data[1] * self.data[1] + self.data[2] * self.data[2])
            .sqrt()
    }

    pub fn normalized(&self) -> Vec3 {
        let magnitude = self.magnitude();
        Vec3 {
            data: [
                self.data[0] / magnitude,
                self.data[1] / magnitude,
                self.data[2] / magnitude,
            ],
        }
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 { data:[
            self.data[1]*other.data[2] - self.data[2]*other.data[1],
            self.data[2]*other.data[0] - self.data[0]*other.data[2],
            self.data[0]*other.data[1] - self.data[1]*other.data[0]
        ] }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            data: [
                self.data[0] - other.data[0],
                self.data[1] - other.data[1],
                self.data[2] - other.data[2],
            ],
        }
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

    #[test]
    fn sub() {
        let left = Vec3::new(1.0, 2.0, 3.0);
        let right = Vec3::new(1.0, 2.0, 3.0);
        let result = left - right;
        assert_eq!(0f32, result.x());
        assert_eq!(0f32, result.y());
        assert_eq!(0f32, result.z());
    }

    #[test]
    fn magnitude() {
        let vec = Vec3::new(3.0, 4.0, 5.0);
        assert_eq!(7.0710678118654755, vec.magnitude())
    }

    #[test]
    fn normalize() {
        let vec = Vec3::new(3.0, 4.0, 5.0);
        let normalized = vec.normalized();
        assert_eq!(1.0, normalized.magnitude());
        assert_eq!(0.4242640687119285, normalized.x());
        assert_eq!(0.565685424949238, normalized.y());
        assert_eq!(0.7071067811865475, normalized.z());
    }

    fn assert_componentwise_equality(left: &Vec3, right: &Vec3) {
        println!("Left: {:?}", left);
        println!("right: {:?}", right);

        assert_eq!(left.data[0], right.data[0]);
        assert_eq!(left.data[1], right.data[1]);
        assert_eq!(left.data[2], right.data[2]);
    }

    #[test]
    fn cross() {
        let i = Vec3::new(1.0, 0.0, 0.0);
        let j = Vec3::new(0.0, 1.0, 0.0);
        let k = Vec3::new(0.0, 0.0, 1.0);

        assert_componentwise_equality(&i.cross(&j), &k);
        assert_componentwise_equality(&j.cross(&k), &i);
        assert_componentwise_equality(&k.cross(&i), &j);
    }
}
