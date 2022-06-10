use super::{helper::*, mock::*};
use crate::{Config as SuperConfig, Error, Role};
use frame_support::{assert_noop, assert_ok, traits::ReservableCurrency};
pub use sp_std::{boxed::Box, mem::size_of};

#[test]
fn delete_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			},
		));
		let supersig_id = get_account_id(0);
		let bob_balance = Balances::free_balance(BOB());
		let amount = 10_000u64;
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			amount
		));
		assert_ok!(Supersig::delete_supersig(
			Origin::signed(supersig_id.clone()),
			supersig_id.clone(),
			BOB()
		));

		assert_eq!(Supersig::total_members(0), 0);
		assert_eq!(Supersig::nonce_call(0), 0);
		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Supersig::votes(0, 0), 0);
		assert_eq!(frame_system::Pallet::<Test>::providers(&supersig_id), 0);

		let reserve = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((3u32).into())
			.saturating_mul(<Test as SuperConfig>::PricePerByte::get());
		assert_eq!(
			Balances::free_balance(BOB()),
			bob_balance + amount + reserve
		);
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::SupersigRemoved(supersig_id))
		);
	})
}

#[test]
fn delete_supersig_with_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			},
		));
		let supersig_id = get_account_id(0);
		let bob_balance = Balances::free_balance(BOB());
		let amount = 10_000u64;
		let call = Call::Nothing(NoCall::do_nothing {
			nothing: "test".into(),
		});
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			amount
		));
		assert_ok!(Supersig::submit_call(
			Origin::signed(BOB()),
			supersig_id.clone(),
			Box::new(call.clone())
		));
		assert_ok!(Supersig::delete_supersig(
			Origin::signed(supersig_id.clone()),
			supersig_id.clone(),
			BOB()
		));

		assert_eq!(Supersig::total_members(0), 0);
		assert_eq!(Supersig::nonce_call(0), 0);
		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Supersig::votes(0, 0), 0);
		assert_eq!(frame_system::Pallet::<Test>::providers(&supersig_id), 0);

		let reserve = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((3u32).into())
			.saturating_mul(<Test as SuperConfig>::PricePerByte::get());
		assert_eq!(
			Balances::free_balance(BOB()),
			bob_balance + amount + reserve
		);
		assert_eq!(Balances::reserved_balance(BOB()), 0);
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::SupersigRemoved(supersig_id))
		);
	})
}

#[test]
fn delete_supersig_not_allowed() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			},
		));
		let supersig_id = get_account_id(0);
		assert_noop!(
			Supersig::delete_supersig(Origin::signed(ALICE()), supersig_id, BOB()),
			Error::<Test>::NotAllowed
		);
	})
}

#[test]
fn delete_supersig_unknown_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			},
		));
		let bad_supersig_id = get_account_id(1);
		assert_noop!(
			Supersig::delete_supersig(
				Origin::signed(bad_supersig_id.clone()),
				bad_supersig_id,
				BOB()
			),
			Error::<Test>::SupersigNotFound
		);
	})
}

#[test]
fn cannot_delete_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			},
		));
		let supersig_id = get_account_id(0);
		let amount = 10_000u64;
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			amount
		));
		assert_ok!(Balances::reserve(&supersig_id, amount));
		assert_noop!(
			Supersig::delete_supersig(Origin::signed(supersig_id.clone()), supersig_id, BOB()),
			Error::<Test>::SupersigHaveLockedFunds
		);
	})
}

#[test]
fn cannot_liquidate_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			},
		));
		let supersig_id = get_account_id(0);

		let call = Call::Balances(pallet_balances::Call::transfer_all {
			dest: ALICE(),
			keep_alive: false,
		});

		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));

		assert_ok!(Supersig::approve_call(
			Origin::signed(BOB()),
			supersig_id.clone(),
			0
		));

		assert_ok!(Supersig::approve_call(
			Origin::signed(CHARLIE()),
			supersig_id.clone(),
			0
		));

		assert!(Supersig::calls(0, 0).is_none());

		assert!(System::account_exists(&supersig_id));
	});
}
