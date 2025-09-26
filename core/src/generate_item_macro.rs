use syn::{Attribute, Visibility, Ident, parse_quote};
use quote::{ToTokens, quote, format_ident};

use crate::{get_macro_ident, sanitize};

pub fn generate_item_macro <T>
(
	item_ident: &Ident,
	item_type: &Ident,
	item_visibility: &Visibility,
	item: &T
)
-> proc_macro2::TokenStream
where T: ToTokens
{
	let macro_ident = get_macro_ident (item_ident);

	let mangled_macro_ident = format_ident! ("__{}__", macro_ident);

	let export_attribute: Option <Attribute> = match item_visibility
	{
		Visibility::Public (_) => Some (parse_quote! (#[macro_export])),
		_ => None
	};

	let sanitized_item = sanitize (item);

	quote!
	{
		#[doc (hidden)]
		#export_attribute
		macro_rules! #mangled_macro_ident
		{
			(
				$this_item_path: path: $($this_item_types: tt)|*
				(
					$next_item_path: path: $($next_item_types: tt)|*,
					$($item_args: tt)*
				)
				$inner_macro_path: path
				{$($items: tt)*}
				[$($tokens: tt)*]
			) =>
			{
				macrospace::check_item_type!
				(
					$this_item_path: #item_type == $($this_item_types)|*
					{
						macrospace::invoke_item_macro!
						(
							$next_item_path: $($next_item_types)|*
							($($item_args)*)
							$inner_macro_path
							{$($items)* #sanitized_item}
							[$($tokens)*]
						);
					}
				);
			};
			(
				$this_item_path: path: $($this_item_types: tt)|*
				(
					$next_item_path: path: $($next_item_types: tt)|*
				)
				$inner_macro_path: path
				{$($items: tt)*}
				[$($tokens: tt)*]
			) =>
			{
				macrospace::check_item_type!
				(
					$this_item_path: #item_type == $($this_item_types)|*
					{
						macrospace::invoke_item_macro!
						(
							$next_item_path: $($next_item_types)|*
							()
							$inner_macro_path
							{$($items)* #sanitized_item}
							[$($tokens)*]
						);
					}
				);
			};
			(
				$this_item_path: path: $($this_item_types: tt)|*
				()
				$inner_macro_path: path
				{$($items: tt)*}
				[$($tokens: tt)*]
			) =>
			{
				macrospace::check_item_type!
				(
					$this_item_path: #item_type == $($this_item_types)|*
					{$inner_macro_path! ({$($items)* #sanitized_item} [$($tokens)*]);}
				);
			}
		}

		#item_visibility use #mangled_macro_ident as #macro_ident;
	}
}
