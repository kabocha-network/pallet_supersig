//! Benchmarking setup for pallet-supersig
#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::Pallet;
use frame_benchmarking::{account as benchmark_account, benchmarks};
use frame_support::{assert_ok, storage::bounded_vec::*, traits::Get, PalletId};
use frame_system::RawOrigin;
use sp_std::vec;

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}

benchmarks! {
	create_supersig {
		let z in 0 .. T::MaxAccountsPerTransaction::get() - 1;

		let alice: T::AccountId = get_account::<T>("ALICE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));

		let mut members: BoundedVec<_, _> = vec!{(alice.clone(), Role::Standard)}.try_into().unwrap();
		let oui = "oui";
		for i in 0 .. z {
			let acc = benchmark_account(oui, i, 0);
			members.try_push((acc, Role::Standard)).unwrap();
		}
	}: _(RawOrigin::Signed(alice.clone()), members.clone())
	verify {
		assert_eq!(Pallet::<T>::nonce_supersig(), 1);
	}

	submit_call {
		let z in 0 .. 100_000;
		let call = frame_system::Call::remark {
			remark: vec![0; z as usize]
		}.into();

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));


		let supersig_id = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account_truncating(0);
		let members: BoundedVec<_, _> = vec!{(alice.clone(), Role::Standard), (bob, Role::Standard), (charlie, Role::Standard)}.try_into().unwrap();

		assert_ok!(
			Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(),
			members)
		);
	}: _(RawOrigin::Signed(alice.clone()), supersig_id, Box::new(call))
	verify {
		assert_eq!(Pallet::<T>::nonce_supersig(), 1);
		assert_eq!(Pallet::<T>::nonce_call(0), 1);
	}

	approve_call {
		let call = frame_system::Call::remark {
			remark: vec![0; 0]
		}.into();

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));
		T::Currency::make_free_balance_be(&bob, val.saturating_mul(4_000_000_000u32.into()));

		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account_truncating(0);

		let members: BoundedVec<_, _> = vec!{(alice.clone(), Role::Standard), (bob.clone(), Role::Standard), (charlie, Role::Standard)}.try_into().unwrap();

		assert_ok!(
			Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(),
			members)
		);
		assert_ok!(Pallet::<T>::submit_call(RawOrigin::Signed(alice).into(), supersig_id.clone(), Box::new(call)));

	}: _(RawOrigin::Signed(bob.clone()), supersig_id, 0)
	verify {
		assert_eq!(Pallet::<T>::nonce_supersig(), 1);
		assert_eq!(Pallet::<T>::nonce_call(0), 1);
		assert_eq!(Pallet::<T>::votes(0, 0), 1);
	}

	remove_call {
		let call = frame_system::Call::remark {
			remark: vec![0; 0]
		}.into();

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));

		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account_truncating(0);

		let members: BoundedVec<_, _> = vec!{(alice.clone(), Role::Standard), (bob, Role::Standard), (charlie, Role::Standard)}.try_into().unwrap();

		assert_ok!(
			Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(),
			members)
		);
		assert_ok!(Pallet::<T>::submit_call(RawOrigin::Signed(alice.clone()).into(), supersig_id.clone(), Box::new(call)));
		assert_ok!(Pallet::<T>::approve_call(RawOrigin::Signed(alice.clone()).into(), supersig_id.clone(), 0));
	}: _(RawOrigin::Signed(alice.clone()), supersig_id, 0)
	verify {
		assert_eq!(Pallet::<T>::nonce_supersig(), 1);
		assert_eq!(Pallet::<T>::nonce_call(0), 1);
		assert!(Pallet::<T>::calls(0, 0).is_none());
	}

	add_members {
		let z in 0 .. T::MaxAccountsPerTransaction::get();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));

		let initial_members: BoundedVec<_, _> = vec!{(alice.clone(), Role::Standard), (bob, Role::Standard), (charlie, Role::Standard)}.try_into().unwrap();
		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice).into(), initial_members.clone()));

		let mut new_members: BoundedVec<_, _> = Vec::new().try_into().unwrap();
		let oui = "oui";
		for i in 0 .. z {
			let acc = benchmark_account(oui, i, 0);
			new_members.try_push((acc, Role::Standard)).unwrap();
		}
		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account_truncating(0);
		T::Currency::make_free_balance_be(&supersig_id, val.saturating_mul(4_000_000_000u32.into()));

	}: _(RawOrigin::Signed(supersig_id.clone()), new_members.clone())
	verify {
		assert_eq!(Pallet::<T>::total_members(0), 3 + z);
	}

	remove_members {
		let z in 0 .. T::MaxAccountsPerTransaction::get();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));

		let initial_members: BoundedVec<_, _> = vec!{(alice.clone(), Role::Standard), (bob, Role::Standard), (charlie, Role::Standard)}.try_into().unwrap();
		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice).into(), initial_members.clone()));

		let mut new_members: BoundedVec<_, _> = Vec::new().try_into().unwrap();
		let oui = "oui";
		for i in 0 .. z {
			let acc = benchmark_account(oui, i, 0);
			new_members.try_push((acc, Role::Standard)).unwrap();
		}
		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account_truncating(0);
		T::Currency::make_free_balance_be(&supersig_id, val.saturating_mul(4_000_000_000u32.into()));
		assert_ok!(Pallet::<T>::add_members(RawOrigin::Signed(supersig_id.clone()).into(), new_members.clone()));

		let members_to_remove: BoundedVec<T::AccountId, _> = new_members.into_iter().map(|(a, r)| a).collect::<Vec<_>>().try_into().unwrap();
	}: _(RawOrigin::Signed(supersig_id.clone()), members_to_remove.clone())
	verify {
		assert_eq!(Pallet::<T>::total_members(0), 3);
	}

	delete_supersig {
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));
		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account_truncating(0);
		T::Currency::make_free_balance_be(&supersig_id, val.saturating_mul(4_000_000_000u32.into()));

		let members: BoundedVec<_, _> = vec!{(alice.clone(), Role::Standard), (bob.clone(), Role::Standard), (charlie, Role::Standard)}.try_into().unwrap();

		assert_ok!(
			Pallet::<T>::create_supersig(RawOrigin::Signed(alice).into(),
			members)
		);
	}: _(RawOrigin::Signed(supersig_id.clone()), bob)
	verify {

		assert_eq!(Pallet::<T>::total_members(0), 0);
		assert_eq!(Pallet::<T>::nonce_call(0), 0);
		assert!(Pallet::<T>::calls(0, 0).is_none());
		assert_eq!(Pallet::<T>::votes(0, 0), 0);
		assert_eq!(frame_system::Pallet::<T>::consumers(&supersig_id), 0);
		assert_eq!(frame_system::Pallet::<T>::providers(&supersig_id), 0);
	}

	leave_supersig {
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 1_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));

		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account_truncating(0);
		let members: BoundedVec<_, _> = vec!{(alice.clone(), Role::Standard), (bob.clone(), Role::Standard), (charlie.clone(), Role::Standard)}.try_into().unwrap();

		assert_ok!(
			Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(),
			members)
		);
	}: _(RawOrigin::Signed(alice.clone()), supersig_id)
	verify {
		assert_eq!(Pallet::<T>::total_members(0), 2);
		assert_eq!(Pallet::<T>::members(0, alice), Role::NotMember);
		assert_eq!(Pallet::<T>::members(0, bob), Role::Standard);
		assert_eq!(Pallet::<T>::members(0, charlie), Role::Standard);
	}
}
