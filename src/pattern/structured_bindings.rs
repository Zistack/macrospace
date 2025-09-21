use std::borrow::BorrowMut;
use std::collections::hash_map::{HashMap, Entry};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};

use syn::Ident;

#[derive (Copy, Clone, Debug)]
pub enum StructuredBindingType
{
	Value,
	Index,
	Optional,
	ZeroOrMore,
	OneOrMore
}

impl Display for StructuredBindingType
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		match self
		{
			Self::Value => f . write_str ("value"),
			Self::Index => f . write_str ("index"),
			Self::Optional => f . write_str ("optional"),
			Self::ZeroOrMore => f . write_str ("zero_or_more"),
			Self::OneOrMore => f . write_str ("one_or_more")
		}
	}
}

#[derive (Clone, Debug, PartialEq, Eq)]
pub enum StructuredBinding <V>
{
	Value (V),
	Index (usize),
	Optional (Option <Box <StructuredBinding <V>>>),
	ZeroOrMore (Vec <StructuredBinding <V>>),
	OneOrMore (Vec <StructuredBinding <V>>)
}

impl <V> StructuredBinding <V>
{
	pub fn ty (&self) -> StructuredBindingType
	{
		match self
		{
			Self::Value (_) => StructuredBindingType::Value,
			Self::Index (_) => StructuredBindingType::Index,
			Self::Optional (_) => StructuredBindingType::Optional,
			Self::ZeroOrMore (_) => StructuredBindingType::ZeroOrMore,
			Self::OneOrMore (_) => StructuredBindingType::OneOrMore
		}
	}

	pub fn map <F, FF, O> (self, mut f: F) -> StructuredBinding <O>
	where
		F: BorrowMut <FF>,
		FF: FnMut (V) -> O
	{
		match self
		{
			Self::Value (v) => StructuredBinding::Value ((f . borrow_mut ()) (v)),
			Self::Index (len) => StructuredBinding::Index (len),
			Self::Optional (option) => StructuredBinding::Optional
			(
				option . map
				(
					|boxed_v|
					Box::new ((*boxed_v) . map::<&mut FF, FF, O> (f . borrow_mut ()))
				)
			),
			Self::ZeroOrMore (vec) => StructuredBinding::ZeroOrMore
			(
				vec
					. into_iter ()
					. map (|binding| binding . map::<&mut FF, FF, O> (f . borrow_mut ()))
					. collect ()
			),
			Self::OneOrMore (vec) => StructuredBinding::OneOrMore
			(
				vec
					. into_iter ()
					. map (|binding| binding . map::<&mut FF, FF, O> (f . borrow_mut ()))
					. collect ()
			)
		}
	}
}

fn write_vec <V> (vec: &Vec <StructuredBinding <V>>, f: &mut Formatter <'_>)
-> Result <(), std::fmt::Error>
where V: Display
{
	f . write_char ('[')?;

	let mut vec_iter = vec . iter ();

	if let Some (last) = vec_iter . next_back ()
	{
		for item in vec_iter
		{
			Display::fmt (item, f)?;
			f . write_str (", ")?;
		}

		Display::fmt (last, f)?;
	}

	f . write_char (']')?;

	Ok (())
}

impl <V> Display for StructuredBinding <V>
where V: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		match self
		{
			Self::Value (v) => Display::fmt (v, f),
			Self::Index (len) => Display::fmt (len, f),
			Self::Optional (option) => match option
			{
				Some (boxed_binding) => f . write_fmt
				(
					format_args! ("Some ({})", &*boxed_binding)
				),
				None => f . write_str ("None")
			},
			Self::ZeroOrMore (vec) => write_vec (vec, f),
			Self::OneOrMore (vec) => write_vec (vec, f)
		}
	}
}

#[derive (Clone, Debug)]
pub struct StructuredBindings <V>
{
	map: HashMap <Ident, StructuredBinding <V>>
}

impl <V> StructuredBindings <V>
{
	pub fn new () -> Self
	{
		Self {map: HashMap::new ()}
	}

	pub fn add_binding (&mut self, ident: Ident, binding: StructuredBinding <V>)
	-> Result <(), ParameterBindingMismatch <V>>
	where V: Clone + PartialEq
	{
		match self . map . entry (ident . clone ())
		{
			Entry::Vacant (vacant) =>
			{
				vacant . insert (binding);
			},
			Entry::Occupied (occupied) => if *occupied . get () != binding
			{
				return Err
				(
					ParameterBindingMismatch::new
					(
						ident,
						occupied . get () . clone (),
						binding
					)
				);
			}
		}

		Ok (())
	}

	pub fn add_value_binding (&mut self, ident: Ident, value: V)
	-> Result <(), ParameterBindingMismatch <V>>
	where V: Clone + PartialEq
	{
		self . add_binding (ident, StructuredBinding::Value (value))
	}

