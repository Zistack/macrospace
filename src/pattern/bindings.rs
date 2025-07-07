use proc_macro2::TokenStream;
use syn::parse::ParseStream;

pub trait MatchBindings <P>
{
	type Error;

	fn parse_parameter_binding
	(
		&mut self,
		parameter: P,
		input: ParseStream <'_>
	)
	-> Result <(), Self::Error>;

	fn try_merge (&mut self, other: Self) -> Result <(), Self::Error>;
}

pub trait SubstitutionBindings <P>
{
	type Error;

	fn write_parameter_tokens (&self, parameter: P, tokens: &mut TokenStream)
	-> Result <(), Self::Error>;
}
