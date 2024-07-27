use syn::{Ident, Path, Token, parse};
use syn::token::Brace;
use syn::parse::{Result, Error};
use syn::ext::IdentExt;
use syn_derive::Parse;

use macrospace_core::ItemTypeSpec;

#[allow (dead_code)]
#[derive (Parse)]
struct CheckItemTypeInput
{
	item_path: Path,
	colon_token: Token! [:],
	#[parse (Ident::parse_any)]
	item_type: Ident,

	eq_token: Token! [==],

	expected_item_type_spec: ItemTypeSpec,

	#[syn (braced)]
	brace_token: Brace,
	#[syn (in = brace_token)]
	success_tokens: proc_macro2::TokenStream
}

fn check_item_type
(
	item_path: Path,
	item_type: Ident,
	expected_item_type_spec: ItemTypeSpec,
	success_tokens: proc_macro2::TokenStream
)
-> proc_macro2::TokenStream
{
	if expected_item_type_spec . contains (&item_type)
	{
		success_tokens
	}
	else
	{
		Error::new_spanned
		(
			item_path,
			format!
			(
				"Expected item of type {}, found {}",
				expected_item_type_spec,
				item_type
			)
		)
			. into_compile_error ()
	}
}

fn try_check_item_type_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let CheckItemTypeInput
	{
		item_path,
		item_type,
		expected_item_type_spec,
		success_tokens,
		..
	}
		= parse (input)?;

	let tokens = check_item_type
	(
		item_path,
		item_type,
		expected_item_type_spec,
		success_tokens
	);

	Ok (tokens)
}

pub fn check_item_type_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_check_item_type_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
