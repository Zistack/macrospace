use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::repeat;

use syn
::{
	Lifetime,
	Type,
	Expr,
	PathArguments,
	GenericArgument,
	GenericParam,
	TypeParam,
	ConstParam,
	Token,
	parse_quote
};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn::fold
::{
	Fold,
	fold_lifetime,
	fold_type,
	fold_type_param,
	fold_expr,
	fold_const_param
};

use crate::generics::get_num_required_arguments;

use super::parameter::Parameter;
use super::argument::Argument;

#[derive (Clone, Debug)]
pub struct ArgumentMismatchError
{
	parameter: Parameter,
	self_argument: Argument,
	other_argument: Argument
}

impl Display for ArgumentMismatchError
{
	fn fmt (&self, f: &mut Formatter <'_>)
	-> std::result::Result <(), std::fmt::Error>
	{
		f . write_fmt
		(
			format_args!
			(
				"arguments for parameter {} do not match: {} != {}",
				self . parameter,
				self . self_argument,
				self . other_argument
			)
		)
	}
}

impl std::error::Error for ArgumentMismatchError
{
}

#[derive (Clone, PartialEq, Eq)]
pub struct Substitutions
{
	pub parameters: HashMap <Parameter, Argument>
}

impl Substitutions
{
	pub fn new () -> Self
	{
		Self {parameters: HashMap::new ()}
	}

	pub fn try_from_generics
	(
		item_parameters: &Punctuated <GenericParam, Token! [,]>,
		path_arguments: &Punctuated <GenericArgument, Token! [,]>
	)
	-> Result <Self>
	{
		let num_available_arguments = item_parameters . len ();
		let num_required_arguments =
			get_num_required_arguments (&item_parameters);
		let num_provided_arguments = path_arguments . len ();

		if num_provided_arguments < num_required_arguments
		{
			return Err
			(
				Error::new_spanned
				(
					path_arguments,
					format!
					(
						"Item requires {} arguments, {} were provided",
						num_required_arguments,
						num_provided_arguments
					)
				)
			);
		}

		if num_provided_arguments > num_available_arguments
		{
			return Err
			(
				Error::new_spanned
				(
					path_arguments,
					format!
					(
						"Item only takes {} arguments, {} were provided",
						num_available_arguments,
						num_provided_arguments
					)
				)
			);
		}

		let mut substitutions = Self::default ();

		for (item_parameter, maybe_path_argument)
		in item_parameters
			. iter ()
			. cloned ()
			. zip
			(
				path_arguments
					. into_iter ()
					. map (Option::from)
					. chain (repeat (None))
			)
		{
			if let Some (path_argument) = maybe_path_argument
			{
				substitutions . parameters . insert
				(
					Parameter::from (item_parameter),
					Argument::try_from (path_argument)?
				);
			}
			else
			{
				substitutions . parameters . insert
				(
					Parameter::from (item_parameter . clone ()),

					// If someone somehow manages to mix the parameters in silly
					// ways, attempting to pull the default arguments could
					// still fail.
					Argument::try_from_default_value (item_parameter)?
				);
			}
		}

		// In the event that the default arguments contain references to other
		// generic parameters, we've got to substitute in all of those values
		// properly.  This could theoretically take an unbounded number of
		// steps, though most of the time it should take about 1 in practice,
		// with 1 more to verify that there are no more substitutions needed.
		const MAX_ITERATIONS: usize = 100;
		let mut num_iterations = 0;
		loop
		{
			let new_substitutions = substitutions
				. fold_substitutions (substitutions . clone ());

			num_iterations += 1;

			if new_substitutions == substitutions
			{
				return Ok (substitutions);
			}

			if num_iterations >= MAX_ITERATIONS
			{
				return Err
				(
					Error::new_spanned
					(
						item_parameters,
						"Iteration limit reached evaluating default arguments"
					)
				);
			}

			substitutions = new_substitutions;
		}
	}

	pub fn scrubber
	(
		prefix: &str,
		generic_parameters: &Punctuated <GenericParam, Token! [,]>
	)
	-> Self
	{
		let mut parameters = HashMap::new ();

		for generic_param in generic_parameters
		{
			let parameter = Parameter::from (generic_param . clone ());
			let argument = Argument::hygenic_from_parameter
			(
				prefix,
				generic_param . clone ()
			);

			parameters . insert (parameter, argument);
		}

		Self {parameters}
	}

	pub fn try_merge (mut self, other: Self)
	-> std::result::Result <Self, ArgumentMismatchError>
	{
		for (other_parameter, other_argument) in other . parameters
		{
			let self_argument = self . parameters . get (&other_parameter);

			match self_argument
			{
				Some (argument) => if *argument != other_argument
				{
					return Err
					(
						ArgumentMismatchError
						{
							parameter: other_parameter,
							self_argument: argument . clone (),
							other_argument
						}
					);
				},
				None =>
				{
					self
						. parameters
						. insert (other_parameter, other_argument);
				}
			}
		}

		Ok (self)
	}

