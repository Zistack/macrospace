use syn::{ItemUse, parse};
use syn::parse::{Nothing, Result, Error};
use syn::fold::Fold;
use quote::ToTokens;

use crate::transform_use::TransformUse;

fn try_import_impl
(
	attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> Result <proc_macro2::TokenStream>
{
	let _: Nothing = parse (attr)?;

	let mut tokens = proc_macro2::TokenStream::from (item . clone ());

	let item_use: ItemUse = parse (item)?;

	TransformUse {} . fold_item_use (item_use) . to_tokens (&mut tokens);

	Ok (tokens)
}

pub fn import_impl
(
	attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> proc_macro::TokenStream
{
	try_import_impl (attr, item)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}

fn try_import_exclusive_impl
(
	attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> Result <proc_macro2::TokenStream>
{
	let _: Nothing =  parse (attr)?;

	let item_use: ItemUse = parse (item)?;

	Ok (TransformUse {} . fold_item_use (item_use) . into_token_stream ())
}

pub fn import_exclusive_impl
(
	attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> proc_macro::TokenStream
{
	try_import_exclusive_impl (attr, item)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
