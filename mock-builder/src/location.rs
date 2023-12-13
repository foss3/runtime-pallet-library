use frame_support::StorageHasher;

use super::util::TypeSignature;

/// Indicate how to perform the localtion hash
/// See `FunctionLocation::hash()`
#[derive(Clone, Copy, Debug)]
pub enum TraitInfo {
	/// Create hash with trait info, panics if it has not.
	Yes,

	/// Create hash with no trait info
	No,

	/// Create the hash with the trait info if it has trait info
	/// or not if it has none.
	Whatever,
}

/// Absolute string identification of function.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FunctionLocation {
	location: String,
	trait_info: Option<String>,
}

impl FunctionLocation {
	/// Creates a location for the function which created the given closure used
	/// as a locator
	pub fn from<F: Fn()>(_: F) -> Self {
		let location = std::any::type_name::<F>();
		let location = &location[..location.len() - "::{{closure}}".len()];

		// Remove generic attributes from signature if it has any
		let location = location
			.ends_with('>')
			.then(|| {
				let mut count = 0;
				for (i, c) in location.chars().rev().enumerate() {
					if c == '>' {
						count += 1;
					} else if c == '<' {
						count -= 1;
						if count == 0 {
							return location.split_at(location.len() - i - 1).0;
						}
					}
				}
				panic!("Expected '<' symbol to close '>'");
			})
			.unwrap_or(location);

		Self {
			location: location.into(),
			trait_info: Default::default(),
		}
	}

	/// Normalize the location, allowing to identify the function
	/// no matter if it belongs to a trait or not.
	pub fn normalize(self) -> Self {
		let (path, name) = self.location.rsplit_once("::").expect("always '::'");
		let (path, trait_info) = match path.strip_prefix('<') {
			Some(struct_as_trait_path) => {
				let struct_path = struct_as_trait_path
					.split_once(" as")
					.expect("always ' as'")
					.0;

				let trait_name = struct_as_trait_path
					.rsplit_once("::")
					.expect("always '::'")
					.1
					.strip_suffix('>')
					.unwrap();

				// Remove generic from trait name
				let trait_name = trait_name
					.split_once('<')
					.map(|(fst, _)| fst)
					.unwrap_or(trait_name);

				(struct_path, Some(trait_name.to_owned()))
			}
			None => (path, None),
		};

		Self {
			location: format!("{path}::{name}"),
			trait_info,
		}
	}

	/// Remove the prefix from the function name.
	pub fn strip_name_prefix(self, prefix: &str) -> Self {
		let (path, name) = self.location.rsplit_once("::").expect("always ::");
		let name = name.strip_prefix(prefix).unwrap_or_else(|| {
			panic!(
				"Function '{name}' should have a '{prefix}' prefix. Location: {}",
				self.location
			)
		});

		Self {
			location: format!("{path}::{name}"),
			trait_info: self.trait_info,
		}
	}

	/// Remove the trait name from the function name and add such information to
	/// the location. The location is expected to have the following structure:
	/// `<path>::<TraitInfo>_<name>`
	pub fn assimilate_trait_prefix(self) -> Self {
		let (path, name) = self.location.rsplit_once("::").expect("always ::");
		let (trait_info, name) = match name.chars().next().unwrap().is_uppercase() {
			true => {
				let (trait_info, name) = name.split_once('_').expect("always '_' after trait name");
				(Some(trait_info.to_owned()), name)
			}
			false => (None, name),
		};

		Self {
			location: format!("{path}::{name}"),
			trait_info,
		}
	}

	/// Add a representation of the function input and output types
	pub fn append_type_signature<I, O>(self) -> Self {
		Self {
			location: format!("{}:{}", self.location, TypeSignature::new::<I, O>()),
			trait_info: self.trait_info,
		}
	}

