use super::{helper::*, mock::*};
use crate::{Config as SuperConfig, Error, Role};
use frame_support::{assert_noop, assert_ok};
pub use sp_std::{boxed::Box, mem::size_of};

#[test]
fn remove_members() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Member),
				(BOB(), Role::Member),
				(CHARLIE(), Role::Member),
				(PAUL(), Role::Member),
			},
		));
		let supersig_id = get_account_id(0);
		assert_ok!(Supersig::remove_members(
			Origin::signed(supersig_id.clone()),
			supersig_id.clone(),
			vec!(BOB(), CHARLIE())
		));
		assert_eq!(Supersig::members(0, ALICE()), Role::Member);
		assert_eq!(Supersig::members(0, BOB()), Role::NotMember);
		assert_eq!(Supersig::members(0, CHARLIE()), Role::NotMember);
		assert_eq!(Supersig::members(0, PAUL()), Role::Member);
		assert_eq!(Supersig::members_number(0), 2);

		let reserve = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((Supersig::members_number(0) as u32).into())
			.saturating_mul(<Test as SuperConfig>::PricePerBytes::get());
		assert_eq!(Balances::reserved_balance(get_account_id(0)), reserve);
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::UsersRemoved(
				supersig_id,
				vec!(BOB(), CHARLIE())
			))
		);
	})
}

#[test]
fn remove_users_not_allowed() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Member),
				(BOB(), Role::Member),
				(CHARLIE(), Role::Member),
				(PAUL(), Role::Member),
			},
		));
		let supersig_id = get_account_id(0);
		assert_noop!(
			Supersig::remove_members(Origin::signed(ALICE()), supersig_id, vec!(BOB(), CHARLIE())),
			Error::<Test>::NotAllowed
		);
	})
}

#[test]
fn remove_users_unknown_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Member),
				(BOB(), Role::Member),
				(CHARLIE(), Role::Member),
				(PAUL(), Role::Member),
			},
		));
		let bad_supersig_id = get_account_id(1);
		assert_noop!(
			Supersig::remove_members(
				Origin::signed(bad_supersig_id.clone()),
				bad_supersig_id,
				vec!(BOB(), CHARLIE())
			),
			Error::<Test>::SupersigNotFound
		);
	})
}

#[test]
fn remove_users_leaving_1_users() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Member),
				(BOB(), Role::Member),
				(CHARLIE(), Role::Member),
			},
		));
		let supersig_id = get_account_id(0);
		assert_noop!(
			Supersig::remove_members(
				Origin::signed(supersig_id.clone()),
				supersig_id,
				vec!(ALICE(), BOB())
			),
			Error::<Test>::CannotRemoveUsers
		);
	})
}
