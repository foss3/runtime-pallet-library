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
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use parity_scale_codec::EncodeLike;
use sp_std::{boxed::Box, vec, vec::Vec};

use super::*;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	where_clause {
		where
			T::AccountId: EncodeLike<<T as frame_system::Config>::AccountId>,
	}

	remark {
		let c in 1 .. T::MaxRemarksPerCall::get();
		let mut remarks = Vec::new();

		for i in 0 .. c {
			remarks.push(T::Remark::default())
		}

		let remarks = BoundedVec::<T::Remark, T::MaxRemarksPerCall>::try_from(remarks).expect("can build remarks");

		let caller: T::AccountId = account("acc_0", 0, 0);
		let call: <T as Config>::RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();
	}: remark(RawOrigin::Signed(caller), remarks.clone(), Box::new(call.clone()))
	verify {
		assert_last_event::<T>(Event::Remark {
			remarks,
			call
		}.into())
	}
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime);
