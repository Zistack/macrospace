use std::fmt::{Display, Formatter, Write};

use proc_macro2::{Punct, TokenStream, Spacing, Span};
use syn::buffer::Cursor;
use syn::parse::{ParseStream, Error};
use quote::{ToTokens, TokenStreamExt};

use super::cursor_parse::CursorParse;

#[derive (Clone, Debug)]
pub struct PunctGroup
{
	puncts: Vec <(char, Span)>
}

impl CursorParse for PunctGroup
{
	fn parse_from_cursor (mut cursor: Cursor <'_>)
	-> Option <(Self, Cursor <'_>)>
	{
		let mut puncts = Vec::new ();

		while let Some ((punct, next_cursor)) = cursor . punct ()
		{
			puncts . push ((punct . as_char (), punct . span ()));

			cursor = next_cursor;

			if let Spacing::Alone = punct . spacing ()
			{
				break;
			}
		}

		Some ((Self {puncts}, cursor))
	}
}

impl PartialEq for PunctGroup
{
	fn eq (&self, other: &Self) -> bool
	{
		if self . puncts . len () != other . puncts . len () { return false; }

		for ((self_char, _), (other_char, _))
		in self . puncts . iter () . zip (other . puncts . iter ())
		{
			if self_char != other_char { return false; }
		}

		true
	}
}

impl Eq for PunctGroup
{
}

impl Display for PunctGroup
{
	fn fmt (&self, f: &mut Formatter <'_>)
	-> std::result::Result <(), std::fmt::Error>
	{
		f . write_char ('`')?;
		for (punct_char, _) in &self . puncts
		{
			f . write_char (*punct_char)?;
		}
		f . write_char ('`')?;

		Ok (())
	}
}

impl ToTokens for PunctGroup
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		let mut punct_iter = self . puncts . iter ();

		let (last_char, last_span) = match punct_iter . next_back ()
		{
			Some ((char, span)) => (char, span),
			None => { return; }
		};

		for (punct_char, punct_span) in &self . puncts
		{
			let mut punct = Punct::new (*punct_char, Spacing::Joint);
			punct . set_span (*punct_span);
			tokens . append (punct);
		}

		let mut punct = Punct::new (*last_char, Spacing::Alone);
		punct . set_span (*last_span);
		tokens . append (punct);
	}
}

impl PunctGroup
{
	pub fn expect_from (&self, input: ParseStream <'_>)
	-> syn::parse::Result <()>
	{
		input . step
		(
			|cursor|
			{
				match PunctGroup::parse_from_cursor (*cursor)
				{
					None => Err
					(
						Error::new
						(
							cursor . span (),
							format! ("expected {}", self)
						)
					),
					Some ((punct_group, cursor)) => if punct_group == *self
					{
						Ok (((), cursor))
					}
					else
					{
						Err
						(
							Error::new_spanned
							(
								punct_group,
								format! ("expected {}", self)
							)
						)
					}
				}
			}
		)
	}
}
