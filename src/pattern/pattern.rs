use std::fmt::{Debug, Display, Formatter};

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream, Parser};
use quote::ToTokens;

use super::{
	ParameterSchema,
	StructuredBindings,
	StructuredBindingTypeMismatch,
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
	parameter_schema: ParameterSchema
}

impl <T> Parse for Pattern <T>
where T: Clone + Parse + ToTokens
{
	fn parse (input: ParseStream <'_>) -> syn::Result <Self>
	{
		let pattern_buffer: PatternBuffer <T> = input . parse ()?;

		let parameter_schema = pattern_buffer
			. extract_schema ()
			. map_err (Into::<syn::Error>::into)?;

		parameter_schema
			. assert_parameters_disjoint ()
			. map_err (Into::<syn::Error>::into)?;

		Ok (Self {pattern_buffer, parameter_schema})
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
	pub fn is_parameters_superschema (&self, other: &Self) -> bool
	{
		self . parameter_schema . is_superschema (&other . parameter_schema)
	}

	pub fn visit_pattern <V> (&self, visitor: &mut V)
	-> Result <(), <V as PatternVisitor <T>>::Error>
	where V: PatternVisitor <T>
	{
		self . pattern_buffer . visit (visitor)
	}

	pub fn match_input <V> (&self, input: ParseStream <'_>)
	-> syn::Result <StructuredBindings <V>>
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
	-> Result <TokenStream, SubstitutionError <T, V>>
	where T: TokenizeBinding <V>
	{
		let mut substitution_visitor =
			SubstitutionVisitor::new (bindings . view ());

		self . visit_pattern (&mut substitution_visitor)?;

		Ok (substitution_visitor . into_tokens ())
	}

	pub fn specialize <V> (&self, bindings: &StructuredBindings <V>)
	-> Result <Self, StructuredBindingTypeMismatch>
	where T: Clone + Debug
	{
		let mut pattern_buffer = PatternBuffer::new ();

		self
			. pattern_buffer
			. specialize (&bindings . view (), &mut pattern_buffer)?;

		// Because repetitions are fully instantiated if all bindings exist, the
		// only remaining repetitions must yet have unbound parameters, and so
		// this schema extraction will never return an error.
		let parameter_schema = pattern_buffer . extract_schema () . unwrap ();

		Ok (Self {pattern_buffer, parameter_schema})
	}
}
