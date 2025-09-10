use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use proc_macro2::{TokenStream, Group, Punct, Literal, Delimiter};
use proc_macro2::extra::DelimSpan;
use syn::Ident;
use quote::{ToTokens, TokenStreamExt};

use super::{
	Parameter,
	StructuredBindingView,
	StructuredBindingLookupError,
	StructuredBindingTypeMismatch,
	ParameterBindingNotFound,
	PatternVisitor,
	OptionalVisitor,
	ZeroOrMoreVisitor,
	OneOrMoreVisitor,
	TokenizeBinding
};

pub struct SubstitutionVisitor <'a, V>
{
	bindings: StructuredBindingView <'a, V>,
	tokens: TokenStream
}

impl <'a, V> SubstitutionVisitor <'a, V>
{
	pub fn new (bindings: StructuredBindingView <'a, V>) -> Self
	{
		Self {bindings, tokens: TokenStream::new ()}
	}

	pub fn into_tokens (self) -> TokenStream
	{
		self . tokens
	}
}

impl <'a, V, T> PatternVisitor <T> for SubstitutionVisitor <'a, V>
where T: TokenizeBinding <V>
{
	type Error = SubstitutionError <T, V>;
	type OptionalVisitor = SubstitutionOptionalVisitor <'a, V>;
	type ZeroOrMoreVisitor = SubstitutionXOrMoreVisitor <'a, V>;
	type OneOrMoreVisitor = SubstitutionXOrMoreVisitor <'a, V>;
	type GroupVisitor = SubstitutionVisitor <'a, V>;

	fn visit_parameter (&mut self, parameter: &Parameter <T>)
	-> Result <(), Self::Error>
	{
		let value = self . bindings . get_value (&parameter . ident)?;

		parameter
			. extra_tokens
			. tokenize (value, &mut self . tokens)
			. map_err (SubstitutionError::Tokenize)?;

		Ok (())
	}

	fn pre_visit_optional <'b, I> (&mut self, repetition_parameters: I)
	-> Result <Self::OptionalVisitor, Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		Ok
		(
			Self::OptionalVisitor::new
			(
				self . bindings . project (repetition_parameters)?
			)
		)
	}

	fn post_visit_optional <'b, I>
	(
		&mut self,
		_repetition_parameters: I,
		optional_visitor: Self::OptionalVisitor
	)
	-> Result <(), Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		self . tokens . extend (optional_visitor . tokens);

		Ok (())
	}

	fn pre_visit_zero_or_more <'b, I> (&mut self, repetition_parameters: I)
	-> Result <Self::ZeroOrMoreVisitor, Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		Ok
		(
			Self::ZeroOrMoreVisitor::new
			(
				self . bindings . project (repetition_parameters)?
			)
		)
	}

	fn post_visit_zero_or_more <'b, I>
	(
		&mut self,
		_repetition_parameters: I,
		zero_or_more_visitor: Self::ZeroOrMoreVisitor
	)
	-> Result <(), Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		self . tokens . extend (zero_or_more_visitor . tokens);

		Ok (())
	}

	fn pre_visit_one_or_more <'b, I> (&mut self, repetition_parameters: I)
	-> Result <Self::OneOrMoreVisitor, Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		Ok
		(
			Self::OneOrMoreVisitor::new
			(
				self . bindings . project (repetition_parameters)?
			)
		)
	}

	fn post_visit_one_or_more <'b, I>
	(
		&mut self,
		_repetition_parameters: I,
		one_or_more_visitor: Self::OneOrMoreVisitor
	)
	-> Result <(), Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		self . tokens . extend (one_or_more_visitor . tokens);

		Ok (())
	}

	fn pre_visit_group
	(
		&mut self,
		_delimiter: Delimiter,
		_group_span: DelimSpan
	)
	-> Result <Self::GroupVisitor, Self::Error>
	{
		Ok (Self::GroupVisitor::new (self . bindings . clone ()))
	}

	fn post_visit_group
	(
		&mut self,
		delimiter: Delimiter,
		_group_span: DelimSpan,
		group_visitor: Self::GroupVisitor
	)
	-> Result <(), Self::Error>
	{
		self . tokens . append (Group::new (delimiter, group_visitor . tokens));

		Ok (())
	}

	fn visit_ident (&mut self, ident: &Ident) -> Result <(), Self::Error>
	{
		ident . to_tokens (&mut self . tokens);

		Ok (())
	}

	fn visit_punct (&mut self, punct: &Punct) -> Result <(), Self::Error>
	{
		punct . to_tokens (&mut self . tokens);

		Ok (())
	}

	fn visit_literal (&mut self, literal: &Literal) -> Result <(), Self::Error>
	{
		literal . to_tokens (&mut self . tokens);

		Ok (())
	}
}

