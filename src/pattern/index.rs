use std::fmt::{Display, Formatter};

use syn::{Ident, Token};
use syn_derive::{Parse, ToTokens};

#[derive (Clone, Debug, Parse, ToTokens)]
pub struct Index
{
	pub dollar_token: Token! [$],
	pub hash_token: Token! [#],
	pub ident: Ident
}

impl Display for Index
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt (format_args! ("$#{}", &self . ident))
	}
}
