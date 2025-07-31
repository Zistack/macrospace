use proc_macro2::TokenStream;

pub trait DummyTokens <P>
{
	fn dummy_tokens (paramer: &P) -> TokenStream;
}
