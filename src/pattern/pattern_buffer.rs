use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use itertools::Itertools;
use proc_macro2::{TokenStream, Punct, Literal};
use syn::Ident;
use syn::parse::{Parse, ParseStream};
use quote::ToTokens;

use super::{
	Parameter,
	ParameterSchema,
	StructuredBindingView,
	ParameterBindingTypeMismatch,
	OptionalPattern,
	ZeroOrMorePattern,
	OneOrMorePattern,
	RepetitionPattern,
	GroupPattern,
	PatternItem,
	PatternVisitor,
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

	fn assert_nested_schema_nonempty <R>
	(
		nested_schema: &ParameterSchema,
		repetition: &R
	)
	-> Result <(), NoParameterInRepetition <T>>
	where R: Clone + Into <RepetitionPattern <T>>
	{
		match nested_schema . is_empty ()
		{
			true => Err
			(
				NoParameterInRepetition::new (repetition . clone () . into ())
			),
			false => Ok (())
		}
	}

	pub fn extract_schema (&self)
	-> Result <ParameterSchema, NoParameterInRepetition <T>>
	where T: Clone
	{
		let mut schema = ParameterSchema::new ();

		for ident in self . referenced_identifiers ()
		{
			schema . add_parameter (ident . clone ());
		}

		let mut optional_schema = ParameterSchema::new ();
		let mut zero_or_more_schema = ParameterSchema::new ();
		let mut one_or_more_schema = ParameterSchema::new ();

		for sub_pattern_item
		in self
			. sub_pattern_indices
			. iter ()
			. map (|i| &self . pattern_items [*i])
		{
			match sub_pattern_item
			{
				PatternItem::Optional (optional) =>
				{
					let nested_schema = optional . extract_schema ()?;
					Self::assert_nested_schema_nonempty (&nested_schema, optional)?;
					optional_schema . merge (nested_schema);
				},
				PatternItem::ZeroOrMore (zero_or_more) =>
				{
					let nested_schema = zero_or_more . extract_schema ()?;
					Self::assert_nested_schema_nonempty (&nested_schema, zero_or_more)?;
					zero_or_more_schema . merge (nested_schema);
				},
				PatternItem::OneOrMore (one_or_more) =>
				{
					let nested_schema = one_or_more . extract_schema ()?;
					Self::assert_nested_schema_nonempty (&nested_schema, one_or_more)?;
					one_or_more_schema . merge (nested_schema);
				},
				PatternItem::Group (group) =>
					schema . merge (group . extract_schema ()?),
				_ => unreachable! ()
			}
		}

		if ! optional_schema . is_empty ()
		{
			schema . optional_parameters = Some (Box::new (optional_schema));
		}

		if ! zero_or_more_schema . is_empty ()
		{
			schema . zero_or_more_parameters = Some (Box::new (zero_or_more_schema));
		}

		if ! one_or_more_schema . is_empty ()
		{
			schema . one_or_more_parameters = Some (Box::new (one_or_more_schema));
		}

		Ok (schema)
	}

	pub fn visit <V> (&self, visitor: &mut V) -> Result <(), V::Error>
	where V: PatternVisitor <T>
	{
		for pattern_item in &self . pattern_items
		{
			pattern_item . visit (visitor)?;
		}

		Ok (())
	}

	pub fn specialize <'a, V>
	(
		&self,
		bindings: &StructuredBindingView <'a, V>,
		pattern_buffer: &mut PatternBuffer <T>
	)
	-> Result <(), ParameterBindingTypeMismatch>
	where T: Clone
	{
		for pattern_item in &self . pattern_items
		{
			pattern_item . specialize (bindings, pattern_buffer)?;
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

#[derive (Clone, Debug)]
pub struct NoParameterInRepetition <T>
{
	repetition: RepetitionPattern <T>
}

impl <T> NoParameterInRepetition <T>
{
	pub fn new (repetition: RepetitionPattern <T>) -> Self
	{
		Self {repetition}
	}
}

impl <T> Display for NoParameterInRepetition <T>
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_str ("No parameter in repetition")
	}
}

impl <T> Error for NoParameterInRepetition <T>
where T: Debug
{
}

impl <T> Into <syn::Error> for NoParameterInRepetition <T>
where T: ToTokens
{
	fn into (self) -> syn::Error
	{
		syn::Error::new_spanned (&self . repetition, &self)
	}
}
