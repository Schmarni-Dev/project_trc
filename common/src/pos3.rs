use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Default, serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Hash, Eq)]
pub struct Pos3 {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}
#[allow(dead_code)]
impl Pos3 {
    pub const ZERO: Pos3 = Pos3 { x: 0, y: 0, z: 0 };
    pub fn zero() -> Pos3 {
        Pos3::new(0, 0, 0)
    }
    pub fn new(x: i16, y: i16, z: i16) -> Pos3 {
        Pos3 { x: x, y: y, z: z }
    }
    pub fn multiply(&self, vector: Pos3) -> Pos3 {
        Pos3::new(self.x * vector.x, self.y * vector.y, self.z * vector.z)
    }
    pub fn scale(&self, scaler: i16) -> Pos3 {
        Pos3::new(self.x * scaler, self.y * scaler, self.z * scaler)
    }
}

impl PartialEq for Pos3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}
impl Add for Pos3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Pos3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}
impl AddAssign for Pos3 {
    // type Output = Self;
    fn add_assign(&mut self, other: Self) {
        *self = Pos3::new(self.x + other.x, self.y + other.y, self.z + other.z);
    }
}
impl Sub for Pos3 {
    fn sub(self, rhs: Self) -> Self::Output {
        Pos3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
    type Output = Self;
}

impl SubAssign for Pos3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Pos3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
