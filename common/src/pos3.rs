use bevy_ecs::component::Component;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

#[derive(
    Default,
    serde::Serialize,
    serde::Deserialize,
    Clone,
    Copy,
    Debug,
    Hash,
    Eq,
    PartialEq,
    Component,
)]
pub struct Pos3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[allow(dead_code)]
impl Pos3 {
    pub const ZERO: Pos3 = Pos3 { x: 0, y: 0, z: 0 };
    pub const X: Pos3 = Pos3 { x: 1, y: 0, z: 0 };
    pub const NEG_X: Pos3 = Pos3 { x: -1, y: 0, z: 0 };
    pub const Y: Pos3 = Pos3 { x: 0, y: 1, z: 0 };
    pub const NEG_Y: Pos3 = Pos3 { x: 0, y: -1, z: 0 };
    pub const Z: Pos3 = Pos3 { x: 0, y: 0, z: 1 };
    pub const NEG_Z: Pos3 = Pos3 { x: 0, y: 0, z: -1 };
    pub fn new(x: i32, y: i32, z: i32) -> Pos3 {
        Pos3 { x, y, z }
    }
    pub fn to_string_repr(&self) -> String {
        format!("{};{};{}", self.x, self.y, self.z)
    }
    pub fn from_str_repr(str: &str) -> Option<Self> {
        let nums = str.split(';');
        let mut nums = nums.map(|v| v.parse::<i32>());
        let x = nums.next()?.ok()?;
        let y = nums.next()?.ok()?;
        let z = nums.next()?.ok()?;
        Some(Pos3::new(x, y, z))
    }
}
impl Mul for Pos3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Pos3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}
impl Mul<i32> for Pos3 {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Pos3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}
impl<'a> Add<&'a Pos3> for Pos3 {
    type Output = Self;
    fn add(self, other: &'a Self) -> Self {
        Pos3::new(self.x + other.x, self.y + other.y, self.z + other.z)
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
