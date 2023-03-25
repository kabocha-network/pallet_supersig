use super::{helper::*, mock::*};
use crate::{Error, Role};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
pub use sp_std::boxed::Box;

#[test]
fn remove_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);

		let call = frame_system::Call::remark {
			remark: "test".into(),
		};
		assert_ok!(Supersig::propose_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.into())
		));
		assert!(Supersig::calls(0, 0).is_some());
		assert_ok!(Supersig::remove_call(
			RawOrigin::Signed(supersig_account.clone()).into(),
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
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);

		let call = frame_system::Call::remark {
			remark: "test".into(),
		};
		assert_ok!(Supersig::propose_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.into())
		));
		assert_noop!(
			Supersig::remove_call(RawOrigin::Signed(BOB()).into(), supersig_account, 0),
			Error::<Test>::NotAllowed
		);
	})
}

#[test]
fn remove_unknown_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);

		let call = frame_system::Call::remark {
			remark: "test".into(),
		};
		assert_ok!(Supersig::propose_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.into())
		));
		assert_noop!(
			Supersig::remove_call(
				RawOrigin::Signed(supersig_account.clone()).into(),
				supersig_account,
				1
			),
			Error::<Test>::CallNotFound
		);
	})
}
