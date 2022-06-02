use super::{helper::*, mock::*};
use crate::Error;
use frame_support::{assert_noop, assert_ok};
pub use sp_std::boxed::Box;

#[test]
fn leave_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
		));
		let supersig_id = get_account_id(0);

		assert_ok!(Supersig::leave_supersig(
			Origin::signed(ALICE()),
			supersig_id.clone()
		));

		assert_eq!(
			Supersig::supersigs(0).unwrap().members,
			vec!(BOB(), CHARLIE())
		);
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
			vec!(ALICE(), BOB())
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
			vec!(ALICE(), BOB(), CHARLIE()),
		));
		let bad_supersig_id = get_account_id(1);

		assert_noop!(
			Supersig::leave_supersig(Origin::signed(CHARLIE()), bad_supersig_id),
			Error::<Test>::SupersigNotFound
		);
	})
}
