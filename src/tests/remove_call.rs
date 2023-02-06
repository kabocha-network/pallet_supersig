use super::{helper::*, mock::*};
use crate::{Error, Role};
use frame_support::{assert_noop, assert_ok};
pub use sp_std::boxed::Box;
use frame_system::{Call, Origin};

#[test]
fn remove_call() {
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

		let call = Call::Nothing(NoCall::do_nothing {
			nothing: "test".into(),
		});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			Box::new(call)
		));
		assert!(Supersig::calls(0, 0).is_some());
		assert_ok!(Supersig::remove_call(
			Origin::signed(supersig_account.clone()),
			supersig_account.clone(),
			0
		));
		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Supersig::votes(0, 0), 0);
		assert!(!Supersig::members_votes((0, 0, CHARLIE())));
		assert!(!Supersig::members_votes((0, 0, BOB())));
		assert_eq!(
			last_event(),
			RuntimeEvent::Supersig(crate::Event::CallRemoved(supersig_account, 0))
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
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);

		let call = Call::Nothing(NoCall::do_nothing {
			nothing: "test".into(),
		});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			Box::new(call)
		));
		assert_noop!(
			Supersig::remove_call(Origin::signed(BOB()), supersig_account, 0),
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
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);

		let call = Call::Nothing(NoCall::do_nothing {
			nothing: "test".into(),
		});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			Box::new(call)
		));
		assert_noop!(
			Supersig::remove_call(
				Origin::signed(supersig_account.clone()),
				supersig_account,
				1
			),
			Error::<Test>::CallNotFound
		);
	})
}
