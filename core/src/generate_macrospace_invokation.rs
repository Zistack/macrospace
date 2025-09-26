use syn::Path;
use quote::{ToTokens, quote};

use crate::{ItemArgument, get_macro_path, sanitize};

pub fn generate_macrospace_invokation <I, T>
(
	inner_macro_path: Path,
	required_items: I,
	additional_tokens: T
)
-> proc_macro2::TokenStream
where
	I: IntoIterator <Item = ItemArgument>,
	T: ToTokens
{
	let mut item_paths = Vec::new ();
	let mut item_type_specs = Vec::new ();

	for item_argument in required_items
	{
		item_paths . push (item_argument . path);
		item_type_specs . push (item_argument . type_spec);
	}

	let mut item_paths = item_paths . into_iter ();
	let mut item_type_specs = item_type_specs . into_iter ();

	let sanitized_additional_tokens = sanitize (additional_tokens);

	if let (Some (first_item), Some (first_type_spec))
		= (item_paths . next (), item_type_specs . next ())
	{
		let first_macro = get_macro_path (&first_item);

		quote!
		{
			#first_macro!
			(
				#first_item: #first_type_spec
				(#(#item_paths: #item_type_specs),*)
				#inner_macro_path
				{}
				[#sanitized_additional_tokens]
			);
		}
	}
	else
	{
		quote!
		{
			#inner_macro_path! ({} [#sanitized_additional_tokens]);
		}
	}
}
