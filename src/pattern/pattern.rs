use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};

use proc_macro2::TokenStream;
use syn::Ident;
use syn::parse::{Parse, ParseStream, Parser};
use quote::ToTokens;

use super::{
	StructuredBindings,
	IndexBindings,
	VisitationError,
	SpecializationError,
	PatternBuffer,
	PatternVisitor,
	ParseBinding,
	MatchVisitor,
	TokenizeBinding,
	SubstitutionVisitor,
	SubstitutionError
};

#[derive (Clone, Debug)]
pub struct Pattern <T>
{
	pattern_buffer: PatternBuffer <T>,
	parameters: HashSet <Ident>
}

impl <T> Parse for Pattern <T>
where T: Clone + Parse + ToTokens
{
	fn parse (input: ParseStream <'_>) -> syn::Result <Self>
	{
		let pattern_buffer: PatternBuffer <T> = input . parse ()?;

		pattern_buffer . validate () . map_err (Into::<syn::Error>::into)?;

		let parameters = pattern_buffer
			. referenced_identifiers ()
			. cloned ()
			. collect ();

		Ok (Self {pattern_buffer, parameters})
	}
}

impl <T> ToTokens for Pattern <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		self . pattern_buffer . to_tokens (tokens);
	}
}

impl <T> Display for Pattern <T>
where T: ToTokens
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		Display::fmt (&self . pattern_buffer . to_token_stream (), f)
	}
}

impl <T> Pattern <T>
{
	pub fn assert_parameters_superset <O> (&self, other: &Pattern <O>)
	-> Result <(), Ident>
	{
		for ident in &other . parameters
		{
			if ! self . parameters . contains (ident)
			{
				return Err (ident . clone ())
			}
		}

		Ok (())
	}

	pub fn visit_pattern <V> (&self, visitor: &mut V)
	-> Result <(), VisitationError <<V as PatternVisitor <T>>::Error>>
	where V: PatternVisitor <T>
	{
		let index_bindings = IndexBindings::new ();

		self . pattern_buffer . visit (&index_bindings, visitor)
	}

	pub fn match_input <V> (&self, input: ParseStream <'_>)
	-> Result <StructuredBindings <V>, VisitationError <syn::Error>>
	where
		T: ParseBinding <V>,
		V: Clone + PartialEq + Display
	{
		let mut match_visitor = MatchVisitor::new (input);

		self . visit_pattern (&mut match_visitor)?;

		Ok (match_visitor . into_bindings ())
	}

	pub fn match_tokens <V> (&self, tokens: TokenStream)
	-> syn::Result <StructuredBindings <V>>
	where
		T: ParseBinding <V>,
		V: Clone + PartialEq + Display
	{
		let parser = |input: ParseStream <'_>|
		{
			self . match_input (input) . map_err (Into::into)
		};

		parser . parse2 (tokens)
	}

	pub fn substitute <V> (&self, bindings: &StructuredBindings <V>)
	-> Result <TokenStream, VisitationError <SubstitutionError <T::Error>>>
	where T: TokenizeBinding <V>
	{
		let mut substitution_visitor =
			SubstitutionVisitor::new (bindings . view ());

		self . visit_pattern (&mut substitution_visitor)?;

		Ok (substitution_visitor . into_tokens ())
	}

	pub fn specialize <V> (&self, bindings: &StructuredBindings <V>)
	-> Result <Self, SpecializationError <T::Error>>
	where T: Clone + Parse + Debug + TokenizeBinding <V>
	{
		let index_bindings = IndexBindings::new ();
		let mut pattern_buffer = PatternBuffer::new ();

		self . pattern_buffer . specialize
		(
			&index_bindings,
			&bindings . view (),
			&mut pattern_buffer
		)?;

		// No validation required.  A valid pattern, when specialized, will
		// always produce a valid pattern.

		let parameters = pattern_buffer
			. referenced_identifiers ()
			. cloned ()
			. collect ();

		Ok (Self {pattern_buffer, parameters})
	}
}
