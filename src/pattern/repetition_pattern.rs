use std::error::Error;
use std::fmt::{Display, Formatter};

use proc_macro2::{TokenStream, Punct};
use syn::{Ident, Token, parenthesized};
use syn::parse::{Parse, ParseStream};
use syn_derive::{Parse, ToTokens};
use quote::ToTokens;

use super::{
	ParameterSchema,
	NoParameterInRepetition,
	StructuredBindingView,
	IndexBindings,
	VisitationError,
	SpecializationError,
	PatternBuffer,
	PatternVisitor,
	OptionalVisitor,
	ZeroOrMoreVisitor,
	OneOrMoreVisitor,
	TokenizeBinding
};

#[derive (Clone, Debug)]
pub struct OptionalPattern <T>
{
	pub dollar_token: syn::token::Dollar,
	pub paren_token: syn::token::Paren,
	pub inner_pattern: PatternBuffer <T>,
	pub question_token: syn::token::Question
}

impl <T> Parse for OptionalPattern <T>
where T: Parse
{
	fn parse (input: ParseStream <'_>) -> syn::Result <Self>
	{
		let dollar_token = input . parse ()?;

		let content;
		let paren_token = parenthesized! (content in input);
		let inner_pattern = content . parse ()?;

		let question_token = input . parse ()?;

		Ok (Self {dollar_token, paren_token, inner_pattern, question_token})
	}
}

impl <T> ToTokens for OptionalPattern <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		self . dollar_token . to_tokens (tokens);

		self . paren_token . surround
		(
			tokens,
			|inner_tokens| self . inner_pattern . to_tokens (inner_tokens)
		);

		self . question_token . to_tokens (tokens);
	}
}

impl <T> OptionalPattern <T>
{
	pub fn referenced_identifiers (&self) -> impl Iterator <Item = &Ident>
	{
		self . inner_pattern . referenced_identifiers ()
	}

	pub fn extract_schema (&self)
	-> Result <ParameterSchema, NoParameterInRepetition <T>>
	where T: Clone
	{
		self . inner_pattern . extract_schema ()
	}

	pub fn visit <V> (&self, index_bindings: &IndexBindings, visitor: &mut V)
	-> Result <(), VisitationError <V::Error>>
	where V: PatternVisitor <T>
	{
		let mut optional_visitor = visitor . pre_visit_optional
		(
			self . inner_pattern . referenced_identifiers ()
		)
			. map_err (VisitationError::Visitor)?;

		if let Some (mut once_visitor) = optional_visitor
			. pre_visit_once ()
			. map_err (VisitationError::Visitor)?
		{
			let visit_result = self
				. inner_pattern
				. visit (index_bindings, &mut once_visitor);

			optional_visitor . post_visit_once (once_visitor, visit_result)?;
		}

		visitor . post_visit_optional
		(
			self . inner_pattern .  referenced_identifiers (),
			optional_visitor
		)
			. map_err (VisitationError::Visitor)?;

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
		match bindings . project
		(
			self . inner_pattern . referenced_identifiers ()
		)
		{
			Ok (projected_bindings) =>
				if let Some (optional_bindings) =
					projected_bindings . get_optional_view ()?
			{
				self . inner_pattern . specialize
				(
					index_bindings,
					&optional_bindings,
					pattern_buffer
				)?;
			},
			Err (_) => pattern_buffer . append_optional (self . clone ())
		}

		Ok (())
	}
}

#[derive (Clone, Debug, Parse, ToTokens)]
pub struct RepetitionIndex
{
	#[syn (bracketed)]
	pub bracket_token: syn::token::Bracket,
	#[syn (in = bracket_token)]
	pub ident: Ident
}

#[derive (Clone, Debug)]
pub struct ZeroOrMorePattern <T>
{
	pub dollar_token: syn::token::Dollar,
	pub repetition_index: Option <RepetitionIndex>,
	pub paren_token: syn::token::Paren,
	pub inner_pattern: PatternBuffer <T>,
	pub interspersed_token: Option <Punct>,
	pub star_token: syn::token::Star
}

