use std::fmt::{Display, Formatter};

use proc_macro2::TokenStream;
use syn::{Ident, Token};
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream};
use syn_derive::{Parse, ToTokens};
use quote::ToTokens;

use super::cursor_parse::CursorParse;

#[derive (Clone, Debug)]
pub struct TypedParameter <T>
{
	pub dollar_token: syn::token::Dollar,
	pub ident: Ident,
	pub colon_token: syn::token::Colon,
	pub ty: T
}

impl <T> Parse for TypedParameter <T>
where T: Parse
{
	fn parse (input: ParseStream <'_>) -> syn::parse::Result <Self>
	{
		Ok
		(
			Self
			{
				dollar_token: input . parse ()?,
				ident: input . parse ()?,
				colon_token: input . parse ()?,
				ty: input . parse ()?
			}
		)
	}
}

impl <T> ToTokens for TypedParameter <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		self . dollar_token . to_tokens (tokens);
		self . ident . to_tokens (tokens);
		self . colon_token . to_tokens (tokens);
		self . ty . to_tokens (tokens);
	}
}

impl <T> CursorParse for TypedParameter <T>
where T: CursorParse
{
	fn parse_from_cursor (cursor: Cursor <'_>) -> Option <(Self, Cursor <'_>)>
	{
		let (dollar_token, cursor) = match cursor . punct ()
		{
			Some ((punct, cursor)) if punct . as_char () == '$' =>
			(
				syn::token::Dollar {spans: [punct . span ()]},
				cursor
			),
			_ => { return None; }
		};

		let (ident, cursor) = match cursor . ident ()
		{
			Some ((ident, cursor)) => (ident, cursor),
			_ => { return None; }
		};

		let (colon_token, cursor) = match cursor . punct ()
		{
			Some ((punct, cursor)) if punct . as_char () == ':' =>
			(
				syn::token::Colon {spans: [punct . span ()]},
				cursor
			),
			_ => { return None; }
		};

		let (ty, cursor) = match T::parse_from_cursor (cursor)
		{
			Some ((ty, cursor)) => (ty, cursor),
			_ => { return None; }
		};

		Some ((Self {dollar_token, ident, colon_token, ty}, cursor))
	}
}

impl <T> Display for TypedParameter <T>
where T: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt (format_args! ("`${}: {}`", self . ident, self . ty))
	}
}

#[derive (Clone, Debug, Parse, ToTokens)]
pub struct UntypedParameter
{
	pub dollar_token: Token! [$],
	pub ident: Ident
}

impl CursorParse for UntypedParameter
{
	fn parse_from_cursor (cursor: Cursor <'_>) -> Option <(Self, Cursor <'_>)>
	{
		let (dollar_token, cursor) = match cursor . punct ()
		{
			Some ((punct, cursor)) if punct . as_char () == '$' =>
			(
				syn::token::Dollar {spans: [punct . span ()]},
				cursor
			),
			_ => { return None; }
		};

		let (ident, cursor) = match cursor . ident ()
		{
			Some ((ident, cursor)) => (ident, cursor),
			_ => { return None; }
		};

		Some ((Self {dollar_token, ident}, cursor))
	}
}

impl Display for UntypedParameter
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt (format_args! ("`${}`", self . ident))
	}
}
