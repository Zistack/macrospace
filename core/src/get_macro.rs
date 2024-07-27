use syn::{Ident, Path};
use quote::format_ident;

pub fn get_macro_ident (ident: &Ident) -> Ident
{
	format_ident! ("macrospace_apply_{}", ident)
}

pub fn get_macro_path (path: &Path) -> Path
{
	let mut macro_path = path . clone ();

	if let Some (segment) = macro_path . segments . last_mut ()
	{
		segment . ident = get_macro_ident (&segment . ident);
	}

	macro_path
}
