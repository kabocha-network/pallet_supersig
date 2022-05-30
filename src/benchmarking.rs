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
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 1_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(10000u32.into()));


	}: _(RawOrigin::Signed(alice.clone()), vec!(alice.clone(), bob, charlie))
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

		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(), vec!(alice.clone(), bob, charlie)));
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

		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(), vec!(alice.clone(), bob, charlie)));
		assert_ok!(Pallet::<T>::submit_call(RawOrigin::Signed(alice.clone()).into(), supersig_id.clone(), Box::new(call)));
	}: _(RawOrigin::Signed(alice.clone()), supersig_id, 0)
	verify {
		assert_eq!(Pallet::<T>::nonce_supersig(), 1);
		assert_eq!(Pallet::<T>::nonce_call(0), 1);
		assert_eq!(Pallet::<T>::votes(0, 0), 1);
		assert!(Pallet::<T>::users_votes((0, 0, alice)));
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

		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(), vec!(alice.clone(), bob, charlie)));
		assert_ok!(Pallet::<T>::submit_call(RawOrigin::Signed(alice.clone()).into(), supersig_id.clone(), Box::new(call)));
		assert_ok!(Pallet::<T>::approve_call(RawOrigin::Signed(alice.clone()).into(), supersig_id.clone(), 0));
	}: _(RawOrigin::Signed(alice.clone()), supersig_id, 0)
	verify {
		assert_eq!(Pallet::<T>::nonce_supersig(), 1);
		assert_eq!(Pallet::<T>::nonce_call(0), 1);
		assert!(Pallet::<T>::calls(0, 0).is_none());
	}

	add_members {
		let z in 0 .. 1_000;
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 1_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(10000u32.into()));

		let mut new_users: Vec<T::AccountId> = Vec::new();

		for i in 1..z {
			let new = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(10 + new_users.len() as u64);
			new_users.push(new);
		}
		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);

		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(), vec!(alice.clone(), bob.clone(), charlie.clone())));
	}: _(RawOrigin::Signed(supersig_id.clone()), supersig_id.clone(), new_users.clone())
	verify {
		let mut new_users = new_users;
		let mut tmp = vec!(alice, bob, charlie);
		tmp.append(new_users.as_mut());
		assert_eq!(Pallet::<T>::supersigs(0).unwrap().members, tmp);
	}
	remove_members {
		let z in 0 .. 1_000;
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 1_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(10000u32.into()));

		let mut new_users: Vec<T::AccountId> = Vec::new();

		for i in 1..z {
			let new = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(10 + new_users.len() as u64);
			new_users.push(new);
		}
		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);

		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(), vec!(alice.clone(), bob.clone(), charlie.clone())));
		assert_ok!(Pallet::<T>::add_members(RawOrigin::Signed(supersig_id.clone()).into(), supersig_id.clone(), new_users.clone()));
	}: _(RawOrigin::Signed(supersig_id.clone()), supersig_id.clone(), new_users)
	verify {
		assert_eq!(Pallet::<T>::supersigs(0).unwrap().members, vec!(alice.clone(), bob.clone(), charlie));
	}
	remove_supersig {
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let val: BalanceOf<T> = 1_000_000_000u32.into();
		T::Currency::make_free_balance_be(&alice, val.saturating_mul(10000u32.into()));
		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);

		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(), vec!(alice, bob.clone(), charlie)));
	}: _(RawOrigin::Signed(supersig_id.clone()), supersig_id.clone(), bob)
	verify {

		assert_eq!(Pallet::<T>::supersigs(0), None);
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

		let supersig_id: T::AccountId = <<T as Config>::PalletId as Get<PalletId>>::get().into_sub_account(0);
		assert_ok!(Pallet::<T>::create_supersig(RawOrigin::Signed(alice.clone()).into(), vec!(alice, bob.clone(), charlie.clone())));
    }: _(RawOrigin::Signed(supersig_id.clone()), supersig_id.clone())
    verify {
        assert_eq!(Pallet::<T>::supersigs(0).unwrap().members, vec!(bob, charlie));
    }
}
