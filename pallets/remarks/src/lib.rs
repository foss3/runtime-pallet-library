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
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::GetDispatchInfo,
	pallet_prelude::*,
	traits::{IsSubType, OriginTrait},
	BoundedVec,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_runtime::traits::Dispatchable;
use sp_std::boxed::Box;
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod weights;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching call type.
		type RuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>
			+ IsSubType<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>;

		/// Weight information.
		type WeightInfo: WeightInfo;

		/// The type attached to the remark event.
		type Remark: Parameter + Member + Default;

		/// Type that restrains the maximum remarks that can be attached to a
		/// call.
		type MaxRemarks: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A remark was made.
		Remark {
			remarks: BoundedVec<T::Remark, T::MaxRemarks>,
			call: <T as Config>::RuntimeCall,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No remarks were provided.
		NoRemarks,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add remarks to a call.
		///
		/// The weight calculation is similar to the one in the Proxy.proxy.
		#[pallet::call_index(0)]
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(T::WeightInfo::remark()
				.saturating_add(dispatch_info.weight),
			dispatch_info.class)
		})]
		pub fn remark(
			origin: OriginFor<T>,
			remarks: BoundedVec<T::Remark, T::MaxRemarks>,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResult {
			ensure!(!remarks.is_empty(), Error::<T>::NoRemarks);

			let mut filtered_origin = origin;

			// Nested remark calls are not allowed.
			filtered_origin.add_filter(move |c: &<T as frame_system::Config>::RuntimeCall| {
				let c = <T as Config>::RuntimeCall::from_ref(c);
				!matches!(c.is_sub_type(), Some(Call::remark { .. }))
			});

			call.clone()
				.dispatch(filtered_origin)
				.map(|_| ())
				.map_err(|e| e.error)?;

			Self::deposit_event(Event::Remark {
				remarks,
				call: *call,
			});

			Ok(())
		}
	}
}
