use super::{helper::*, mock::*};
use crate::{Config as SuperConfig, Error, Role};
use frame_support::{assert_noop, assert_ok, traits::ReservableCurrency};
use frame_system::RawOrigin;
pub use sp_std::{boxed::Box, mem::size_of};

#[test]
fn delete_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);
		let bob_balance = Balances::free_balance(BOB());
		let amount = 10_000u64;
		assert_ok!(Balances::transfer(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			amount
		));
		assert_ok!(Supersig::delete_supersig(
			RawOrigin::Signed(supersig_account.clone()).into(),
			BOB()
		));

		assert_eq!(Supersig::total_members(0), 0);
		assert_eq!(Supersig::nonce_call(0), 0);
		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Supersig::votes(0, 0), 0);
		assert_eq!(
			frame_system::Pallet::<Test>::providers(&supersig_account),
			0
		);

		let reserve = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((3u32).into())
			.saturating_mul(<Test as SuperConfig>::DepositPerByte::get());
		assert_eq!(
			Balances::free_balance(BOB()),
			bob_balance + amount + reserve
		);
		assert_eq!(
			last_event(),
			RuntimeEvent::Supersig(crate::Event::SupersigRemoved(supersig_account))
		);
	})
}

#[test]
fn delete_supersig_with_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);
		let bob_balance = Balances::free_balance(BOB());
		let amount = 10_000u64;
		let call = frame_system::Call::remark {
			remark: "test".into(),
		};
		assert_ok!(Balances::transfer(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			amount
		));
		assert_ok!(Supersig::propose_call(
			RawOrigin::Signed(BOB()).into(),
			supersig_account.clone(),
			Box::new(call.into())
		));
		assert_ok!(Supersig::delete_supersig(
			RawOrigin::Signed(supersig_account.clone()).into(),
			BOB()
		));

		assert_eq!(Supersig::total_members(0), 0);
		assert_eq!(Supersig::nonce_call(0), 0);
		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Supersig::votes(0, 0), 0);
		assert_eq!(
			frame_system::Pallet::<Test>::providers(&supersig_account),
			0
		);

		let reserve = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((3u32).into())
			.saturating_mul(<Test as SuperConfig>::DepositPerByte::get());
		assert_eq!(
			Balances::free_balance(BOB()),
			bob_balance + amount + reserve
		);
		assert_eq!(Balances::reserved_balance(BOB()), 0);
		assert_eq!(
			last_event(),
			RuntimeEvent::Supersig(crate::Event::SupersigRemoved(supersig_account))
		);
	})
}

#[test]
fn delete_supersig_unknown_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let bad_supersig_account = get_supersig_account(1);
		assert_noop!(
			Supersig::delete_supersig(RawOrigin::Signed(bad_supersig_account).into(), BOB()),
			Error::<Test>::NotSupersig
		);
	})
}

#[test]
fn cannot_delete_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);
		let amount = 10_000u64;
		assert_ok!(Balances::transfer(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			amount
		));
		assert_ok!(Balances::reserve(&supersig_account, amount));
		assert_noop!(
			Supersig::delete_supersig(RawOrigin::Signed(supersig_account).into(), BOB()),
			Error::<Test>::SupersigHaveLockedFunds
		);
	})
}

#[test]
fn cannot_liquidate_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);

		let call = pallet_balances::Call::transfer_all {
			dest: ALICE(),
			keep_alive: false,
		};

		assert_ok!(Supersig::propose_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.into())
		));

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(BOB()).into(),
			supersig_account.clone(),
			0
		));

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(CHARLIE()).into(),
			supersig_account.clone(),
			0
		));

		assert!(Supersig::calls(0, 0).is_none());

		assert!(System::account_exists(&supersig_account));
	});
}
