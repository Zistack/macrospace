use syn::{Path, Token, parse};
use syn::token::Paren;
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn_derive::Parse;
use quote::quote;

use macrospace_core::{ItemArgument, MultiItemMacroInput, get_macro_path};

#[allow (dead_code)]
#[derive (Parse)]
struct InvokeItemMacro
{
	this_item: ItemArgument,
	#[syn (parenthesized)]
	paren_token: Paren,
	#[syn (in = paren_token)]
	#[parse (Punctuated::parse_terminated)]
	next_items: Punctuated <ItemArgument, Token! [,]>,
	inner_macro_path: Path,
	inner_macro_input: MultiItemMacroInput <proc_macro2::TokenStream>
}

pub fn try_invoke_item_macro_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let tokens = proc_macro2::TokenStream::from (input . clone ());

	let InvokeItemMacro
	{
		this_item,
		..
	}
		= parse (input)?;

	let this_item_macro_path = get_macro_path (&this_item . path);

	Ok (quote! { #this_item_macro_path! (#tokens); })
}

pub fn invoke_item_macro_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_invoke_item_macro_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