pub struct SubstitutionOptionalVisitor <'a, V>
{
	bindings: StructuredBindingView <'a, V>,
	tokens: TokenStream
}

impl <'a, V> SubstitutionOptionalVisitor <'a, V>
{
	pub fn new (bindings: StructuredBindingView <'a, V>) -> Self
	{
		Self {bindings, tokens: TokenStream::new ()}
	}
}

impl <'a, V, T> OptionalVisitor <T> for SubstitutionOptionalVisitor <'a, V>
where T: TokenizeBinding <V>
{
	type Error = SubstitutionError <T, V>;
	type OnceVisitor = SubstitutionVisitor <'a, V>;

	fn pre_visit_once (&mut self)
	-> Result <Option <Self::OnceVisitor>, Self::Error>
	{
		Ok
		(
			self
				. bindings
				. get_optional_view ()?
				. map (SubstitutionVisitor::new)
		)
	}

	fn post_visit_once
	(
		&mut self,
		once_visitor: Self::OnceVisitor,
		visit_result: Result <(), Self::Error>
	)
	-> Result <(), Self::Error>
	{
		visit_result?;

		self . tokens = once_visitor . tokens;

		Ok (())
	}
}

pub struct SubstitutionXOrMoreVisitor <'a, V>
{
	bindings: StructuredBindingView <'a, V>,
	repetition_index: usize,
	tokens: TokenStream
}

impl <'a, V> SubstitutionXOrMoreVisitor <'a, V>
{
	pub fn new (bindings: StructuredBindingView <'a, V>) -> Self
	{
		Self {bindings, repetition_index: 0, tokens: TokenStream::new ()}
	}
}

impl <'a, V, T> ZeroOrMoreVisitor <T> for SubstitutionXOrMoreVisitor <'a, V>
where T: TokenizeBinding <V>
{
	type Error = SubstitutionError <T, V>;
	type IterationVisitor = SubstitutionVisitor <'a, V>;

	fn pre_visit_iteration (&mut self)
	-> Result <Option <Self::IterationVisitor>, Self::Error>
	{
		Ok
		(
			self
				. bindings
				. get_zero_or_more_view (self . repetition_index)?
				. map (SubstitutionVisitor::new)
		)
	}

	fn post_visit_iteration
	(
		&mut self,
		iteration_visitor: Self::IterationVisitor,
		visit_result: Result <(), Self::Error>
	)
	-> Result <(), Self::Error>
	{
		visit_result?;

		self . tokens . extend (iteration_visitor . tokens);

		self . repetition_index += 1;

		Ok (())
	}

	fn visit_maybe_punct (&mut self, punct: &Punct)
	-> Result <bool, Self::Error>
	{
		if self
			. bindings
			. get_zero_or_more_view (self . repetition_index)?
			. is_some ()
		{
			punct . to_tokens (&mut self . tokens);

			Ok (true)
		}
		else
		{
			Ok (false)
		}
	}
}

impl <'a, V, T> OneOrMoreVisitor <T> for SubstitutionXOrMoreVisitor <'a, V>
where T: TokenizeBinding <V>
{
	type Error = SubstitutionError <T, V>;
	type IterationVisitor = SubstitutionVisitor <'a, V>;

	fn pre_visit_first (&mut self)
	-> Result <Self::IterationVisitor, Self::Error>
	{
		Ok
		(
			SubstitutionVisitor::new
			(
				self
					. bindings
					. get_one_or_more_first_view ()?
			)
		)
	}

	fn pre_visit_iteration (&mut self)
	-> Result <Option <Self::IterationVisitor>, Self::Error>
	{
		Ok
		(
			self
				. bindings
				. get_one_or_more_view (self . repetition_index)?
				. map (SubstitutionVisitor::new)
		)
	}

	fn post_visit_iteration
	(
		&mut self,
		iteration_visitor: Self::IterationVisitor,
		visit_result: Result <(), Self::Error>
	)
	-> Result <(), Self::Error>
	{
		visit_result?;

		self . tokens . extend (iteration_visitor . tokens);

		self . repetition_index += 1;

		Ok (())
	}

	fn visit_maybe_punct (&mut self, punct: &Punct)
	-> Result <bool, Self::Error>
	{
		if self
			. bindings
			. get_one_or_more_view (self . repetition_index)?
			. is_some ()
		{
			punct . to_tokens (&mut self . tokens);

			Ok (true)
		}
		else
		{
			Ok (false)
		}
	}
}

pub enum SubstitutionError <T, V>
where T: TokenizeBinding <V>
{
	Lookup (StructuredBindingLookupError),
	Tokenize (T::Error)
}

impl <T, V> Clone for SubstitutionError <T, V>
where
	T: TokenizeBinding <V>,
	T::Error: Clone
{
	fn clone (&self) -> Self
	{
		match self
		{
			Self::Lookup (e) => Self::Lookup (e . clone ()),
			Self::Tokenize (e) => Self::Tokenize (e . clone ())
		}
	}
}

impl <T, V> From <StructuredBindingLookupError> for SubstitutionError <T, V>
where T: TokenizeBinding <V>
{
	fn from (e: StructuredBindingLookupError) -> Self
	{
		Self::Lookup (e)
	}
}

impl <T, V> From <StructuredBindingTypeMismatch> for SubstitutionError <T, V>
where T: TokenizeBinding <V>
{
	fn from (e: StructuredBindingTypeMismatch) -> Self
	{
		Self::Lookup (e . into ())
	}
}

impl <T, V> From <ParameterBindingNotFound> for SubstitutionError <T, V>
where T: TokenizeBinding <V>
{
	fn from (e: ParameterBindingNotFound) -> Self
	{
		Self::Lookup (e . into ())
	}
}

impl <T, V> Debug for SubstitutionError <T, V>
where
	T: TokenizeBinding <V>,
	T::Error: Debug
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		match self
		{
			Self::Lookup (e) =>
				f . debug_tuple ("Lookup") . field (e) . finish (),
			Self::Tokenize (e) =>
				f . debug_tuple ("Tokenize") . field (e) . finish ()
		}
	}
}

impl <T, V> Display for SubstitutionError <T, V>
where
	T: TokenizeBinding <V>,
	T::Error: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		match self
		{
			Self::Lookup (e) => Display::fmt (e, f),
			Self::Tokenize (e) => Display::fmt (e, f)
		}
	}
}

impl <T, V> Error for SubstitutionError <T, V>
where
	T: TokenizeBinding <V>,
	T::Error: Debug + Display
{
}

/*
// I think?
impl <T, V> Into <syn::Error> for SubstitutionError <T, V>
where
	T: TokenizeBinding <V>,
	T::Error: Into <syn::Error>
{
	fn into (self) -> syn::Error
	{
		match self
		{
			Self::Lookup (e) => e . into (),
			Self::Tokenize (e) => e . into ()
		}
	}
}
*/