	pub fn add_index_len (&mut self, ident: Ident, len: usize)
	-> Result <(), ParameterBindingMismatch <V>>
	where V: Clone + PartialEq
	{
		self . add_binding (ident, StructuredBinding::Index (len))
	}

	pub fn add_optional_bindings <'a, I>
	(
		&mut self,
		idents: I,
		optional_bindings: Option <StructuredBindings <V>>
	)
	-> Result <(), StructuredBindingMergeError <V>>
	where
		I: IntoIterator <Item = &'a Ident>,
		V: Clone + PartialEq
	{
		match optional_bindings
		{
			Some (mut bindings) =>
			{
				for ident in idents
				{
					match bindings . map . remove (ident)
					{
						Some (binding) => self . add_binding
						(
							ident . clone (),
							StructuredBinding::Optional
							(
								Some (Box::new (binding))
							)
						)?,
						None => return Err
						(
							ParameterBindingNotFound::new (ident . clone ())
								. into ()
						)
					}
				}

				Ok (())
			},
			None =>
			{
				for ident in idents
				{
					self . add_binding
					(
						ident . clone (),
						StructuredBinding::Optional (None)
					)?;
				}

				Ok (())
			}
		}
	}

	pub fn add_zero_or_more_bindings <'a, I>
	(
		&mut self,
		idents: I,
		mut zero_or_more_bindings: Vec <StructuredBindings <V>>
	)
	-> Result <(), StructuredBindingMergeError <V>>
	where
		I: IntoIterator <Item = &'a Ident>,
		V: Clone + PartialEq
	{
		for ident in idents
		{
			let mut binding_vec = Vec::new ();

			for bindings in &mut zero_or_more_bindings
			{
				match bindings . map . remove (ident)
				{
					Some (binding) => binding_vec . push (binding),
					None => return Err
					(
						ParameterBindingNotFound::new (ident . clone ())
							. into ()
					)
				}
			}

			self . add_binding
			(
				ident . clone (),
				StructuredBinding::ZeroOrMore (binding_vec)
			)?;
		}

		Ok (())
	}

	pub fn add_one_or_more_bindings <'a, I>
	(
		&mut self,
		idents: I,
		mut one_or_more_bindings: Vec <StructuredBindings <V>>
	)
	-> Result <(), StructuredBindingMergeError <V>>
	where
		I: IntoIterator <Item = &'a Ident>,
		V: Clone + PartialEq
	{
		let mut idents = idents . into_iter ();

		if one_or_more_bindings . len () < 1
		{
			return if let Some (ident) = idents . next ()
			{
				Err (ParameterBindingNotFound::new (ident . clone ()) . into ())
			}
			else
			{
				// I suspect that this should never happen.
				Ok (())
			}
		}

		for ident in idents
		{
			let mut binding_vec = Vec::new ();

			for bindings in &mut one_or_more_bindings
			{
				match bindings . map . remove (ident)
				{
					Some (binding) => binding_vec . push (binding),
					None => return Err
					(
						ParameterBindingNotFound::new (ident . clone ())
							. into ()
					)
				}
			}

			self . add_binding
			(
				ident . clone (),
				StructuredBinding::OneOrMore (binding_vec)
			)?;
		}

		Ok (())
	}

	pub fn merge (&mut self, other: Self)
	-> Result <(), StructuredBindingMergeError <V>>
	where V: Clone + PartialEq
	{
		for (other_ident, other_binding) in other . map
		{
			self . add_binding (other_ident, other_binding)?;
		}

		Ok (())
	}

	pub fn map <F, FF, O> (self, mut f: F) -> StructuredBindings <O>
	where
		F: BorrowMut <FF>,
		FF: FnMut (V) -> O
	{
		StructuredBindings
		{
			map: self
				. map
				. into_iter ()
				. map
				(
					|(ident, binding)|
					(ident, binding . map::<&mut FF, FF, O> (f . borrow_mut ()))
				)
				. collect ()
		}
	}

	pub fn view (&self) -> StructuredBindingView <'_, V>
	{
		let mut map = HashMap::new ();

		for (ident, binding) in &self . map
		{
			map . insert (ident . clone (), binding);
		}

		StructuredBindingView {map}
	}
}

#[derive (Clone, Debug)]
pub struct ParameterBindingMismatch <V>
{
	parameter: Ident,
	first_value: StructuredBinding <V>,
	second_value: StructuredBinding <V>
}

impl <V> ParameterBindingMismatch <V>
{
	pub fn new
	(
		parameter: Ident,
		first_value: StructuredBinding <V>,
		second_value: StructuredBinding <V>
	)
	-> Self
	{
		Self {parameter, first_value, second_value}
	}
}

