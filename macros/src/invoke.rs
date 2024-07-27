use syn::{Path, Token, parse};
use syn::token::{Paren, Bracket};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn_derive::Parse;
use quote::quote;

use macrospace_core::{ItemArgument, generate_macrospace_invokation};

fn invoke <I>
(
	inner_macro_path: Path,
	required_items: I,
	additional_tokens: proc_macro2::TokenStream
)
-> proc_macro2::TokenStream
where I: IntoIterator <Item = ItemArgument>
{
	let macrospace_invokation = generate_macrospace_invokation
	(
		inner_macro_path,
		required_items,
		additional_tokens
	);

	quote!
	{
		quote::quote!
		{
			#macrospace_invokation
		}
	}
}

#[allow (dead_code)]
#[derive (Parse)]
struct InvokeInput
{
	inner_macro_path: Path,

	#[syn (parenthesized)]
	paren_token: Paren,
	#[syn (in = paren_token)]
	#[parse (Punctuated::parse_terminated)]
	required_items: Punctuated <ItemArgument, Token! [,]>,

	#[syn (bracketed)]
	bracket_token: Bracket,
	#[syn (in = bracket_token)]
	additional_tokens: proc_macro2::TokenStream
}

fn try_invoke_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let InvokeInput
	{
		inner_macro_path,
		required_items,
		additional_tokens,
		..
	}
		= parse (input)?;

	let tokens = invoke
	(
		inner_macro_path,
		required_items,
		additional_tokens
	);

	Ok (tokens)
}

pub fn invoke_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_invoke_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
