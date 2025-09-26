use std::collections::HashSet;

use itertools::Itertools;
use proc_macro2::{TokenStream, Punct, Literal};
use syn::Ident;
use syn::parse::{Parse, ParseStream};
use quote::ToTokens;

use super::{
	Parameter,
	Index,
	StructuredBindingView,
	IndexBindings,
	OptionalPattern,
	ZeroOrMorePattern,
	OneOrMorePattern,
	RepetitionPattern,
	NoParameterInRepetition,
	GroupPattern,
	PatternItem,
	VisitationError,
	SpecializationError,
	PatternVisitor,
	TokenizeBinding
};

#[derive (Clone, Debug)]
pub struct PatternBuffer <T>
{
	pattern_items: Vec <PatternItem <T>>,
	parameters: HashSet <Ident>,
	sub_pattern_indices: Vec <usize>
}

impl <T> Parse for PatternBuffer <T>
where T: Parse
{
	fn parse (input: ParseStream <'_>) -> syn::Result <Self>
	{
		let mut pattern_buffer = Self::new ();

		while ! input . is_empty ()
		{
			pattern_buffer . append_item (input . parse ()?);
		}

		Ok (pattern_buffer)
	}
}

impl <T> PatternBuffer <T>
{
	pub fn new () -> Self
	{
		Self
		{
			pattern_items: Vec::new (),
			parameters: HashSet::new (),
			sub_pattern_indices: Vec::new ()
		}
	}

	pub fn append_parameter (&mut self, parameter: Parameter <T>)
	{
		self . parameters . insert (parameter . ident . clone ());

		self . pattern_items . push (PatternItem::Parameter (parameter));
	}

	pub fn append_index (&mut self, index: Index)
	{
		self . parameters . insert (index . ident . clone ());

		self . pattern_items . push (PatternItem::Index (index));
	}

	pub fn append_optional (&mut self, optional: OptionalPattern <T>)
	{
		self . sub_pattern_indices . push (self . pattern_items . len ());
		self . pattern_items . push (PatternItem::Optional (optional));
	}

	pub fn append_zero_or_more
	(
		&mut self,
		zero_or_more: ZeroOrMorePattern <T>
	)
	{
		self . sub_pattern_indices . push (self . pattern_items . len ());
		self . pattern_items . push (PatternItem::ZeroOrMore (zero_or_more));
	}

	pub fn append_one_or_more
	(
		&mut self,
		one_or_more: OneOrMorePattern <T>
	)
	{
		self . sub_pattern_indices . push (self . pattern_items . len ());
		self . pattern_items . push (PatternItem::OneOrMore (one_or_more));
	}

	pub fn append_repetition (&mut self, repetition: RepetitionPattern <T>)
	{
		match repetition
		{
			RepetitionPattern::Optional (optional) =>
				self . append_optional (optional),
			RepetitionPattern::ZeroOrMore (zero_or_more) =>
				self . append_zero_or_more (zero_or_more),
			RepetitionPattern::OneOrMore (one_or_more) =>
				self . append_one_or_more (one_or_more)
		}
	}

	pub fn append_group (&mut self, group: GroupPattern <T>)
	{
		self . sub_pattern_indices . push (self . pattern_items . len ());
		self . pattern_items . push (PatternItem::Group (group));
	}

	pub fn append_ident (&mut self, ident: Ident)
	{
		self . pattern_items . push (PatternItem::Ident (ident));
	}

	pub fn append_punct (&mut self, punct: Punct)
	{
		self . pattern_items . push (PatternItem::Punct (punct));
	}

	pub fn append_literal (&mut self, literal: Literal)
	{
		self . pattern_items . push (PatternItem::Literal (literal));
	}

	pub fn append_item (&mut self, pattern_item: PatternItem <T>)
	{
		match pattern_item
		{
			PatternItem::Parameter (parameter) =>
				self . append_parameter (parameter),
			PatternItem::Index (index) => self . append_index (index),
			PatternItem::Optional (optional) =>
				self . append_optional (optional),
			PatternItem::ZeroOrMore (zero_or_more) =>
				self . append_zero_or_more (zero_or_more),
			PatternItem::OneOrMore (one_or_more) =>
				self . append_one_or_more (one_or_more),
			PatternItem::Group (group) => self . append_group (group),
			PatternItem::Ident (ident) => self . append_ident (ident),
			PatternItem::Punct (punct) => self . append_punct (punct),
			PatternItem::Literal (literal) => self . append_literal (literal)
		}
	}

	// Panics if called on a pattern item which does _not_ have parameters.
	fn sub_pattern_referenced_identifiers (sub_pattern_item: &PatternItem <T>)
	-> Box <dyn Iterator <Item = &Ident> + '_>
	{
		match sub_pattern_item
		{
			PatternItem::Optional (optional) => Box::new (optional . referenced_identifiers ()),
			PatternItem::ZeroOrMore (zero_or_more) => Box::new (zero_or_more . referenced_identifiers ()),
			PatternItem::OneOrMore (one_or_more) => Box::new (one_or_more . referenced_identifiers ()),
			PatternItem::Group (group) => Box::new (group . referenced_identifiers ()),
			_ => unreachable! ()
		}
	}

	pub fn referenced_identifiers (&self) -> impl Iterator <Item = &Ident>
	{
		self
			. parameters
			. iter ()
			. chain
			(
				self . sub_pattern_indices . iter () . flat_map
				(
					|i| Self::sub_pattern_referenced_identifiers (&self . pattern_items [*i])
				)
			)
			. unique ()
	}

	pub fn validate (&self) -> Result <(), NoParameterInRepetition <T>>
	where T: Clone
	{
		for pattern_item in &self . pattern_items
		{
			pattern_item . validate ()?;
		}

		Ok (())
	}

	pub fn visit <V> (&self, index_bindings: &IndexBindings, visitor: &mut V)
	-> Result <(), VisitationError <V::Error>>
	where V: PatternVisitor <T>
	{
		for pattern_item in &self . pattern_items
		{
			pattern_item . visit (index_bindings, visitor)?;
		}

		Ok (())
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
		for pattern_item in &self . pattern_items
		{
			pattern_item . specialize (index_bindings, bindings, pattern_buffer)?;
		}

		Ok (())
	}
}

impl <T> ToTokens for PatternBuffer <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		for pattern_item in &self . pattern_items
		{
			pattern_item . to_tokens (tokens);
		}
	}
}
