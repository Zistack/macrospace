use proc_macro2::{TokenStream, Delimiter, Group};
use proc_macro2::extra::DelimSpan;
use syn::{Ident, parse2};
use syn::parse::{Parse, ParseStream};
use quote::{ToTokens, TokenStreamExt};

use super::{
	NoParameterInRepetition,
	StructuredBindingView,
	IndexBindings,
	VisitationError,
	SpecializationError,
	PatternBuffer,
	PatternVisitor,
	TokenizeBinding
};

#[derive (Clone, Debug)]
pub struct GroupPattern <T>
{
	pub delimiter: Delimiter,
	pub delim_span: DelimSpan,
	pub inner_pattern: PatternBuffer <T>
}

impl <T> Parse for GroupPattern <T>
where T: Parse
{
	fn parse (input: ParseStream <'_>) -> syn::Result <Self>
	{
		Ok (Self::try_from (input . parse::<Group> ()?)?)
	}
}

impl <T> GroupPattern <T>
{
	pub fn referenced_identifiers (&self) -> impl Iterator <Item = &Ident>
	{
		self . inner_pattern . referenced_identifiers ()
	}

	pub fn validate (&self) -> Result <(), NoParameterInRepetition <T>>
	where T: Clone
	{
		self . inner_pattern . validate ()
	}

	pub fn visit <V> (&self, index_bindings: &IndexBindings, visitor: &mut V)
	-> Result <(), VisitationError <V::Error>>
	where V: PatternVisitor <T>
	{
		let mut group_visitor = visitor . pre_visit_group
		(
			self . delimiter,
			self . delim_span
		)
			. map_err (VisitationError::Visitor)?;

		self . inner_pattern . visit (index_bindings, &mut group_visitor)?;

		visitor . post_visit_group
		(
			self . delimiter,
			self . delim_span, group_visitor
		)
			. map_err (VisitationError::Visitor)?;

		Ok (())
	}

	pub fn specialize <'a, V>
	(
		&self,
		index_bindings: &IndexBindings,
		bindings: &StructuredBindingView <'a, V>,
		pattern_buffer: &mut PatternBuffer <T>
	)
	-> Result <(), SpecializationError <T::Error>>
	where T: Clone + Parse + TokenizeBinding <V>
	{
		let delimiter = self . delimiter;
		let delim_span = self . delim_span;
		let mut inner_pattern = PatternBuffer::new ();

		self . inner_pattern . specialize (index_bindings, bindings, &mut inner_pattern)?;

		pattern_buffer . append_group
		(
			Self {delimiter, delim_span, inner_pattern}
		);

		Ok (())
	}
}

impl <T> ToTokens for GroupPattern <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		let group = Group::new
		(
			self . delimiter,
			self . inner_pattern . to_token_stream ()
		);

		tokens . append (group);
	}
}

impl <T> TryFrom <Group> for GroupPattern <T>
where T: Parse
{
	type Error = syn::Error;

	fn try_from (group: Group) -> Result <Self, Self::Error>
	{
		let delimiter = group . delimiter ();
		let delim_span = group . delim_span ();
		let inner_pattern = parse2 (group . stream ())?;

		Ok (Self {delimiter, delim_span, inner_pattern})
	}
}
