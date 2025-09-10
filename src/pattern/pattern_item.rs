use proc_macro2::{TokenStream, TokenTree, Punct, Literal};
use syn::{Ident, Token};
use syn::parse::{Parse, ParseStream};
use quote::ToTokens;

use super::{
	Parameter,
	StructuredBindingView,
	StructuredBindingTypeMismatch,
	OptionalPattern,
	ZeroOrMorePattern,
	OneOrMorePattern,
	RepetitionPattern,
	GroupPattern,
	PatternBuffer,
	PatternVisitor
};

#[derive (Clone, Debug)]
pub enum PatternItem <T>
{
	Parameter (Parameter <T>),
	Optional (OptionalPattern <T>),
	ZeroOrMore (ZeroOrMorePattern <T>),
	OneOrMore (OneOrMorePattern <T>),
	Group (GroupPattern <T>),
	Ident (Ident),
	Punct (Punct),
	Literal (Literal)
}

impl <T> Parse for PatternItem <T>
where T: Parse
{
	fn parse (input: ParseStream <'_>) -> syn::Result <Self>
	{
		if input . peek (Token! [$])
		{
			if input . peek2 (Ident)
			{
				Ok (Self::Parameter (input . parse ()?))
			}
			else if input . peek2 (syn::token::Paren)
			{
				let repetition: RepetitionPattern <T> = input . parse ()?;

				match repetition
				{
					RepetitionPattern::Optional (optional_pattern) =>
						Ok (Self::Optional (optional_pattern)),
					RepetitionPattern::ZeroOrMore (zero_or_more_pattern) =>
						Ok (Self::ZeroOrMore (zero_or_more_pattern)),
					RepetitionPattern::OneOrMore (one_or_more_pattern) =>
						Ok (Self::OneOrMore (one_or_more_pattern))
				}
			}
			else
			{
				Err
				(
					syn::Error::new
					(
						input . span (),
						"Expected `$` to be followed by either a parameter or a repetition pattern"
					)
				)
			}
		}
		else
		{
			match input . parse ()?
			{
				TokenTree::Group (group) =>
					Ok (Self::Group (GroupPattern::try_from (group)?)),
				TokenTree::Ident (ident) => Ok (Self::Ident (ident)),
				TokenTree::Punct (punct) => Ok (Self::Punct (punct)),
				TokenTree::Literal (literal) => Ok (Self::Literal (literal))
			}
		}
	}
}

impl <T> PatternItem <T>
{
	pub fn visit <V> (&self, visitor: &mut V) -> Result <(), V::Error>
	where V: PatternVisitor <T>
	{
		match self
		{
			Self::Parameter (parameter) => visitor . visit_parameter (parameter),
			Self::Optional (optional) => optional . visit (visitor),
			Self::ZeroOrMore (zero_or_more) => zero_or_more . visit (visitor),
			Self::OneOrMore (one_or_more) => one_or_more . visit (visitor),
			Self::Group (group) => group . visit (visitor),
			Self::Ident (ident) => visitor . visit_ident (ident),
			Self::Punct (punct) => visitor . visit_punct (punct),
			Self::Literal (literal) => visitor . visit_literal (literal)
		}
	}

	pub fn specialize <'a, V>
	(
		&self,
		bindings: &StructuredBindingView <'a, V>,
		pattern_buffer: &mut PatternBuffer <T>
	)
	-> Result <(), StructuredBindingTypeMismatch>
	where T: Clone
	{
		match self
		{
			Self::Parameter (parameter) =>
				Ok (pattern_buffer . append_parameter (parameter . clone ())),
			Self::Optional (optional) =>
				optional . specialize (bindings, pattern_buffer),
			Self::ZeroOrMore (zero_or_more) =>
				zero_or_more . specialize (bindings, pattern_buffer),
			Self::OneOrMore (one_or_more) =>
				one_or_more . specialize (bindings, pattern_buffer),
			Self::Group (group) =>
				group . specialize (bindings, pattern_buffer),
			Self::Ident (ident) =>
				Ok (pattern_buffer . append_ident (ident . clone ())),
			Self::Punct (punct) =>
				Ok (pattern_buffer . append_punct (punct . clone ())),
			Self::Literal (literal) =>
				Ok (pattern_buffer . append_literal (literal . clone ()))
		}
	}
}

impl <T> ToTokens for PatternItem <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		match self
		{
			Self::Parameter (parameter) => parameter . to_tokens (tokens),
			Self::Optional (optional) => optional . to_tokens (tokens),
			Self::ZeroOrMore (zero_or_more) => zero_or_more . to_tokens (tokens),
			Self::OneOrMore (one_or_more) => one_or_more . to_tokens (tokens),
			Self::Group (group) => group . to_tokens (tokens),
			Self::Ident (ident) => ident . to_tokens (tokens),
			Self::Punct (punct) => punct . to_tokens (tokens),
			Self::Literal (literal) => literal . to_tokens (tokens)
		}
	}
}
