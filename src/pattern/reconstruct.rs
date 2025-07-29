use proc_macro2::{TokenStream, Group, Delimiter};
use syn::buffer::{TokenBuffer, Cursor};
use syn::parse::{Result, Error};
use quote::{ToTokens, TokenStreamExt};

use super::{
	CursorParse,
	DummyTokens,
	PunctGroup,
	expect_tokens_from_cursor
};

fn reconstruct_step <'a, 'b, P>
(
	parsed_input_cursor: Cursor <'a>,
	orig_pattern_cursor: Cursor <'b>,
	output_tokens: &mut TokenStream
)
-> Result <(Cursor <'a>, Cursor <'b>)>
where P: CursorParse + DummyTokens + ToTokens
{
	if let Some ((parameter, next_pattern_cursor)) =
		P::parse_from_cursor (orig_pattern_cursor)
	{
		let next_parsed_cursor = expect_tokens_from_cursor
		(
			parsed_input_cursor,
			parameter . dummy_tokens ()
		)?;

		parameter . to_tokens (output_tokens);

		return Ok ((next_parsed_cursor, next_pattern_cursor));
	}

	// If I used match-based logic like with the expect code, I could do things
	// slightly differently here and avoid the call to unreachable! ();

	if let Some ((parsed_ident, next_parsed_cursor)) =
		parsed_input_cursor . ident ()
	{
		if let Some ((orig_ident, next_pattern_cursor)) =
			orig_pattern_cursor . ident ()
		{
			if parsed_ident == orig_ident
			{
				output_tokens . append (parsed_ident);
				return Ok ((next_parsed_cursor, next_pattern_cursor))
			}
		}

		return Err
		(
			Error::new
			(
				orig_pattern_cursor . span (),
				format! ("expected `{}`", parsed_ident)
			)
		);
	}

	if let Some ((parsed_literal, next_parsed_cursor)) =
		parsed_input_cursor . literal ()
	{
		if let Some ((orig_literal, next_pattern_cursor)) =
			orig_pattern_cursor . literal ()
		{
			if parsed_literal . to_string () == orig_literal . to_string ()
			{
				output_tokens . append (parsed_literal);
				return Ok ((next_parsed_cursor, next_pattern_cursor))
			}
		}

		return Err
		(
			Error::new
			(
				orig_pattern_cursor . span (),
				format! ("expected `{}`", parsed_literal)
			)
		);
	}

	if let Some ((parsed_punct, next_parsed_cursor)) =
		PunctGroup::parse_from_cursor (parsed_input_cursor)
	{
		if let Some ((orig_punct, next_pattern_cursor)) =
			PunctGroup::parse_from_cursor (orig_pattern_cursor)
		{
			if parsed_punct == orig_punct
			{
				parsed_punct . to_tokens (output_tokens);
				return Ok ((next_parsed_cursor, next_pattern_cursor))
			}
		}

		return Err
		(
			Error::new
			(
				orig_pattern_cursor . span (),
				format! ("expected `{}`", parsed_punct)
			)
		);
	}

	if let Some ((parsed_group_cursor, parsed_delimiter, parsed_group_span, next_parsed_cursor)) =
		parsed_input_cursor . any_group ()
	{
		if let Some ((orig_group_cursor, orig_delimiter, _, next_orig_cursor)) =
			orig_pattern_cursor . any_group ()
		{
			if parsed_delimiter == orig_delimiter
			{
				let output_group_tokens = reconstruct_stream::<P>
				(
					parsed_group_cursor,
					orig_group_cursor
				)?;

				let mut output_group = Group::new (parsed_delimiter, output_group_tokens);
				output_group . set_span (parsed_group_span . join ());

				output_tokens . append (output_group);

				return Ok ((next_parsed_cursor, next_orig_cursor))
			}
		}

		let expected_char = match parsed_delimiter
		{
			Delimiter::Parenthesis => "(",
			Delimiter::Brace => "{",
			Delimiter::Bracket => "[",
			Delimiter::None => "∅"
		};

		return Err
		(
			Error::new
			(
				orig_pattern_cursor . span (),
				format! ("expected `{}`", expected_char)
			)
		);
	}

	unreachable! ();
}

fn reconstruct_stream <'a, 'b, P>
(
	mut parsed_input_cursor: Cursor <'a>,
	mut orig_pattern_cursor: Cursor <'b>
)
-> Result <TokenStream>
where P: CursorParse + DummyTokens + ToTokens
{
	let mut output_tokens = TokenStream::new ();

	loop
	{
		if parsed_input_cursor . eof ()
		{
			if orig_pattern_cursor . eof ()
			{
				return Ok (output_tokens);
			}
			else
			{
				return Err
				(
					Error::new
					(
						orig_pattern_cursor . span (),
						"expected end of stream"
					)
				)
			}
		}

		(parsed_input_cursor, orig_pattern_cursor) = reconstruct_step::<P>
		(
			parsed_input_cursor,
			orig_pattern_cursor,
			&mut output_tokens
		)?;
	}
}

pub fn reconstruct_pattern_tokens <P>
(
	parsed_tokens: TokenStream,
	orig_pattern_tokens: &TokenBuffer
)
-> Result <TokenStream>
where P: CursorParse + DummyTokens + ToTokens
{
	let parsed_tokens = TokenBuffer::new2 (parsed_tokens);

	reconstruct_stream::<P>
	(
		parsed_tokens . begin (),
		orig_pattern_tokens . begin ()
	)
}
