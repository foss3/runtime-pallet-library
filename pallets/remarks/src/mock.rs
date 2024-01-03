// Copyright 2023 Centrifuge Foundation (centrifuge.io).
//
// This file is part of the Centrifuge chain project.
// Centrifuge is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version (see http://www.gnu.org/licenses).
// Centrifuge is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
use frame_support::{construct_runtime, pallet_prelude::ConstU32, parameter_types, BoundedVec};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

pub use crate as pallet_remarks;
use crate::pallet::Config;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Storage, Event<T>},
		RemarkDispatchHandlerMock: pallet_mock_test,
		Remarks: pallet_remarks::{Pallet, Call, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Event},
	}
);

// System config
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = u64;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = SS58Prefix;
	type SystemWeightInfo = ();
	type Version = ();
}

pub type Balance = u128;

parameter_types! {
	pub const MaxRemarksPerCall: u32 = 3;
}

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum TestRemark {
	SomeId(u32),
	SomeBytes(BoundedVec<u8, ConstU32<16>>),
}

impl Default for TestRemark {
	fn default() -> Self {
		TestRemark::SomeBytes(
			BoundedVec::try_from(vec![1, 2, 3]).expect("can build default test remark"),
		)
	}
}

#[allow(unused_imports)]
#[frame_support::pallet]
mod pallet_mock_test {
	use frame_support::pallet_prelude::*;
	use mock_builder::{execute_call, register_call};

	use crate::{RemarkArgs, RemarkDispatchHandler};

	#[pallet::config]
	pub trait Config: frame_system::Config + crate::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type CallIds<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		<Blake2_128 as frame_support::StorageHasher>::Output,
		mock_builder::CallId,
	>;

	impl<T: Config> Pallet<T> {
		pub fn mock_pre_dispatch_check(f: impl Fn(RemarkArgs<T>) -> DispatchResult + 'static) {
			register_call!(move |t| f(t));
		}

		pub fn mock_post_dispatch_check(f: impl Fn(RemarkArgs<T>) -> DispatchResult + 'static) {
			register_call!(move |t| f(t));
		}
	}

	impl<T: Config> RemarkDispatchHandler<RemarkArgs<T>> for Pallet<T> {
		fn pre_dispatch_check(t: RemarkArgs<T>) -> DispatchResult {
			execute_call!(t)
		}

		fn post_dispatch_check(t: RemarkArgs<T>) -> DispatchResult {
			execute_call!(t)
		}
	}
}

impl pallet_mock_test::Config for Runtime {}

impl Config for Runtime {
	type MaxRemarksPerCall = MaxRemarksPerCall;
	type Remark = TestRemark;
	type RemarkDispatchHandler = pallet_mock_test::Pallet<Runtime>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

// Balances pallet.
parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub MaxLocks: u32 = 2;
}

impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type FreezeIdentifier = ();
	type HoldIdentifier = ();
	type MaxFreezes = ();
	type MaxHolds = ConstU32<1>;
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities =
		GenesisConfig::default().build_storage().unwrap().into();

	// Ensure that we set a block number otherwise no events would be deposited.
	ext.execute_with(|| frame_system::Pallet::<Runtime>::set_block_number(1));

	ext
}

pub fn configure_mocks() {
	RemarkDispatchHandlerMock::mock_pre_dispatch_check(move |_t| Ok(()));
	RemarkDispatchHandlerMock::mock_post_dispatch_check(move |_t| Ok(()));
}