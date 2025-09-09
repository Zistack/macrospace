use std::borrow::Borrow;
use std::fmt::Display;
use std::marker::PhantomData;

use proc_macro2::{Punct, Literal, Delimiter};
use proc_macro2::extra::DelimSpan;
use syn::{Ident, parenthesized, braced, bracketed};
use syn::ext::IdentExt;
use syn::parse::ParseBuffer;
use syn::parse::discouraged::Speculative;

use super::{
	Parameter,
	StructuredBindings,
	PatternVisitor,
	OptionalVisitor,
	ZeroOrMoreVisitor,
	OneOrMoreVisitor,
	ParseBinding
};

pub struct MatchVisitor <'a, S, V>
{
	input: S,
	input_lifetime: PhantomData <&'a S>,
	bindings: StructuredBindings <V>
}

impl <'a, S, V> MatchVisitor <'a, S, V>
{
	pub fn new (input: S) -> Self
	{
		Self
		{
			input,
			input_lifetime: PhantomData::default (),
			bindings: StructuredBindings::new ()
		}
	}

	pub fn into_bindings (self) -> StructuredBindings <V>
	{
		self . bindings
	}
}

impl <'a, S, V, T> PatternVisitor <T> for MatchVisitor <'a, S, V>
where
	S: Borrow <ParseBuffer <'a>>,
	V: Clone + PartialEq + Display,
	T: ParseBinding <V>
{
	type Error = syn::Error;
	type OptionalVisitor = MatchOptionalVisitor <'a, ParseBuffer <'a>, V>;
	type ZeroOrMoreVisitor = MatchXOrMoreVisitor <'a, ParseBuffer <'a>, V>;
	type OneOrMoreVisitor = MatchXOrMoreVisitor <'a, ParseBuffer <'a>, V>;
	type GroupVisitor = MatchVisitor <'a, ParseBuffer <'a>, V>;

	fn visit_parameter (&mut self, parameter: &Parameter <T>)
	-> Result <(), Self::Error>
	{
		let value = parameter
			. extra_tokens
			. parse (self . input . borrow ())?;

		self
			. bindings
			. add_value_binding (parameter . ident . clone (), value)
			. map_err (Into::<syn::Error>::into)?;

		Ok (())
	}

	fn pre_visit_optional <'b, I> (&mut self, _repetition_parameters: I)
	-> Result <Self::OptionalVisitor, Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		Ok (Self::OptionalVisitor::new (self . input . borrow () . fork ()))
	}

	fn post_visit_optional <'b, I>
	(
		&mut self,
		repetition_parameters: I,
		optional_visitor: Self::OptionalVisitor
	)
	-> Result <(), Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		self . input . borrow () . advance_to (&optional_visitor . input);

		self . bindings . add_optional_bindings
		(
			repetition_parameters,
			optional_visitor . bindings
		)
			. map_err (Into::<syn::Error>::into)?;

		Ok (())
	}

	fn pre_visit_zero_or_more <'b, I> (&mut self, _repetition_parameters: I)
	-> Result <Self::ZeroOrMoreVisitor, Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		Ok (Self::ZeroOrMoreVisitor::new (self . input . borrow () . fork ()))
	}

	fn post_visit_zero_or_more <'b, I>
	(
		&mut self,
		repetition_parameters: I,
		zero_or_more_visitor: Self::ZeroOrMoreVisitor
	)
	-> Result <(), Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		self . input . borrow () . advance_to (&zero_or_more_visitor . input);

		self . bindings . add_zero_or_more_bindings
		(
			repetition_parameters,
			zero_or_more_visitor . bindings
		)
			. map_err (Into::<syn::Error>::into)?;

		Ok (())
	}

	fn pre_visit_one_or_more <'b, I> (&mut self, _repetition_parameters: I)
	-> Result <Self::OneOrMoreVisitor, Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		Ok (Self::OneOrMoreVisitor::new (self . input . borrow () . fork ()))
	}

	fn post_visit_one_or_more <'b, I>
	(
		&mut self,
		repetition_parameters: I,
		one_or_more_visitor: Self::OneOrMoreVisitor
	)
	-> Result <(), Self::Error>
	where I: IntoIterator <Item = &'b Ident>
	{
		self . input . borrow () . advance_to (&one_or_more_visitor . input);

		self . bindings . add_one_or_more_bindings
		(
			repetition_parameters,
			one_or_more_visitor . bindings
		)
			. map_err (Into::<syn::Error>::into)?;

		Ok (())
	}

	fn pre_visit_group
	(
		&mut self,
		delimiter: Delimiter,
		group_span: DelimSpan
	)
	-> Result <Self::GroupVisitor, Self::Error>
	{
		let content;

		match delimiter
		{
			Delimiter::Parenthesis =>
			{
				parenthesized! (content in self . input . borrow ());
			}
			Delimiter::Brace =>
			{
				braced! (content in self . input . borrow ());
			}
			Delimiter::Bracket =>
			{
				bracketed! (content in self . input . borrow ());
			}
			Delimiter::None => return Err
			(
				syn::parse::Error::new
				(
					group_span . join (),
					"undelimited groups are not supported"
				)
					. into ()
			)
		}

		Ok (Self::GroupVisitor::new (content))
	}

	fn post_visit_group
	(
		&mut self,
		_delimiter: Delimiter,
		_group_span: DelimSpan,
		group_visitor: Self::GroupVisitor
	)
	-> Result <(), Self::Error>
	{
		self
			. bindings
			. merge (group_visitor . bindings)
			. map_err (Into::into)
	}

	fn visit_ident (&mut self, ident: &Ident) -> Result <(), Self::Error>
	{
		let input_ident: Ident = Ident::parse_any (self . input . borrow ())?;

		if input_ident != *ident
		{
			return Err
			(
				syn::parse::Error::new_spanned
				(
					input_ident,
					format! ("expected `{}`", ident)
				)
			);
		}

		Ok (())
	}

	fn visit_punct (&mut self, punct: &Punct)
	-> Result <(), Self::Error>
	{
		let input_punct: Punct = self . input . borrow () . parse ()?;

		if input_punct . as_char () != punct . as_char ()
		{
			return Err
			(
				syn::Error::new_spanned
				(
					input_punct,
					format! ("expected `{}`", punct)
				)
			);
		}

		Ok (())
	}

	fn visit_literal (&mut self, literal: &Literal) -> Result <(), Self::Error>
	{
		let input_literal: Literal = self . input . borrow () . parse ()?;

		if input_literal . to_string () != literal . to_string ()
		{
			return Err
			(
				syn::parse::Error::new_spanned
				(
					input_literal,
					format! ("expected `{}`", literal)
				)
			);
		}

		Ok (())
	}

	fn visit_end (&mut self) -> Result <(), Self::Error>
	{
		if ! self . input . borrow () . is_empty ()
		{
			return Err
			(
				self
					. input
					. borrow ()
					. error ("expected end of input")
			);
		}

		Ok (())
	}
}

