use proc_macro2::Delimiter;
use proc_macro2::extra::DelimSpan;

use super::{PatternVisitor, ParameterCollector, MergeableBindings};

pub struct CollectVisitor <C>
{
	collector: C
}

impl <C> CollectVisitor <C>
{
	pub fn new () -> Self
	where C: Default
	{
		Self {collector: C::default ()}
	}

	pub fn into_collector (self) -> C
	{
		self . collector
	}
}

impl <C, P> PatternVisitor <P> for CollectVisitor <C>
where C: Default + ParameterCollector <P> + MergeableBindings
{
	type Error = C::Error;
	type SubVisitor = Self;

	fn visit_parameter (&mut self, parameter: P) -> Result <(), Self::Error>
	{
		self . collector . add_parameter (parameter);

		Ok (())
	}

	fn pre_visit_group (&mut self, _delimiter: Delimiter, _group_span: DelimSpan)
	-> Result <Self::SubVisitor, Self::Error>
	{
		Ok (Self::SubVisitor::new ())
	}

	fn post_visit_group
	(
		&mut self,
		_delimiter: Delimiter,
		_group_span: DelimSpan,
		sub_visitor: Self::SubVisitor
	)
	-> Result <(), Self::Error>
	{
		self . collector . try_merge (sub_visitor . collector)
	}
}