	/// Generate a hash of the location
	pub fn hash<Hasher: StorageHasher>(&self, trait_info: TraitInfo) -> Hasher::Output {
		let string = match trait_info {
			TraitInfo::Yes => {
				let trait_info = self
					.trait_info
					.as_ref()
					.expect("Location must have trait info");
				format!("{}{}", self.location, trait_info)
			}
			TraitInfo::No => self.location.clone(),
			TraitInfo::Whatever => {
				let trait_info = self.trait_info.clone().unwrap_or_default();
				format!("{}{}", self.location, trait_info)
			}
		};

		Hasher::hash(string.as_bytes())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const PREFIX: &str = "mock_builder::location::tests";

	trait TraitExample {
		fn method() -> FunctionLocation;
		fn generic_method<A: Into<i32>>(_: impl Into<u32>) -> FunctionLocation;
	}

	trait TraitExampleGen<G1, G2> {
		fn foo() -> FunctionLocation;
	}

	struct Example;

	impl Example {
		fn mock_method() -> FunctionLocation {
			FunctionLocation::from(|| ())
		}

		#[allow(non_snake_case)]
		fn mock_TraitExample_method() -> FunctionLocation {
			FunctionLocation::from(|| ())
		}

		fn mock_generic_method<A: Into<i32>>(_: impl Into<u32>) -> FunctionLocation {
			FunctionLocation::from(|| ())
		}
	}

	impl TraitExample for Example {
		fn method() -> FunctionLocation {
			FunctionLocation::from(|| ())
		}

		fn generic_method<A: Into<i32>>(_: impl Into<u32>) -> FunctionLocation {
			FunctionLocation::from(|| ())
		}
	}

	impl TraitExampleGen<u32, bool> for Example {
		fn foo() -> FunctionLocation {
			FunctionLocation::from(|| ())
		}
	}

	#[test]
	fn function_location() {
		assert_eq!(
			Example::mock_method(),
			FunctionLocation {
				location: format!("{PREFIX}::Example::mock_method"),
				trait_info: None,
			}
		);

		assert_eq!(
			Example::mock_TraitExample_method(),
			FunctionLocation {
				location: format!("{PREFIX}::Example::mock_TraitExample_method"),
				trait_info: None,
			}
		);

		assert_eq!(
			Example::mock_generic_method::<i8>(0u8),
			FunctionLocation {
				location: format!("{PREFIX}::Example::mock_generic_method"),
				trait_info: None,
			}
		);

		assert_eq!(
			Example::method(),
			FunctionLocation {
				location: format!("<{PREFIX}::Example as {PREFIX}::TraitExample>::method"),
				trait_info: None,
			}
		);

		assert_eq!(
			Example::generic_method::<i8>(0u8),
			FunctionLocation {
				location: format!("<{PREFIX}::Example as {PREFIX}::TraitExample>::generic_method"),
				trait_info: None,
			}
		);

		assert_eq!(
			Example::foo(),
			FunctionLocation {
				location: format!(
					"<{PREFIX}::Example as {PREFIX}::TraitExampleGen<u32, bool>>::foo"
				),
				trait_info: None,
			}
		);
	}

	#[test]
	fn normalized() {
		assert_eq!(
			Example::mock_method().normalize(),
			FunctionLocation {
				location: format!("{PREFIX}::Example::mock_method"),
				trait_info: None,
			}
		);

		assert_eq!(
			Example::mock_TraitExample_method().normalize(),
			FunctionLocation {
				location: format!("{PREFIX}::Example::mock_TraitExample_method"),
				trait_info: None,
			}
		);

		assert_eq!(
			Example::method().normalize(),
			FunctionLocation {
				location: format!("{PREFIX}::Example::method"),
				trait_info: Some("TraitExample".into()),
			}
		);

		assert_eq!(
			Example::foo().normalize(),
			FunctionLocation {
				location: format!("{PREFIX}::Example::foo"),
				trait_info: Some("TraitExampleGen".into()),
			}
		);
	}

	#[test]
	fn striped_name_prefix() {
		assert_eq!(
			Example::mock_method().strip_name_prefix("mock_"),
			FunctionLocation {
				location: format!("{PREFIX}::Example::method"),
				trait_info: None
			}
		);
	}

	#[test]
	fn assimilated_trait_prefix() {
		assert_eq!(
			Example::mock_method()
				.strip_name_prefix("mock_")
				.assimilate_trait_prefix(),
			FunctionLocation {
				location: format!("{PREFIX}::Example::method"),
				trait_info: None
			}
		);

		assert_eq!(
			Example::mock_TraitExample_method()
				.strip_name_prefix("mock_")
				.assimilate_trait_prefix(),
			FunctionLocation {
				location: format!("{PREFIX}::Example::method"),
				trait_info: Some("TraitExample".into()),
			}
		);
	}

	#[test]
	fn appended_type_signature() {
		assert_eq!(
			Example::mock_method().append_type_signature::<i8, u8>(),
			FunctionLocation {
				location: format!("{PREFIX}::Example::mock_method:i8->u8"),
				trait_info: None,
			}
		);
	}
}
