use proc_macro2::{Punct, Literal, Delimiter};
use proc_macro2::extra::DelimSpan;
use syn::Ident;

use super::{Parameter, Index, VisitationError};

pub trait PatternVisitor <T>
{
	type Error;

	type OptionalVisitor: OptionalVisitor <T, Error = Self::Error>;
	type ZeroOrMoreVisitor: ZeroOrMoreVisitor <T, Error = Self::Error>;
	type OneOrMoreVisitor: OneOrMoreVisitor <T, Error = Self::Error>;

	type GroupVisitor: PatternVisitor <T, Error = Self::Error>;

	#[allow (unused_variables)]
	fn visit_parameter (&mut self, parameter: &Parameter <T>)
	-> Result <(), Self::Error>
	{
		Ok (())
	}

	#[allow (unused_variables)]
	fn visit_index (&mut self, index: &Index, i: usize)
	-> Result <(), Self::Error>
	{
		Ok (())
	}

	fn pre_visit_optional <'a, I> (&mut self, repetition_parameters: I)
	-> Result <Self::OptionalVisitor, Self::Error>
	where I: IntoIterator <Item = &'a Ident>;

	#[allow (unused_variables)]
	fn post_visit_optional <'a, I>
	(
		&mut self,
		repetition_parameters: I,
		optional_visitor: Self::OptionalVisitor
	)
	-> Result <(), Self::Error>
	where I: IntoIterator <Item = &'a Ident>
	{
		Ok (())
	}

	fn pre_visit_zero_or_more <'a, I> (&mut self, repetition_parameters: I)
	-> Result <Self::ZeroOrMoreVisitor, Self::Error>
	where I: IntoIterator <Item = &'a Ident>;

	#[allow (unused_variables)]
	fn post_visit_zero_or_more <'a, I>
	(
		&mut self,
		repetition_parameters: I,
		repetition_index_len: Option <(&Ident, usize)>,
		zero_or_more_visitor: Self::ZeroOrMoreVisitor
	)
	-> Result <(), Self::Error>
	where I: IntoIterator <Item = &'a Ident>
	{
		Ok (())
	}

	fn pre_visit_one_or_more <'a, I> (&mut self, repetition_parameters: I)
	-> Result <Self::OneOrMoreVisitor, Self::Error>
	where I: IntoIterator <Item = &'a Ident>;

	#[allow (unused_variables)]
	fn post_visit_one_or_more <'a, I>
	(
		&mut self,
		repetition_parameters: I,
		repetition_index_len: Option <(&Ident, usize)>,
		one_or_more_visitor: Self::OneOrMoreVisitor
	)
	-> Result <(), Self::Error>
	where I: IntoIterator <Item = &'a Ident>
	{
		Ok (())
	}

	fn pre_visit_group
	(
		&mut self,
		delimiter: Delimiter,
		group_span: DelimSpan
	)
	-> Result <Self::GroupVisitor, Self::Error>;

	#[allow (unused_variables)]
	fn post_visit_group
	(
		&mut self,
		delimiter: Delimiter,
		group_span: DelimSpan,
		group_visitor: Self::GroupVisitor
	)
	-> Result <(), Self::Error>
	{
		Ok (())
	}

	#[allow (unused_variables)]
	fn visit_ident (&mut self, ident: &Ident) -> Result <(), Self::Error>
	{
		Ok (())
	}

	#[allow (unused_variables)]
	fn visit_punct (&mut self, punct: &Punct) -> Result <(), Self::Error>
	{
		Ok (())
	}

	#[allow (unused_variables)]
	fn visit_literal (&mut self, literal: &Literal) -> Result <(), Self::Error>
	{
		Ok (())
	}

	fn visit_end (&mut self) -> Result <(), Self::Error>
	{
		Ok (())
	}
}

pub trait OptionalVisitor <T>
{
	type Error;
	type OnceVisitor: PatternVisitor <T, Error = Self::Error>;

	fn pre_visit_once (&mut self)
	-> Result <Option <Self::OnceVisitor>, Self::Error>;

	#[allow (unused_variables)]
	fn post_visit_once
	(
		&mut self,
		once_visitor: Self::OnceVisitor,
		visit_result: Result <(), VisitationError <Self::Error>>
	)
	-> Result <(), VisitationError <Self::Error>>
	{
		Ok (())
	}
}

pub trait ZeroOrMoreVisitor <T>
{
	type Error;
	type IterationVisitor: PatternVisitor <T, Error = Self::Error>;

	fn pre_visit_iteration (&mut self)
	-> Result <Option <Self::IterationVisitor>, Self::Error>;

	#[allow (unused_variables)]
	fn post_visit_iteration
	(
		&mut self,
		iteration_visitor: Self::IterationVisitor,
		visit_result: Result <(), VisitationError <Self::Error>>
	)
	-> Result <(), VisitationError <Self::Error>>
	{
		Ok (())
	}

	#[allow (unused_variables)]
	fn visit_maybe_punct (&mut self, punct: &Punct) -> Result <bool, Self::Error>;
}

pub trait OneOrMoreVisitor <T>
{
	type Error;
	type IterationVisitor: PatternVisitor <T, Error = Self::Error>;

	fn pre_visit_first (&mut self)
	-> Result <Self::IterationVisitor, Self::Error>;

	fn pre_visit_iteration (&mut self)
	-> Result <Option <Self::IterationVisitor>, Self::Error>;

	#[allow (unused_variables)]
	fn post_visit_iteration
	(
		&mut self,
		iteration_visitor: Self::IterationVisitor,
		visit_result: Result <(), VisitationError <Self::Error>>
	)
	-> Result <(), VisitationError <Self::Error>>
	{
		Ok (())
	}

	#[allow (unused_variables)]
	fn visit_maybe_punct (&mut self, punct: &Punct) -> Result <bool, Self::Error>;
}
