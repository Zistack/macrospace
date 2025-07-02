use syn::{Ident, Token};
use syn::buffer::Cursor;
use syn_derive::{Parse, ToTokens};

use super::cursor_parse::CursorParse;

#[derive (Clone, Debug, Parse, ToTokens)]
pub struct TypedParameter
{
	dollar_token: Token! [$],
	ident: Ident,
	colon_token: Token! [:],
	ty_ident: Ident
}

impl CursorParse for TypedParameter
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

		let (ty_ident, cursor) = match cursor . ident ()
		{
			Some ((ident, cursor)) => (ident, cursor),
			_ => { return None; }
		};

		Some ((Self {dollar_token, ident, colon_token, ty_ident}, cursor))
	}
}

#[derive (Clone, Debug, Parse, ToTokens)]
pub struct UntypedParameter
{
	dollar_token: Token! [$],
	ident: Ident
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
