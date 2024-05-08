pub trait TraitA {
	fn foo(p1: String, p2: Option<u64>);
	fn bar(p1: u64, p2: bool) -> Result<(), String>;
	fn same_name(p1: bool, p2: i32) -> usize;
}

pub trait TraitB {
	fn qux(p1: String) -> bool;
	fn generic_input<A: Into<i32>>(a: A, b: impl Into<u32>) -> usize;
	fn generic_output<A: Into<i32>>() -> A;
	fn reference(a: &i32) -> &i32;
	fn same_name(p1: i32) -> bool;
}

pub trait TraitGen<A> {
	fn generic() -> u32;
}

pub trait Storage {
	fn set(value: i32);
	fn get() -> i32;
}

#[frame_support::pallet(dev_mode)]
pub mod pallet_mock_test {
	use frame_support::pallet_prelude::*;
	use mock_builder::{execute_call, register_call};

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	type CallIds<T: Config> = StorageMap<_, _, String, mock_builder::CallId>;

	impl<T: Config> Pallet<T> {
		pub fn mock_foo(f: impl Fn(String, Option<u64>) + 'static) {
			register_call!(move |(a, b)| f(a, b));
		}

		pub fn mock_bar(f: impl Fn(u64, bool) -> Result<(), String> + 'static) {
			register_call!(move |(a, b)| f(a, b));
		}

		pub fn mock_qux(f: impl Fn(String) -> bool + 'static) {
			register_call!(f);
		}

		pub fn mock_generic_input<A: Into<i32>, B: Into<u32>>(f: impl Fn(A, B) -> usize + 'static) {
			register_call!(move |(a, b)| f(a, b));
		}

		pub fn mock_generic_output<A: Into<i32>>(f: impl Fn() -> A + 'static) {
			register_call!(move |()| f());
		}

		pub fn mock_reference(f: impl Fn(&i32) -> &i32 + 'static) {
			register_call!(f);
		}

		pub fn mock_set(f: impl Fn(i32) + 'static) {
			register_call!(f);
		}

		pub fn mock_get(f: impl Fn() -> i32 + 'static) {
			register_call!(move |()| f());
		}

		#[allow(non_snake_case)]
		pub fn mock_TraitA_same_name(f: impl Fn(bool, i32) -> usize + 'static) {
			register_call!(move |(a, b)| f(a, b));
		}

		#[allow(non_snake_case)]
		pub fn mock_TraitB_same_name(f: impl Fn(i32) -> bool + 'static) {
			register_call!(f);
		}

		#[allow(non_snake_case)]
		pub fn mock_TraitGen_generic(f: impl Fn() -> u32 + 'static) {
			register_call!(move |()| f());
		}
	}

	impl<T: Config> super::TraitA for Pallet<T> {
		fn foo(a: String, b: Option<u64>) {
			execute_call!((a, b))
		}

		fn bar(a: u64, b: bool) -> Result<(), String> {
			execute_call!((a, b))
		}

		fn same_name(a: bool, b: i32) -> usize {
			execute_call!((a, b))
		}
	}

	impl<T: Config> super::TraitB for Pallet<T> {
		fn qux(a: String) -> bool {
			execute_call!(a)
		}

		fn generic_input<A: Into<i32>>(a: A, b: impl Into<u32>) -> usize {
			execute_call!((a, b))
		}

		fn generic_output<A: Into<i32>>() -> A {
			execute_call!(())
		}

		fn reference(a: &i32) -> &i32 {
			execute_call!(a)
		}

		fn same_name(a: i32) -> bool {
			execute_call!(a)
		}
	}

	impl<T: Config> super::TraitGen<T::AccountId> for Pallet<T> {
		fn generic() -> u32 {
			execute_call!(())
		}
	}

	impl<T: Config> super::Storage for Pallet<T> {
		fn set(a: i32) {
			execute_call!(a)
		}

		fn get() -> i32 {
			execute_call!(())
		}
	}
}

#[frame_support::pallet]
pub mod my_pallet {
	use super::{TraitA, TraitB};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type ActionAB: TraitA + TraitB;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	impl<T: Config> Pallet<T> {
		pub fn my_call(name: &str, value: u64) -> Result<(), String> {
			T::ActionAB::foo(name.into(), Some(value));
			let answer = T::ActionAB::qux(name.into());
			T::ActionAB::bar(value, answer)
		}
	}
}

mod mock {
	use frame_support::{
		derive_impl,
		traits::{ConstU16, ConstU32, ConstU64},
	};
	use sp_core::H256;
	use sp_runtime::traits::{BlakeTwo256, IdentityLookup};

