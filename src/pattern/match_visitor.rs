use std::borrow::Borrow;
use std::marker::PhantomData;

use proc_macro2::{Literal, Delimiter};
use proc_macro2::extra::DelimSpan;
use syn::{Ident, parenthesized, braced, bracketed};
use syn::parse::{ParseBuffer, Error};

use super::{PunctGroup, PatternVisitor, PatternBindings};

pub struct MatchVisitor <'a, S, B>
{
	input: S,
	input_lifetime: PhantomData <&'a S>,
	bindings: B
}

impl <'a, S, B> MatchVisitor <'a, S, B>
{
	pub fn new (input: S) -> Self
	where B: Default
	{
		Self
		{
			input,
			input_lifetime: PhantomData::default (),
			bindings: B::default ()
		}
	}

	pub fn into_bindings (self) -> B
	{
		self . bindings
	}
}

impl <'a, S, B, P> PatternVisitor <P> for MatchVisitor <'a, S, B>
where
	S: Borrow <ParseBuffer <'a>>,
	B: Default + PatternBindings <P, Error = Error>
{
	type Error = Error;
	type SubVisitor = MatchVisitor <'a, ParseBuffer <'a>, B>;

	fn visit_parameter (&mut self, parameter: P) -> Result <(), Self::Error>
	{
		self . bindings . parse_parameter_binding (parameter, self . input . borrow ())
	}

	fn visit_ident (&mut self, ident: Ident) -> Result <(), Self::Error>
	{
		let input_ident: Ident = self . input . borrow () . parse ()?;

		if input_ident != ident
		{
			return Err
			(
				Error::new_spanned (input_ident, format! ("expected {}", ident))
			);
		}

		Ok (())
	}

	fn visit_literal (&mut self, literal: Literal) -> Result <(), Self::Error>
	{
		let input_literal: Literal = self . input . borrow () . parse ()?;

		if input_literal . to_string () != literal . to_string ()
		{
			return Err
			(
				Error::new_spanned
				(
					input_literal,
					format! ("expected {}", literal)
				)
			);
		}

		Ok (())
	}

	fn visit_punct_group (&mut self, punct_group: PunctGroup)
	-> Result <(), Self::Error>
	{
		punct_group . expect_from (self . input . borrow ())
	}

	fn pre_visit_group
	(
		&mut self,
		delimiter: Delimiter,
		group_span: DelimSpan
	)
	-> Result <Self::SubVisitor, Self::Error>
	{
		let content;

		match delimiter
		{
			Delimiter::Parenthesis =>
			{
				parenthesized! (content in self . input . borrow ());
			},
			Delimiter::Brace =>
			{
				braced! (content in self . input . borrow ());
			},
			Delimiter::Bracket =>
			{
				bracketed! (content in self . input . borrow ());
			},
			Delimiter::None => return Err
			(
				Error::new
				(
					group_span . join (),
					"undelimited groups are not supported"
				)
			)
		}

		Ok (Self::SubVisitor::new (content))
	}

	fn post_visit_group
	(
		&mut self,
		_delimiter: Delimiter,
		_group_span: DelimSpan,
		sub_visitor: Self::SubVisitor
	)
	-> Result <(), Self::Error>
	{
		self . bindings . try_merge (sub_visitor . bindings)
	}

	fn visit_end (&mut self) -> Result <(), Self::Error>
	{
		if ! self . input . borrow () . is_empty ()
		{
			return Err
			(
				self . input . borrow () . error ("expected end of input")
			);
		}

		Ok (())
	}
}
