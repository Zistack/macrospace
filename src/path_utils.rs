use syn::{Path, PathArguments, Token};

pub fn as_prefix (mut p: Path) -> Path
{
	if let Some (last_segment) = p . segments . last_mut ()
	{
		if let PathArguments::AngleBracketed (path_arguments) =
			&mut last_segment . arguments
		{
			path_arguments
				. colon2_token
				. get_or_insert (<Token! [::]>::default ());
		}
	}

	p
}

pub fn without_arguments (mut p: Path) -> Path
{
	if let Some (last_segment) = p . segments . last_mut ()
	{
		last_segment . arguments = PathArguments::None;
	}

	p
}
