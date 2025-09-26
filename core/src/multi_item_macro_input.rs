use proc_macro2::TokenStream;
use syn::{braced, bracketed, parse2};
use syn::parse::{Parse, ParseStream, Result};
use quote::ToTokens;

use crate::{sanitize, desanitize};

pub struct MultiItemMacroInput <T>
{
	pub brace_token: syn::token::Brace,
	pub items_tokens: TokenStream,
	pub bracket_token: syn::token::Bracket,
	pub user_data: T
}

impl <T> Parse for MultiItemMacroInput <T>
where T: Parse
{
	fn parse (input: ParseStream <'_>) -> Result <Self>
	{
		let content;
		let brace_token = braced! (content in input);
		let sanitized_items_tokens: TokenStream = content . parse ()?;

		let items_tokens = desanitize (sanitized_items_tokens);

		let content;
		let bracket_token = bracketed! (content in input);
		let sanitized_user_data: TokenStream = content . parse ()?;

		let user_data = parse2 (desanitize (sanitized_user_data))?;

		Ok (Self {brace_token, items_tokens, bracket_token, user_data})
	}
}

impl <T> ToTokens for MultiItemMacroInput <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		self . brace_token . surround
		(
			tokens,
			|inner_tokens|
			sanitize (&self . items_tokens) . to_tokens (inner_tokens)
		);
		self . bracket_token . surround
		(
			tokens,
			|inner_tokens|
			sanitize (&self . user_data) . to_tokens (inner_tokens)
		);
	}
}
