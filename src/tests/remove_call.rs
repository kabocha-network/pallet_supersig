use super::{helper::*, mock::*};
use crate::{Error, Role};
use frame_support::{assert_noop, assert_ok};
pub use sp_std::boxed::Box;

#[test]
fn remove_call() {
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

		let call = Call::Nothing(NoCall::do_nothing {
			nothing: "test".into(),
		});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));
		assert!(Supersig::calls(0, 0).is_some());
		assert_ok!(Supersig::remove_call(
			Origin::signed(supersig_id.clone()),
			supersig_id.clone(),
			0
		));
		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Supersig::votes(0, 0), 0);
		assert!(!Supersig::members_votes((0, 0, CHARLIE())));
		assert!(!Supersig::members_votes((0, 0, BOB())));
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::CallRemoved(supersig_id, 0))
		);
	})
}

#[test]
fn non_allowed_remove_call() {
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

		let call = Call::Nothing(NoCall::do_nothing {
			nothing: "test".into(),
		});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));
		assert_noop!(
			Supersig::remove_call(Origin::signed(BOB()), supersig_id, 0),
			Error::<Test>::NotAllowed
		);
	})
}

#[test]
fn remove_unknown_call() {
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

		let call = Call::Nothing(NoCall::do_nothing {
			nothing: "test".into(),
		});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));
		assert_noop!(
			Supersig::remove_call(Origin::signed(supersig_id.clone()), supersig_id, 1),
			Error::<Test>::CallNotFound
		);
	})
}
