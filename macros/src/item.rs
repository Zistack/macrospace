use syn::{Ident, Visibility, Item, parse};
use syn::parse::{Nothing, Result, Error};
use syn::spanned::Spanned;
use quote::ToTokens;

use macrospace_core::generate_item_macro;

fn item_visibility (item: &Item) -> Result <&Visibility>
{
	match item
	{
		Item::Const (item) => Ok (&item . vis),
		Item::Enum (item) => Ok (&item . vis),
		Item::ExternCrate (item) => Ok (&item . vis),
		Item::Fn (item) => Ok (&item . vis),
		Item::Mod (item) => Ok (&item . vis),
		Item::Static (item) => Ok (&item . vis),
		Item::Struct (item) => Ok (&item . vis),
		Item::Trait (item) => Ok (&item . vis),
		Item::TraitAlias (item) => Ok (&item . vis),
		Item::Type (item) => Ok (&item . vis),
		Item::Union (item) => Ok (&item . vis),
		Item::Use (item) => Ok (&item . vis),
		_ => Err (Error::new_spanned (item, "Unsupported item"))
	}
}

fn item_ident (item: &Item) -> Result <&Ident>
{
	match item
	{
		Item::Const (item) => Ok (&item . ident),
		Item::Enum (item) => Ok (&item . ident),
		Item::ExternCrate (item) => Ok (&item . ident),
		Item::Fn (item) => Ok (&item . sig . ident),
		Item::Mod (item) => Ok (&item . ident),
		Item::Static (item) => Ok (&item . ident),
		Item::Struct (item) => Ok (&item . ident),
		Item::Trait (item) => Ok (&item . ident),
		Item::TraitAlias (item) => Ok (&item . ident),
		Item::Type (item) => Ok (&item . ident),
		Item::Union (item) => Ok (&item . ident),
		_ => Err (Error::new_spanned (item, "Unsupported item"))
	}
}

fn item_type (item: &Item) -> Result <Ident>
{
	match item
	{
		Item::Const (item) =>
			Ok (Ident::new ("const", item . const_token . span ())),
		Item::Enum (item) =>
			Ok (Ident::new ("enum", item . enum_token . span ())),
		Item::ExternCrate (item) =>
			Ok (Ident::new ("extern", item . extern_token . span ())),
		Item::Fn (item) =>
			Ok (Ident::new ("fn", item . sig . fn_token . span ())),
		Item::Mod (item) =>
			Ok (Ident::new ("mod", item . mod_token . span ())),
		Item::Static (item) =>
			Ok (Ident::new ("static", item . static_token . span ())),
		Item::Struct (item) =>
			Ok (Ident::new ("struct", item . struct_token . span ())),
		Item::Trait (item) =>
			Ok (Ident::new ("trait", item . trait_token . span ())),
		Item::TraitAlias (item) =>
			Ok (Ident::new ("alias", item . trait_token . span ())),
		Item::Type (item) =>
			Ok (Ident::new ("type", item . type_token . span ())),
		Item::Union (item) =>
			Ok (Ident::new ("union", item . union_token . span ())),
		Item::Use (item) =>
			Ok (Ident::new ("use", item . use_token . span ())),
		_ => Err (Error::new_spanned (item, "Unsupported item"))
	}
}

fn try_item_impl (attr: proc_macro::TokenStream, item: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let _: Nothing = parse (attr)?;

	let mut tokens = proc_macro2::TokenStream::from (item . clone ());

	let item = parse (item)?;

	generate_item_macro
	(
		item_ident (&item)?,
		&item_type (&item)?,
		item_visibility (&item)?,
		&item
	)
		. to_tokens (&mut tokens);

	Ok (tokens)
}

pub fn item_impl (attr: proc_macro::TokenStream, item: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_item_impl (attr, item)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
