use syn::{
	Path,
	PathArguments,
	GenericArgument,
	Generics,
	WhereClause,
	GenericParam,
	Token
};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};

pub fn combine_generics <const N: usize> (parts: [Generics; N]) -> Generics
{
	let mut params = Punctuated::new ();
	let mut predicates = Punctuated::new ();

	for part in parts
	{
		params . extend (part . params);
		if let Some (where_clause) = part . where_clause
		{
			predicates . extend (where_clause . predicates);
		}
	}

	let where_clause =
		if predicates . is_empty () { None }
		else
		{
			Some
			(
				WhereClause
				{
					where_token: <Token! [where]>::default (),
					predicates
				}
			)
		};

	Generics {lt_token: None, params, gt_token: None, where_clause}
}

pub fn get_num_required_arguments
(
	generic_parameters: &Punctuated <GenericParam, Token! [,]>
)
-> usize
{
	let mut count: usize = 0;

	for parameter in generic_parameters
	{
		match parameter
		{
			GenericParam::Lifetime (_) => count += 1,
			GenericParam::Type (type_parameter)
				if ! type_parameter . default . is_some () => count += 1,
			GenericParam::Const (const_parameter)
				if ! const_parameter . default . is_some () => count += 1,
			_ => {}
		}
	}

	count
}

pub fn get_path_arguments (path: &Path)
-> Result <Punctuated <GenericArgument, Token! [,]>>
{
	path
		. segments
		. last ()
		. ok_or
		(
			Error::new_spanned (path, "Path must be nonempty")
		)
		. and_then
		(
			|last_segment|
			match &last_segment . arguments
			{
				PathArguments::AngleBracketed (arguments) =>
					Ok (arguments . args . clone ()),
				PathArguments::Parenthesized (_) => Err
				(
					Error::new_spanned
					(
						path,
						"Parenthesized path arguments are not supported"
					)
				),
				_ => Ok (Punctuated::new ())
			}
		)
}
