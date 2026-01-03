use syn::{Path, Type, Fields, Member};
use quote::{ToTokens, quote};

use crate::path_utils::as_prefix;

pub fn constructor <T>
(
	ty_path: &Path,
	members: &Vec <Member>,
	values: &Vec <T>
)
-> proc_macro2::TokenStream
where T: ToTokens
{
	let ty_path = as_prefix (ty_path . clone ());

	match members . get (0)
	{
		None => quote! (#ty_path),
		Some (Member::Named (_)) => quote! (#ty_path {#(#members: #values),*}),
		Some (Member::Unnamed (_)) => quote! (#ty_path (#(#values),*))
	}
}

pub fn get_members_and_types (fields: &Fields) -> Vec <(Member, Type)>
{
	match fields
	{
		Fields::Named (named) => named
			. named
			. iter ()
			. map
			(
				|field|
				(
					Member::from (field . ident . clone () . unwrap ()),
					field . ty . clone ()
				)
			)
			. collect (),
		Fields::Unnamed (unnamed) => unnamed
			. unnamed
			. iter ()
			. enumerate ()
			. map (|(i, field)| (Member::from (i), field . ty . clone ()))
			. collect (),
		Fields::Unit => Vec::new ()
	}
}

pub fn get_members_and_types_split (fields: &Fields) -> (Vec <Member>, Vec <Type>)
{
	let mut members = Vec::new ();
	let mut types = Vec::new ();

	match fields
	{
		Fields::Named (named) => for field in &named . named
		{
			members . push (Member::from (field . ident . clone () . unwrap ()));
			types . push (field . ty . clone ());
		},
		Fields::Unnamed (unnamed) => for (i, field) in unnamed . unnamed . iter () . enumerate ()
		{
			members . push (Member::from (i));
			types . push (field . ty . clone ());
		}
		_ => {}
	}

	(members, types)
}

pub fn get_members (fields: &Fields) -> Vec <Member>
{
	fields . members () . collect ()
}

pub fn get_member_types (fields: &Fields) -> Vec <Type>
{
	fields . iter () . map (|field| field . ty . clone ()) . collect ()
}
