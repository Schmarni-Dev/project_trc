
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Item {
	pub amount: u8,
	pub max_stack_size: u8,
	pub item_id: String,
	pub item_name: String,
}
