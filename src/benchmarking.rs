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

benchmarks! {
	create_supersig {
		let z in 0 .. 10;

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));

		let mut members = vec!{(alice.clone(), Role::Standard), (bob, Role::Standard), (charlie, Role::Standard)};
		let oui = "oui";
		for i in 0 .. z {
			let acc = benchmark_account(oui, i, 0);
			members.push((acc, Role::Standard));
		}


	}: _(RawOrigin::Signed(alice.clone()), members.clone())
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

		assert_ok!(
			Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(),
			vec!((alice.clone(), Role::Standard), (bob, Role::Standard), (charlie, Role::Standard)))
		);
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
		T::Currency::make_free_balance_be(&bob, val.saturating_mul(4_000_000_000u32.into()));

		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);

		assert_ok!(
			Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(),
			vec!((alice.clone(), Role::Standard), (bob.clone(), Role::Standard), (charlie, Role::Standard)))
		);
		assert_ok!(Pallet::<T>::submit_call(RawOrigin::Signed(alice.clone()).into(), supersig_id.clone(), Box::new(call)));
		assert_ok!(Pallet::<T>::approve_call(RawOrigin::Signed(alice).into(), supersig_id.clone(), 0));
	}: _(RawOrigin::Signed(bob.clone()), supersig_id, 0)
	verify {
		assert_eq!(Pallet::<T>::nonce_supersig(), 1);
		assert_eq!(Pallet::<T>::nonce_call(0), 1);
		assert_eq!(Pallet::<T>::votes(0, 0), 0);
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

		assert_ok!(
			Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(),
			vec!((alice.clone(), Role::Standard), (bob, Role::Standard), (charlie, Role::Standard)))
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
		let z in 0 .. 10;
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));

		let mut members = sp_std::vec!{(alice.clone(), Role::Standard), (bob, Role::Standard), (charlie, Role::Standard)};
		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice).into(), members.clone()));
		let oui = "oui";
		for i in 0 .. z {
			let acc = benchmark_account(oui, i, 0);
			members.push((acc, Role::Standard));
		}
		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);
		T::Currency::make_free_balance_be(&supersig_id, val.saturating_mul(4_000_000_000u32.into()));

	}: _(RawOrigin::Signed(supersig_id.clone()), supersig_id.clone(), members)
	verify {
		assert_eq!(Pallet::<T>::total_members(0), 3 + z);
	}
	remove_members {
		let z in 0 .. 10;
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));

		let mut members_and_roles = sp_std::vec!{(alice.clone(), Role::Standard), (bob.clone(), Role::Standard), (charlie.clone(), Role::Standard)};
		let mut members = Vec::new();
		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(), members_and_roles.clone()));
		let oui = "oui";
		for i in 0 .. z {
			let acc: T::AccountId = benchmark_account(oui, i, 0);
			members_and_roles.push((acc.clone(), Role::Standard));
		members.push(acc);
		}
		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);
		T::Currency::make_free_balance_be(&supersig_id, val.saturating_mul(4_000_000_000u32.into()));

		assert_ok!(Pallet::<T>::add_members(RawOrigin::Signed(supersig_id.clone()).into(), supersig_id.clone(), members_and_roles.clone()));
	}: _(RawOrigin::Signed(supersig_id.clone()), supersig_id.clone(), members.clone())
	verify {
		assert_eq!(Pallet::<T>::members(0, alice), Role::Standard);
		assert_eq!(Pallet::<T>::members(0, bob), Role::Standard);
		assert_eq!(Pallet::<T>::members(0, charlie), Role::Standard);
	}
	remove_supersig {
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 4_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(4_000_000_000u32.into()));
		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);
		T::Currency::make_free_balance_be(&supersig_id, val.saturating_mul(4_000_000_000u32.into()));

		assert_ok!(
			Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(),
			vec!((alice, Role::Standard), (bob.clone(), Role::Standard), (charlie, Role::Standard)))
		);
	}: _(RawOrigin::Signed(supersig_id.clone()), supersig_id.clone(), bob)
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

		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);
		assert_ok!(
			Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(),
			vec!((alice.clone(), Role::Standard), (bob.clone(), Role::Standard), (charlie.clone(), Role::Standard)))
		);
	}: _(RawOrigin::Signed(alice.clone()), supersig_id)
	verify {
		assert_eq!(Pallet::<T>::total_members(0), 2);
		assert_eq!(Pallet::<T>::members(0, alice), Role::NotMember);
		assert_eq!(Pallet::<T>::members(0, bob), Role::Standard);
		assert_eq!(Pallet::<T>::members(0, charlie), Role::Standard);
	}
}
