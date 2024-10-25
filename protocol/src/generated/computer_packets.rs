use crate::generated::types::Item;
use common::Pos3;
use common::turtle::MoveDirection;
use common::turtle::Orientation;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ComputerSetupToServerPacket {
	pub world: String,
	pub id: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum TurtleToServerPacket {
	SetMaxFuel(
		i32,
	),
	SetPos(
		Pos3,
	),
	SetOrientation(
		Orientation,
	),
	Moved(
		MoveDirection,
	),
	UpdateFuel(
		i32,
	),
	UpdateBlocks {
		up: Option<String>,
		down: Option<String>,
		front: Option<String>,
	},
	UpdateSlotContents {
		slot: u8,
		contents: Option<Item>,
	},
	ComputerToServer(
		ComputerToServerPacket,
	),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ComputerToServerPacket {
	StdOut(
		String,
	),
	Executables(
		Vec<String>,
	),
	UpdateName(
		String,
	),
	ChangeWorld(
		String,
	),
	Ping,
}
