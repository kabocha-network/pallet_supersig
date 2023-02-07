use super::{helper::*, mock::*};
use crate::{Error, Role};
use frame_support::{assert_noop, assert_ok};
pub use sp_std::boxed::Box;
use frame_system::{Origin};

#[test]
fn leave_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);

		assert_ok!(Supersig::leave_supersig(
			Origin::signed(ALICE()),
			supersig_account.clone()
		));
		assert_eq!(Supersig::members(0, ALICE()), Role::NotMember);
		assert_eq!(Supersig::total_members(0), 2);

		assert_eq!(
			last_event(),
			RuntimeEvent::Supersig(crate::Event::SupersigLeaved(supersig_account, ALICE()))
		);
	})
}

#[test]
fn leave_supersig_not_a_member() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);

		assert_noop!(
			Supersig::leave_supersig(Origin::signed(CHARLIE()), supersig_account),
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
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let bad_supersig_account = get_supersig_account(1);

		assert_noop!(
			Supersig::leave_supersig(Origin::signed(CHARLIE()), bad_supersig_account),
			Error::<Test>::NotSupersig
		);
	})
}

#[test]
fn leave_supersig_last_user() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);

		assert_noop!(
			Supersig::leave_supersig(Origin::signed(ALICE()), supersig_account),
			Error::<Test>::InvalidNumberOfMembers
		);
	})
}
