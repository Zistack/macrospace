use proc_macro2::{Literal, Delimiter};
use proc_macro2::extra::DelimSpan;
use syn::Ident;

use super::PunctGroup;

pub trait PatternVisitor <P>
{
	type Error;
	type SubVisitor: PatternVisitor <P, Error = Self::Error>;

	fn visit_parameter (&mut self, parameter: P) -> Result <(), Self::Error>;

	fn visit_ident (&mut self, ident: Ident) -> Result <(), Self::Error>;

	fn visit_literal (&mut self, literal: Literal) -> Result <(), Self::Error>;

	fn visit_punct_group (&mut self, punct_group: PunctGroup)
	-> Result <(), Self::Error>;

	fn pre_visit_group
	(
		&mut self,
		delimiter: Delimiter,
		group_span: DelimSpan
	)
	-> Result <Self::SubVisitor, Self::Error>;

	fn post_visit_group
	(
		&mut self,
		delimiter: Delimiter,
		group_span: DelimSpan,
		sub_visitor: Self::SubVisitor
	)
	-> Result <(), Self::Error>;

	fn visit_end (&mut self) -> Result <(), Self::Error>;
}
