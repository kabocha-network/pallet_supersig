use super::{helper::*, mock::*};
use crate::{Config as SuperConfig, Error, Roles};
use frame_support::{assert_noop, assert_ok};
pub use sp_std::{boxed::Box, mem::size_of};

#[test]
fn add_members() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members = vec![(ALICE(), Roles::Member), (BOB(), Roles::Member)];
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
			vec!((BOB(), Roles::Master), (CHARLIE(), Roles::Member))
		));

		assert_eq!(Supersig::members(0, ALICE()), Roles::Member);
		assert_eq!(Supersig::members(0, BOB()), Roles::Member);
		assert_eq!(Supersig::members(0, CHARLIE()), Roles::Member);
		assert_eq!(Supersig::members_number(0), 3);

		let deposit = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((Supersig::members_number(0) as u32).into())
			.saturating_mul(<Test as SuperConfig>::PricePerBytes::get());
		assert_eq!(Balances::reserved_balance(get_account_id(0)), deposit);
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::UsersAdded(
				supersig_id,
				vec!((CHARLIE(), Roles::Member))
			))
		);
	})
}

#[test]
fn add_users_not_allowed() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members = vec![(ALICE(), Roles::Member), (BOB(), Roles::Member)];
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
		let members = vec![(ALICE(), Roles::Member), (BOB(), Roles::Member)];
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
