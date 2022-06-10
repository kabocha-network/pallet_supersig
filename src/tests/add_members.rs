use super::{helper::*, mock::*};
use crate::{Config as SuperConfig, Error, Role};
use frame_support::{assert_noop, assert_ok};
pub use sp_std::{boxed::Box, mem::size_of};

#[test]
fn add_members() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members = vec![(ALICE(), Role::Standard), (BOB(), Role::Standard)];
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), members,));

		let supersig_id = get_account_id(0);
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			100_000
		));
		assert_ok!(Supersig::add_members(
			Origin::signed(supersig_id.clone()),
			supersig_id.clone(),
			vec!((BOB(), Role::Master), (CHARLIE(), Role::Standard))
		));

		assert_eq!(Supersig::members(0, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(0, BOB()), Role::Master);
		assert_eq!(Supersig::members(0, CHARLIE()), Role::Standard);
		assert_eq!(Supersig::total_members(0), 3);

		let deposit = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((Supersig::total_members(0) as u32).into())
			.saturating_mul(<Test as SuperConfig>::PricePerByte::get());
		assert_eq!(Balances::reserved_balance(get_account_id(0)), deposit);
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::UsersAdded(
				supersig_id,
				vec!((CHARLIE(), Role::Standard))
			))
		);
	})
}

#[test]
fn add_users_not_allowed() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members = vec![(ALICE(), Role::Standard), (BOB(), Role::Standard)];
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			members.clone(),
		));
		let supersig_id = get_account_id(0);
		assert_noop!(
			Supersig::add_members(Origin::signed(ALICE()), supersig_id, members),
			Error::<Test>::NotAllowed
		);
	})
}

#[test]
fn add_users_unknown_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members = vec![(ALICE(), Role::Standard), (BOB(), Role::Standard)];
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			members.clone(),
		));
		let bad_supersig_id = get_account_id(1);
		assert_noop!(
			Supersig::add_members(
				Origin::signed(bad_supersig_id.clone()),
				bad_supersig_id,
				members
			),
			Error::<Test>::SupersigNotFound
		);
	})
}

#[test]
fn add_too_many_users() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec![(ALICE(), Role::Standard), (BOB(), Role::Standard),]
		));
		let supersig_id = get_account_id(1);
		assert_noop!(
			Supersig::add_members(
				Origin::signed(supersig_id.clone()),
				supersig_id,
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
