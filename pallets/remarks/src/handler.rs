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

use std::marker::PhantomData;

use frame_support::{dispatch::DispatchResult, BoundedVec};
use frame_system::pallet_prelude::OriginFor;

use crate::Config;

/// The type used in the RemarkDispatchHandler trait of the pallet's Config.
pub type RemarkArgs<T> = (
	OriginFor<T>,
	BoundedVec<<T as Config>::Remark, <T as Config>::MaxRemarksPerCall>,
	Box<<T as Config>::RuntimeCall>,
);

/// The handler used to check remarks before and after call dispatch.
pub trait RemarkDispatchHandler<T> {
	fn pre_dispatch_check(t: T) -> DispatchResult;

	fn post_dispatch_check(t: T) -> DispatchResult;
}

pub struct NoopRemarkDispatchHandler<T>(PhantomData<T>);

impl<T> RemarkDispatchHandler<RemarkArgs<T>> for NoopRemarkDispatchHandler<T>
where
	T: Config,
{
	fn pre_dispatch_check(_t: RemarkArgs<T>) -> DispatchResult {
		Ok(())
	}

	fn post_dispatch_check(_t: RemarkArgs<T>) -> DispatchResult {
		Ok(())
	}
}