impl <V> Display for ParameterBindingMismatch <V>
where V: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt
		(
			format_args!
			(
				"Cannot bind parameter `{}` to value `{}`: already has value `{}`",
				&self . parameter,
				&self . second_value,
				&self . first_value
			)
		)
	}
}

impl <V> Error for ParameterBindingMismatch <V>
where V: Debug + Display
{
}

impl <V> Into <syn::Error> for ParameterBindingMismatch <V>
where V: Display
{
	fn into (self) -> syn::Error
	{
		syn::Error::new (proc_macro2::Span::call_site (), &self)
	}
}

#[derive (Clone, Debug)]
pub struct ParameterBindingNotFound
{
	parameter: Ident
}

impl ParameterBindingNotFound
{
	pub fn new (parameter: Ident) -> Self
	{
		Self {parameter}
	}
}

impl Display for ParameterBindingNotFound
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt
		(
			format_args!
			(
				"Expected binding for parameter `{}`",
				self . parameter
			)
		)
	}
}

impl Error for ParameterBindingNotFound
{
}

impl Into <syn::Error> for ParameterBindingNotFound
{
	fn into (self) -> syn::Error
	{
		syn::Error::new_spanned (&self . parameter, self . to_string ())
	}
}

#[derive (Clone, Debug)]
pub enum StructuredBindingMergeError <V>
{
	Mismatch (ParameterBindingMismatch <V>),
	NotFound (ParameterBindingNotFound)
}

impl <V> From <ParameterBindingMismatch <V>> for StructuredBindingMergeError <V>
{
	fn from (e: ParameterBindingMismatch <V>) -> Self
	{
		Self::Mismatch (e)
	}
}

impl <V> From <ParameterBindingNotFound> for StructuredBindingMergeError <V>
{
	fn from (e: ParameterBindingNotFound) -> Self
	{
		Self::NotFound (e)
	}
}

impl <V> Display for StructuredBindingMergeError <V>
where V: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		match self
		{
			Self::Mismatch (e) => Display::fmt (e, f),
			Self::NotFound (e) => Display::fmt (e, f)
		}
	}
}

impl <V> Error for StructuredBindingMergeError <V>
where V: Debug + Display
{
}

impl <V> Into <syn::Error> for StructuredBindingMergeError <V>
where V: Display
{
	fn into (self) -> syn::Error
	{
		match self
		{
			Self::Mismatch (e) => e . into (),
			Self::NotFound (e) => e . into ()
		}
	}
}

#[derive (Debug)]
pub struct StructuredBindingView <'a, V>
{
	map: HashMap <Ident, &'a StructuredBinding <V>>
}

impl <'a, V> Clone for StructuredBindingView <'a, V>
{
	fn clone (&self) -> Self
	{
		Self {map: self . map . clone ()}
	}
}

impl <'a, V> StructuredBindingView <'a, V>
{
	pub fn get_value (&self, ident: &Ident)
	-> Result <&V, StructuredBindingLookupError>
	{
		match self . map . get (ident)
		{
			None => Err
			(
				ParameterBindingNotFound::new (ident . clone ()) . into ()
			),
			Some (StructuredBinding::Value (value)) => Ok (value),
			Some (s) => Err
			(
				StructuredBindingTypeMismatch::new
				(
					ident . clone (),
					s . ty (),
					StructuredBindingType::Value
				)
					. into ()
			)
		}
	}

	pub fn get_maybe_value (&self, ident: &Ident)
	-> Result <Option <&V>, StructuredBindingTypeMismatch>
	{
		match self . map . get (ident)
		{
			None => Ok (None),
			Some (StructuredBinding::Value (value)) => Ok (Some (value)),
			Some (s) => Err
			(
				StructuredBindingTypeMismatch::new
				(
					ident . clone (),
					s . ty (),
					StructuredBindingType::Value
				)
					. into ()
			)
		}
	}

	pub fn get_index_len (&self, ident: &Ident)
	-> Result <usize, StructuredBindingLookupError>
	{
		match self . map . get (ident)
		{
			None => Err
			(
				ParameterBindingNotFound::new (ident . clone ()) . into ()
			),
			Some (StructuredBinding::Index (len)) => Ok (*len),
			Some (s) => Err
			(
				StructuredBindingTypeMismatch::new
				(
					ident . clone (),
					s . ty (),
					StructuredBindingType::Index
				)
					. into ()
			)
		}
	}

	pub fn project <'b, I> (&self, idents: I)
	-> Result <Self, ParameterBindingNotFound>
	where I: IntoIterator <Item = &'b Ident>
	{
		let mut map = HashMap::new ();

		for ident in idents
		{
			match self . map . get (ident)
			{
				Some (binding) => map . insert (ident . clone (), *binding),
				None => return Err
				(
					ParameterBindingNotFound::new (ident . clone ())
				)
			};
		}

		Ok (Self {map})
	}