impl <T> Parse for ZeroOrMorePattern <T>
where T: Parse
{
	fn parse (input: ParseStream <'_>) -> syn::Result <Self>
	{
		let dollar_token = input . parse ()?;

		let repetition_index = if input . peek (syn::token::Bracket)
		{
			Some (input . parse ()?)
		}
		else
		{
			None
		};

		let content;
		let paren_token = parenthesized! (content in input);
		let inner_pattern = content . parse ()?;

		let (interspersed_token, star_token) = if input . peek (Token! [*])
		{
			(None, input . parse ()?)
		}
		else
		{
			(Some (input . parse ()?), input . parse ()?)
		};

		Ok
		(
			Self
			{
				dollar_token,
				repetition_index,
				paren_token,
				inner_pattern,
				interspersed_token,
				star_token
			}
		)
	}
}

impl <T> ToTokens for ZeroOrMorePattern <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		self . dollar_token . to_tokens (tokens);

		self . repetition_index . to_tokens (tokens);

		self . paren_token . surround
		(
			tokens,
			|inner_tokens| self . inner_pattern . to_tokens (inner_tokens)
		);

		self . interspersed_token . to_tokens (tokens);
		self . star_token . to_tokens (tokens);
	}
}

impl <T> ZeroOrMorePattern <T>
{
	pub fn referenced_identifiers (&self) -> impl Iterator <Item = &Ident>
	{
		Iterator::chain
		(
			self . repetition_index . iter () . map (|ri| &ri . ident),
			self . inner_pattern . referenced_identifiers ()
		)
	}

	pub fn extract_schema (&self)
	-> Result <ParameterSchema, NoParameterInRepetition <T>>
	where T: Clone
	{
		self . inner_pattern . extract_schema ()
	}

	pub fn visit <V> (&self, index_bindings: &IndexBindings, visitor: &mut V)
	-> Result <(), VisitationError <V::Error>>
	where V: PatternVisitor <T>
	{
		let mut zero_or_more_visitor = visitor . pre_visit_zero_or_more
		(
			self . inner_pattern . referenced_identifiers ()
		)
			. map_err (VisitationError::Visitor)?;

		let binding_scope = match &self . repetition_index
		{
			Some (repetition_index) => Some
			(
				index_bindings . get_binding_scope
				(
					repetition_index . ident . clone ()
				)
			),
			None => None
		};

		while let Some (mut iteration_visitor) =
			zero_or_more_visitor
				. pre_visit_iteration ()
				. map_err (VisitationError::Visitor)?
		{
			let visit_result = self
				. inner_pattern
				. visit (index_bindings, &mut iteration_visitor);
			let should_break = visit_result . is_ok ();

			zero_or_more_visitor
				. post_visit_iteration (iteration_visitor, visit_result)?;

			if let Some (binding_scope) = &binding_scope
			{
				binding_scope . increment ();
			}

			if should_break { break; }

			if let Some (punct) = &self . interspersed_token
			{
				if ! zero_or_more_visitor
					. visit_maybe_punct (punct)
					. map_err (VisitationError::Visitor)?
				{
					break;
				}
			}
		}

		let repetition_index_len = match &self . repetition_index
		{
			Some (repetition_index) => Some
			((
				&repetition_index . ident,
				index_bindings . return_binding_scope (binding_scope . unwrap ())
			)),
			None => None
		};

		visitor . post_visit_zero_or_more
		(
			self . inner_pattern . referenced_identifiers (),
			repetition_index_len,
			zero_or_more_visitor
		)
			. map_err (VisitationError::Visitor)?;

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
		match bindings . project (self . inner_pattern . referenced_identifiers ())
		{
			Ok (projected_bindings) =>
			{
				let binding_scope = match &self . repetition_index
				{
					Some (repetition_index) => Some
					(
						index_bindings . get_binding_scope
						(
							repetition_index . ident . clone ()
						)
					),
					None => None
				};

				let mut index = 0;

				while let Some (zero_or_more_bindings) = projected_bindings . get_zero_or_more_view (index)?
				{
					self . inner_pattern . specialize (index_bindings, &zero_or_more_bindings, pattern_buffer)?;

					if let Some (punct) = &self . interspersed_token
						&& projected_bindings . get_zero_or_more_view (index + 1)? . is_some ()
					{
						pattern_buffer . append_punct (punct . clone ());
					}

					if let Some (binding_scope) = &binding_scope
					{
						binding_scope . increment ();
					}

					index += 1;
				}

				if let Some (repetition_index) = &self . repetition_index
				{
					let len = index_bindings
						. return_binding_scope (binding_scope . unwrap ());

					let expected_len = bindings
						. get_index_len (&repetition_index . ident)?;

					if len != expected_len
					{
						return Err
						(
							RepetitionLenMismatch::new
							(
								repetition_index . ident . clone (),
								len,
								expected_len
							)
								. into ()
						);
					}
				}
			},
			Err (_) => pattern_buffer . append_zero_or_more (self . clone ())
		}

		Ok (())
	}
}

