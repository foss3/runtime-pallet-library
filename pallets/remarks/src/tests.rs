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
use pallet_balances::Call as BalancesCall;
use pallet_proxy::Call as ProxyCall;
use pallet_utility::Call as UtilityCall;
use sp_runtime::DispatchError::BadOrigin;

use crate::{mock::*, *};

mod remark {
	use super::*;

	fn get_test_remarks() -> BoundedVec<TestRemark, MaxRemarksPerCall> {
		BoundedVec::<TestRemark, MaxRemarksPerCall>::try_from(vec![
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
			let origin = RuntimeOrigin::signed(1);
			let remarks = get_test_remarks();

			let call = RuntimeCall::System(SystemCall::remark {
				remark: vec![3, 4, 5],
			});

			let expected_pre_origin = origin.clone();
			let expected_pre_remarks = remarks.clone();
			let expected_pre_call = call.clone();

			RemarkDispatchHandlerMock::mock_pre_dispatch_check(move |t| {
				assert_eq!(t.0.as_signed(), expected_pre_origin.clone().as_signed());
				assert_eq!(t.1, expected_pre_remarks);
				assert_eq!(t.2, Box::new(expected_pre_call.clone()));

				Ok(())
			});

			let expected_post_origin = origin.clone();
			let expected_post_remarks = remarks.clone();
			let expected_post_call = call.clone();

			RemarkDispatchHandlerMock::mock_post_dispatch_check(move |t| {
				assert_eq!(t.0.as_signed(), expected_post_origin.clone().as_signed());
				assert_eq!(t.1, expected_post_remarks);
				assert_eq!(t.2, Box::new(expected_post_call.clone()));

				Ok(())
			});

			assert_ok!(Remarks::remark(
				origin.clone(),
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
	fn pre_dispatch_failure() {
		new_test_ext().execute_with(|| {
			let remarks = get_test_remarks();

			let call = RuntimeCall::System(SystemCall::remark {
				remark: vec![3, 4, 5],
			});

			let expected_error = DispatchError::Other("pre-dispatch error");

			RemarkDispatchHandlerMock::mock_pre_dispatch_check(move |_t| Err(expected_error));

			assert_noop!(
				Remarks::remark(
					RuntimeOrigin::signed(1),
					remarks.clone(),
					call.clone().into()
				),
				expected_error
			);
		});
	}

	#[test]
	fn post_dispatch_failure() {
		new_test_ext().execute_with(|| {
			let remarks = get_test_remarks();

			let call = RuntimeCall::System(SystemCall::remark {
				remark: vec![3, 4, 5],
			});

			RemarkDispatchHandlerMock::mock_pre_dispatch_check(move |_t| Ok(()));

			let expected_error = DispatchError::Other("post-dispatch error");

			RemarkDispatchHandlerMock::mock_post_dispatch_check(move |_t| Err(expected_error));

			assert_noop!(
				Remarks::remark(
					RuntimeOrigin::signed(1),
					remarks.clone(),
					call.clone().into()
				),
				expected_error
			);
		});
	}

	#[test]
	fn nested_remark_call_failure() {
		new_test_ext().execute_with(|| {
			let remarks = get_test_remarks();

			let call = RuntimeCall::Remarks(Call::remark {
				remarks: Default::default(),
				call: Box::new(RuntimeCall::System(SystemCall::remark {
					remark: vec![3, 4, 5],
				})),
			});

			RemarkDispatchHandlerMock::mock_pre_dispatch_check(move |_t| Ok(()));

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
	fn remark_in_batch_call_success() {
		new_test_ext().execute_with(|| {
			let remarks = get_test_remarks();

			let batch_call = RuntimeCall::Utility(UtilityCall::batch {
				calls: vec![
					RuntimeCall::Remarks(Call::remark {
						remarks: Default::default(),
						call: Box::new(RuntimeCall::System(SystemCall::remark {
							remark: vec![1, 2, 3],
						})),
					}),
					RuntimeCall::Remarks(Call::remark {
						remarks: Default::default(),
						call: Box::new(RuntimeCall::System(SystemCall::remark {
							remark: vec![4, 5, 6],
						})),
					}),
				],
			});

			RemarkDispatchHandlerMock::mock_pre_dispatch_check(move |_t| Ok(()));
			RemarkDispatchHandlerMock::mock_post_dispatch_check(move |_t| Ok(()));

			assert_ok!(Remarks::remark(
				RuntimeOrigin::signed(1),
				remarks.clone(),
				batch_call.clone().into()
			));
		});
	}

	#[test]
	fn inner_call_failure() {
		new_test_ext().execute_with(|| {
			let remarks = get_test_remarks();

			let call = RuntimeCall::System(SystemCall::set_heap_pages { pages: 8 });

			RemarkDispatchHandlerMock::mock_pre_dispatch_check(move |_t| Ok(()));

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

	#[test]
	fn inner_proxy_call_failure() {
		new_test_ext().execute_with(|| {
			let remarks = get_test_remarks();

			let proxy_origin = RuntimeOrigin::signed(1);
			let real_origin = RuntimeOrigin::signed(2);
			let transfer_dest = RuntimeOrigin::signed(3);

			let call = RuntimeCall::Proxy(ProxyCall::proxy {
				real: real_origin.as_signed().unwrap(),
				force_proxy_type: None,
				call: Box::new(RuntimeCall::Balances(BalancesCall::transfer {
					dest: transfer_dest.as_signed().unwrap(),
					value: 100,
				})),
			});

			RemarkDispatchHandlerMock::mock_pre_dispatch_check(move |_t| Ok(()));

			assert_noop!(
				Remarks::remark(proxy_origin, remarks.clone(), call.clone().into()),
				pallet_proxy::Error::<Runtime>::NotProxy,
			);
		});
	}
}
