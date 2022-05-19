//! Benchmarking setup for pallet-supersig
#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::Pallet;
use frame_benchmarking::{account as benchmark_account, benchmarks};
use frame_support::{assert_ok, traits::Get};
use frame_system::RawOrigin;
use sp_std::vec;

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}

// pub type BalanceCall = pallet_balances::Call<T>;

benchmarks! {
	create_supersig {
		// let members: Vec<T::AccountId> = Vec::new();
		// for i in 10_000 {
		//     let random_string =
		// }
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 1_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(10000u32.into()));


	}: _(RawOrigin::Signed(alice.clone()), vec!(alice.clone(), bob, charlie), 2)
	verify {
		assert_eq!(Pallet::<T>::nonce_supersig(), 1);
	}
	submit_call {
		let z in 0 .. 10_000;
		let call: <T as Config>::Call = frame_system::Call::<T>::remark {
			remark: vec![0; z as usize]
		}.into();

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));

		let supersig_id = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);

		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(), vec!(alice.clone(), bob, charlie), 2));
	}: _(RawOrigin::Signed(alice.clone()), supersig_id, Box::new(call))
	verify {
		assert_eq!(Pallet::<T>::nonce_supersig(), 1);
		assert_eq!(Pallet::<T>::nonce_call(0), 1);
	}
	approve_call {
		let call: <T as Config>::Call = frame_system::Call::<T>::remark {
			remark: vec![0; 0]
		}.into();

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));

		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);

		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(), vec!(alice.clone(), bob, charlie), 2));
		assert_ok!(Pallet::<T>::submit_call(RawOrigin::Signed(alice.clone()).into(), supersig_id.clone(), Box::new(call)));
	}: _(RawOrigin::Signed(alice.clone()), supersig_id, 0)
	verify {
		assert_eq!(Pallet::<T>::nonce_supersig(), 1);
		assert_eq!(Pallet::<T>::nonce_call(0), 1);
		assert_eq!(Pallet::<T>::votes(0, 0), 1);
		assert!(Pallet::<T>::users_votes((0, 0), alice));
	}

	remove_call {
		let call: <T as Config>::Call = frame_system::Call::<T>::remark {
			remark: vec![0; 0]
		}.into();

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));

		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);

		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(), vec!(alice.clone(), bob, charlie), 2));
		assert_ok!(Pallet::<T>::submit_call(RawOrigin::Signed(alice.clone()).into(), supersig_id.clone(), Box::new(call)));
		assert_ok!(Pallet::<T>::approve_call(RawOrigin::Signed(alice.clone()).into(), supersig_id.clone(), 0));
	}: _(RawOrigin::Signed(alice.clone()), supersig_id, 0)
	verify {
		assert_eq!(Pallet::<T>::nonce_supersig(), 1);
		assert_eq!(Pallet::<T>::nonce_call(0), 1);
		assert!(Pallet::<T>::calls(0, 0).is_none());
	}
}
