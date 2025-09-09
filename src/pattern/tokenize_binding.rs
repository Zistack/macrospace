use proc_macro2::TokenStream;

pub trait TokenizeBinding <V>
{
	type Error;

	fn tokenize_binding (&self, binding: &V, tokens: &mut TokenStream)
	-> Result <(), Self::Error>;
}
