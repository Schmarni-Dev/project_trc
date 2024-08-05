
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Pos3 {
	pub x: u32,
	pub y: u32,
	pub z: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Quat {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub w: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct NewQuat(
	pub Quat,
);

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Alive;

pub enum Live {
	Alive {
		x: u32,
		y: u32,
		hellow_wowld: String,
	},
	Dying(
		f32,
	),
	Dead,
}
