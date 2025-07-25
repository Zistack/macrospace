use std::fmt::{Display, Formatter};

use syn::{Ident, Lifetime, GenericArgument, GenericParam, Token, parse2};
use syn::parse::{Result, Error};
use syn_derive::{Parse, ToTokens};
use quote::ToTokens;

#[derive (Clone, Debug, PartialEq, Eq, Hash, Parse, ToTokens)]
pub enum Parameter
{
	#[parse (peek = Lifetime)]
	Lifetime (Lifetime),

	#[parse (peek = Ident)]
	Type (Ident),

	#[parse (peek = Token! [const])]
	Const (Token! [const], Ident)
}

impl From <GenericParam> for Parameter
{
	fn from (generic_param: GenericParam) -> Self
	{
		match generic_param
		{
			GenericParam::Lifetime (lifetime_param) =>
				Parameter::Lifetime (lifetime_param . lifetime),
			GenericParam::Type (type_param) =>
				Parameter::Type (type_param . ident),
			GenericParam::Const (const_param) =>
				Parameter::Const (const_param . const_token, const_param . ident)
		}
	}
}

impl <'a> From <&'a GenericParam> for Parameter
{
	fn from (generic_param: &'a GenericParam) -> Self
	{
		match generic_param
		{
			GenericParam::Lifetime (lifetime_param) =>
				Parameter::Lifetime (lifetime_param . lifetime . clone ()),
			GenericParam::Type (type_param) =>
				Parameter::Type (type_param . ident . clone ()),
			GenericParam::Const (const_param) => Parameter::Const
			(
				const_param . const_token,
				const_param . ident . clone ()
			)
		}
	}
}

impl TryFrom <GenericArgument> for Parameter
{
	type Error = Error;

	fn try_from (generic_argument: GenericArgument) -> Result <Self>
	{
		match generic_argument
		{
			GenericArgument::Lifetime (lifetime) =>
				Ok (Parameter::Lifetime (lifetime)),
			GenericArgument::Type (ty) =>
			{
				let ident: Ident = parse2 (ty . to_token_stream ())?;

				Ok (Parameter::Type (ident))
			},
			GenericArgument::Const (expr) =>
			{
				let ident: Ident = parse2 (expr . to_token_stream ())?;

				Ok (Parameter::Const (<Token! [const]>::default (), ident))
			},
			_ => Err
			(
				Error::new_spanned
				(
					generic_argument,
					"constraints make no sense in this context"
				)
			)
		}
	}
}

impl Display for Parameter
{
	fn fmt (&self, f: &mut Formatter <'_>)
	-> std::result::Result <(), std::fmt::Error>
	{
		match self
		{
			Self::Lifetime (lifetime) => Display::fmt (lifetime, f),
			Self::Type (ty_ident) => Display::fmt (ty_ident, f),
			Self::Const (_const_token, const_ident) =>
			{
				f . write_fmt (format_args! ("const {}", const_ident))
			}
		}
	}
}