	use super::{my_pallet, pallet_mock_test};

	frame_support::construct_runtime!(
		pub struct Runtime {
			System: frame_system,
			MockTest: pallet_mock_test,
			MyPallet: my_pallet,
		}
	);

	#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
	impl frame_system::Config for Runtime {
		type Block = frame_system::mocking::MockBlock<Runtime>;
	}

	impl pallet_mock_test::Config for Runtime {}

	impl my_pallet::Config for Runtime {
		type ActionAB = pallet_mock_test::Pallet<Runtime>;
	}
}

mod test {
	use frame_support::assert_ok;

	use super::{mock::*, Storage, TraitA, TraitB, TraitGen};

	#[test]
	fn basic() {
		System::externalities().execute_with(|| {
			MockTest::mock_qux(|p1| &p1 == "hello");

			assert_eq!(MockTest::qux("hello".into()), true);
		});
	}

	#[test]
	fn correct_flow() {
		System::externalities().execute_with(|| {
			MockTest::mock_foo(|p1, _| assert_eq!("hello", &p1));
			MockTest::mock_qux(|p1| &p1 == "hello");
			MockTest::mock_bar(|_, p2| match p2 {
				true => Ok(()),
				false => Err("err".into()),
			});

			assert_ok!(MyPallet::my_call("hello".into(), 42));
		});
	}

	#[test]
	#[should_panic]
	fn wrong() {
		System::externalities().execute_with(|| {
			MockTest::mock_foo(|p1, _| assert_eq!("hello", &p1));

			assert_ok!(MyPallet::my_call("bye".into(), 42));
		});
	}

	#[test]
	#[should_panic]
	fn mock_not_configured() {
		System::externalities().execute_with(|| {
			assert_ok!(MyPallet::my_call("hello".into(), 42));
		});
	}

	#[test]
	#[should_panic]
	fn previous_mock_data_is_destroyed() {
		correct_flow();
		// The storage is dropped at this time. Mocks no longer found from here.
		mock_not_configured();
	}

	#[test]
	fn generic_input() {
		System::externalities().execute_with(|| {
			MockTest::mock_generic_input(|p1: i8, p2: u8| {
				assert_eq!(p1, 1);
				assert_eq!(p2, 2);
				8
			});
			MockTest::mock_generic_input(|p1: i16, p2: u16| {
				assert_eq!(p1, 3);
				assert_eq!(p2, 4);
				16
			});

			assert_eq!(MockTest::generic_input(1i8, 2u8), 8);
			assert_eq!(MockTest::generic_input(3i16, 4u16), 16);
		});
	}

	#[test]
	#[should_panic]
	fn generic_input_not_found() {
		System::externalities().execute_with(|| {
			MockTest::mock_generic_input(|p1: i8, p2: u8| {
				assert_eq!(p1, 3);
				assert_eq!(p2, 4);
				8
			});

			MockTest::generic_input(3i16, 4u16);
		});
	}

	#[test]
	fn generic_output() {
		System::externalities().execute_with(|| {
			MockTest::mock_generic_output(|| 8i8);
			MockTest::mock_generic_output(|| 16i16);

			assert_eq!(MockTest::generic_output::<i8>(), 8);
			assert_eq!(MockTest::generic_output::<i16>(), 16);
		});
	}

	#[test]
	fn reference() {
		System::externalities().execute_with(|| {
			MockTest::mock_reference(|a| a);

			assert_eq!(MockTest::reference(&42), &42);
		});
	}

	#[test]
	fn get_last_set() {
		System::externalities().execute_with(|| {
			MockTest::mock_set(|v| MockTest::mock_get(move || v));

			MockTest::set(23);
			assert_eq!(MockTest::get(), 23);

			MockTest::set(42);
			assert_eq!(MockTest::get(), 42);
		});
	}

	#[test]
	fn method_with_same_name() {
		System::externalities().execute_with(|| {
			MockTest::mock_TraitA_same_name(|a, b| {
				assert_eq!(a, true);
				assert_eq!(b, 42);
				23
			});
			MockTest::mock_TraitB_same_name(|a| {
				assert_eq!(a, 23);
				true
			});

			assert_eq!(<MockTest as TraitA>::same_name(true, 42), 23);
			assert_eq!(<MockTest as TraitB>::same_name(23), true);
		});
	}

	#[test]
	fn method_from_generic_trait_long_path() {
		System::externalities().execute_with(|| {
			MockTest::mock_TraitGen_generic(|| 23);

			assert_eq!(MockTest::generic(), 23);
		});
	}
}
