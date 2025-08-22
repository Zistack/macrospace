use syn::{Path, PathArguments, GenericArgument, Token};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use quote::ToTokens;

pub fn get_path_arguments (path: &Path)
-> Result <Option <&Punctuated <GenericArgument, Token! [,]>>>
{
	path
		. segments
		. last ()
		. ok_or (Error::new_spanned (path, "Path must be nonempty"))
		. and_then
		(
			|last_segment| match &last_segment . arguments
			{
				PathArguments::AngleBracketed (arguments) =>
					Ok (Some (&arguments . args)),
				PathArguments::Parenthesized (_) => Err
				(
					Error::new_spanned
					(
						path,
						"Parenthesized path arguments are not supported"
					)
				),
				_ => Ok (None)
			}
		)
}

pub fn get_path_arguments_mut (path: &mut Path)
-> Result <Option <&mut Punctuated <GenericArgument, Token! [,]>>>
{
	let path_tokens = path . to_token_stream ();

	match path . segments . last_mut ()
	{
		Some (last_segment) => match &mut last_segment . arguments
		{
			PathArguments::AngleBracketed (arguments) =>
				Ok (Some (&mut arguments . args)),
			PathArguments::Parenthesized (_) => Err
			(
				Error::new_spanned
				(
					path_tokens,
					"Parenthesized path arguments are not supported"
				)
			),
			_ => Ok (None)
		},
		None => Err (Error::new_spanned (path_tokens, "Path must be nonempty"))
	}
}

pub fn as_prefix (mut p: Path) -> Path
{
	if let Some (last_segment) = p . segments . last_mut ()
	{
		if let PathArguments::AngleBracketed (path_arguments) =
			&mut last_segment . arguments
		{
			path_arguments
				. colon2_token
				. get_or_insert (<Token! [::]>::default ());
		}
	}

	p
}

pub fn without_arguments (mut p: Path) -> Path
{
	if let Some (last_segment) = p . segments . last_mut ()
	{
		last_segment . arguments = PathArguments::None;
	}

	p
}
