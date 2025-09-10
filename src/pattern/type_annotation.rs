use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use proc_macro2::TokenStream;
use syn::Ident;
use syn::parse::{Parse, ParseStream};
use quote::ToTokens;

use super::{ParseBinding, TokenizeBinding};

#[derive (Copy, Clone, Debug, PartialEq, Eq)]
pub struct TypeAnnotation <T>
{
	colon_token: syn::token::Colon,
	ty: T
}

impl <T> Parse for TypeAnnotation <T>
where T: Parse
{
	fn parse (input: ParseStream <'_>) -> syn::Result <Self>
	{
		let colon_token = input . parse ()?;
		let ty = input . parse ()?;

		Ok (Self {colon_token, ty})
	}
}

impl <T> ToTokens for TypeAnnotation <T>
where T: ToTokens
{
	fn to_tokens (&self, tokens: &mut TokenStream)
	{
		self . colon_token . to_tokens (tokens);
		self . ty . to_tokens (tokens);
	}
}

impl <T> Display for TypeAnnotation <T>
where T: Display
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt (format_args! (": {}", self . ty))
	}
}

impl <T, V> ParseBinding <V> for TypeAnnotation <T>
where T: ParseBinding <V>
{
	fn parse (&self, input: ParseStream <'_>) -> syn::Result <V>
	{
		self . ty . parse (input)
	}
}

impl <T, V> TokenizeBinding <V> for TypeAnnotation <T>
where T: TokenizeBinding <V>
{
	type Error = T::Error;

	fn tokenize (&self, ident: &Ident, binding: &V, tokens: &mut TokenStream)
	-> Result <(), Self::Error>
	{
		self . ty . tokenize (ident, binding, tokens)
	}
}

#[derive (Clone, Debug)]
pub struct ParameterBindingTypeMismatch <V, T>
{
	parameter: Ident,
	value: V,
	found: T,
	expected: T
}

impl <V, T> ParameterBindingTypeMismatch <V, T>
{
	pub fn new (parameter: Ident, value: V, found: T, expected: T) -> Self
	{
		Self {parameter, value, found, expected}
	}
}

impl <V, T> Display for ParameterBindingTypeMismatch <V, T>
where T: Display
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

impl <V, T> Error for ParameterBindingTypeMismatch <V, T>
where
	T: Debug + Display,
	V: Debug
{
}

impl <V, T> Into <syn::Error> for ParameterBindingTypeMismatch <V, T>
where
	V: ToTokens,
	T: Display
{
	fn into (self) -> syn::Error
	{
		syn::Error::new_spanned (&self . value, &self)
	}
}
