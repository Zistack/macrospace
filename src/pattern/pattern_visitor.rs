use proc_macro2::{Literal, Delimiter};
use proc_macro2::extra::DelimSpan;
use syn::Ident;

use super::PunctGroup;

pub trait PatternVisitor <P>
{
	type Error;
	type SubVisitor: PatternVisitor <P, Error = Self::Error>;

	#[allow (unused_variables)]
	fn visit_parameter (&mut self, parameter: P) -> Result <(), Self::Error>
	{
		Ok (())
	}

	#[allow (unused_variables)]
	fn visit_ident (&mut self, ident: Ident) -> Result <(), Self::Error>
	{
		Ok (())
	}

	#[allow (unused_variables)]
	fn visit_literal (&mut self, literal: Literal) -> Result <(), Self::Error>
	{
		Ok (())
	}

	#[allow (unused_variables)]
	fn visit_punct_group (&mut self, _punct_group: PunctGroup)
	-> Result <(), Self::Error>
	{
		Ok (())
	}

	fn pre_visit_group
	(
		&mut self,
		delimiter: Delimiter,
		group_span: DelimSpan
	)
	-> Result <Self::SubVisitor, Self::Error>;

	#[allow (unused_variables)]
	fn post_visit_group
	(
		&mut self,
		delimiter: Delimiter,
		group_span: DelimSpan,
		sub_visitor: Self::SubVisitor
	)
	-> Result <(), Self::Error>
	{
		Ok (())
	}

	fn visit_end (&mut self) -> Result <(), Self::Error>
	{
		Ok (())
	}
}
