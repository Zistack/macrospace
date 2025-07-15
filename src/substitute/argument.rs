use std::fmt::{Display, Formatter};

use syn::{Lifetime, Type, Expr, GenericArgument, GenericParam, parse_quote};
use syn::parse::{Result, Error};
use syn_derive::{Parse, ToTokens};
use quote::ToTokens;

use super::parameter::Parameter;

#[derive (Clone, Debug, PartialEq, Eq, Hash, Parse, ToTokens)]
pub enum Argument
{
	#[parse (peek = Lifetime)]
	Lifetime (Lifetime),

	#[parse (peek_func = |input| input . fork () . parse::<Type> () . is_ok ())]
	Type (Type),

	Const (Expr)
}

impl Argument
{
	pub fn try_from_default_value (generic_param: GenericParam) -> Result <Self>
	{
		match generic_param
		{
			GenericParam::Lifetime (lifetime_param) => Err
			(
				Error::new_spanned
				(
					lifetime_param,
					"Lifetime parameters cannot have default values"
				)
			),
			GenericParam::Type (type_param) =>
				if let Some (ty) = type_param . default
			{
				Ok (Argument::Type (ty))
			}
			else
			{
				Err
				(
					Error::new_spanned
					(
						type_param,
						"Type parameter lacks a default argument"
					)
				)
			},
			GenericParam::Const (const_param) =>
				if let Some (expr) = const_param . default
			{
				Ok (Argument::Const (expr))
			}
			else
			{
				Err
				(
					Error::new_spanned
					(
						const_param,
						"Const parameter lacks a default_argument"
					)
				)
			}
		}
	}
}

impl From <GenericParam> for Argument
{
	fn from (generic_param: GenericParam) -> Self
	{
		match generic_param
		{
			GenericParam::Lifetime (lifetime_param) =>
				Argument::Lifetime (lifetime_param . lifetime),
			GenericParam::Type (type_param) =>
			{
				let ident = type_param . ident;
				Argument::Type (parse_quote! (#ident))
			},
			GenericParam::Const (const_param) =>
			{
				let ident = const_param . ident;
				Argument::Const (parse_quote! (#ident))
			}
		}
	}
}

impl <'a> From <&'a GenericParam> for Argument
{
	fn from (generic_param: &'a GenericParam) -> Self
	{
		match generic_param
		{
			GenericParam::Lifetime (lifetime_param) =>
				Argument::Lifetime (lifetime_param . lifetime . clone ()),
			GenericParam::Type (type_param) =>
			{
				let ident = &type_param . ident;
				Argument::Type (parse_quote! (#ident))
			},
			GenericParam::Const (const_param) =>
			{
				let ident = &const_param . ident;
				Argument::Const (parse_quote! (#ident))
			}
		}
	}
}

impl TryFrom <GenericArgument> for Argument
{
	type Error = Error;

	fn try_from (generic_argument: GenericArgument) -> Result <Self>
	{
		match generic_argument
		{
			GenericArgument::Lifetime (lifetime) =>
				Ok (Argument::Lifetime (lifetime)),
			GenericArgument::Type (ty) => Ok (Argument::Type (ty)),
			GenericArgument::Const (expr) => Ok (Argument::Const (expr)),
			_ => Err
			(
				Error::new_spanned
				(
					generic_argument,
					"Constraints make no sense in this context"
				)
			)
		}
	}
}

impl <'a> TryFrom <&'a GenericArgument> for Argument
{
	type Error = Error;

	fn try_from (generic_argument: &'a GenericArgument) -> Result <Self>
	{
		match generic_argument
		{
			GenericArgument::Lifetime (lifetime) =>
				Ok (Argument::Lifetime (lifetime . clone ())),
			GenericArgument::Type (ty) => Ok (Argument::Type (ty . clone ())),
			GenericArgument::Const (expr) => Ok (Argument::Const (expr . clone ())),
			_ => Err
			(
				Error::new_spanned
				(
					generic_argument,
					"Constraints make no sense in this context"
				)
			)
		}
	}
}

impl <'a> From <&'a Parameter> for Argument
{
	fn from (info: &'a Parameter) -> Self
	{
		match info
		{
			Parameter::Lifetime (lifetime) =>
				Argument::Lifetime (lifetime . clone ()),
			Parameter::Type (ident) =>
				Argument::Type (parse_quote! (#ident)),
			Parameter::Const (_, ident) =>
				Argument::Const (parse_quote! (#ident))
		}
	}
}

impl Display for Argument
{
	fn fmt (&self, f: &mut Formatter <'_>)
	-> std::result::Result <(), std::fmt::Error>
	{
		match self
		{
			Self::Lifetime (lifetime) => Display::fmt (lifetime, f),
			Self::Type (ty) => Display::fmt (&ty . to_token_stream (), f),
			Self::Const (expr) => Display::fmt (&expr . to_token_stream (), f)
		}
	}
}
