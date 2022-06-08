use super::{helper::*, mock::*};
use crate::{Error, Roles};
use frame_support::{assert_noop, assert_ok};
pub use sp_std::boxed::Box;

#[test]
fn leave_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Roles::Member),
				(BOB(), Roles::Member),
				(CHARLIE(), Roles::Member),
			},
		));
		let supersig_id = get_account_id(0);

		assert_ok!(Supersig::leave_supersig(
			Origin::signed(ALICE()),
			supersig_id.clone()
		));
		assert_eq!(Supersig::members(0, ALICE()), Roles::NotMember);
		assert_eq!(Supersig::members_number(0), 2);

		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::SupersigLeaved(supersig_id, ALICE()))
		);
	})
}

#[test]
fn leave_supersig_not_a_member() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Roles::Member),
				(BOB(), Roles::Member),
			},
		));
		let supersig_id = get_account_id(0);

		assert_noop!(
			Supersig::leave_supersig(Origin::signed(CHARLIE()), supersig_id),
			Error::<Test>::NotMember
		);
	})
}

#[test]
fn leave_unknown_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Roles::Member),
				(BOB(), Roles::Member),
				(CHARLIE(), Roles::Member),
			},
		));
		let bad_supersig_id = get_account_id(1);

		assert_noop!(
			Supersig::leave_supersig(Origin::signed(CHARLIE()), bad_supersig_id),
			Error::<Test>::SupersigNotFound
		);
	})
}

#[test]
fn leave_supersig_last_user() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Roles::Member),
				(BOB(), Roles::Member),
			},
		));
		let supersig_id = get_account_id(0);

		assert_noop!(
			Supersig::leave_supersig(Origin::signed(ALICE()), supersig_id),
			Error::<Test>::CannotRemoveUsers
		);
	})
}
