// Copyright 2023 Centrifuge Foundation (centrifuge.io).
// This file is part of Centrifuge chain project.

// Centrifuge is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version (see http://www.gnu.org/licenses).

// Centrifuge is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

//! `mock-builder` allows you to mock traits in Substrate environments.
//! It does it by helping you creating types that implements those traits with
//! mocked methods. Using them for testing allow you to customize each test
//! case, getting organized and accurate tests for your pallet.
//!
//! # Motivation
//!
//! Pallets have dependencies. Programming in a
//! [loosely coupled](https://docs.substrate.io/build/pallet-coupling)
//! way is great for getting rid of those dependencies for the implementation.
//! Nevertheless, those dependencies still exist in testing because when the
//! `mock.rs` file is defined, you're forced to give some implementations for
//! the associated types of your pallet `Config`.
//!
//! Then, you are mostly forced to use other pallet configurations
//! getting a [tight coupling](https://docs.substrate.io/build/pallet-coupling/)
//! with them. It has some downsides:
//! - You need to learn how to configure other pallets.
//! - You need to know how those pallets work, because they affect directly the
//!   behavior of the pallet you're testing.
//! - The way they work can give you non-completed tests. It means that some
//!   paths of your pallet can not be tested because some dependency works in a
//!   specific way.
//! - You need a lot of effort maintaining your tests because each time one
//!   dependency changes, it can easily break your tests.
//!
//! This doesn't scale well. Frequently some pallet dependencies need in turn to
//! configure their own dependent pallets, making this problem even worse.
//!
//! This is why mocking is so important. It lets you get rid of all these
//! dependencies and related issues, obtaining **loose coupling tests**.
//!
//! There are other crates focusing on this problem,
//! such as [`mockall`](https://docs.rs/mockall/latest/mockall/),
//! but they do not know about the Substrate storage life cycle used by
//! your pallets. This crate gives you a mock type ready to use in your current
//! tests, without worring about setting up or freeing any inner state used by
//! the mock.
//!
//! ## Usage
//!
//! Suppose that in our pallet, which we'll call it `my_pallet`, we have an
//! associated type in our `Config`, called `AB`, which implements traits
//! `TraitA` and `TraitB`. Those traits are defined as follows:
//!
//! ```
//! trait TraitA {
//!     type AssocA;
//!
//!     fn foo() -> Self::AssocA;
//! }
//!
//! trait TraitB {
//!     type AssocB;
//!
//!     fn bar(a: u64, b: Self::AssocB) -> u32;
//! }
//! ```
//!
//! We have a really huge pallet that implements a specific behavior for those
//! traits, but we want to get rid of such dependency so we
//! [generate a mock](#mock-type-creation), and we configure it:
//!
//! ```ignore
//! pub type MyMock = my_mock::MyMock<Runtime>;
//! impl my_mock::Config for Runtime {
//!     type AssocA = bool;
//!     type AssocB = u8;
//! }
//!
//! impl my_pallet::Config for Runtime {
//!     type AB = MyMock // Tell our pallet we want to use this mock.
//! }
//! ```
//!
//! Later in our use case, we can give a behavior for both `foo()` and `bar()`
//! methods in their analogous methods `mock_foo()` and `mock_bar()` which
//! accept a closure.
//!
//! ```ignore
//! #[test]
//! fn correct() {
//!     new_test_ext().execute_with(|| {
//!         MockDep::mock_foo(|| true);
//!         MockDep::mock_bar(|a, b| {
//!             assert_eq!(a, 42);
//!             assert_eq!(b, false);
//!             23
//!         });
//!
//!         // This method will call foo() and bar() under the hood, running the
//!         // closures we just have defined.
//!         MyPallet::my_call();
//!     });
//! }
//! ```
//!
//! Take a look to the [pallet
//! tests](https://github.com/foss3/runtime-pallet-library/blob/main/mock-builder/tests/pallet.rs)
//! to have a user view of how to use a *mock pallet*.
//! It supports any kind of trait, with reference
//! parameters and generics at trait level and method level.
//!
//! ## Mock type creation
//!
//! **NOTE: There is a working progress on this part to generate *mock types*
//! automatically using procedural macros. Once done, all this part could be
//! auto-generated.**
//!
//! This crate exports two macros [`register_call!()`] and [`execute_call!()`]
//! that allow you to build a *mock pallet*.
//!
//! - [`register_call!()`] registers a closure where you can define the
//! mock behavior for that method. The method which registers the closure must
//! have the name of the trait method you want to mock prefixed with `mock_`.
//!
//! - [`execute_call!()`] is placed in the trait method implementation and will
//!   call the closure previously registered by [`register_call!()`]
//!
//! Following the above example, generating a *mock pallet* for both `TraitA`
//! and `TraitB` is done as follows:
//! ```
//! # trait TraitA {
//! #     type AssocA;
//! #
//! #     fn foo() -> Self::AssocA;
//! # }
//! #
//! # trait TraitB {
//! #     type AssocB;
//! #
//! #     fn bar(a: u64, b: Self::AssocB) -> u32;
//! # }
//!
//! use mock_builder::{execute_call, register_call};
//!
//! // This trait is optional, but usually you would need a considerable
//! // number of types for your mock. Follows this pattern to configure things
//! // can help you managing generics and scales better for new additions.
//! pub trait Config {
//!     type AssocA;
//!     type AssocB;
//! }
//!
//! // You can also add extra types to create different types.
//! // Similar to the Substrate `pallet::Instance`
//! pub struct Mock<T>(std::marker::PhantomData<T>);
//!
//! impl<T: Config> Mock<T> {
//!     fn mock_foo(f: impl Fn() -> T::AssocA + 'static) {
//!         register_call!(move |()| f())
//!     }
//!
//!     fn mock_bar(f: impl Fn(u64, T::AssocB) -> u32 + 'static) {
//!         register_call!(move |(a, b)| f(a, b))
//!     }
//! }
//!
//! impl<T: Config> TraitA for Mock<T> {
//!     type AssocA = T::AssocA;
//!
//!     fn foo() -> Self::AssocA {
//!         execute_call!(())
//!     }
//! }
//!
//! impl<T: Config> TraitB for Mock<T> {
//!     type AssocB = T::AssocB;
//!
//!     fn bar(a: u64, b: Self::AssocB) -> u32 {
//!         execute_call!((a, b))
//!     }
//! }
//! ```
//!
//! If types for the closure of `mock_*` method and trait method don't match,
//! you will obtain a runtime error in your tests.
//!
//! ## Mock Patterns
//!
//! #### Storage pattern
//! In some cases it's pretty common making a mock that returns a value that was
//! set previously by another mock. For this case you can define your "getter"
//! mock inside the definition of the "setter" mock, as follows:
//! ```ignore
//! MyMock::mock_set(|value| MyMock::mock_get(move || value));
//! ```
//!
//! Any call to `get()` will return the last value given to `set()`.
//!
//! #### Check internal calls are ordered
//! If you want to test some mock methods are called in some order, you can
//! define them nested. They must be called in that expected order.
//! ```ignore
//! MyMock::mock_first(|| {
//!     MyMock::mock_second(|| {
//!         MyMock::mock_third(|| {
//!             //...
//!         })
//!     })
//! });
//!
//!
//! // The next method only will be succesful
//! // if it makes the internal calls in order
//! MyPallet::calls_first_then_second_then_third();
//! ```

