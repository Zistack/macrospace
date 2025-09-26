use syn::{Ident, Index, Expr, Token, parse};
use syn::parse::{Result, Error};
use syn_derive::Parse;
use quote::{quote, format_ident};

fn parse_args (n: Index, input: Expr) -> proc_macro2::TokenStream
{
	let item_var_names: Vec <Ident> =
		(0..(n . index)) . map (|i| format_ident! ("x{}", i)) . collect ();

	quote!
	{
		(
			|input|
			{
				use syn::parse::Parser;

				let macrospace::MultiItemMacroInput {items_tokens, user_data, ..}
					= syn::parse (input)?;

				let parse_items = |input: syn::parse::ParseStream <'_>|
				{
					#(let #item_var_names = input . parse ()?;)*

					syn::Result::Ok ((#(#item_var_names),*))
				};

				syn::Result::Ok ((parse_items . parse2 (items_tokens)?, user_data))
			}
		)
		(#input)
	}
}

#[allow (dead_code)]
#[derive (Parse)]
struct ParseArgsInput
{
	n: Index,
	comma_token: Token! [,],
	input: Expr
}

fn try_parse_args_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let ParseArgsInput {n, input, ..} = parse (input)?;

	Ok (parse_args (n, input))
}

pub fn parse_args_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_parse_args_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
