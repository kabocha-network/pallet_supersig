use super::{helper::*, mock::*};
use crate::{Config as SuperConfig, Error, Role};
use frame_support::{assert_noop, assert_ok};
pub use sp_std::{boxed::Box, cmp::min, mem::size_of};

#[test]
fn create_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members = vec![
			(ALICE(), Role::Standard),
			(BOB(), Role::Standard),
			(CHARLIE(), Role::Standard),
		];
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), members,));

		assert_eq!(Balances::free_balance(get_account_id(0)), 0u64);
		let deposit = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((3u32).into())
			.saturating_mul(<Test as SuperConfig>::PricePerBytes::get());

		assert_eq!(Balances::reserved_balance(get_account_id(0)), deposit);
		assert_eq!(Supersig::nonce_supersig(), 1);
		assert_eq!(Supersig::members(0, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(0, BOB()), Role::Standard);
		assert_eq!(Supersig::members(0, CHARLIE()), Role::Standard);
		assert_eq!(Supersig::total_members(0), 3);
		assert_eq!(
			frame_system::Pallet::<Test>::providers(&get_account_id(0)),
			1
		);
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::SupersigCreated(get_account_id(0)))
		);
	});
}

#[test]
fn create_supersig_with_master() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members = vec![
			(ALICE(), Role::Standard),
			(BOB(), Role::Master),
			(CHARLIE(), Role::Master),
		];
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), members,));

		assert_eq!(Balances::free_balance(get_account_id(0)), 0u64);
		let deposit = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((3u32).into())
			.saturating_mul(<Test as SuperConfig>::PricePerBytes::get());

		assert_eq!(Balances::reserved_balance(get_account_id(0)), deposit);
		assert_eq!(Supersig::nonce_supersig(), 1);
		assert_eq!(Supersig::members(0, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(0, BOB()), Role::Master);
		assert_eq!(Supersig::members(0, CHARLIE()), Role::Master);
		assert_eq!(Supersig::total_members(0), 3);
		assert_eq!(
			frame_system::Pallet::<Test>::providers(&get_account_id(0)),
			1
		);
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::SupersigCreated(get_account_id(0)))
		);
	});
}

#[test]
fn create_multiple_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members = vec![
			(ALICE(), Role::Standard),
			(BOB(), Role::Standard),
			(CHARLIE(), Role::Standard),
		];
		let members2 = vec![(ALICE(), Role::Standard), (BOB(), Role::Master)];
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), members,));
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), members2,));

		assert_eq!(Supersig::nonce_supersig(), 2);

		assert_eq!(Balances::free_balance(get_account_id(0)), 0u64);
		assert_eq!(Balances::free_balance(get_account_id(1)), 0u64);
		Balances::transfer(Origin::signed(ALICE()), get_account_id(1), 10_000).unwrap();

		assert_eq!(Balances::free_balance(get_account_id(0)), 0u64);
		assert_eq!(Balances::free_balance(get_account_id(1)), 10_000);

		assert_eq!(Supersig::members(0, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(0, BOB()), Role::Standard);
		assert_eq!(Supersig::members(0, CHARLIE()), Role::Standard);
		assert_eq!(Supersig::total_members(0), 3);

		assert_eq!(Supersig::members(1, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(1, BOB()), Role::Master);
		assert_eq!(Supersig::members(1, CHARLIE()), Role::NotMember);
		assert_eq!(Supersig::total_members(1), 2);
	});
}

#[test]
fn create_with_empty_list() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(
			Supersig::create_supersig(Origin::signed(ALICE()), vec!()),
			Error::<Test>::InvalidSupersig
		);
	});
}

#[test]
fn create_with_too_many_users() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(
			Supersig::create_supersig(
				Origin::signed(ALICE()),
				vec![
					(ALICE(), Role::Standard),
					(BOB(), Role::Standard),
					(CHARLIE(), Role::Standard),
					(PAUL(), Role::Standard),
					(DONALD(), Role::Standard),
				]
			),
			Error::<Test>::TooManyUsers
		);
	})
}
