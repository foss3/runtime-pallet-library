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
use frame_benchmarking::{account, impl_benchmark_test_suite, v2::*};
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use sp_std::{boxed::Box, cmp::min, vec, vec::Vec};

use super::*;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks()]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn remark(
		n: Linear<1, { min(10, T::MaxRemarksPerCall::get()) }>,
	) -> Result<(), BenchmarkError> {
		let mut remarks = Vec::new();

		for _ in 0..n {
			remarks.push(T::Remark::default())
		}

		let remarks = BoundedVec::<T::Remark, T::MaxRemarksPerCall>::try_from(remarks)
			.expect("can build remarks");

		let caller: T::AccountId = account("acc_0", 0, 0);
		let call: <T as Config>::RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();

		#[cfg(test)]
		crate::mock::configure_mocks();

		#[extrinsic_call]
		remark(
			RawOrigin::Signed(caller),
			remarks.clone(),
			Box::new(call.clone()),
		);

		assert_last_event::<T>(Event::Remark { remarks, call }.into());

		Ok(())
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::System::externalities(),
		crate::mock::Runtime
	);
}
