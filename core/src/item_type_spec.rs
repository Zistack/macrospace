use std::collections::HashSet;
use std::fmt::{Display, Formatter, Result};

use itertools::Itertools;
use syn::{Ident, Token};
use syn::punctuated::Punctuated;
use syn::ext::IdentExt;
use syn_derive::{Parse, ToTokens};
use quote::TokenStreamExt;

#[derive (Parse, ToTokens, Debug)]
pub struct ItemTypeSpec
{
	#[parse (
		|input| Ok
		(
			HashSet::from_iter
			(
				Punctuated
					::<Ident, Token! [|]>
					::parse_separated_nonempty_with (input, Ident::parse_any)?
			)
		)
	)]
	#[to_tokens (
		|tokens, types: &HashSet <Ident>|
		tokens . append_separated (types . iter (), <Token! [|]>::default ())
	)]
	pub types: HashSet <Ident>
}

impl ItemTypeSpec
{
	pub fn contains (&self, ty: &Ident) -> bool
	{
		self . types . contains (ty)
	}
}

impl Display for ItemTypeSpec
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result
	{
		match self . types . len ()
		{
			0 =>
			{
				write! (f, "nothing")
			},
			_ =>
			{
				write! (f, "{}", self . types . iter () . format (", "))
			}
		}
	}
}

pub struct ItemTypeMismatch;
