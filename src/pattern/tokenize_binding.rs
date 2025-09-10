use proc_macro2::TokenStream;
use syn::Ident;

pub trait TokenizeBinding <V>
{
	type Error;

	fn tokenize (&self, ident: &Ident, binding: &V, tokens: &mut TokenStream)
	-> Result <(), Self::Error>;
}
