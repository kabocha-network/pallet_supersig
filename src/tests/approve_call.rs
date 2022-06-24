use super::{helper::*, mock::*};
use crate::{Error, Role};
use frame_support::{assert_noop, assert_ok};
use pallet_balances::WeightInfo;
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
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap(),
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

		assert_ok!(Supersig::approve_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			0
		));
		assert_eq!(Supersig::votes(0, 0), 1);
		assert!(Supersig::members_votes((0, 0, ALICE())));
		assert!(!Supersig::members_votes((0, 0, CHARLIE())));
		assert!(!Supersig::members_votes((0, 0, BOB())));
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::CallVoted(supersig_account, 0, ALICE()))
		);
	})
}

#[test]
fn approve_call_until_threshold() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
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
			Origin::signed(ALICE()),
			supersig_account.clone(),
			1_000_000_000,
		));

		let bob_balance = Balances::free_balance(BOB());

		let amount_transferd = 100_000;
		let call = Call::Balances(pallet_balances::Call::transfer {
			dest: BOB(),
			value: amount_transferd,
		});

		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			Box::new(call)
		));

		assert_ok!(Supersig::approve_call(
			Origin::signed(BOB()),
			supersig_account.clone(),
			0
		));

		let supersig_balance = Balances::free_balance(&supersig_account);
		assert_ok!(Supersig::approve_call(
			Origin::signed(ALICE()),
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

		assert_eq!(bob_balance + 100_000, Balances::free_balance(BOB()));
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::CallExecutionAttempted(
				supersig_account.clone(),
				0,
				Ok(Ok(()))
			))
		);

		assert_eq!(
			Balances::free_balance(&supersig_account),
			supersig_balance
				- amount_transferd
				- pallet_transaction_payment::<T>:: <TestRuntime as pallet_balances::Config>::WeightInfo::transfer()
		);
	})
}

#[test]
fn approve_call_as_master() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
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
			Origin::signed(ALICE()),
			supersig_account.clone(),
			1_000_000_000
		));

		let bob_balance = Balances::free_balance(BOB());

		let call = Call::Balances(pallet_balances::Call::transfer {
			dest: BOB(),
			value: 100_000,
		});

		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			Box::new(call)
		));

		assert_ok!(Supersig::approve_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			0
		));
		assert_ok!(Supersig::approve_call(
			Origin::signed(BOB()),
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

		assert_eq!(bob_balance + 100_000, Balances::free_balance(BOB()));
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::CallExecutionAttempted(
				supersig_account,
				0,
				Ok(Ok(()))
			))
		);
	});
}

#[test]
fn approve_supersig_doesnt_exist() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let supersig_account = get_supersig_account(0);

		let call = Call::Nothing(NoCall::do_nothing {
			nothing: "test".into(),
		});
		assert_ok!(Supersig::submit_call(
			Origin::signed(CHARLIE()),
			supersig_account,
			Box::new(call)
		));
		assert_noop!(
			Supersig::approve_call(Origin::signed(CHARLIE()), get_supersig_account(3), 0),
			Error::<TestRuntime>::NotSupersig
		);
	})
}

#[test]
fn user_already_voted() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let supersig_account = get_supersig_account(0);

		let call = Call::Nothing(NoCall::do_nothing {
			nothing: "test".into(),
		});
		assert_ok!(Supersig::submit_call(
			Origin::signed(CHARLIE()),
			supersig_account.clone(),
			Box::new(call)
		));
		assert_ok!(Supersig::approve_call(
			Origin::signed(CHARLIE()),
			supersig_account.clone(),
			0
		));
		assert_noop!(
			Supersig::approve_call(Origin::signed(CHARLIE()), supersig_account, 0),
			Error::<TestRuntime>::AlreadyVoted
		);
	})
}

#[test]
fn approve_not_a_member() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
			}
			.try_into()
			.unwrap(),
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
			Supersig::approve_call(Origin::signed(CHARLIE()), supersig_account, 0),
			Error::<TestRuntime>::NotMember
		);
	})
}