pub struct MatchOptionalVisitor <'a, S, V>
{
	input: S,
	input_lifetime: PhantomData <&'a S>,
	bindings: Option <StructuredBindings <V>>
}

impl <'a, S, V> MatchOptionalVisitor <'a, S, V>
{
	fn new (input: S) -> Self
	{
		Self
		{
			input,
			input_lifetime: PhantomData::default (),
			bindings: None
		}
	}
}

impl <'a, S, V, T> OptionalVisitor <T> for MatchOptionalVisitor <'a, S, V>
where
	S: Borrow <ParseBuffer <'a>>,
	V: Clone + PartialEq + Display,
	T: ParseBinding <V>
{
	type Error = syn::Error;
	type OnceVisitor = MatchVisitor <'a, ParseBuffer <'a>, V>;

	fn pre_visit_once (&mut self)
	-> Result <Option <Self::OnceVisitor>, Self::Error>
	{
		Ok (Some (Self::OnceVisitor::new (self . input . borrow () . fork ())))
	}

	fn post_visit_once
	(
		&mut self,
		once_visitor: Self::OnceVisitor,
		visit_result: Result <(), Self::Error>
	)
	-> Result <(), Self::Error>
	{
		if visit_result . is_ok ()
		{
			self . input . borrow () . advance_to (&once_visitor . input);
			self . bindings = Some (once_visitor . bindings);
		}

		Ok (())
	}
}

