use super::{helper::*, mock::*};
use crate::{Error, Role};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
pub use sp_std::boxed::Box;

////////////
//
// approve_call() tests
//
////////////

#[test]
fn approve_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let supersig_account = get_supersig_account(0);
		let call = frame_system::Call::remark {
			remark: "test".into(),
		};
		assert_ok!(Supersig::submit_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.into())
		));

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			0
		));
		assert_eq!(Supersig::votes(0, 0), 1);
		assert!(Supersig::members_votes((0, 0, ALICE())));
		assert!(!Supersig::members_votes((0, 0, CHARLIE())));
		assert!(!Supersig::members_votes((0, 0, BOB())));
		assert_eq!(
			last_event(),
			RuntimeEvent::Supersig(crate::Event::CallVoted(supersig_account, 0, ALICE()))
		);
	})
}

#[test]
fn approve_call_until_threshold() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let supersig_account = get_supersig_account(0);
		assert_ok!(Balances::transfer(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			100_000
		));

		let bob_balance = Balances::free_balance(BOB());

		let call = pallet_balances::Call::transfer {
			dest: BOB(),
			value: 100_000,
		};

		assert_ok!(Supersig::submit_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.into())
		));

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(BOB()).into(),
			supersig_account.clone(),
			0
		));

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			0
		));

		// the call have been approved, so it is executed, and then the call is deleted from
		// storage

		assert_eq!(Supersig::votes(0, 0), 0);
		assert!(!Supersig::members_votes((0, 0, ALICE())));
		assert!(!Supersig::members_votes((0, 0, BOB())));
		assert!(!Supersig::members_votes((0, 0, CHARLIE())));

		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Balances::reserved_balance(ALICE()), 0);
		let supersig_event = last_event();

		assert_eq!(bob_balance + 100_000, Balances::free_balance(BOB()));

		let dispatch_result = Balances::transfer(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			100_000,
		);

		assert_eq!(
			supersig_event,
			RuntimeEvent::Supersig(crate::Event::CallExecutionAttempted(
				supersig_account,
				0,
				dispatch_result
			))
		);
	})
}

#[test]
fn approve_call_as_master() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Master),
				(CHARLIE(), Role::Standard),
				(PAUL(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let supersig_account = get_supersig_account(0);
		assert_ok!(Balances::transfer(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			100_000
		));

		let bob_balance = Balances::free_balance(BOB());

		let call = pallet_balances::Call::transfer {
			dest: BOB(),
			value: 100_000,
		};

		assert_ok!(Supersig::submit_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.into())
		));

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			0
		));
		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(BOB()).into(),
			supersig_account.clone(),
			0
		));

		// the call have been approved, so it is executed, and then the call is deleted from
		// storage

		assert_eq!(Supersig::votes(0, 0), 0);
		assert!(!Supersig::members_votes((0, 0, ALICE())));
		assert!(!Supersig::members_votes((0, 0, BOB())));
		assert!(!Supersig::members_votes((0, 0, CHARLIE())));

		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Balances::reserved_balance(ALICE()), 0);

		let supersig_event = last_event();

		assert_eq!(bob_balance + 100_000, Balances::free_balance(BOB()));

		let dispatch_result = Balances::transfer(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			100_000,
		);

		assert_eq!(
			supersig_event,
			RuntimeEvent::Supersig(crate::Event::CallExecutionAttempted(
				supersig_account,
				0,
				dispatch_result
			))
		);
	});
}

#[test]
fn approve_failing_call_as_master() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Master),
				(CHARLIE(), Role::Standard),
				(PAUL(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let supersig_account = get_supersig_account(0);
		assert_ok!(Balances::transfer(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			100_000
		));

		let call = pallet_balances::Call::transfer {
			dest: BOB(),
			value: 10_000_000,
		};

		assert_ok!(Supersig::submit_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.into())
		));

		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			0
		));
		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(BOB()).into(),
			supersig_account.clone(),
			0
		));

		assert_eq!(Supersig::votes(0, 0), 0);
		assert!(!Supersig::members_votes((0, 0, ALICE())));
		assert!(!Supersig::members_votes((0, 0, BOB())));
		assert!(!Supersig::members_votes((0, 0, CHARLIE())));

		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Balances::reserved_balance(ALICE()), 0);

		let supersig_event = last_event();

		assert_eq!(
			supersig_event,
			RuntimeEvent::Supersig(crate::Event::CallExecutionAttempted(
				supersig_account,
				0,
				Err(pallet_balances::Error::<Test>::InsufficientBalance.into())
			))
		);
	});
}

#[test]
fn approve_supersig_doesnt_exist() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let supersig_account = get_supersig_account(0);

		let call = frame_system::Call::remark {
			remark: "test".into(),
		};
		assert_ok!(Supersig::submit_call(
			RawOrigin::Signed(CHARLIE()).into(),
			supersig_account,
			Box::new(call.into())
		));
		assert_noop!(
			Supersig::approve_call(
				RawOrigin::Signed(CHARLIE()).into(),
				get_supersig_account(3),
				0
			),
			Error::<Test>::NotSupersig
		);
	})
}

#[test]
fn user_already_voted() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let supersig_account = get_supersig_account(0);

		let call = frame_system::Call::remark {
			remark: "test".into(),
		};
		assert_ok!(Supersig::submit_call(
			RawOrigin::Signed(CHARLIE()).into(),
			supersig_account.clone(),
			Box::new(call.into())
		));
		assert_ok!(Supersig::approve_call(
			RawOrigin::Signed(CHARLIE()).into(),
			supersig_account.clone(),
			0
		));
		assert_noop!(
			Supersig::approve_call(RawOrigin::Signed(CHARLIE()).into(), supersig_account, 0),
			Error::<Test>::AlreadyVoted
		);
	})
}

#[test]
fn approve_not_a_member() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let supersig_account = get_supersig_account(0);

		let call = frame_system::Call::remark {
			remark: "test".into(),
		};
		assert_ok!(Supersig::submit_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.into())
		));
		assert_noop!(
			Supersig::approve_call(RawOrigin::Signed(CHARLIE()).into(), supersig_account, 0),
			Error::<Test>::NotMember
		);
	})
}
