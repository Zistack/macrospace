use std::cell::RefCell;
use std::collections::HashMap;

use syn::Ident;

use super::ParameterBindingNotFound;

#[derive (Clone, Debug)]
pub struct IndexBindings
{
	map: RefCell <HashMap <Ident, usize>>
}

impl IndexBindings
{
	pub fn new () -> Self
	{
		Self {map: RefCell::new (HashMap::new ())}
	}

	pub fn get_binding_scope (&self, ident: Ident) -> IndexBindingScope <'_>
	{
		IndexBindingScope::new (ident, self)
	}

	pub fn get_index (&self, ident: &Ident)
	-> Result <usize, ParameterBindingNotFound>
	{
		match self . map . borrow () . get (ident)
		{
			Some (index) => Ok (*index),
			None => Err (ParameterBindingNotFound::new (ident . clone ()))
		}
	}

	pub fn get_maybe_index (&self, ident: &Ident) -> Option <usize>
	{
		self . map . borrow () . get (ident) . copied ()
	}

	pub fn return_binding_scope <'a>
	(
		&'a self,
		binding_scope: IndexBindingScope <'a>
	)
	-> usize
	{
		if ! std::ptr::eq (self, binding_scope . bindings)
		{
			panic! ("Cannot return binding scope to index bindings that it was not retrieved from");
		}

		self . map . borrow_mut () . remove (&binding_scope . ident) . unwrap ()
	}
}

#[derive (Debug)]
pub struct IndexBindingScope <'a>
{
	ident: Ident,
	bindings: &'a IndexBindings
}

impl <'a> IndexBindingScope <'a>
{
	fn new (ident: Ident, bindings: &'a IndexBindings) -> Self
	{
		bindings . map . borrow_mut () . insert (ident . clone (), 0);

		Self {ident, bindings}
	}

	pub fn increment (&self)
	{
		*self
			. bindings
			. map
			. borrow_mut ()
			. get_mut (&self . ident)
			. unwrap () += 1;
	}
}
