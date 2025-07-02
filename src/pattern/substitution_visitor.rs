use proc_macro2::{Literal, Delimiter, Group, TokenStream};
use proc_macro2::extra::DelimSpan;
use syn::Ident;
use quote::{ToTokens, TokenStreamExt};

use super::{PunctGroup, PatternVisitor, PatternBindings};

pub struct SubstitutionVisitor <'a, B>
{
	bindings: &'a B,
	tokens: TokenStream
}

impl <'a, B> SubstitutionVisitor <'a, B>
{
	pub fn new (bindings: &'a B) -> Self
	{
		Self {bindings, tokens: TokenStream::new ()}
	}

	pub fn into_tokens (self) -> TokenStream
	{
		self . tokens
	}
}

impl <'a, B, P> PatternVisitor <P> for SubstitutionVisitor <'a, B>
where B: PatternBindings <P>
{
	type Error = B::Error;
	type SubVisitor = SubstitutionVisitor <'a, B>;

	fn visit_parameter (&mut self, parameter: P) -> Result <(), Self::Error>
	{
		self . bindings . write_parameter_tokens (parameter, &mut self . tokens)
	}

	fn visit_ident (&mut self, ident: Ident) -> Result <(), Self::Error>
	{
		ident . to_tokens (&mut self . tokens);

		Ok (())
	}

	fn visit_literal (&mut self, literal: Literal) -> Result <(), Self::Error>
	{
		self . tokens . append (literal);

		Ok (())
	}

	fn visit_punct_group (&mut self, punct_group: PunctGroup)
	-> Result <(), Self::Error>
	{
		punct_group . to_tokens (&mut self . tokens);

		Ok (())
	}

	fn pre_visit_group
	(
		&mut self,
		_delimiter: Delimiter,
		_group_span: DelimSpan
	)
	-> Result <Self::SubVisitor, Self::Error>
	{
		Ok (Self::SubVisitor::new (self . bindings))
	}

	fn post_visit_group
	(
		&mut self,
		delimiter: Delimiter,
		_group_span: DelimSpan,
		sub_visitor: Self::SubVisitor
	)
	-> Result <(), Self::Error>
	{
		let group_tokens = sub_visitor . tokens;

		self . tokens . append (Group::new (delimiter, group_tokens));

		Ok (())
	}

	fn visit_end (&mut self) -> Result <(), Self::Error>
	{
		Ok (())
	}
}