#[derive (Clone, Debug)]
pub struct OneOrMorePattern <T>
{
	pub dollar_token: syn::token::Dollar,
	pub repetition_index: Option <RepetitionIndex>,
	pub paren_token: syn::token::Paren,
	pub inner_pattern: PatternBuffer <T>,
	pub interspersed_token: Option <Punct>,
	pub plus_token: syn::token::Plus
}

impl <T> Parse for OneOrMorePattern <T>
where T: Parse
{
	fn parse (input: ParseStream <'_>) -> syn::Result <Self>
	{
		let dollar_token = input . parse ()?;

		let repetition_index = if input . peek (syn::token::Bracket)
		{
			Some (input . parse ()?)
		}
		else
		{
			None
		};

		let content;
		let paren_token = parenthesized! (content in input);
		let inner_pattern = content . parse ()?;

		let (interspersed_token, plus_token) = if input . peek (Token! [+])
		{
			(None, input . parse ()?)
		}
		else
		{
			(Some (input . parse ()?), input . parse ()?)
		};

		Ok
		(
			Self
			{
				dollar_token,
				repetition_index,
				paren_token,
				inner_pattern,
				interspersed_token,
				plus_token
			}
		)
	}
}

impl <T> ToTokens for OneOrMorePattern <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		self . dollar_token . to_tokens (tokens);

		self . repetition_index . to_tokens (tokens);

		self . paren_token . surround
		(
			tokens,
			|inner_tokens| self . inner_pattern . to_tokens (inner_tokens)
		);

		self . interspersed_token . to_tokens (tokens);
		self . plus_token . to_tokens (tokens);
	}
}

impl <T> OneOrMorePattern <T>
{
	pub fn referenced_identifiers (&self) -> impl Iterator <Item = &Ident>
	{
		Iterator::chain
		(
			self . repetition_index . iter () . map (|ri| &ri . ident),
			self . inner_pattern . referenced_identifiers ()
		)
	}

	pub fn extract_schema (&self)
	-> Result <ParameterSchema, NoParameterInRepetition <T>>
	where T: Clone
	{
		self . inner_pattern . extract_schema ()
	}

