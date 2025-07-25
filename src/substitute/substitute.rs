use syn::{
	Path,
	Generics,
	GenericParam,
	LifetimeParam,
	TypeParam,
	ItemStruct,
	ItemEnum,
	ItemTrait,
	ItemFn,
	DeriveInput,
	parse_quote
};
use syn::punctuated::Punctuated;
use syn::parse::Result;
use syn::fold::Fold;

use crate::generics::get_path_arguments;

use super::substitutions::Substitutions;

fn remove_parameters_from_generics (generics: Generics) -> Generics
{
	let Generics {lt_token, params, gt_token, where_clause} = generics;
	let mut new_generics = Generics
	{
		lt_token,
		params: Punctuated::new (),
		gt_token,
		where_clause
	};

	for param in params
	{
		match param
		{
			GenericParam::Lifetime (lifetime_param) =>
			{
				let LifetimeParam {lifetime, bounds, ..} = lifetime_param;

				if ! bounds . is_empty ()
				{
					new_generics
						. make_where_clause ()
						. predicates
						. push (parse_quote! (#lifetime: #bounds));
				}
			},
			GenericParam::Type (type_param) =>
			{
				let TypeParam {ident, bounds, ..} = type_param;

				if ! bounds . is_empty ()
				{
					new_generics
						. make_where_clause ()
						. predicates
						. push (parse_quote! (#ident: #bounds));
				}
			},
			GenericParam::Const (_const_param) => {}
		}
	}

	new_generics
}

pub fn substitute_arguments_for_struct
(
	mut struct_item: ItemStruct,
	struct_path: &Path
)
-> Result <(Substitutions, ItemStruct)>
{
	let mut substitutions = Substitutions::try_from_path_arguments
	(
		&struct_item . generics . params,
		&get_path_arguments (struct_path)?
	)?;

	struct_item . generics =
		remove_parameters_from_generics (struct_item . generics);

	let struct_item = substitutions . fold_item_struct (struct_item);

	Ok ((substitutions, struct_item))
}

pub fn substitute_arguments_for_enum
(
	mut enum_item: ItemEnum,
	enum_path: &Path
)
-> Result <(Substitutions, ItemEnum)>
{
	let mut substitutions = Substitutions::try_from_path_arguments
	(
		&enum_item . generics . params,
		&get_path_arguments (enum_path)?
	)?;

	enum_item . generics =
		remove_parameters_from_generics (enum_item . generics);

	let enum_item = substitutions . fold_item_enum (enum_item);

	Ok ((substitutions, enum_item))
}

pub fn substitute_arguments_for_trait
(
	mut trait_item: ItemTrait,
	trait_path: &Path
)
-> Result <(Substitutions, ItemTrait)>
{
	let mut substitutions = Substitutions::try_from_path_arguments
	(
		&trait_item . generics . params,
		&get_path_arguments (trait_path)?
	)?;

	trait_item . generics =
		remove_parameters_from_generics (trait_item . generics);

	let trait_item = substitutions . fold_item_trait (trait_item);

	Ok ((substitutions, trait_item))
}

pub fn substitute_arguments_for_fn (mut fn_item: ItemFn, fn_path: &Path)
-> Result <(Substitutions, ItemFn)>
{
	let mut substitutions = Substitutions::try_from_path_arguments
	(
		&fn_item . sig . generics . params,
		&get_path_arguments (fn_path)?
	)?;

	fn_item . sig . generics =
		remove_parameters_from_generics (fn_item . sig . generics);

	let fn_item = substitutions . fold_item_fn (fn_item);

	Ok ((substitutions, fn_item))
}

pub fn substitute_arguments_for_derive_input
(
	mut derive_input: DeriveInput,
	derive_input_path: &Path
)
-> Result <(Substitutions, DeriveInput)>
{
	let mut substitutions = Substitutions::try_from_path_arguments
	(
		&derive_input . generics . params,
		&get_path_arguments (derive_input_path)?
	)?;

	derive_input . generics =
		remove_parameters_from_generics (derive_input . generics);

	let derive_input = substitutions . fold_derive_input (derive_input);

	Ok ((substitutions, derive_input))
}