	pub fn fold_parameter_value (&mut self, node: Argument) -> Argument
	{
		match node
		{
			Argument::Lifetime (lifetime) =>
				Argument::Lifetime (self . fold_lifetime (lifetime)),
			Argument::Type (ty) =>
				Argument::Type (self . fold_type (ty)),
			Argument::Const (expr) =>
				Argument::Const (self . fold_expr (expr))
		}
	}

	pub fn fold_substitutions (&mut self, node: Substitutions) -> Substitutions
	{
		Substitutions
		{
			parameters: node
				. parameters
				. into_iter ()
				. map (|(info, value)| (info, self . fold_parameter_value (value)))
				. collect ()
		}
	}
}

impl Default for Substitutions
{
	fn default () -> Self
	{
		Self::new ()
	}
}

impl FromIterator <(Parameter, Argument)> for Substitutions
{
	fn from_iter <I> (iter: I) -> Self
	where I: IntoIterator <Item = (Parameter, Argument)>
	{
		Self {parameters: HashMap::from_iter (iter)}
	}
}

macro_rules! make_type_key
{
	($ident: expr) => { &Parameter::Type ($ident) }
}

macro_rules! make_const_key
{
	($ident: expr) =>
	{
		&Parameter::Const (<Token! [const]>::default (), $ident)
	}
}

macro_rules! fold_qpath
{
	($fold_qpath: ident, $QPath: ident, $PVariant: ident, $make_key: ident) =>
	{
		fn $fold_qpath (&mut self, node: $QPath) -> $QPath
		{
			if let $QPath::Path (qpath) = &node
			{
				if qpath . qself . is_none ()
				{
					if let Some (ident) = qpath . path . get_ident ()
					{
						if let Some (Argument::$PVariant (ty)) = self
							. parameters
							. get ($make_key! (ident . clone ()))
						{
							return ty . clone ();
						}
					}
				}

				if let Some (first_segment) = qpath . path . segments . first ()
				{
					if let PathArguments::None = first_segment . arguments
					{
						let maybe_parameter_value = self
							. parameters
							. get ($make_key! (first_segment . ident . clone ()))
							. cloned ();

						if let Some (Argument::$PVariant (ty)) =
							maybe_parameter_value
						{
							let tail_segments = qpath
								. path
								. segments
								. iter ()
								. skip (1)
								. cloned ()
								. map (|segment| self . fold_path_segment (segment));

							return parse_quote! (<#ty>#(::#tail_segments)*);
						}
					}
				}
			}

			$fold_qpath (self, node)
		}
	}
}

impl Fold for Substitutions
{
	fn fold_lifetime (&mut self, node: Lifetime) -> Lifetime
	{
		if let Some (Argument::Lifetime (lifetime)) =
			self . parameters . get (&Parameter::Lifetime (node . clone ()))
		{
			return lifetime . clone ();
		}

		fold_lifetime (self, node)
	}

	fold_qpath! (fold_type, Type, Type, make_type_key);

	fn fold_type_param (&mut self, node: TypeParam) -> TypeParam
	{
		if let Some (Argument::Type (ty)) = self
			. parameters
			. get (&Parameter::Type (node . ident . clone ()))
		{
			if let Type::Path (type_path) = ty
			{
				if type_path . qself . is_none ()
				{
					if let Some (ident) = type_path . path . get_ident ()
					{
						return TypeParam
						{
							attrs: node . attrs,
							ident: ident . clone (),
							colon_token: node . colon_token,
							bounds: node
								. bounds
								. into_iter ()
								. map (|bound| self . fold_type_param_bound (bound))
								. collect (),
							eq_token: node . eq_token,
							default: node
								. default
								. map (|ty| self . fold_type (ty))
						};
					}
				}
			}
		}

		fold_type_param (self, node)
	}

	fold_qpath! (fold_expr, Expr, Const, make_const_key);

	fn fold_const_param (&mut self, node: ConstParam) -> ConstParam
	{
		if let Some (Argument::Const (expr)) = self
			. parameters
			. get
			(
				&Parameter::Const
				(
					<Token! [const]>::default (),
					node . ident . clone ()
				)
			)
		{
			if let Expr::Path (expr_path) = expr
			{
				if expr_path . qself . is_none ()
				{
					if let Some (ident) = expr_path . path . get_ident ()
					{
						return ConstParam
						{
							attrs: node . attrs,
							const_token: node . const_token,
							ident: ident . clone (),
							colon_token: node . colon_token,
							ty: self . fold_type (node . ty),
							eq_token: node . eq_token,
							default: node
								. default
								. map (|expr| self . fold_expr (expr))
						};
					}
				}
			}
		}

		fold_const_param (self, node)
	}
}
