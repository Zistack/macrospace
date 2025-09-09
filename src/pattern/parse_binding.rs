use syn::parse::{ParseStream, Result};

pub trait ParseBinding <V>
{
	fn parse (&self, input: ParseStream <'_>) -> Result <V>;
}