	pub fn get_optional_view (&self)
	-> Result <Option <Self>, StructuredBindingTypeMismatch>
	{
		let mut map = HashMap::new ();

		for (ident, binding) in &self . map
		{
			match binding
			{
				StructuredBinding::Optional (Some (boxed_binding)) =>
					map . insert (ident . clone (), &**boxed_binding),
				StructuredBinding::Optional (None) => return Ok (None),
				_ => return Err
				(
					StructuredBindingTypeMismatch::new
					(
						ident . clone (),
						binding . ty (),
						StructuredBindingType::Optional
					)
				)
			};
		}

		Ok (Some (Self {map}))
	}

	pub fn get_zero_or_more_view (&self, index: usize)
	-> Result <Option <Self>, StructuredBindingTypeMismatch>
	{
		let mut map = HashMap::new ();

		for (ident, binding) in &self . map
		{
			match binding
			{
				StructuredBinding::ZeroOrMore (binding_vec) =>
					match binding_vec . get (index)
				{
					Some (binding) => map . insert (ident . clone (), binding),
					None => return Ok (None)
				},
				_ => return Err
				(
					StructuredBindingTypeMismatch::new
					(
						ident . clone (),
						binding . ty (),
						StructuredBindingType::ZeroOrMore
					)
				)
			};
		}

		Ok (Some (Self {map}))
	}

	pub fn get_one_or_more_first_view (&self)
	-> Result <Self, StructuredBindingLookupError>
	{
		let mut map = HashMap::new ();

		for (ident, binding) in &self . map
		{
			match binding
			{
				StructuredBinding::OneOrMore (binding_vec) =>
					match binding_vec . get (0)
				{
					Some (binding) => map . insert (ident . clone (), binding),
					None => return Err
					(
						ParameterBindingNotFound::new (ident . clone ())
							. into ()
					)
				},
				_ => return Err
				(
					StructuredBindingTypeMismatch::new
					(
						ident . clone (),
						binding . ty (),
						StructuredBindingType::OneOrMore
					)
						. into ()
				)
			};
		}

		Ok (Self {map})
	}

	pub fn get_one_or_more_view (&self, index: usize)
	-> Result <Option <Self>, StructuredBindingTypeMismatch>
	{
		let mut map = HashMap::new ();

		for (ident, binding) in &self . map
		{
			match binding
			{
				StructuredBinding::OneOrMore (binding_vec) =>
					match binding_vec . get (index)
				{
					Some (binding) => map . insert (ident . clone (), binding),
					None => return Ok (None)
				},
				_ => return Err
				(
					StructuredBindingTypeMismatch::new
					(
						ident . clone (),
						binding . ty (),
						StructuredBindingType::OneOrMore
					)
				)
			};
		}

		Ok (Some (Self {map}))
	}
}

#[derive (Clone, Debug)]
pub struct StructuredBindingTypeMismatch
{
	parameter: Ident,
	found: StructuredBindingType,
	expected: StructuredBindingType
}

impl StructuredBindingTypeMismatch
{
	pub fn new
	(
		parameter: Ident,
		found: StructuredBindingType,
		expected: StructuredBindingType
	)
	-> Self
	{
		Self {parameter, found, expected}
	}
}

impl Display for StructuredBindingTypeMismatch
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt
		(
			format_args!
			(
				"Expected parameter `{}` to have binding of type `{}`: found binding of type `{}`",
				self . parameter,
				self . expected,
				self . found
			)
		)
	}
}

impl Error for StructuredBindingTypeMismatch
{
}

impl Into <syn::Error> for StructuredBindingTypeMismatch
{
	fn into (self) -> syn::Error
	{
		syn::Error::new_spanned (&self . parameter, &self)
	}
}

#[derive (Clone, Debug)]
pub enum StructuredBindingLookupError
{
	TypeMismatch (StructuredBindingTypeMismatch),
	NotFound (ParameterBindingNotFound)
}

impl From <StructuredBindingTypeMismatch> for StructuredBindingLookupError
{
	fn from (e: StructuredBindingTypeMismatch) -> Self
	{
		Self::TypeMismatch (e)
	}
}

impl From <ParameterBindingNotFound> for StructuredBindingLookupError
{
	fn from (e: ParameterBindingNotFound) -> Self
	{
		Self::NotFound (e)
	}
}

impl Display for StructuredBindingLookupError
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		match self
		{
			Self::TypeMismatch (e) => Display::fmt (e, f),
			Self::NotFound (e) => Display::fmt (e, f)
		}
	}
}

impl Error for StructuredBindingLookupError
{
}

impl Into <syn::Error> for StructuredBindingLookupError
{
	fn into (self) -> syn::Error
	{
		match self
		{
			Self::TypeMismatch (e) => e . into (),
			Self::NotFound (e) => e . into ()
		}
	}
}
