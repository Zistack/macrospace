use syn::token::{Brace, Bracket};
use syn::parse::Parse;
use syn_derive::{Parse, ToTokens};
use quote::ToTokens;

#[derive (Parse, ToTokens)]
pub struct MultiItemMacroInput <T>
where T: Parse + ToTokens
{
	#[syn (braced)]
	pub brace: Brace,

	#[syn (in = brace)]
	pub items: proc_macro2::TokenStream,

	#[syn (bracketed)]
	pub bracket: Bracket,

	#[syn (in = bracket)]
	pub user_data: T
}
