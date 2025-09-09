use std::fmt::{Display, Formatter};

use proc_macro2::TokenStream;
use syn::Ident;
use syn::parse::{Parse, ParseStream};
use quote::ToTokens;

#[derive (Clone, Debug, PartialEq, Eq, Hash)]
pub struct Parameter <T>
{
	pub dollar_token: syn::token::Dollar,
	pub ident: Ident,
	pub extra_tokens: T
}

impl <T> Parse for Parameter <T>
where T: Parse
{
	fn parse (input: ParseStream <'_>) -> syn::Result <Self>
	{
		let dollar_token = input . parse ()?;
		let ident = input . parse ()?;
		let extra_tokens = input . parse ()?;

		Ok (Self {dollar_token, ident, extra_tokens})
	}
}

impl <T> ToTokens for Parameter <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		self . dollar_token . to_tokens (tokens);
		self . ident . to_tokens (tokens);
		self . extra_tokens . to_tokens (tokens);
	}
}

impl <T> Display for Parameter <T>
where T: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt (format_args! ("${} {}", self . ident, self . extra_tokens))
	}
}
