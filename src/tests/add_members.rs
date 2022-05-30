use crate::Error;
use super::mock::*;
use super::helper::*;
use frame_support::{assert_noop, assert_ok};
pub use sp_std::boxed::Box;


#[test]
fn add_members() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
		));
		let supersig_id = get_account_id(0);
		assert_ok!(Supersig::add_members(
			Origin::signed(supersig_id.clone()),
			supersig_id,
			vec!(BOB(), CHARLIE())
		));
		let supersig = Supersig::supersigs(0).unwrap();
		assert_eq!(supersig.members, vec!(ALICE(), BOB(), CHARLIE()));
	})
}

#[test]
fn add_users_not_allowed() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
		));
		let supersig_id = get_account_id(0);
		assert_noop!(
			Supersig::add_members(Origin::signed(ALICE()), supersig_id, vec!(BOB(), CHARLIE())),
			Error::<Test>::NotAllowed
		);
	})
}

#[test]
fn add_users_unknown_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
		));
		let bad_supersig_id = get_account_id(1);
		assert_noop!(
			Supersig::add_members(
				Origin::signed(bad_supersig_id.clone()),
				bad_supersig_id,
				vec!(BOB(), CHARLIE())
			),
			Error::<Test>::SupersigNotFound
		);
	})
}
