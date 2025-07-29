use proc_macro2::{TokenStream, TokenTree, Delimiter};
use syn::buffer::Cursor;
use syn::parse::{Parser, ParseStream, Result, Error};

fn assert_token_tree
(
	input_token_tree: TokenTree,
	expected_token_tree: TokenTree
)
-> Result <()>
{
	match (input_token_tree, expected_token_tree)
	{
		(TokenTree::Group (input_group), TokenTree::Group (expected_group)) =>
		{
			let input_delimiter = input_group . delimiter ();
			let expected_delimiter = expected_group . delimiter ();

			if input_delimiter != expected_delimiter
			{
				let expected_char = match expected_delimiter
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
						input_group . span_open (),
						format! ("expected `{}`", expected_char)
					)
				);
			}

			let parser = |input: ParseStream <'_>|
			{
				expect_tokens (input, expected_group . stream ())
			};

			return parser . parse2 (input_group . stream ());
		},
		(TokenTree::Ident (input_ident), TokenTree::Ident (expected_ident)) =>
		{
			if input_ident != expected_ident
			{
				return Err
				(
					Error::new_spanned
					(
						input_ident,
						format! ("expected `{}`", expected_ident)
					)
				);
			}
		},
		(TokenTree::Punct (input_punct), TokenTree::Punct (expected_punct)) =>
		{
			if input_punct . as_char () != expected_punct . as_char ()
			{
				return Err
				(
					Error::new_spanned
					(
						input_punct,
						format! ("expected `{}`", expected_punct)
					)
				);
			}
		},
		(TokenTree::Literal (input_literal), TokenTree::Literal (expected_literal)) =>
		{
			if input_literal . to_string () != expected_literal . to_string ()
			{
				return Err
				(
					Error::new_spanned
					(
						input_literal,
						format! ("expected `{}`", expected_literal)
					)
				);
			}
		},
		(input @ _, TokenTree::Group (expected_group)) =>
		{
			let expected_char = match expected_group . delimiter ()
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
					input . span (),
					format! ("expected `{}`", expected_char)
				)
			);
		},
		(input @ _, expected @ _) =>
		{
			return Err
			(
				Error::new
				(
					input . span (),
					format! ("expected `{}`", expected)
				)
			);
		}
	}

	Ok (())
}

pub fn expect_token_tree
(
	input: ParseStream <'_>,
	expected_token_tree: TokenTree
)
-> Result <()>
{
	let input_token_tree = input . parse ()?;

	assert_token_tree (input_token_tree, expected_token_tree)
}

pub fn expect_tokens (input: ParseStream <'_>, expected_tokens: TokenStream)
-> Result <()>
{
	for expected_token_tree in expected_tokens
	{
		expect_token_tree (input, expected_token_tree)?;
	}

	Ok (())
}

pub fn expect_token_tree_from_cursor
(
	input_cursor: Cursor <'_>,
	expected_token_tree: TokenTree
)
-> Result <Cursor <'_>>
{
	if let Some ((input_token_tree, next_input_cursor)) =
		input_cursor . token_tree ()
	{
		assert_token_tree (input_token_tree, expected_token_tree)?;

		Ok (next_input_cursor)
	}
	else
	{
		Err
		(
			Error::new
			(
				input_cursor . span (),
				format! ("expected `{}`", expected_token_tree)
			)
		)
	}
}

pub fn expect_tokens_from_cursor
(
	mut input_cursor: Cursor <'_>,
	expected_tokens: TokenStream
)
-> Result <Cursor <'_>>
{
	for expected_token_tree in expected_tokens
	{
		input_cursor = expect_token_tree_from_cursor
		(
			input_cursor,
			expected_token_tree
		)?;
	}

	Ok (input_cursor)
}
