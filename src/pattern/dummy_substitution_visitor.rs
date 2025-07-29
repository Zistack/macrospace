use std::convert::Infallible;

use proc_macro2::{TokenStream, Literal, Group, Delimiter};
use proc_macro2::extra::DelimSpan;
use syn::Ident;
use quote::{ToTokens, TokenStreamExt};

use super::{PatternVisitor, PunctGroup, DummyTokens, ParameterCollector, MergeableBindings};

pub struct DummySubstitutionVisitor
{
	tokens: TokenStream
}

impl DummySubstitutionVisitor
{
	pub fn new () -> Self
	{
		Self {tokens: TokenStream::new ()}
	}

	pub fn into_tokens (self) -> TokenStream
	{
		self . tokens
	}
}

impl <P> PatternVisitor <P> for DummySubstitutionVisitor
where P: DummyTokens
{
	type Error = Infallible;
	type SubVisitor = Self;

	fn visit_parameter (&mut self, parameter: P) -> Result <(), Self::Error>
	{
		self . tokens . extend (parameter . dummy_tokens ());

		Ok (())
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

	fn pre_visit_group (&mut self, _delimiter: Delimiter, _group_span: DelimSpan)
	-> Result <Self::SubVisitor, Self::Error>
	{
		Ok (Self::SubVisitor::new ())
	}

	fn post_visit_group
	(
		&mut self, delimiter: Delimiter,
		_group_span: DelimSpan,
		sub_visitor: Self::SubVisitor
	)
	-> Result <(), Self::Error>
	{
		let group_tokens = sub_visitor . tokens;

		self . tokens . append (Group::new (delimiter, group_tokens));

		Ok (())
	}
}

pub struct DummySubstitutionCollectorVisitor <C>
{
	collector: C,
	tokens: TokenStream
}

impl <C> DummySubstitutionCollectorVisitor <C>
{
	pub fn new () -> Self
	where C: Default
	{
		Self {collector: C::default (), tokens: TokenStream::new ()}
	}

	pub fn into_collector_and_tokens (self) -> (C, TokenStream)
	{
		(self . collector, self . tokens)
	}
}

impl <C, P> PatternVisitor <P> for DummySubstitutionCollectorVisitor <C>
where
	C: Default + ParameterCollector <P> + MergeableBindings,
	P: DummyTokens
{
	type Error = C::Error;
	type SubVisitor = Self;

	fn visit_parameter (&mut self, parameter: P) -> Result <(), Self::Error>
	{
		self . tokens . extend (parameter . dummy_tokens ());

		self . collector . add_parameter (parameter);

		Ok (())
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

	fn pre_visit_group (&mut self, _delimiter: Delimiter, _group_span: DelimSpan)
	-> Result <Self::SubVisitor, Self::Error>
	{
		Ok (Self::SubVisitor::new ())
	}

	fn post_visit_group
	(
		&mut self, delimiter: Delimiter,
		_group_span: DelimSpan,
		sub_visitor: Self::SubVisitor
	)
	-> Result <(), Self::Error>
	{
		let group_tokens = sub_visitor . tokens;

		self . tokens . append (Group::new (delimiter, group_tokens));

		self . collector . try_merge (sub_visitor . collector)
	}
}
