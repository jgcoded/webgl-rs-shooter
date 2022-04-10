use std::{
    ops::{Add, AddAssign, Sub},
};

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

    pub fn scaled(&self, u: f32) -> Vec3 {
        Vec3 {
            data: [u * self.data[0], u * self.data[1], u * self.data[2]],
        }
    }

    pub fn length(&self) -> f32 {
        (self.data[0] * self.data[0] + self.data[1] * self.data[1] + self.data[2] * self.data[2])
            .sqrt()
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            data: [
                self.data[0] + other.data[0],
                self.data[1] + other.data[1],
                self.data[2] + other.data[2],
            ],
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        self.data[0] += other.data[0];
        self.data[1] += other.data[1];
        self.data[2] += other.data[2];
    }
}

impl Copy for Vec3 {}

impl Clone for Vec3 {
    fn clone(&self) -> Vec3 {
        Self {
            data: [self.data[0], self.data[1], self.data[2]],
        }
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
    fn add() {
        let left = Vec3::new(1.0, 2.0, 3.0);
        let right = Vec3::new(1.0, 2.0, 3.0);
        let result = left + right;
        assert_eq!(2f32, result.x());
        assert_eq!(4f32, result.y());
        assert_eq!(6f32, result.z());
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
}
