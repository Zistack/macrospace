use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use proc_macro2::{TokenStream, TokenTree, Punct, Literal};
use syn::{Ident, Token};
use syn::parse::{Parse, Parser, ParseStream};
use quote::ToTokens;

use super::{
	Parameter,
	Index,
	ParameterBindingNotFound,
	StructuredBindingView,
	StructuredBindingTypeMismatch,
	StructuredBindingLookupError,
	IndexBindings,
	OptionalPattern,
	ZeroOrMorePattern,
	OneOrMorePattern,
	RepetitionPattern,
	NoParameterInRepetition,
	RepetitionLenMismatch,
	GroupPattern,
	PatternBuffer,
	PatternVisitor,
	TokenizeBinding
};

#[derive (Clone, Debug)]
pub enum PatternItem <T>
{
	Parameter (Parameter <T>),
	Index (Index),
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
			else if input . peek2 (syn::token::Pound)
			{
				Ok (Self::Index (input . parse ()?))
			}
			else if input . peek2 (syn::token::Bracket) || input . peek2 (syn::token::Paren)
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
	pub fn validate (&self) -> Result <(), NoParameterInRepetition <T>>
	where T: Clone
	{
		match self
		{
			Self::Optional (optional) => optional . validate (),
			Self::ZeroOrMore (zero_or_more) => zero_or_more . validate (),
			Self::OneOrMore (one_or_more) => one_or_more . validate (),
			Self::Group (group) => group . validate (),
			_ => Ok (())
		}
	}

	pub fn visit <V> (&self, index_bindings: &IndexBindings, visitor: &mut V)
	-> Result <(), VisitationError <V::Error>>
	where V: PatternVisitor <T>
	{
		match self
		{
			Self::Parameter (parameter) => visitor
				. visit_parameter (parameter)
				. map_err (VisitationError::Visitor),
			Self::Index (index) => visitor
				. visit_index
				(
					index,
					index_bindings . get_index (&index . ident)?
				)
				. map_err (VisitationError::Visitor),
			Self::Optional (optional) => optional
				. visit (index_bindings, visitor),
			Self::ZeroOrMore (zero_or_more) => zero_or_more
				. visit (index_bindings, visitor),
			Self::OneOrMore (one_or_more) => one_or_more
				. visit (index_bindings, visitor),
			Self::Group (group) => group . visit (index_bindings, visitor),
			Self::Ident (ident) => visitor
				. visit_ident (ident)
				. map_err (VisitationError::Visitor),
			Self::Punct (punct) => visitor
				. visit_punct (punct)
				. map_err (VisitationError::Visitor),
			Self::Literal (literal) => visitor
				. visit_literal (literal)
				. map_err (VisitationError::Visitor)
		}
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
		match self
		{
			Self::Parameter (parameter) =>
			{
				match bindings . get_maybe_value (&parameter . ident)?
				{
					Some (value) =>
					{
						let mut value_tokens = TokenStream::new ();

						parameter . extra_tokens . tokenize
						(
							&parameter . ident,
							value,
							&mut value_tokens
						)
							. map_err (SpecializationError::Tokenize)?;

						let parser = |input: ParseStream <'_>|
						{
							while ! input . is_empty ()
							{
								pattern_buffer . append_item (input . parse ()?);
							}

							Ok (())
						};

						parser . parse2 (value_tokens)?;
					},
					None => pattern_buffer . append_parameter (parameter . clone ())
				}

				Ok (())
			},
			Self::Index (index) =>
			{
				match index_bindings . get_maybe_index (&index . ident)
				{
					Some (i) => pattern_buffer . append_literal (Literal::usize_unsuffixed (i)),
					None => pattern_buffer . append_index (index . clone ())
				}

				Ok (())
			},
			Self::Optional (optional) =>
				optional . specialize (index_bindings, bindings, pattern_buffer),
			Self::ZeroOrMore (zero_or_more) =>
				zero_or_more . specialize (index_bindings, bindings, pattern_buffer),
			Self::OneOrMore (one_or_more) =>
				one_or_more . specialize (index_bindings, bindings, pattern_buffer),
			Self::Group (group) =>
				group . specialize (index_bindings, bindings, pattern_buffer),
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
			Self::Index (index) => index . to_tokens (tokens),
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

#[derive (Clone, Debug)]
pub enum VisitationError <E>
{
	IndexLookup (ParameterBindingNotFound),
	Visitor (E)
}

impl <E> From <ParameterBindingNotFound> for VisitationError <E>
{
	fn from (e: ParameterBindingNotFound) -> Self
	{
		Self::IndexLookup (e)
	}
}

impl <E> Display for VisitationError <E>
where E: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		match self
		{
			Self::IndexLookup (e) => Display::fmt (e, f),
			Self::Visitor (e) => Display::fmt (e, f)
		}
	}
}

impl <E> Error for VisitationError <E>
where E: Debug + Display
{
}

impl <E> Into <syn::Error> for VisitationError <E>
where E: Into <syn::Error>
{
	fn into (self) -> syn::Error
	{
		match self
		{
			Self::IndexLookup (e) => e . into (),
			Self::Visitor (e) => e . into ()
		}
	}
}

#[derive (Clone, Debug)]
pub enum SpecializationError <E>
{
	Lookup (StructuredBindingLookupError),
	LenMismatch (RepetitionLenMismatch),
	Tokenize (E),
	Parse (syn::Error)
}

impl <E> From <StructuredBindingLookupError> for SpecializationError <E>
{
	fn from (e: StructuredBindingLookupError) -> Self
	{
		Self::Lookup (e)
	}
}

impl <E> From <StructuredBindingTypeMismatch> for SpecializationError <E>
{
	fn from (e: StructuredBindingTypeMismatch) -> Self
	{
		Self::Lookup (e . into ())
	}
}

impl <E> From <RepetitionLenMismatch> for SpecializationError <E>
{
	fn from (e: RepetitionLenMismatch) -> Self
	{
		Self::LenMismatch (e)
	}
}

impl <E> From <syn::Error> for SpecializationError <E>
{
	fn from (e: syn::Error) -> Self
	{
		Self::Parse (e)
	}
}

impl <E> Display for SpecializationError <E>
where E: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		match self
		{
			Self::Lookup (e) => Display::fmt (e, f),
			Self::LenMismatch (e) => Display::fmt (e, f),
			Self::Tokenize (e) => Display::fmt (e, f),
			Self::Parse (e) => Display::fmt (e, f)
		}
	}
}

impl <E> Error for SpecializationError <E>
where E: Debug + Display
{
}

impl <E> Into <syn::Error> for SpecializationError <E>
where E: Into <syn::Error>
{
	fn into (self) -> syn::Error
	{
		match self
		{
			Self::Lookup (e) => e . into (),
			Self::LenMismatch (e) => e . into (),
			Self::Tokenize (e) => e . into (),
			Self::Parse (e) => e
		}
	}
}
