use proc_macro2::{TokenStream, TokenTree, Group, Span};
use quote::TokenStreamExt;

pub fn strip_spans (tokens: TokenStream) -> TokenStream
{
	let mut new_tokens = TokenStream::new ();

	for mut token in tokens
	{
		if let TokenTree::Group (group) = token
		{
			new_tokens . append
			(
				Group::new
				(
					group . delimiter (),
					strip_spans (group . stream ())
				)
			);
		}
		else
		{
			token . set_span (Span::call_site ());
			new_tokens . append (token);
		}
	}

	new_tokens
}
