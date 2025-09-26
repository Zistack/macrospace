use proc_macro2::{TokenStream, TokenTree, Ident, Punct, Group, Spacing};
use quote::{ToTokens, TokenStreamExt};

const DOLLAR_ALONE: &str = "__sanitized_dollar_token_alone__";
const DOLLAR_JOINT: &str = "__sanitized_dollar_token_joint__";

pub fn sanitize <T> (raw_tokens: T) -> TokenStream
where T: ToTokens
{
	let mut sanitized_tokens = TokenStream::new ();

	for token in raw_tokens . into_token_stream ()
	{
		match token
		{
			TokenTree::Punct (punct) if punct . as_char () == '$' =>
			{
				match punct . spacing ()
				{
					Spacing::Alone => sanitized_tokens . append
					(
						Ident::new (DOLLAR_ALONE, punct . span ())
					),
					Spacing::Joint => sanitized_tokens . append
					(
						Ident::new (DOLLAR_JOINT, punct . span ())
					)
				}
			},
			TokenTree::Group (group) =>
			{
				let mut sanitized_group = Group::new
				(
					group . delimiter (),
					sanitize (group . stream ())
				);
				sanitized_group . set_span (group . span ());
				sanitized_tokens . append (sanitized_group);
			},
			t @ _ => sanitized_tokens . append (t)
		}
	}

	sanitized_tokens
}

pub fn desanitize <T> (sanitized_tokens: T) -> TokenStream
where T: ToTokens
{
	let mut desanitized_tokens = TokenStream::new ();

	for token in sanitized_tokens . into_token_stream ()
	{
		match token
		{
			TokenTree::Ident (ident) =>
			{
				if ident == DOLLAR_ALONE
				{
					let mut dollar_token = Punct::new ('$', Spacing::Alone);
					dollar_token . set_span (ident . span ());
					desanitized_tokens . append (dollar_token);
				}
				else if ident == DOLLAR_JOINT
				{
					let mut dollar_token = Punct::new ('$', Spacing::Joint);
					dollar_token . set_span (ident . span ());
					desanitized_tokens . append (dollar_token);
				}
				else
				{
					desanitized_tokens . append (ident);
				}
			},
			TokenTree::Group (group) =>
			{
				let mut desanitized_group = Group::new
				(
					group . delimiter (),
					desanitize (group . stream ())
				);
				desanitized_group . set_span (group . span ());
				desanitized_tokens . append (desanitized_group);
			},
			t @ _ => desanitized_tokens . append (t)
		}
	}

	desanitized_tokens
}
