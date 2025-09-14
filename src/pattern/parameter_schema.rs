use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use syn::Ident;

#[derive (Clone, Debug)]
pub struct ParameterSchema
{
	pub parameters: HashSet <Ident>,
	pub optional_parameters: Option <Box <ParameterSchema>>,
	pub zero_or_more_parameters: Option <Box <ParameterSchema>>,
	pub one_or_more_parameters: Option <Box <ParameterSchema>>
}

impl ParameterSchema
{
	pub fn new () -> Self
	{
		Self
		{
			parameters: HashSet::new (),
			optional_parameters: None,
			zero_or_more_parameters: None,
			one_or_more_parameters: None
		}
	}

	pub fn add_parameter (&mut self, ident: Ident)
	{
		self . parameters . insert (ident);
	}

	fn merge_nested_parameters
	(
		self_nested_parameters: &mut Option <Box <Self>>,
		other_nested_parameters: Option <Box <Self>>
	)
	{
		match (self_nested_parameters, other_nested_parameters)
		{
			(Some (self_boxed_parameters), Some (other_boxed_parameters)) =>
				self_boxed_parameters . merge (*other_boxed_parameters),
			(self_boxed_parameters @ _, Some (other_boxed_parameters)) =>
				*self_boxed_parameters = Some (other_boxed_parameters),
			_ => {}
		}
	}

	pub fn merge (&mut self, other: Self)
	{
		for other_parameter in other . parameters
		{
			self . parameters . insert (other_parameter);
		}

		Self::merge_nested_parameters
		(
			&mut self . optional_parameters,
			other . optional_parameters
		);
		Self::merge_nested_parameters
		(
			&mut self . zero_or_more_parameters,
			other . zero_or_more_parameters
		);
		Self::merge_nested_parameters
		(
			&mut self . one_or_more_parameters,
			other . one_or_more_parameters
		);
	}

	fn nested_parameters_are_empty (nested_parameters: &Option <Box <Self>>)
	-> bool
	{
		match nested_parameters
		{
			Some (boxed_parameter_schema) =>
				boxed_parameter_schema . is_empty (),
			None => true
		}
	}

	pub fn is_empty (&self) -> bool
	{
		self . parameters . is_empty ()
			&& Self::nested_parameters_are_empty (&self . optional_parameters)
			&& Self::nested_parameters_are_empty (&self . zero_or_more_parameters)
			&& Self::nested_parameters_are_empty (&self . one_or_more_parameters)
	}

	fn assert_nested_parameters_disjoint (nested_parameters: &Option <Box <Self>>)
	-> Result <Option <HashSet <Ident>>, ParameterUsedInIncompatibleRepetitions>
	{
		match nested_parameters
		{
			Some (boxed_parameter_schema) => Ok
			(
				Some (boxed_parameter_schema . assert_parameters_disjoint ()?)
			),
			None => Ok (None)
		}
	}

	fn assert_parameter_sets_disjoint (a: &HashSet <Ident>, b: &HashSet <Ident>)
	-> Result <(), ParameterUsedInIncompatibleRepetitions>
	{
		match a . intersection (b) . next ()
		{
			Some (ident) => Err
			(
				ParameterUsedInIncompatibleRepetitions::new (ident . clone ())
			),
			None => Ok (())
		}
	}

	pub fn assert_parameters_disjoint (&self)
	-> Result <HashSet <Ident>, ParameterUsedInIncompatibleRepetitions>
	{
		let mut disjoint_parameters = self . parameters . clone ();

		if let Some (optional_parameters) =
			Self::assert_nested_parameters_disjoint (&self . optional_parameters)?
		{
			Self::assert_parameter_sets_disjoint (&disjoint_parameters, &optional_parameters)?;
			disjoint_parameters . extend (optional_parameters);
		}

		if let Some (zero_or_more_parameters) =
			Self::assert_nested_parameters_disjoint (&self . zero_or_more_parameters)?
		{
			Self::assert_parameter_sets_disjoint (&disjoint_parameters, &zero_or_more_parameters)?;
			disjoint_parameters . extend (zero_or_more_parameters);
		}

		if let Some (one_or_more_parameters) =
			Self::assert_nested_parameters_disjoint (&self . one_or_more_parameters)?
		{
			Self::assert_parameter_sets_disjoint (&disjoint_parameters, &one_or_more_parameters)?;
			disjoint_parameters . extend (one_or_more_parameters);
		}

		Ok (disjoint_parameters)
	}

	fn nested_get_any_ident (nested_schema: &Option <Box <Self>>)
	-> Option <&Ident>
	{
		match nested_schema
		{
			Some (boxed_schema) => boxed_schema . get_any_ident (),
			None => None
		}
	}

	fn get_any_ident (&self) -> Option <&Ident>
	{
		if ! self . parameters . is_empty ()
		{
			Some (self . parameters . iter () . next () . unwrap ())
		}
		else if let Some (ident) =
			Self::nested_get_any_ident (&self . optional_parameters)
		{
			Some (ident)
		}
		else if let Some (ident) =
			Self::nested_get_any_ident (&self . zero_or_more_parameters)
		{
			Some (ident)
		}
		else if let Some (ident) =
			Self::nested_get_any_ident (&self . one_or_more_parameters)
		{
			Some (ident)
		}
		else
		{
			None
		}
	}

	fn assert_nested_superschema
	(
		self_nested_schema: &Option <Box <Self>>,
		other_nested_schema: &Option <Box <Self>>
	)
	-> Result <(), Ident>
	{
		match (self_nested_schema, other_nested_schema)
		{
			(Some (self_boxed_schema), Some (other_boxed_schema)) =>
				self_boxed_schema . assert_superschema (other_boxed_schema),
			(None, Some (other_boxed_schema)) => Err
			(
				other_boxed_schema . get_any_ident () . unwrap () . clone ()
			),
			_ => Ok (())
		}
	}

	pub fn assert_superschema (&self, other: &Self) -> Result <(), Ident>
	{
		for other_parameter in &other . parameters
		{
			if ! self . parameters . contains (other_parameter)
			{
				return Err (other_parameter . clone ());
			}
		}

		Self::assert_nested_superschema
		(
			&self . optional_parameters,
			&other . optional_parameters
		)?;
		Self::assert_nested_superschema
		(
			&self . zero_or_more_parameters,
			&other . zero_or_more_parameters
		)?;
		Self::assert_nested_superschema
		(
			&self . one_or_more_parameters,
			&other . one_or_more_parameters
		)?;

		Ok (())
	}

	pub fn assert_subschema (&self, other: &Self) -> Result <(), Ident>
	{
		other . assert_superschema (self)
	}
}

#[derive (Clone, Debug)]
pub struct ParameterUsedInIncompatibleRepetitions
{
	parameter: Ident
}

impl ParameterUsedInIncompatibleRepetitions
{
	fn new (parameter: Ident) -> Self
	{
		Self {parameter}
	}
}

impl Display for ParameterUsedInIncompatibleRepetitions
{
	fn fmt (&self, f: &mut Formatter <'_>) -> Result <(), std::fmt::Error>
	{
		f . write_fmt
		(
			format_args!
			(
				"Parameter `{}` used in incompatible repetitions",
				self . parameter
			)
		)
	}
}

impl Error for ParameterUsedInIncompatibleRepetitions
{
}

impl Into <syn::Error> for ParameterUsedInIncompatibleRepetitions
{
	fn into (self) -> syn::Error
	{
		syn::Error::new_spanned (&self . parameter, &self)
	}
}
