use proc_macro2::TokenStream;

pub trait DummyTokens
{
	fn dummy_tokens (&self) -> TokenStream;
}
