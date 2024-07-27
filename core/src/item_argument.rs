use syn::{Path, Token};
use syn_derive::{Parse, ToTokens};

use crate::ItemTypeSpec;

#[derive (Parse, ToTokens)]
pub struct ItemArgument
{
	pub path: Path,
	pub colon_token: Token! [:],
	pub type_spec: ItemTypeSpec
}
