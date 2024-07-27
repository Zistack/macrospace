use proc_macro::TokenStream;

mod transform_use;

mod check_item_type;
mod invoke_item_macro;
mod item;
mod import;
mod invoke;
mod parse_args;

#[doc (hidden)]
#[proc_macro]
pub fn check_item_type (input: TokenStream) -> TokenStream
{
	check_item_type::check_item_type_impl (input)
}

#[doc (hidden)]
#[proc_macro]
pub fn invoke_item_macro (input: TokenStream) -> TokenStream
{
	invoke_item_macro::invoke_item_macro_impl (input)
}

#[proc_macro_attribute]
pub fn item (attr: TokenStream, item: TokenStream) -> TokenStream
{
	item::item_impl (attr, item)
}

#[proc_macro_attribute]
pub fn import (attr: TokenStream, item: TokenStream) -> TokenStream
{
	import::import_impl (attr, item)
}

#[proc_macro]
pub fn invoke (input: TokenStream) -> TokenStream
{
	invoke::invoke_impl (input)
}

#[proc_macro]
pub fn parse_args (input: TokenStream) -> TokenStream
{
	parse_args::parse_args_impl (input)
}
