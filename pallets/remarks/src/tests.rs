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
use frame_support::{assert_noop, assert_ok, pallet_prelude::ConstU32, BoundedVec};
use frame_system::Call as SystemCall;
use sp_runtime::DispatchError::BadOrigin;

use crate::{mock::*, *};

mod remark {
	use super::*;

	fn get_test_remarks() -> BoundedVec<TestRemark, MaxRemarks> {
		BoundedVec::<TestRemark, MaxRemarks>::try_from(vec![
			TestRemark::SomeId(1),
			TestRemark::SomeId(2),
			TestRemark::SomeBytes(
				BoundedVec::<u8, ConstU32<16>>::try_from(vec![0, 1, 2])
					.expect("can build remark bytes"),
			),
		])
		.expect("can build remarks")
	}

	#[test]
	fn success() {
		new_test_ext().execute_with(|| {
			let remarks = get_test_remarks();

			let call = RuntimeCall::System(SystemCall::remark {
				remark: vec![3, 4, 5],
			});

			assert_ok!(Remarks::remark(
				RuntimeOrigin::signed(1),
				remarks.clone(),
				call.clone().into()
			));

			let expected_event: RuntimeEvent = Event::<Runtime>::Remark { remarks, call }.into();

			System::events()
				.iter()
				.find(|e| e.event == expected_event)
				.expect("remark event is present");
		});
	}

	#[test]
	fn no_remarks() {
		new_test_ext().execute_with(|| {
			let call = RuntimeCall::System(SystemCall::remark {
				remark: vec![3, 4, 5],
			});

			assert_noop!(
				Remarks::remark(
					RuntimeOrigin::signed(1),
					Default::default(),
					call.clone().into()
				),
				Error::<Runtime>::NoRemarks,
			);
		});
	}

	#[test]
	fn nested_call_failure() {
		new_test_ext().execute_with(|| {
			let remarks = get_test_remarks();

			let call = RuntimeCall::Remarks(Call::remark {
				remarks: Default::default(),
				call: Box::new(RuntimeCall::System(SystemCall::remark {
					remark: vec![3, 4, 5],
				})),
			});

			assert_noop!(
				Remarks::remark(
					RuntimeOrigin::signed(1),
					remarks.clone(),
					call.clone().into()
				),
				frame_system::Error::<Runtime>::CallFiltered
			);
		});
	}

	#[test]
	fn inner_call_failure() {
		new_test_ext().execute_with(|| {
			let remarks = get_test_remarks();

			let call = RuntimeCall::System(SystemCall::set_heap_pages { pages: 8 });

			assert_noop!(
				Remarks::remark(
					RuntimeOrigin::signed(1),
					remarks.clone(),
					call.clone().into()
				),
				BadOrigin
			);
		});
	}
}