	pub fn visit <V> (&self, index_bindings: &IndexBindings, visitor: &mut V)
	-> Result <(), VisitationError <V::Error>>
	where V: PatternVisitor <T>
	{
		let mut one_or_more_visitor = visitor . pre_visit_one_or_more
		(
			self . inner_pattern . referenced_identifiers ()
		)
			. map_err (VisitationError::Visitor)?;

		let binding_scope = match &self . repetition_index
		{
			Some (repetition_index) => Some
			(
				index_bindings . get_binding_scope
				(
					repetition_index . ident . clone ()
				)
			),
			None => None
		};

		let mut first_visitor = one_or_more_visitor
			. pre_visit_first ()
			. map_err (VisitationError::Visitor)?;

		self . inner_pattern . visit (index_bindings, &mut first_visitor)?;

		one_or_more_visitor
			. post_visit_iteration (first_visitor, Ok (()))?;

		if let Some (binding_scope) = &binding_scope
		{
			binding_scope . increment ();
		}

		if let Some (punct) = &self . interspersed_token
		{
			if ! one_or_more_visitor
				. visit_maybe_punct (punct)
				. map_err (VisitationError::Visitor)?
			{
				let repetition_index_len = match &self . repetition_index
				{
					Some (repetition_index) => Some
					((
						&repetition_index . ident,
						index_bindings . return_binding_scope (binding_scope . unwrap ())
					)),
					None => None
				};

				visitor . post_visit_one_or_more
				(
					self . inner_pattern . referenced_identifiers (),
					repetition_index_len,
					one_or_more_visitor
				)
					. map_err (VisitationError::Visitor)?;

				return Ok (());
			}
		}

		while let Some (mut iteration_visitor) = one_or_more_visitor
			. pre_visit_iteration ()
			. map_err (VisitationError::Visitor)?
		{
			let visit_result = self
				. inner_pattern
				. visit (index_bindings, &mut iteration_visitor);
			let should_break = visit_result . is_ok ();

			one_or_more_visitor
				. post_visit_iteration (iteration_visitor, visit_result)?;

			if let Some (binding_scope) = &binding_scope
			{
				binding_scope . increment ();
			}

			if should_break { break; }

			if let Some (punct) = &self . interspersed_token
			{
				if ! one_or_more_visitor
					. visit_maybe_punct (punct)
					. map_err (VisitationError::Visitor)?
				{
					break;
				}
			}
		}

		let repetition_index_len = match &self . repetition_index
		{
			Some (repetition_index) => Some
			((
				&repetition_index . ident,
				index_bindings . return_binding_scope (binding_scope . unwrap ())
			)),
			None => None
		};

		visitor . post_visit_one_or_more
		(
			self . inner_pattern . referenced_identifiers (),
			repetition_index_len,
			one_or_more_visitor
		)
			. map_err (VisitationError::Visitor)?;

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
		match bindings . project (self . inner_pattern . referenced_identifiers ())
		{
			Ok (projected_bindings) =>
			{
				let binding_scope = match &self . repetition_index
				{
					Some (repetition_index) => Some
					(
						index_bindings . get_binding_scope
						(
							repetition_index . ident . clone ()
						)
					),
					None => None
				};

				let one_or_more_bindings =
					projected_bindings . get_one_or_more_view (0)? . unwrap ();

				self . inner_pattern . specialize
				(
					index_bindings,
					&one_or_more_bindings,
					pattern_buffer
				)?;

				if let Some (punct) = &self . interspersed_token
					&& projected_bindings
						. get_one_or_more_view (1)?
						. is_some ()
				{
					pattern_buffer . append_punct (punct . clone ());
				}

				if let Some (binding_scope) = &binding_scope
				{
					binding_scope . increment ();
				}

				let mut index = 1;

				while let Some (one_or_more_bindings) =
					projected_bindings . get_one_or_more_view (index)?
				{
					self . inner_pattern . specialize
					(
						index_bindings,
						&one_or_more_bindings,
						pattern_buffer
					)?;

					if let Some (punct) = &self . interspersed_token
						&& projected_bindings
							. get_one_or_more_view (index + 1)?
							. is_some ()
					{
						pattern_buffer . append_punct (punct . clone ());
					}

					if let Some (binding_scope) = &binding_scope
					{
						binding_scope . increment ();
					}

					index += 1;
				}

				if let Some (repetition_index) = &self . repetition_index
				{
					let len = index_bindings
						. return_binding_scope (binding_scope . unwrap ());

					let expected_len = bindings
						. get_index_len (&repetition_index . ident)?;

					if len != expected_len
					{
						return Err
						(
							RepetitionLenMismatch::new
							(
								repetition_index . ident . clone (),
								len,
								expected_len
							)
								. into ()
						);
					}
				}
			},
			Err (_) => pattern_buffer . append_one_or_more (self . clone ())
		}

		Ok (())
	}
}

#[derive (Clone, Debug)]
pub enum RepetitionPattern <T>
{
	Optional (OptionalPattern <T>),
	ZeroOrMore (ZeroOrMorePattern <T>),
	OneOrMore (OneOrMorePattern <T>)
}

