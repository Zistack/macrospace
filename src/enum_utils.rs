use syn::{Variant, Fields, Ident, Type, Token};
use syn::parse::{Result, Error};
use syn::punctuated::Punctuated;

pub fn get_variant_type (variant: &Variant) -> Result <&Type>
{
	match &variant . fields
	{
		Fields::Unnamed (unnamed) if unnamed . unnamed . len () == 1 =>
		{
			Ok (&unnamed . unnamed [0] . ty)
		}
		_ => Err
		(
			Error::new_spanned
			(
				&variant . fields,
				"Expected tuple variant with exactly one field"
			)
		)
	}
}

pub fn get_variant_types (variants: &Punctuated <Variant, Token! [,]>)
-> Result <Vec <Type>>
{
	let mut variant_types = Vec::new ();

	for variant in variants
	{
		variant_types . push (get_variant_type (variant)? . clone ());
	}

	Ok (variant_types)
}

pub fn get_variants (variants: &Punctuated <Variant, Token! [,]>) -> Vec <Ident>
{
	variants . iter () . map (|variant| variant . ident . clone ()) . collect ()
}