/// Provide functions for register/execute calls
pub mod storage;

/// Provide functions for handle fuction locations
pub mod location;

mod util;

use location::{FunctionLocation, TraitInfo};
pub use storage::CallId;

/// Prefix that the register functions should have.
pub const MOCK_FN_PREFIX: &str = "mock_";

const HELP_MSG: &str = "Be sure your mock_<method> matches your trait <method> name and type.";

/// Register a mock function into the mock function storage.
/// This function should be called with a locator used as a function
/// identification.
pub fn register<Locator, F, I, O, Insert>(locator: Locator, f: F, insert: Insert)
where
	Locator: Fn(),
	F: Fn(I) -> O + 'static,
	Insert: Fn(&[u8], &CallId),
{
	let location = FunctionLocation::from(locator)
		.normalize()
		.strip_name_prefix(MOCK_FN_PREFIX)
		.assimilate_trait_prefix()
		.append_type_signature::<I, O>();

	insert(
		location.get(TraitInfo::Whatever).as_bytes(),
		&storage::register_call(f),
	)
}

/// Execute a function from the function storage.
/// This function should be called with a locator used as a function
/// identification.
pub fn execute<Locator, I, O, Get>(locator: Locator, input: I, get: Get) -> O
where
	Locator: Fn(),
	Get: Fn(&[u8]) -> Option<CallId>,
{
	let location = FunctionLocation::from(locator)
		.normalize()
		.append_type_signature::<I, O>();

	let call_id = get(location.get(TraitInfo::Yes).as_bytes())
		.or_else(|| get(location.get(TraitInfo::No).as_bytes()))
		.unwrap_or_else(|| {
			panic!("mock-builder ERROR: Mock was not found at: {location:#?}\n{HELP_MSG}")
		});

	storage::execute_call(call_id, input).unwrap_or_else(|err| {
		panic!("mock-builder ERROR: {err} at: {location:#?}\n{HELP_MSG}");
	})
}

/// Register a mock function into the mock function storage.
/// Same as `register()` but it uses as locator who calls this macro.
#[macro_export]
macro_rules! register_call {
	($f:expr) => {{
		$crate::register(|| (), $f, frame_support::storage::unhashed::put);
	}};
}

/// Execute a function from the function storage.
/// Same as `execute()` but it uses as locator who calls this macro.
#[macro_export]
macro_rules! execute_call {
	($input:expr) => {{
		$crate::execute(|| (), $input, frame_support::storage::unhashed::get)
	}};
}
