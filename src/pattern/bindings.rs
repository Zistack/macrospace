use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use proc_macro2::TokenStream;
use syn::Ident;
use syn::parse::ParseStream;
use quote::ToTokens;

use super::TypedParameter;

pub trait MatchBindings <P>
{
	fn parse_parameter_binding
	(
		&mut self,
		parameter: P,
		input: ParseStream <'_>
	)
	-> Result <(), syn::parse::Error>;
}

pub trait MergeableBindings
{
	type Error;

	fn try_merge (&mut self, other: Self) -> Result <(), Self::Error>;
}

#[derive (Clone, Debug)]
pub struct ParameterBindingMismatch <T>
{
	ident: Ident,
	first_value: T,
	second_value: T
}

impl <T> ParameterBindingMismatch <T>
{
	pub fn new (ident: Ident, first_value: T, second_value: T) -> Self
	{
		Self {ident, first_value, second_value}
	}
}

impl <T> Display for ParameterBindingMismatch <T>
where T: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt
		(
			format_args!
			(
				"parameter `{}` is bound to two different values: {} != {}",
				self . ident,
				self . first_value,
				self . second_value
			)
		)
	}
}

impl <T> Error for ParameterBindingMismatch <T>
where T: Debug + Display
{
}

impl <T> Into <syn::parse::Error> for ParameterBindingMismatch <T>
where T: ToTokens
{
	fn into (self) -> syn::parse::Error
	{
		syn::parse::Error::new_spanned
		(
			self . second_value,
			format!
			(
				"parameter {} is already bound to `{}`",
				self . ident,
				self . first_value . to_token_stream ()
			)
		)
	}
}

pub trait ParameterCollector <P>
{
	fn add_parameter (&mut self, parameter: P);
}

pub trait SubstitutionBindings <P>
{
	type Error;

	fn write_parameter_tokens (&self, parameter: P, tokens: &mut TokenStream)
	-> Result <(), Self::Error>;
}

#[derive (Copy, Clone, Debug)]
pub struct ParameterNotFound <P>
{
	parameter: P
}

impl <P> ParameterNotFound <P>
{
	pub fn new (parameter: P) -> Self
	{
		Self {parameter}
	}
}

impl <P> Display for ParameterNotFound <P>
where P: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt
		(
			format_args!
			(
				"no binding found for parameter: {}",
				self . parameter
			)
		)
	}
}

impl <P> Error for ParameterNotFound <P>
where P: Debug + Display
{
}

impl <P> Into <syn::parse::Error> for ParameterNotFound <P>
where P: ToTokens
{
	fn into (self) -> syn::parse::Error
	{
		syn::parse::Error::new_spanned
		(
			self . parameter,
			"no binding found for parameter"
		)
	}
}

#[derive (Clone, Debug)]
pub struct ParameterTypeMismatch <T>
{
	value_type: T,
	parameter: TypedParameter <T>
}

impl <T> ParameterTypeMismatch <T>
{
	pub fn new (value_type: T, parameter: TypedParameter <T>) -> Self
	{
		Self {value_type, parameter}
	}
}

impl <T> Display for ParameterTypeMismatch <T>
where T: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt
		(
			format_args!
			(
				"parameter `{}` has value of type `{}`, but value of type `{}` was requested by the pattern",
				&self . parameter . ident,
				&self . value_type,
				&self . parameter . ty
			)
		)
	}
}

impl <T> Error for ParameterTypeMismatch <T>
where T: Debug + Display,
{
}

impl <T> Into <syn::parse::Error> for ParameterTypeMismatch <T>
where T: Display + ToTokens
{
	fn into (self) -> syn::parse::Error
	{
		syn::parse::Error::new_spanned (&self . parameter, self . to_string ())
	}
}

#[derive (Clone, Debug)]
pub enum SubstitutionError <T>
{
	NotFound (ParameterNotFound <TypedParameter <T>>),
	TypeMismatch (ParameterTypeMismatch <T>)
}

impl <T> From <ParameterNotFound <TypedParameter <T>>> for SubstitutionError <T>
{
	fn from (e: ParameterNotFound <TypedParameter <T>>) -> Self
	{
		Self::NotFound (e)
	}
}

impl <T> From <ParameterTypeMismatch <T>> for SubstitutionError <T>
{
	fn from (e: ParameterTypeMismatch <T>) -> Self
	{
		Self::TypeMismatch (e)
	}
}

impl <T> Display for SubstitutionError <T>
where T: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		match self
		{
			Self::NotFound (e) => Display::fmt (e, f),
			Self::TypeMismatch (e) => Display::fmt (e, f)
		}
	}
}

impl <T> Error for SubstitutionError <T>
where T: Debug + Display
{
}

impl <T> Into <syn::parse::Error> for SubstitutionError <T>
where T: Display + ToTokens
{
	fn into (self) -> syn::parse::Error
	{
		match self
		{
			Self::NotFound (e) => e . into (),
			Self::TypeMismatch (e) => e . into ()
		}
	}
}