impl <T> Parse for RepetitionPattern <T>
where T: Parse
{
	fn parse (input: ParseStream <'_>) -> syn::Result <Self>
	{
		let dollar_token = input . parse ()?;

		let repetition_index = if input . peek (syn::token::Bracket)
		{
			Some (input . parse ()?)
		}
		else
		{
			None
		};

		let content;
		let paren_token = parenthesized! (content in input);

		let inner_pattern = content . parse ()?;

		let punct: Punct = input . parse ()?;

		match punct . as_char ()
		{
			'?' if repetition_index . is_none () =>
			{
				let question_token = syn::token::Question
				{
					spans: [punct . span ()]
				};

				let optional = OptionalPattern
				{
					dollar_token,
					paren_token,
					inner_pattern,
					question_token
				};

				return Ok (Self::Optional (optional));
			},
			'*' =>
			{
				let star_token = syn::token::Star {spans: [punct . span ()]};

				let zero_or_more = ZeroOrMorePattern
				{
					dollar_token,
					repetition_index,
					paren_token,
					inner_pattern,
					interspersed_token: None,
					star_token
				};

				return Ok (Self::ZeroOrMore (zero_or_more));
			},
			'+' =>
			{
				let plus_token = syn::token::Plus {spans: [punct . span ()]};

				let one_or_more = OneOrMorePattern
				{
					dollar_token,
					repetition_index,
					paren_token,
					inner_pattern,
					interspersed_token: None,
					plus_token
				};

				return Ok (Self::OneOrMore (one_or_more));
			},
			_ => {}
		}

		let lookahead = input . lookahead1 ();

		if lookahead . peek (Token! [*])
		{
			let zero_or_more = ZeroOrMorePattern
			{
				dollar_token,
				repetition_index,
				paren_token,
				inner_pattern,
				interspersed_token: Some (punct),
				star_token: input . parse ()?
			};

			Ok (Self::ZeroOrMore (zero_or_more))
		}
		else if lookahead . peek (Token! [+])
		{
			let one_or_more = OneOrMorePattern
			{
				dollar_token,
				repetition_index,
				paren_token,
				inner_pattern,
				interspersed_token: Some (punct),
				plus_token: input . parse ()?
			};

			Ok (Self::OneOrMore (one_or_more))
		}
		else
		{
			Err (lookahead . error ())
		}
	}
}

impl <T> From <OptionalPattern <T>> for RepetitionPattern <T>
{
	fn from (optional: OptionalPattern <T>) -> Self
	{
		Self::Optional (optional)
	}
}

impl <T> From <ZeroOrMorePattern <T>> for RepetitionPattern <T>
{
	fn from (zero_or_more: ZeroOrMorePattern <T>) -> Self
	{
		Self::ZeroOrMore (zero_or_more)
	}
}

impl <T> From <OneOrMorePattern <T>> for RepetitionPattern <T>
{
	fn from (one_or_more: OneOrMorePattern <T>) -> Self
	{
		Self::OneOrMore (one_or_more)
	}
}

impl <T> ToTokens for RepetitionPattern <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		match self
		{
			Self::Optional (optional) => optional . to_tokens (tokens),
			Self::ZeroOrMore (zero_or_more) => zero_or_more . to_tokens (tokens),
			Self::OneOrMore (one_or_more) => one_or_more . to_tokens (tokens)
		}
	}
}

#[derive (Clone, Debug)]
pub struct RepetitionLenMismatch
{
	ident: Ident,
	found: usize,
	expected: usize
}

impl RepetitionLenMismatch
{
	pub fn new (ident: Ident, found: usize, expected: usize) -> Self
	{
		Self {ident, found, expected}
	}
}

impl Display for RepetitionLenMismatch
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt
		(
			format_args!
			(
				"Expected index `{}` to count to `{}`: counted to `{}`",
				self . ident,
				self . expected,
				self . found
			)
		)
	}
}

impl Error for RepetitionLenMismatch
{
}

impl Into <syn::Error> for RepetitionLenMismatch
{
	fn into (self) -> syn::Error
	{
		syn::Error::new_spanned (&self . ident, &self)
	}
}