pub struct MatchXOrMoreVisitor <'a, S, V>
{
	input: S,
	input_lifespan: PhantomData <&'a S>,
	bindings: Vec <StructuredBindings <V>>
}

impl <'a, S, V> MatchXOrMoreVisitor <'a, S, V>
{
	fn new (input: S) -> Self
	{
		Self
		{
			input,
			input_lifespan: PhantomData::default (),
			bindings: Vec::new ()
		}
	}
}

impl <'a, S, V, T> ZeroOrMoreVisitor <T> for MatchXOrMoreVisitor <'a, S, V>
where
	S: Borrow <ParseBuffer <'a>>,
	V: Clone + PartialEq + Display,
	T: ParseBinding <V>
{
	type Error = syn::Error;
	type IterationVisitor = MatchVisitor <'a, ParseBuffer <'a>, V>;

	fn pre_visit_iteration (&mut self)
	-> Result <Option <Self::IterationVisitor>, Self::Error>
	{
		Ok (Some (Self::IterationVisitor::new (self . input . borrow () . fork ())))
	}

	fn post_visit_iteration
	(
		&mut self,
		iteration_visitor: Self::IterationVisitor,
		visit_result: Result <(), Self::Error>
	)
	-> Result <(), Self::Error>
	{
		if visit_result . is_ok ()
		{
			self . input . borrow () . advance_to (&iteration_visitor . input);
			self . bindings . push (iteration_visitor . bindings);
		}

		Ok (())
	}

	fn visit_maybe_punct (&mut self, punct: &Punct) -> Result <bool, Self::Error>
	{
		let speculative = self . input . borrow () . fork ();

		if let Ok (input_punct) = speculative . parse::<Punct> ()
			&& input_punct . as_char () == punct . as_char ()
		{
			self . input . borrow () . advance_to (&speculative);

			Ok (true)
		}
		else
		{
			Ok (false)
		}
	}
}

impl <'a, S, V, T> OneOrMoreVisitor <T> for MatchXOrMoreVisitor <'a, S, V>
where
	S: Borrow <ParseBuffer <'a>>,
	V: Clone + PartialEq + Display,
	T: ParseBinding <V>
{
	type Error = syn::Error;
	type IterationVisitor = MatchVisitor <'a, ParseBuffer <'a>, V>;

	fn pre_visit_first (&mut self)
	-> Result <Self::IterationVisitor, Self::Error>
	{
		Ok (Self::IterationVisitor::new (self . input . borrow () . fork ()))
	}

	fn pre_visit_iteration (&mut self)
	-> Result <Option <Self::IterationVisitor>, Self::Error>
	{
		Ok (Some (Self::IterationVisitor::new (self . input . borrow () . fork ())))
	}

	fn post_visit_iteration
	(
		&mut self,
		iteration_visitor: Self::IterationVisitor,
		visit_result: Result <(), Self::Error>
	)
	-> Result <(), Self::Error>
	{
		if visit_result . is_ok ()
		{
			self . input . borrow () . advance_to (&iteration_visitor . input);
			self . bindings . push (iteration_visitor . bindings);
		}

		Ok (())
	}

	fn visit_maybe_punct (&mut self, punct: &Punct) -> Result <bool, Self::Error>
	{
		let speculative = self . input . borrow () . fork ();

		if let Ok (input_punct) = speculative . parse::<Punct> ()
			&& input_punct . as_char () == punct . as_char ()
		{
			self . input . borrow () . advance_to (&speculative);

			Ok (true)
		}
		else
		{
			Ok (false)
		}
	}
}
