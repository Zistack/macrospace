use syn::{Type, Fields, Member};
use quote::{ToTokens, quote};

pub fn constructor <T>
(
	ty: &Type,
	members: &Vec <Member>,
	values: &Vec <T>
)
-> proc_macro2::TokenStream
where T: ToTokens
{
	match members . get (0)
	{
		None => quote! (#ty),
		Some (Member::Named (_)) => quote! (#ty {#(#members: #values),*}),
		Some (Member::Unnamed (_)) => quote! (#ty (#(#values),*))
	}
}

pub fn get_member_types (fields: &Fields) -> Vec <(Member, Type)>
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
