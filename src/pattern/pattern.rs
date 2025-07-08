use std::marker::PhantomData;

use proc_macro2::TokenStream;
use syn::buffer::{Cursor, TokenBuffer};
use syn::parse::{Parse, ParseStream, Parser};
use quote::{ToTokens, TokenStreamExt};

use super::{
	PunctGroup,
	MatchBindings,
	MergeableBindings,
	SubstitutionBindings,
	PatternVisitor
};
use super::cursor_parse::CursorParse;
use super::match_visitor::MatchVisitor;
use super::substitution_visitor::SubstitutionVisitor;

pub struct Pattern <P>
{
	pattern_tokens: TokenBuffer,
	_parameter_type: PhantomData <P>
}

impl <P> From <TokenStream> for Pattern <P>
{
	fn from (tokens: TokenStream) -> Self
	{
		Self
		{
			pattern_tokens: TokenBuffer::new2 (tokens),
			_parameter_type: PhantomData::default ()
		}
	}
}

impl <P> Parse for Pattern <P>
{
	fn parse (input: ParseStream <'_>) -> syn::parse::Result <Self>
	{
		Ok (<Self as From <TokenStream>>::from (input . parse ()?))
	}
}

impl <P> ToTokens for Pattern <P>
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		let mut cursor = self . pattern_tokens . begin ();
		while let Some ((token_tree, next_cursor)) = cursor . token_tree ()
		{
			tokens . append (token_tree);
			cursor = next_cursor;
		}
	}
}

impl <P> Pattern <P>
where P: CursorParse
{
	fn visit_pattern_cursor <V> (mut cursor: Cursor <'_>, visitor: &mut V)
	-> Result <(), <V as PatternVisitor <P>>::Error>
	where V: PatternVisitor <P>
	{
		loop
		{
			if let Some ((parameter, next_cursor)) = P::parse_from_cursor (cursor)
			{
				visitor . visit_parameter (parameter)?;

				cursor = next_cursor;

				continue;
			}

			if let Some ((ident, next_cursor)) = cursor . ident ()
			{
				visitor . visit_ident (ident)?;

				cursor = next_cursor;

				continue;
			}

			if let Some ((literal, next_cursor)) = cursor . literal ()
			{
				visitor . visit_literal (literal)?;

				cursor = next_cursor;

				continue;
			}

			if let Some ((punct_group, next_cursor)) =
				PunctGroup::parse_from_cursor (cursor)
			{
				visitor . visit_punct_group (punct_group)?;

				cursor = next_cursor;

				continue;
			}

			if let Some ((group_cursor, delimiter, group_span, next_cursor)) =
				cursor . any_group ()
			{
				let mut sub_visitor = visitor . pre_visit_group
				(
					delimiter,
					group_span
				)?;

				Self::visit_pattern_cursor (group_cursor, &mut sub_visitor)?;

				visitor . post_visit_group
				(
					delimiter,
					group_span,
					sub_visitor
				)?;

				cursor = next_cursor;

				continue;
			}

			if cursor . eof () { return visitor . visit_end (); }

			unreachable! ();
		}
	}

	pub fn visit_pattern <V> (&self, visitor: &mut V)
	-> Result <(), <V as PatternVisitor <P>>::Error>
	where V: PatternVisitor <P>
	{
		let cursor = self . pattern_tokens . begin ();

		Self::visit_pattern_cursor (cursor, visitor)
	}

	pub fn match_input <B> (&self, input: ParseStream <'_>)
	-> syn::parse::Result <B>
	where
		B: Default + MatchBindings <P> + MergeableBindings,
		B::Error: Into <syn::parse::Error>
	{
		let mut match_visitor = MatchVisitor::new (input);

		self . visit_pattern (&mut match_visitor)?;

		Ok (match_visitor . into_bindings ())
	}

	pub fn match_tokens <B> (&self, tokens: TokenStream)
	-> syn::parse::Result <B>
	where
		B: Default + MatchBindings <P> + MergeableBindings,
		B::Error: Into <syn::parse::Error>
	{
		(|input: ParseStream <'_>| { self . match_input (input) . map_err (Into::into) })
			. parse2 (tokens)
	}

	pub fn substitute <B> (&self, bindings: B) -> Result <TokenStream, B::Error>
	where B: SubstitutionBindings <P>
	{
		let mut substitution_visitor = SubstitutionVisitor::new (&bindings);

		self . visit_pattern (&mut substitution_visitor)?;

		Ok (substitution_visitor . into_tokens ())
	}
}
